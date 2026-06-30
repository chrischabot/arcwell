use crate::*;

pub(crate) fn provider(store: Store, args: ProviderCommand) -> Result<()> {
    match args.command {
        ProviderSubcommand::Probe { providers } => {
            print_json(&store.provider_credential_probe(&providers)?)
        }
    }
}

pub(crate) fn doctor(store: Store, args: DoctorArgs) -> Result<()> {
    let service_plist_path = if args.strict {
        Some(service_plist_path()?)
    } else {
        None
    };
    let mut report = store.doctor(DoctorOptions {
        strict: args.strict,
        max_worker_heartbeat_age_seconds: args.max_worker_heartbeat_age_seconds,
        max_dead_lettered_jobs: args.max_dead_lettered_jobs,
        max_backup_age_seconds: args.max_backup_age_seconds,
        service_plist_path: service_plist_path.clone(),
    })?;
    if args.strict {
        if let Some(path) = service_plist_path {
            report
                .failures
                .extend(service_plist_contract_failures(&path));
            report.ok = report.health.ok && report.failures.is_empty();
        }
    }
    print_json(&report)?;
    if args.strict && !report.ok {
        bail!("doctor strict failed");
    }
    Ok(())
}

const SERVICE_LABEL: &str = "com.arcwell.worker";

pub(crate) fn service(store: Store, args: ServiceCommand) -> Result<()> {
    match args.command {
        ServiceSubcommand::Install {
            max_jobs_per_tick,
            idle_sleep_ms,
            no_load,
        } => {
            let paths = store.paths().clone();
            let plist_path = service_plist_path()?;
            let log_dir = paths.home.join("logs");
            fs::create_dir_all(&log_dir)
                .with_context(|| format!("creating {}", log_dir.display()))?;
            if let Some(parent) = plist_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("creating {}", parent.display()))?;
            }
            let binary = std::env::current_exe().context("resolving current executable")?;
            let plist = launch_agent_plist(
                &binary,
                &paths.home,
                &log_dir,
                max_jobs_per_tick,
                idle_sleep_ms,
            );
            fs::write(&plist_path, plist)
                .with_context(|| format!("writing {}", plist_path.display()))?;
            let enable = if no_load {
                json!({ "attempted": false })
            } else {
                let enable = enable_service_label()?;
                if !json_bool(&enable, "ok") {
                    print_json(&json!({
                        "ok": false,
                        "label": SERVICE_LABEL,
                        "plist": plist_path,
                        "log_dir": log_dir,
                        "enable": enable,
                        "load": { "attempted": false }
                    }))?;
                    bail!("launchctl enable failed for {SERVICE_LABEL}");
                }
                enable
            };
            let load = if no_load {
                json!({ "attempted": false })
            } else {
                run_launchctl(&[
                    "bootstrap",
                    &format!("gui/{}", current_uid()?),
                    &plist_path.to_string_lossy(),
                ])
            };
            let ok = !json_bool(&load, "attempted") || json_bool(&load, "ok");
            print_json(&json!({
                "ok": ok,
                "label": SERVICE_LABEL,
                "plist": plist_path,
                "log_dir": log_dir,
                "enable": enable,
                "load": load
            }))?;
            if !ok {
                bail!("launchctl bootstrap failed for {SERVICE_LABEL}");
            }
            Ok(())
        }
        ServiceSubcommand::Status => {
            let plist_path = service_plist_path()?;
            let heartbeat = store.latest_worker_heartbeat()?;
            let heartbeat_events = store.list_worker_heartbeat_events(50)?;
            let launchctl = run_launchctl(&[
                "print",
                &format!("gui/{}/{}", current_uid()?, SERVICE_LABEL),
            ]);
            print_json(&json!({
                "label": SERVICE_LABEL,
                "installed": plist_path.exists(),
                "plist": plist_path,
                "heartbeat": heartbeat,
                "heartbeat_events": heartbeat_events,
                "launchctl": launchctl
            }))
        }
        ServiceSubcommand::RecurrenceAudit {
            min_span_hours,
            max_gap_seconds,
        } => {
            let min_span_seconds = min_span_hours
                .checked_mul(60 * 60)
                .context("min-span-hours is too large")?;
            let audit = store.audit_worker_recurrence(min_span_seconds, max_gap_seconds)?;
            print_json(&audit)?;
            if !audit.ok {
                bail!("worker recurrence audit failed");
            }
            Ok(())
        }
        ServiceSubcommand::Restart => {
            let plist_path = service_plist_path()?;
            let restart = run_launchctl(&[
                "kickstart",
                "-k",
                &format!("gui/{}/{}", current_uid()?, SERVICE_LABEL),
            ]);
            let (enable, bootstrap) = if json_bool(&restart, "ok") {
                (json!({ "attempted": false }), json!({ "attempted": false }))
            } else if plist_path.exists() {
                let enable = enable_service_label()?;
                let bootstrap = if json_bool(&enable, "ok") {
                    run_launchctl(&[
                        "bootstrap",
                        &format!("gui/{}", current_uid()?),
                        &plist_path.to_string_lossy(),
                    ])
                } else {
                    json!({ "attempted": false })
                };
                (enable, bootstrap)
            } else {
                (json!({ "attempted": false }), json!({ "attempted": false }))
            };
            let ok = json_bool(&restart, "ok") || json_bool(&bootstrap, "ok");
            print_json(&json!({
                "ok": ok,
                "label": SERVICE_LABEL,
                "plist": plist_path,
                "restart": restart,
                "enable": enable,
                "bootstrap": bootstrap
            }))?;
            if !ok {
                bail!("launchctl restart/bootstrap failed for {SERVICE_LABEL}");
            }
            Ok(())
        }
        ServiceSubcommand::Logs => {
            let paths = store.paths();
            let stdout_path = paths.home.join("logs").join("worker.out.log");
            let stderr_path = paths.home.join("logs").join("worker.err.log");
            print_json(&json!({
                "stdout_path": stdout_path,
                "stderr_path": stderr_path,
                "stdout": read_tail_status(&stdout_path, 4000),
                "stderr": read_tail_status(&stderr_path, 4000)
            }))
        }
        ServiceSubcommand::Uninstall { no_unload } => {
            let plist_path = service_plist_path()?;
            let unload = if no_unload {
                json!({ "attempted": false })
            } else {
                run_launchctl(&[
                    "bootout",
                    &format!("gui/{}/{}", current_uid()?, SERVICE_LABEL),
                ])
            };
            let removed = if plist_path.exists() {
                fs::remove_file(&plist_path)
                    .with_context(|| format!("removing {}", plist_path.display()))?;
                true
            } else {
                false
            };
            print_json(&json!({
                "ok": true,
                "label": SERVICE_LABEL,
                "plist": plist_path,
                "removed": removed,
                "unload": unload
            }))
        }
    }
}

pub(crate) fn service_plist_path() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME is not set")?;
    Ok(PathBuf::from(home)
        .join("Library")
        .join("LaunchAgents")
        .join(format!("{SERVICE_LABEL}.plist")))
}

pub(crate) fn launch_agent_plist(
    binary: &std::path::Path,
    home: &std::path::Path,
    log_dir: &std::path::Path,
    max_jobs_per_tick: usize,
    idle_sleep_ms: u64,
) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{label}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{binary}</string>
    <string>--home</string>
    <string>{home}</string>
    <string>worker</string>
    <string>run</string>
    <string>--max-jobs-per-tick</string>
    <string>{max_jobs_per_tick}</string>
    <string>--idle-sleep-ms</string>
    <string>{idle_sleep_ms}</string>
  </array>
  <key>KeepAlive</key>
  <true/>
  <key>RunAtLoad</key>
  <true/>
  <key>StandardOutPath</key>
  <string>{stdout}</string>
  <key>StandardErrorPath</key>
  <string>{stderr}</string>
</dict>
</plist>
"#,
        label = xml_escape(SERVICE_LABEL),
        binary = xml_escape(&binary.to_string_lossy()),
        home = xml_escape(&home.to_string_lossy()),
        max_jobs_per_tick = max_jobs_per_tick.clamp(1, 100),
        idle_sleep_ms = idle_sleep_ms.max(250),
        stdout = xml_escape(&log_dir.join("worker.out.log").to_string_lossy()),
        stderr = xml_escape(&log_dir.join("worker.err.log").to_string_lossy())
    )
}

pub(crate) fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub(crate) fn current_uid() -> Result<String> {
    let output = ProcessCommand::new("id")
        .arg("-u")
        .output()
        .context("running id -u")?;
    if !output.status.success() {
        bail!("id -u failed");
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub(crate) fn enable_service_label() -> Result<Value> {
    Ok(run_launchctl(&[
        "enable",
        &format!("gui/{}/{}", current_uid()?, SERVICE_LABEL),
    ]))
}

pub(crate) fn run_launchctl(args: &[&str]) -> Value {
    match ProcessCommand::new("launchctl").args(args).output() {
        Ok(output) => json!({
            "attempted": true,
            "ok": output.status.success(),
            "status": output.status.code(),
            "stdout": String::from_utf8_lossy(&output.stdout).trim(),
            "stderr": String::from_utf8_lossy(&output.stderr).trim()
        }),
        Err(error) => json!({
            "attempted": true,
            "ok": false,
            "error": error.to_string()
        }),
    }
}

pub(crate) fn read_tail(path: &std::path::Path, max_bytes: usize) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    let start = bytes.len().saturating_sub(max_bytes);
    Ok(String::from_utf8_lossy(&bytes[start..]).to_string())
}

pub(crate) fn read_tail_status(path: &std::path::Path, max_bytes: usize) -> Value {
    match read_tail(path, max_bytes) {
        Ok(tail) => json!({ "ok": true, "tail": tail }),
        Err(error) => json!({ "ok": false, "error": error.to_string() }),
    }
}

pub(crate) fn json_bool(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

pub(crate) fn service_plist_contract_failures(path: &std::path::Path) -> Vec<String> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) => {
            return vec![format!(
                "service plist is unreadable for contract validation: {} ({error})",
                path.display()
            )];
        }
    };
    let mut failures = Vec::new();
    match plist_string_after_key(&contents, "Label") {
        Some(label) if label == SERVICE_LABEL => {}
        Some(label) => failures.push(format!(
            "service plist label mismatch: expected {SERVICE_LABEL}, found {label}"
        )),
        None => failures.push("service plist is missing Label".to_string()),
    }
    let args = plist_array_strings_after_key(&contents, "ProgramArguments");
    if args.is_empty() {
        failures.push("service plist is missing ProgramArguments".to_string());
    } else {
        let binary = std::path::PathBuf::from(&args[0]);
        match fs::metadata(&binary) {
            Ok(metadata) if metadata.is_file() => {}
            Ok(_) => failures.push(format!(
                "service binary path is not a file: {}",
                binary.display()
            )),
            Err(error) => failures.push(format!(
                "service binary is missing or unreadable: {} ({error})",
                binary.display()
            )),
        }
        if !args.windows(2).any(|window| window == ["worker", "run"]) {
            failures.push("service plist does not run `arcwell worker run`".to_string());
        }
    }
    for key in ["StandardOutPath", "StandardErrorPath"] {
        match plist_string_after_key(&contents, key) {
            Some(value) => {
                let log_path = std::path::PathBuf::from(value);
                match log_path.parent() {
                    Some(parent) if parent.is_dir() => {}
                    Some(parent) => failures.push(format!(
                        "service {key} parent directory is missing: {}",
                        parent.display()
                    )),
                    None => failures.push(format!("service {key} has no parent directory")),
                }
            }
            None => failures.push(format!("service plist is missing {key}")),
        }
    }
    for key in ["KeepAlive", "RunAtLoad"] {
        if !contents.contains(&format!("<key>{key}</key>")) {
            failures.push(format!("service plist is missing {key}"));
        }
    }
    failures
}

pub(crate) fn plist_string_after_key(contents: &str, key: &str) -> Option<String> {
    let needle = format!("<key>{key}</key>");
    let after_key = contents.get(contents.find(&needle)? + needle.len()..)?;
    let start = after_key.find("<string>")? + "<string>".len();
    let after_start = after_key.get(start..)?;
    let end = after_start.find("</string>")?;
    Some(xml_unescape(&after_start[..end]))
}

pub(crate) fn plist_array_strings_after_key(contents: &str, key: &str) -> Vec<String> {
    let needle = format!("<key>{key}</key>");
    let Some(after_key) = contents
        .find(&needle)
        .and_then(|index| contents.get(index + needle.len()..))
    else {
        return Vec::new();
    };
    let Some(after_array) = after_key
        .find("<array>")
        .and_then(|index| after_key.get(index + "<array>".len()..))
    else {
        return Vec::new();
    };
    let Some(array_contents) = after_array.split("</array>").next() else {
        return Vec::new();
    };
    let mut values = Vec::new();
    let mut rest = array_contents;
    while let Some(start_index) = rest.find("<string>") {
        let after_start = &rest[start_index + "<string>".len()..];
        let Some(end_index) = after_start.find("</string>") else {
            break;
        };
        values.push(xml_unescape(&after_start[..end_index]));
        rest = &after_start[end_index + "</string>".len()..];
    }
    values
}

pub(crate) fn xml_unescape(value: &str) -> String {
    value
        .replace("&apos;", "'")
        .replace("&quot;", "\"")
        .replace("&gt;", ">")
        .replace("&lt;", "<")
        .replace("&amp;", "&")
}

pub(crate) fn worker(store: Store, args: WorkerCommand) -> Result<()> {
    match args.command {
        WorkerSubcommand::RunOnce { max_jobs } => print_json(&store.run_worker_once(max_jobs)?),
        WorkerSubcommand::Run {
            max_jobs_per_tick,
            idle_sleep_ms,
            max_ticks,
        } => {
            let mut ticks = 0usize;
            let mut processed = 0usize;
            let mut completed = 0usize;
            let mut failed = 0usize;
            let mut deferred = 0usize;
            let mut dead_lettered = 0usize;
            loop {
                if max_ticks.is_some_and(|limit| ticks >= limit.clamp(1, 10_000)) {
                    break;
                }
                let report = store.run_worker_once(max_jobs_per_tick)?;
                ticks += 1;
                processed += report.processed;
                completed += report.completed;
                failed += report.failed;
                deferred += report.deferred;
                dead_lettered += report.dead_lettered;
                if report.processed > 0 {
                    println!("{}", serde_json::to_string(&report)?);
                }
                std::thread::sleep(std::time::Duration::from_millis(
                    idle_sleep_ms.clamp(250, 60_000),
                ));
            }
            if max_ticks.is_some() {
                println!(
                    "{}",
                    serde_json::to_string(&json!({
                        "status": "completed",
                        "ticks": ticks,
                        "processed": processed,
                        "completed": completed,
                        "failed": failed,
                        "deferred": deferred,
                        "dead_lettered": dead_lettered,
                        "max_jobs_per_tick": max_jobs_per_tick.clamp(1, 100),
                        "idle_sleep_ms": idle_sleep_ms.clamp(250, 60_000),
                    }))?
                );
            }
            Ok(())
        }
    }
}

pub(crate) fn profile(store: Store, args: ProfileCommand) -> Result<()> {
    match args.command {
        ProfileSubcommand::Set {
            key,
            value,
            sensitivity,
            source,
        } => {
            store.set_profile(&key, &value, &sensitivity, &source)?;
            print_json(&json!({ "ok": true, "key": key }))
        }
        ProfileSubcommand::Get { key } => print_json(&store.get_profile(&key)?),
        ProfileSubcommand::Search { query } => print_json(&store.search_profile(&query)?),
        ProfileSubcommand::List => print_json(&store.list_profile()?),
        ProfileSubcommand::Delete { key } => {
            print_json(&json!({ "ok": store.delete_profile(&key)?, "key": key }))
        }
    }
}

pub(crate) fn memory(store: Store, args: MemoryCommand) -> Result<()> {
    match args.command {
        MemorySubcommand::Add {
            text,
            kind,
            sensitivity,
            source,
            confidence,
        } => {
            let id = store.add_memory(&text, &kind, &sensitivity, &source, confidence)?;
            print_json(&json!({ "ok": true, "id": id }))
        }
        MemorySubcommand::Search { query } => print_json(&store.search_memories(&query)?),
        MemorySubcommand::List { limit } => print_json(&store.list_memories(limit)?),
        MemorySubcommand::Delete { id } => {
            print_json(&json!({ "ok": store.delete_memory(&id)?, "id": id }))
        }
        MemorySubcommand::Mem0Add {
            text,
            user_id,
            source,
            sensitivity,
            infer,
        } => print_json(&store.mem0_add_memory(
            &text,
            user_id.as_deref(),
            &source,
            &sensitivity,
            infer,
        )?),
        MemorySubcommand::Mem0Search {
            query,
            user_id,
            limit,
        } => print_json(&store.mem0_search_memories(&query, user_id.as_deref(), limit)?),
        MemorySubcommand::Mem0Update { id, text, user_id } => {
            print_json(&store.mem0_update_memory(&id, &text, user_id.as_deref())?)
        }
        MemorySubcommand::Mem0Delete { id, user_id } => {
            print_json(&store.mem0_delete_memory(&id, user_id.as_deref())?)
        }
        MemorySubcommand::Mem0History { id } => print_json(&store.mem0_history(&id)?),
        MemorySubcommand::Mem0ForgetUser { user_id } => {
            print_json(&store.mem0_forget_user(user_id.as_deref())?)
        }
        MemorySubcommand::Recall {
            query,
            user_id,
            limit,
        } => print_json(&store.memory_recall_context(&query, user_id.as_deref(), limit)?),
        MemorySubcommand::Capture {
            text,
            source,
            user_id,
            auto_apply,
            infer,
        } => print_json(&store.capture_memory_from_text(
            &text,
            &source,
            user_id.as_deref(),
            auto_apply,
            infer,
        )?),
        MemorySubcommand::Events { limit } => {
            print_json(&store.list_memory_lifecycle_events(limit)?)
        }
        MemorySubcommand::Decisions { limit } => print_json(&store.list_memory_decisions(limit)?),
        MemorySubcommand::Tombstones { limit } => {
            print_json(&store.list_memory_forget_tombstones(limit)?)
        }
        MemorySubcommand::EvalCorpus => print_json(&personal_memory_eval_corpus()),
        MemorySubcommand::Dream => print_json(&store.dream_reconcile_memories()?),
        MemorySubcommand::HookRecall {
            event,
            query,
            user_id,
            limit,
        } => {
            let input = read_stdin_lossy()?;
            let query = query.unwrap_or_else(|| {
                hook_text_from_input(&input).unwrap_or_else(|| {
                    format!(
                        "Codex hook {event}: recall stable user preferences and project context"
                    )
                })
            });
            let recall = store.memory_recall_context(&query, user_id.as_deref(), limit)?;
            print_json(&json!({
                "ok": true,
                "event": event,
                "additionalContext": recall.context,
                "recall": recall
            }))
        }
        MemorySubcommand::HookCapture {
            event,
            text,
            user_id,
            auto_apply,
            infer,
        } => {
            let input = read_stdin_lossy()?;
            let text = text
                .or_else(|| hook_text_from_input(&input))
                .unwrap_or_default();
            if text.trim().is_empty() {
                print_json(&json!({ "ok": true, "event": event, "skipped": "empty hook input" }))
            } else {
                let auto_apply = auto_apply
                    || std::env::var("ARCWELL_MEMORY_HOOK_AUTO_APPLY").as_deref() == Ok("1");
                let infer =
                    infer || std::env::var("ARCWELL_MEMORY_HOOK_INFER").as_deref() == Ok("1");
                let capture = store.capture_memory_from_text(
                    &text,
                    &format!("codex-hook:{event}"),
                    user_id.as_deref(),
                    auto_apply,
                    infer,
                )?;
                print_json(&json!({ "ok": true, "event": event, "capture": capture }))
            }
        }
    }
}
