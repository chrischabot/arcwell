# Moving codex-swift Addons Back To Regular Codex

Date: 2026-06-19

## Assumptions And Success Criteria

Assumptions:

- "Regular Codex" means current OpenAI Codex CLI/app/IDE, not a fork of `openai/codex`.
- The goal is to preserve the useful personal-assistant capabilities, not every daemon/runtime detail of the Swift port.
- Custom local daemons are acceptable, especially for memory, Telegram, X, and watch/ingest jobs.
- Remote Cloudflare Workers are acceptable when always-on HTTPS, cheap storage, OAuth/DCR, or public MCP reachability matters.
- Existing Codex automations are acceptable replacements for codex-swift cron where they can schedule regular Codex work.
- The preferred distribution shape is open-sourceable service packages such as `agent-memory`, `agent-llm-wiki`, `agent-telegram`, `agent-x`, `agent-mail`, and a shared `agent-channel-kit`, not a private monolithic "personal Codex" fork/plugin.
- Capabilities should be usable by other agents where practical, especially Claude Desktop, with MCP as the primary portability layer.
- Drop the MLX/local-inference lane for the first extraction daemon. Use current best hosted GPT/OpenAI models per job instead, with model names kept in config rather than buried in prompts.

Success criteria:

- No persistent fork of Codex is required for the top-priority capabilities.
- Capabilities install through regular Codex surfaces: plugin, skill, MCP server, hook, automation, or Codex SDK/app-server client.
- Shared services expose MCP-compatible tools/resources so Claude Desktop and other MCP clients can use them without Codex-specific code.
- Long-running, inbound, or credential-heavy work lives outside Codex as local sidecars or Cloudflare Workers with narrow MCP/SDK interfaces.
- Always-on internet-facing work can keep bounded, short-lived queues at the edge and sync into the local service when the laptop is online.
- Personal memory remains reviewable, deletable, and portable across agents; wiki outputs remain inspectable, source-backed, and portable across agents.
- Conversation-derived profile/preferences are inspectable, editable, deletable, exportable, and backed by summaries/source cards rather than raw transcript dumps or hidden prompt sludge.
- The system treats competent effort as part of respect: it consults relevant memory/profile/wiki state, uses available tools/connectors when they materially improve accuracy, chooses adequate reasoning effort, and surfaces uncertainty or blockers instead of cheaply guessing.
- Destructive or external-write actions keep explicit authorization boundaries instead of being hidden behind prompts.

## What codex-swift Added

The port is two things mixed together:

1. A replacement runtime for Codex: Swift app-server protocol implementation, multi-process supervisor, workers, broker, auth, sandboxing, persistence, WebGateway UI, observability, workflows, and parity machinery.
2. A personal-agent capability suite layered on that runtime: Memory Wiki, mem0 memory, source ingestion, X import/monitoring, Telegram/Gmail channels, Google Workspace tools, push, cron, media generation, and related UI pages.

For de-porting, the first group mostly should not move. It duplicates regular Codex runtime behavior. The second group is the valuable part.

Local inventory anchors:

- Addon thesis and shipped portfolio: `ADDONS.md`, `docs/guides/addons-and-plugins.md`.
- Memory and wiki: `docs/features/memory.md`, `llm-wiki.md`, `Sources/MemoryStore`, `Sources/MemoryMCP`, `Sources/WikiIngest`, `Sources/WikiResearch`, `Sources/codex-memory`.
- X integration: `docs/X_INTEGRATION.md`, `Sources/Connectors/X*`, `Sources/WikiIngest/X*`, `Sources/codex-memory/X*`.
- Channels: `docs/features/channels.md`, `Sources/Channels`, `Sources/Gmail`.
- Google Workspace: `docs/features/connectors.md`, `Sources/Connectors`, `Sources/GoogleWorkspace`.
- Proactive outputs: `docs/features/push.md`, `docs/features/cron.md`, `docs/features/media.md`, `Sources/Push`, `Sources/Cron`, `Sources/Media`.
- Web UI: `www/ARCHITECTURE.md`, `www/src/pages/Wiki*`, `www/src/components/wiki`.
- Workflow engine: `docs/features/workflows.md`, `Sources/Workflows`.

## Current Regular Codex Extension Surfaces

Current Codex already has most of the right packaging seams:

- Skills are the reusable workflow format; plugins are the installable distribution unit for skills and apps. See OpenAI's [Agent Skills docs](https://developers.openai.com/codex/skills).
- Plugins bundle skills, app integrations, and MCP servers, with examples including Gmail and Google Drive plugins. See [Codex Plugins](https://developers.openai.com/codex/plugins).
- MCP is the live tool/context boundary. Codex supports stdio and streamable HTTP MCP servers, OAuth for HTTP servers, tool allow/deny lists, and plugin-bundled MCP config. See [Codex MCP](https://developers.openai.com/codex/mcp) and the [MCP tools spec](https://modelcontextprotocol.io/specification/2025-06-18/server/tools).
- Automations can use plugins and skills, and thread automations are designed for polling connected sources in an existing thread. See [Codex Automations](https://developers.openai.com/codex/app/automations).
- Codex SDK and app-server are the right way for a sidecar to create/resume Codex threads and stream events when an external channel needs Codex as the brain. See [Codex SDK](https://developers.openai.com/codex/sdk) and [Codex App Server](https://developers.openai.com/codex/app-server).
- Hooks can observe or enforce lifecycle events and can be bundled by plugins. This is useful for capture/export, but not for long-running services.
- Codex built-in Memories exist, but they are local recall files and background-generated summaries. They are not a replacement for the source-backed Memory Wiki or the Swift mem0 implementation.
- Cloudflare Workers are a viable remote MCP host. Cloudflare's Agents docs describe remote MCP over Streamable HTTP with auth, `createMcpHandler()` for stateless servers, and `McpAgent` for stateful/per-session Durable Object-backed servers.

Implication: the migration should be MCP-first at runtime, host-adapter-first for installation, and daemon/Worker-backed for long-running work. Do not patch upstream Codex unless a missing extension surface is proven to block everything else.

## Boundary Model

Use one consistent placement rule: capabilities are open-source `agent-*` packages; stateful work runs in local daemons or bounded Cloudflare Workers; agents interact through MCP and skills; host-specific glue lives under host adapters.

| Surface | Owns | Does not own | Examples |
|---|---|---|---|
| `agent-*` package | Distribution boundary, docs, schemas, fixtures, threat model, host adapters | Runtime authority by itself | `agent-memory`, `agent-llm-wiki`, `agent-garderobe` |
| Rust local daemon | Durable local state, SQLite, filesystem/wiki files, job queues, watchers, private processing, local HTTP/MCP | Public webhooks, OAuth callback stability, host-specific prompts | `memoryd`, `wikid`, `controllerd`, `opsd` |
| CLI | Setup, import/export, doctor, backup/restore, migrations, one-shot admin/debug commands | Background listening, agent reasoning, long-lived state mutation without logs | `agent memory doctor`, `agent wiki ingest`, `agent backup restore-plan` |
| MCP server | Agent-facing tools/resources, typed job handles, status, permissions, async job control | Long-running business logic that can outlive the MCP call unless delegated to daemon | `memory_search`, `wiki_ingest`, `controller_thread_status` |
| Skill | Procedural guidance: when to use which tools, quality gates, prompt discipline, host-specific workflows | Storage, credentials, background jobs, business logic | `$deep-research`, `$memory-review`, `$ops-diagnose` |
| Codex plugin/host adapter | Codex packaging: skills, MCP config, hooks where available, app-server/SDK bridge, marketplace metadata | Core domain behavior or portable service identity | `hosts/codex/plugin.json`, Codex thread bridge |
| Claude host adapter | Claude setup notes, MCP connector config, manual-capture guidance, remote MCP/OAuth notes | Codex-only lifecycle promises such as hooks/automations/app-server control | `hosts/claude/README.md` |
| Cloudflare Worker / DO / D1 / Queue / R2 | Always-on HTTPS, OAuth/DCR, public webhooks, bounded edge inbox, remote MCP for small structured domains, short-lived blob staging | Large private corpora, OS keychain secrets, local checkout access, high-authority Codex actions | Telegram webhook, Email Worker, X OAuth callback, Garderobe remote MCP |
| Automation / scheduler | Triggering existing tools or daemon jobs on a schedule | Owning cursors/state/business rules | Codex automation calls `wiki_watch_run_due`; Cloudflare cron enqueues X monitor |
| Hook | Lightweight host lifecycle observation/enforcement | Heavy extraction, long jobs, network polling, private memory reconciliation | pre-turn profile recall, post-turn capture candidate enqueue |
| Ops UI | Human visual inspection, logs, health, history, retries, approvals, config | Primary agent API or hidden state not exposed through MCP | local `/jobs`, `/sources`, `/errors`, `/approvals` |

Boundary rules:

- If it must keep running while Codex/Claude is not actively thinking, it belongs in a daemon or Worker, not a skill.
- If it needs private local files, SQLite, OS keychain secrets, or large corpora, prefer a local daemon.
- If it needs public HTTPS, OAuth/DCR, inbound webhooks, or cheap always-on capture, prefer Cloudflare.
- If an agent needs to call it, expose it through MCP with small, typed tools and resources.
- If a human needs to install, repair, migrate, or audit it, expose it through CLI and ops UI.
- If the behavior is "how the agent should think or sequence tools", encode it as a skill, not daemon code.
- If the behavior is Codex-only or Claude-only, keep it in `hosts/codex` or `hosts/claude`, never in the portable core.
- Avoid the word "addon" for the new architecture except when referring to codex-swift history. The target unit is an `agent-*` service package.

Placement decision tree:

1. Does it need to receive an event while the laptop/agent is not active?
   Put the ingress in Cloudflare if it is internet-facing; put the processor in a local daemon if it touches private state.
2. Does it need to maintain durable state, run a queue, watch files, poll APIs, or resume after crashes?
   Put it in a daemon/Worker and expose controls through MCP/CLI.
3. Is it a deterministic user/admin command?
   Put it in the CLI, backed by the same APIs as the daemon/MCP server.
4. Is it an agent-callable operation?
   Put a small, typed MCP tool/resource in front of the daemon/Worker.
5. Is it guidance about when/how to call tools or how to write/research/review?
   Put it in a skill or host-specific agent instructions.
6. Is it a Codex thread/control operation?
   Put it in `agent-controller` plus `hosts/codex`, with a degraded read-only MCP path for other agents.
7. Is it a public web UI or webhook?
   Put it in `worker/` only if it can operate with bounded state and scoped secrets; otherwise terminate at the edge and drain locally.

## Deployment Toolbox

Use two hosting lanes, chosen per capability:

| Lane | Best for | Avoid for |
|---|---|---|
| Local sidecar | Private laptop data, local files, SQLite storage, direct Codex SDK/app-server control, Telegram bridge that needs local workspace context | Public OAuth callbacks unless tunneled; always-on remote access |
| Cloudflare Worker / D1 / Durable Object / Queue / R2 | Always-on HTTPS MCP, OAuth/DCR, small structured stores, cheap scheduled jobs, admin UI, public webhook callbacks, short-lived edge inboxes, tools usable from multiple MCP clients | Large local corpora, OS-keychain secrets, local filesystem indexing, heavy inference jobs |

`/Users/chabotc/Projects/garderobe` is the working example of the Cloudflare lane:

- Single-user wardrobe MCP server on Cloudflare Workers.
- D1 for structured inventory, KV for OAuth token/grant storage, Durable Object binding for the MCP agent/session, cron trigger for nightly work.
- `/mcp` endpoint and `/admin` UI.
- OAuth 2.1 with Dynamic Client Registration via `@cloudflare/workers-oauth-provider`.
- `agents`, `@modelcontextprotocol/sdk`, and `zod` tool definitions.

This pattern fits small, structured personal domains very well: wardrobe, reading lists, content ideas, lightweight CRM, source capture inboxes, and possibly X-monitor state. It is less ideal for the full Memory Wiki if that wiki depends on local files, large corpora, or private desktop context. A hybrid is attractive: Cloudflare handles public OAuth/webhook/MCP front doors and structured state; local `wikid` handles heavyweight indexing and private filesystem ingestion.

Cloudflare pieces that fit the hybrid:

- Workers for public HTTPS endpoints, webhooks, OAuth callbacks, admin UI, and lightweight MCP tools.
- D1 for event metadata, cursors, OAuth state, account mappings, and small app state.
- Durable Objects for per-user/session coordination, ordering-sensitive cursors, long-poll/WebSocket sessions, and stateful MCP sessions.
- Queues for buffering webhook/email/crawl events and decoupling fast public handlers from slower processing. Cloudflare documents Queues as a Workers-integrated reliable buffer with messages retained until a consumer successfully processes them.
- R2 for temporary blobs: email attachments, rendered pages, screenshots, crawl artifacts, and export bundles. Keep lifecycle/TTL policies explicit.
- Email Workers for inbound email routing to code; this makes `agent-mail` possible without running an always-on mail server.
- Browser Run/Browser Rendering for edge-side page screenshots, PDFs, and structured extraction when the ingestion source is web-only and does not need local credentials.

## Open-Source Package Shape

The more reusable shape is not one `codex-personal` plugin. It is a family of small agent apps with a shared contract:

```text
agent-<domain>/
  crates/
    domain/        pure domain types, policy, validation, store traits
    daemon/        long-running local service and local HTTP API
    mcp/           MCP tool/resource server, usually thin over daemon API
    cli/           setup, doctor, import/export, migrations, backup/debug
  worker/          optional Cloudflare Worker: webhook, OAuth, queue, remote MCP
  ui/              optional local/edge browser UI; human ops, not hidden state
  hosts/
    codex/         Codex plugin manifest, skills, hooks, app-server/SDK adapter
    claude/        Claude Desktop/Code MCP setup, connector notes, degraded-mode docs
  schemas/         generated JSON Schema/OpenAPI/TypeScript clients
  docs/            install, threat model, limits, host matrix, examples
  test/fixtures    mocked platform events and replayable sync tests
```

Naming decision: use `agent-*`, not `codex-app-*`, for repo and package identity. Codex is one host adapter for these services, while Claude Desktop/Code and other MCP clients should be first-class consumers wherever the capability does not depend on Codex-only thread APIs. Use `codex-*` only for code that specifically targets Codex manifests, skills, SDK/app-server behavior, or compatibility shims.

Within a package, keep ownership crisp:

- `domain` crates contain no network clients, no host assumptions, and no Cloudflare-specific bindings.
- `daemon` crates own durable state and job execution, but not agent-facing prompt choreography.
- `mcp` crates translate agent tool calls into daemon operations and typed resources.
- `cli` crates call the same domain/daemon APIs as MCP, so doctor/import/restore paths exercise real code.
- `worker` code can enqueue, validate, and serve small remote MCP tools, but should not fork a second copy of local business logic unless the package is intentionally edge-native, as with `agent-garderobe`.
- `hosts/*` can add prompts, skills, config, and compatibility shims, but cannot become the only implementation of a portable capability.

Each package should be usable independently, but they can share a tiny Rust runtime crate family once two or three apps prove the same abstractions:

- `agent-envelope`: canonical event envelope, idempotency key, source identity, payload limits, provenance fields.
- `agent-channel`: shared channel contract for inbound/outbound messages, prompt-injection fencing, formatting, attachments, receipts, and routing policy.
- `agent-search`: optional search-provider abstraction for host-native web search, OpenAI web search/deep-research, Claude web search, Brave Search, Perplexity Search/Sonar, and future providers.
- `agent-edge-protocol`: Cloudflare auth helpers, D1 schemas, queue/R2 helpers, retention enforcement, and generated TypeScript bindings for Workers.
- `agent-local`: drain protocol, ack/retry, local secret loading, Codex SDK/app-server client wrapper.
- `agent-mcp`: common MCP error shapes, job resources, async job status conventions.
- `agent-backup`: encrypted snapshot, restore, integrity-check, and retention helpers for local SQLite stores, wiki files, ledgers, and configs.
- `agent-secrets`: credential storage/refresh/expiry/rotation contract across OS keychain, 0600 local files, and scoped Cloudflare secrets.
- `agent-cost`: per-job/model/provider token and spend ledger, budgets, estimates, alerts, and kill switches.

Recommended open-source apps:

| Package | Edge component | Local component | Agent surface |
|---|---|---|---|
| `agent-memory` | Usually none; optional encrypted edge backup/sync later | Personal mem0-style memory via `mem0-rs`, SQLite, and MCP/HTTP | Personal memory MCP + review skills |
| `agent-profile` | Usually none; optional encrypted backup/sync later | Inspectable preference/profile store for output style, recurring constraints, taste, sizes/settings, decision criteria, and support preferences | Profile/preference MCP + host prompt hints |
| `agent-backup` | Optional encrypted R2/object-store snapshots | Local encrypted versioned backups, restore, integrity checks, stale-backup alerts | Backup/restore MCP + doctor checks |
| `agent-secrets` | Scoped edge secret metadata/health only | OS keychain/0600 secret abstraction, OAuth refresh/expiry/rotation, grant health | Credential health MCP + setup/doctor skills |
| `agent-cost` | Optional edge job cost mirror | Local cost ledger, per-package/source/model budgets, run estimates, global kill switch | Cost/status MCP + ops UI cards |
| `agent-llm-wiki` | Capture endpoints, optional Browser Run fetch/render, queue/R2 staging, remote MCP for small queries | Knowledge corpus store, ingest pipeline, embeddings/extraction, UI/API, queue drain | Wiki MCP + research/ingest skills |
| `agent-telegram` | Telegram webhook, owner validation, queue, optional outbound send relay | Codex thread bridge, workspace policy, local approvals, streaming/replies | Skills for channel policy; MCP for send/status |
| `agent-controller` | Optional status inbox and remote-control front door; no public write surface by default | Project/thread registry, active-run monitor, summary index, natural-language router, Codex SDK/app-server client | MCP tools for project/thread status, routing, creation, resume/follow-up |
| `agent-ops-ui` | Optional read-only Cloudflare status mirror for always-on health | Local web UI for ingestion, jobs, history, logs, health, recent errors, and service config | MCP status/card tools first; in-app browser preview for visual/interactive inspection |
| `agent-conversation-import` | Usually none; optional encrypted import staging later | Consentful Claude/Codex export summarizer, redactor, profile/wiki/source-card writer, import ledger | Import/review MCP + migration skills |
| `agent-decision-support` | Optional price/watch capture later | Recommendation rationale ledger, option comparisons, purchase/return state, sizing/fit decisions, source snapshots | Compare/decide/revisit skills |
| `agent-routine-context` | Weather/calendar/location capture where explicitly configured | Opt-in daily context for weather, calendar, routines, health-adjacent constraints, outfit/travel planning | Daily brief/planning MCP |
| `agent-x` | OAuth callback, cron monitor, cursor/spend state, queue of imported items | Wiki writer, local enrichment, reports | X MCP + monitor/report skills |
| `agent-mail` | Email Worker, attachment staging, loop prevention metadata, queue | Mail drain, Gmail/IMAP/workspace archive writer, optional Codex thread bridge | Mail capture MCP + triage skills |
| `agent-garderobe` | Existing Cloudflare Worker/D1/OAuth remote MCP copied from `/Users/chabotc/Projects/garderobe` | Optional local mirror/export later | Wardrobe/outfit MCP + weather/style skills |
| `agent-content` | Optional publish/webhook capture for comments/social analytics | Writing style profile, editorial calendar, blog/social draft pipeline, asset ledger | Blog/post/report/slides/video skills + wiki/style MCP |
| `agent-workspace-context` | Optional Cloudflare email/docs capture inbox | Google Docs/Gmail/Contacts/Drive context index via official connectors first | Workspace-to-wiki capture + relationship/context skills |
| `agent-browser-workflows` | Usually none; optional remote browser capture | Browser/computer-use profiles, task recipes, screenshot/action ledgers | Browser workflow skills using host browser/computer tools |
| `agent-media` | Optional asset delivery/webhook callbacks | Slide/video/image generation ledger, source assets, exports | Media generation/publishing skills |
| `agent-deep-research` | Optional Cloudflare capture/queue for research seeds, scheduled checks, and source snapshots | Mostly none; uses host-agent native web search, Codex/Claude research workflows, wiki MCP, and optional search adapters | Deep-research skills + optional custom agents |
| `agent-channel-kit` | Shared Cloudflare helpers for webhook validation, queueing, delivery receipt storage | Shared local interfaces for channel drains, formatting, prompt-injection protection, and delivery outcomes | No generic app by default; concrete channels expose tools |
| `agent-search-kit` | Usually none; optional remote provider proxy if keys should stay off laptops | Optional MCP/search adapter wrapping Brave Search, Perplexity Search/Sonar, and provider result normalization | Search MCP tools/resources usable by Codex and Claude Desktop |
| `agent-quality-kit` | None | Reusable verification contracts, acceptance checks, replay fixtures, and source/visual/test validation helpers | Skills for bug reproduction, tests, citations, visual inspection, uncertainty surfacing |
| `agent-voice-mobile` | Optional channel front door for voice/mobile capture | Mostly none at first; uses concrete channel packages and controller | Voice/mobile capture hints and notification policies |
| `agent-runtime` | Shared only after repeated patterns appear | Shared only after repeated patterns appear | Installer/doctor helpers |

This lets other people adopt only what they need. Someone can run just `agent-llm-wiki` locally, or deploy `agent-telegram` on Cloudflare with their own bot token, without inheriting your X/Gmail/memory choices.

### Package Surface Matrix

Use these as defaults unless an implementation spike proves a better split:

| Package | Daemon | CLI | MCP | Skills | Cloudflare | Host adapter |
|---|---|---|---|---|---|---|
| `agent-profile` | Optional lightweight profile service; can start file/SQLite-backed | edit/import/export/doctor | read/search/update/review profile slices | when to consult profile; profile hygiene | backup only | prompt hints and relevant-slice loading |
| `agent-memory` | Required for capture/reconcile/dream jobs | import/export/review/debug/bench | recall, add/update/delete, lifecycle jobs | memory review/capture/dream/debug | optional encrypted backup later | Codex hooks where available; Claude manual capture |
| `agent-llm-wiki` | Required for ingest/search/index/librarian | ingest/reindex/reembed/audit | search/read/ingest/job/status/source cards | research/ingest/librarian workflows | capture inbox, Browser Run snapshots, remote small queries | host search/wiki guidance |
| `agent-controller` | Required for project/thread registry and Codex bridge | project/thread admin and doctor | status, resolve, continue, create, approvals | channel routing and status workflows | optional read-only status/front door | Codex app-server/SDK bridge; Claude read-only/status |
| `agent-channel-kit` | Library, not normally a daemon | fixture/replay helpers | only shared schemas if useful | channel safety guidance | shared webhook/receipt helpers | shared host docs |
| `agent-telegram` | Local bridge when driving Codex/local projects | setup/test-send/replay | send/status/thread map | Telegram policy and formatting | webhook, validation, queue, outbound relay | controller integration |
| `agent-mail` | Local drain/processor if mail becomes a channel | setup/replay/archive | mail capture/status/send-draft where allowed | triage/archive workflows | Email Worker, attachment staging | Gmail/Workspace connector hints |
| `agent-x` | Local enricher/wiki writer or Worker-native monitor by policy | oauth/import/report/replay | import/status/report | signal/report workflows | OAuth callback, cron/cursors/queue | wiki/content integration |
| `agent-content` | Usually not a daemon unless publishing queues exist | draft/export/publish-dry-run | style read, draft, rewrite, variants, calendar | writing/social/report workflows | optional analytics/comment webhooks | Codex/Claude writing guidance |
| `agent-garderobe` | Optional local mirror later | import/export/audit | outfit/wardrobe/weather/packing tools | outfit/packing/audit workflows | primary remote MCP/D1/OAuth app | Codex/Claude MCP setup |
| `agent-ops-ui` | Required local ops/status service | doctor/logs/open-ui | health/jobs/errors/logs/approvals/cards | ops status/diagnose/ui workflows | optional read-only status mirror | browser-use guidance |
| `agent-backup` | Scheduled local snapshot service | backup/verify/restore-plan/restore | status/create/verify/restore-plan | backup hygiene | optional encrypted R2 target | doctor warnings |
| `agent-secrets` | Usually library plus health checker | login/logout/rotate/doctor | credential health, not secret readout | setup/recovery guidance | scoped edge secret metadata | connector setup docs |
| `agent-cost` | Local ledger/guard | report/budget/kill-switch | cost summaries/budget controls | cost-aware effort guidance | optional status mirror | effort-router integration |
| `agent-deep-research` | No daemon initially | run/export/eval optional | uses wiki/search MCP rather than many custom tools | primary surface | optional seed capture | host-native search/subagent instructions |

The elegant default is thin surfaces over one owner of state. If both a CLI and MCP tool can mutate something, they should call the same daemon/domain API and produce the same audit ledger entry.

### Tool Surface And Host Capability Budget

The package family must not assume every MCP server and every tool is loaded into every agent session. A large tool surface will make Codex and Claude worse at choosing tools and will spend context on tools that are irrelevant to the current task.

Rules:

- Default host profiles should expose a small high-level tool set, then reveal detailed tools through skills, package-specific profiles, or a domain dispatcher.
- Tool names must be globally namespaced (`memory_*`, `wiki_*`, `controller_*`, `ops_*`, `profile_*`) and avoid vague collisions like many unrelated `status` tools.
- Each package should declare a minimal, recommended, and full MCP profile for Codex and Claude.
- Claude Desktop parity needs an explicit matrix. MCP tools/resources and remote connectors work well; Codex-style skills, hooks, automations, and lifecycle capture may degrade to manual commands or be unavailable.
- Never require all `agent-*` servers in one Codex/Claude session. Load by workflow: coding, personal profile/memory, wiki/research, channel ops, wardrobe, or media.

## Project / Thread Controller

Chat channels should not bind permanently to one workspace. They should talk to a shared meta-controller that can route user intent across projects, threads, and running work.

Package: `agent-controller`.

Responsibilities:

- Project registry: known projects, aliases, paths, repo metadata, default sandbox/profile, owner policy, and human-readable summaries.
- Thread registry: known Codex threads, project association, title, last activity, status, active/idle/archived state, current goal, branch/worktree, and latest summary.
- Active-run monitor: subscribe to app-server/SDK events where possible, poll/inspect status otherwise, and keep a lightweight run ledger.
- Summary index: maintain short, source-linked summaries of threads and projects so a channel can answer "how's the de-porting of codex-swift going?" without dumping raw transcripts.
- Natural-language router: resolve phrases like "the video project", "de-porting of codex swift", "that Telegram thing", or "hyper-agent" to likely project/thread candidates.
- Follow-up context: remember the last resolved project/thread per channel conversation so "and the video project?" works.
- Creation interface: create a new project/thread/work item from a channel message according to explicit owner/channel policy, with confirmation required only for actions outside the configured trust envelope.
- Friction budget: trusted owner channels should not leave work stuck behind invisible approvals. If an action cannot proceed automatically, the controller must send a clear channel notification and preserve resumable state.

MCP/tool surface:

- `controller_list_projects(query?, status?)`
- `controller_project_status(project_ref)`
- `controller_list_threads(project_ref?, status?)`
- `controller_thread_status(thread_ref)`
- `controller_resolve_reference(text, channel_context?)`
- `controller_summarize(ref, depth?)`
- `controller_continue_thread(thread_ref, message, mode?)`
- `controller_create_project(name, description, repo?, default_channel_policy?)`
- `controller_create_thread(project_ref, message, sandbox?, approval_mode?)`
- `controller_list_pending(channel_context?)`
- `controller_approve(pending_id)`
- `controller_cancel(pending_id)`

Routing behavior:

```text
channel message
  -> channel identity + conversation context
  -> controller_resolve_reference()
  -> if one confident match: status/read-only action
  -> if multiple plausible matches: ask a clarifying question
  -> if creation requested: propose or execute according to configured channel policy
  -> if approval is required: notify channel immediately with exact blocked action
```

Examples:

- "How's the de-porting of codex swift going?" resolves to the `codex-swift` project and this migration/report thread, then returns status, blockers, and next steps.
- "And the video project?" uses channel follow-up context plus project aliases to switch target and return that project's latest status.
- "Create a new project for hyper-agent where ..." creates a project/thread directly when the owner channel policy allows it; otherwise it returns the exact blocked action and approval path.

Safety rules:

- Status/summarization is read-only by default.
- Ambiguous routing asks, it does not guess silently.
- Channel text cannot grant itself workspace access or permissions.
- Creating/resuming write-capable Codex turns requires owner identity and configured approval policy. For trusted owner channels, low-risk writes can be preauthorized.
- Controller summaries should cite their source thread/project metadata and timestamps.
- Raw transcript access should be opt-in; channel replies should use compact summaries unless the user asks for details.
- Long-running or approval-blocked work must be visible through channel status, timeout, and resume commands. No quiet background hangs.

Codex implementation path:

- Use Codex SDK/app-server to start/resume/fork/list/archive threads where available. The Codex app-server docs describe thread APIs for starting, resuming, forking, listing/archiving conversations, and `turn/start` events for streaming progress.
- Store controller state in local SQLite first. Optionally mirror non-sensitive status summaries to Cloudflare for always-on read-only status.
- Expose controller tools by MCP so Claude Desktop can inspect project/thread status too, even if only Codex can actually resume Codex threads.

## Channel Contract

Do not build `agent-push`. "Push" is not a domain boundary. The real abstraction is a channel with inbound and outbound behavior, platform identity, formatting rules, prompt-injection controls, and delivery receipts.

Use `agent-channel-kit` as the shared package and keep concrete platforms in their own packages:

- `agent-telegram`: Telegram Bot API, chats, users, groups, message formatting, files, bot-token/webhook quirks.
- `agent-discord`: Discord users/guilds/channels/threads, slash commands, embeds, markdown, attachments, permissions.
- `agent-slack`: Slack workspaces/channels/threads, Block Kit, app mentions, interactive actions, workspace auth.
- `agent-mail` or `agent-gmail-channel`: inbound/outbound email, MIME, threading, loop prevention, reply quoting, unsubscribe/auto-reply hazards.

Shared channel model:

```text
platform event
  -> authenticated channel identity
  -> normalized inbound message
  -> prompt-injection screened context
  -> project/thread controller route
  -> Codex thread / automation / tool call
  -> outbound message intent
  -> platform formatter
  -> delivery attempt
  -> receipt
```

`agent-channel-kit` should define:

- Inbound envelope: `channel`, `platform`, `account_id`, `conversation_id`, `sender_id`, `message_id`, `received_at`, `text`, `attachments`, `reply_to`, `raw_ref`, and trust labels.
- Outbound intent: `target`, `thread_key`, `body`, `format`, `attachments`, `idempotency_key`, `reply_policy`, `approval_policy`, and expiry.
- Delivery outcomes: `sent`, `suppressed`, `partial_failed`, `failed`, with platform message ids and retry metadata.
- Formatting adapters: internal Markdown-ish body to Telegram MarkdownV2/HTML, Discord markdown/embed, Slack Block Kit/plain text, email MIME/plain/HTML.
- Prompt-injection controls: treat user/channel content as untrusted, quote or fence inbound text, strip platform commands from source text, separate identity metadata from message body, and prevent inbound text from granting permissions.
- Authorization policy: owner allowlists, per-channel scopes, group/channel routing, read-only defaults for unknown senders, and write/send approval requirements.
- Thread binding: stable mapping from platform conversation to controller-resolved project/thread context, with explicit reset/fork/switch behavior.

Concrete channel packages should expose their own tools and skills, not a vague global `push_send`. For example, `agent-telegram` can expose `telegram_send`, `telegram_status`, and `telegram_thread_map`; Discord would expose Discord-shaped tools. A higher-level automation can still say "notify me", but the configured channel adapter should resolve that to a concrete platform and target.

## Approval And Friction Policy

The UX target is minimal friction for trusted owner channels. A background agent that silently waits for approval is a product failure.

Use configurable approval tiers:

| Tier | Intended use | Behavior |
|---|---|---|
| `read_only` | Unknown senders, group chats, public channels | Summaries/status/search only; no writes or external sends. |
| `confirm_writes` | Default conservative mode | Reads are automatic; writes send a compact approval request. |
| `trusted_owner_fast` | Personal Telegram/owner channel | Preauthorize low-risk writes such as calendar creates, named-channel notifications, thread status checks, and continuing known threads. |
| `dangerous_confirm` | High-impact actions | Always require explicit confirmation: deletes, broad bulk edits, money/spend changes, credential changes, public posts, repo destructive operations, or ambiguous project creation. |

Rules:

- The owner should be able to configure per-channel and per-tool defaults, e.g. `calendar.create = trusted_owner_fast`, `calendar.delete = dangerous_confirm`, `github.push = confirm_writes`.
- If a write is preauthorized, execute it and send a receipt with exactly what changed.
- If a write is blocked, notify the channel immediately with the exact action, why it is blocked, and a short approval/resume command.
- Timeouts are first-class. A channel turn that waits longer than the configured limit should send "still working" or "blocked" status rather than disappearing.
- Approval prompts should be durable. If the user switches away, the controller can resume from the pending approval id instead of restarting the whole task.
- For calendar and reminder writes from a trusted owner channel, default can be no-confirm if the request has enough date/time/title context and no ambiguous attendees/calendars. Missing details still ask a targeted question.
- For ambiguous natural language, ask one clarifying question rather than making the user watch a long failed run.

## UI Strategy

Split knowledge editing from operations:

- Durable Memory Wiki pages should be Markdown files on disk. Use Obsidian for browsing, linking, manual editing, and review.
- Operational surfaces need their own UI: ingestion jobs, queue status, source health, watch history, recent errors, logs, service config, connector auth state, pending approvals, and controller/project/thread status.

Codex Desktop does not currently document a general extension point for arbitrary custom panels inside the app. The documented UI-adjacent surfaces are:

- Plugins/skills/MCP/connectors for capabilities.
- The task sidebar for plan, sources, generated artifacts, and task summaries.
- Preview of non-code artifacts such as PDFs, spreadsheets, documents, and presentations.
- The in-app browser for local development servers, file-backed previews, and public pages that do not require sign-in.
- Browser Use for Codex to operate local dev-server/file-backed pages.

So the recommended approach is:

- Build `agent-ops-ui` as a standalone local web app served by the local daemon/controller.
- Treat MCP tools/resources as the primary agent interface for ops work.
- Open the local web UI in Codex's in-app browser only for visual inspection, comments, and browser-use workflows.
- Expose all important ops data through MCP tools/resources so Codex, Claude Desktop, and other agents can query it without the UI.
- Return compact status cards in chat using MCP `structuredContent` plus Markdown fallbacks. Treat these as message-stream summaries, not a replacement for the full operations UI.
- Optionally mirror non-sensitive health/status summaries to Cloudflare for always-on read-only access.

Agent-facing skills/hints:

- `$ops-status`: quick health/readout path. Use MCP tools first, summarize current service health, queue depth, active jobs, recent failures, and pending approvals. Do not open the browser unless the user asks for visual inspection or interaction.
- `$ops-diagnose`: investigation path. Pull recent errors/logs/job history through MCP, identify likely root cause, then open the ops UI in the in-app browser only when screenshots, filtering, or interactive inspection would materially help.
- `$ops-ui`: secondary visual/interactive path. Start or locate the local ops UI, open it in Codex's in-app browser, inspect the relevant page, and use Browser Use to filter, click, retry, cancel, approve, or inspect job details only when MCP is insufficient or the user asks for UI inspection.
- `$wiki-ingest-ops`: ingestion operations path. Use MCP to start/check jobs; use the ops UI for job history, source health, failed records, logs, and retry/cancel flows.
- `$controller-status`: project/thread status path. Use controller MCP tools for project/thread summaries first; use ops UI if the user wants a dashboard view or active-run inspection.

MCP status surface:

- `ops_health()`
- `ops_jobs_list(status?, source?, limit?)`
- `ops_job_read(job_id)`
- `ops_job_retry(job_id | filter)`
- `ops_job_cancel(job_id)`
- `ops_sources_list(status?)`
- `ops_source_read(source_id)`
- `ops_recent_errors(limit?, severity?)`
- `ops_logs_query(service?, since?, level?, text?)`
- `ops_approvals_list(channel_context?)`
- `ops_render_card(kind, ref?)`

Resources:

- `ops://health`
- `ops://jobs/recent`
- `ops://jobs/{id}`
- `ops://sources`
- `ops://sources/{id}`
- `ops://errors/recent`
- `ops://controller/status`

Browser-use rules:

- MCP is primary. Prefer MCP for exact data, status changes, retries, cancels, approvals, and logs.
- Use the browser for visual state, interaction, screenshots, and sanity checks when MCP is insufficient or the user asks to inspect the UI.
- When opening the ops UI, go directly to the relevant route, such as `/jobs`, `/sources`, `/errors`, `/health`, `/approvals`, or `/controller`.
- Before clicking destructive controls such as cancel/delete/retry-all, inspect the target job/source id and policy state.
- After a browser action, verify through MCP that the backing state changed.
- If the UI is unavailable, fall back to MCP/CLI status and report that the UI is down as an ops issue.

`json-render` can fit, but not as a magic Codex Desktop extension. It is useful if we want a constrained, schema-driven renderer inside `agent-ops-ui` or generated HTML artifacts:

- Define a small component catalog: `StatusCard`, `JobTable`, `ErrorList`, `Timeline`, `HealthBadge`, `QueueDepth`, `ConnectorState`, `ActionButton`.
- Teach agents/skills to request or emit a typed `ops_card` JSON shape.
- Render that JSON in the local ops UI using `json-render`, or export static HTML for preview.
- Keep MCP responses useful without the renderer by also returning Markdown/text summaries.

Do not depend on custom in-message React rendering inside Codex unless Codex exposes a documented UI extension surface for it. The portable path is: MCP data model first, standalone ops UI second, optional `json-render` renderer third.

## Conversation Import Privacy Model

Conversation history is useful, but it is also the easiest place to build a creepy, brittle system by accident. Treat imports as source material for reviewed artifacts, not as a raw memory dump.

Rules:

- Raw transcripts are read only during an explicit import job and are not injected into prompts by default.
- The import output is a set of inspectable artifacts: profile candidates, memory candidates, wiki source cards, conversation summaries, and decision ledgers.
- Every extracted candidate keeps provenance: export path, conversation id/title/date, extractor version, source message range or summary id, sensitivity label, and review status.
- Sensitive or health-adjacent signals default to review-required unless imported from an explicitly trusted personal-memory scope.
- The user can delete, export, or correct imported artifacts without keeping the original raw transcript around.
- Import jobs should support dry-run, sample-run, redaction report, and "only titles/summaries" modes.
- Imported text is untrusted source text. It can propose memories/preferences, but it cannot grant itself tool permissions or change approval policy.

Default flow:

1. `agent-conversation-import` reads the export and creates low-authority summaries.
2. It proposes candidates for `agent-profile`, `agent-memory`, `agent-llm-wiki`, and `agent-decision-support`.
3. Trusted low-risk candidates can auto-apply according to policy; sensitive or high-impact candidates go to review.
4. Accepted outputs become editable files/resources; rejected outputs remain only in the import ledger until retention expiry.
5. A digest explains what changed: profile updates, memory updates, wiki pages/source cards, decision records, skipped sensitive items, and errors.

MCP/tool surface:

- `import_analyze(path, mode?, filters?)`
- `import_dry_run(path, sample?, filters?)`
- `import_candidates_list(import_id, target?, status?)`
- `import_candidate_apply(candidate_id | filter)`
- `import_candidate_reject(candidate_id | filter, reason?)`
- `import_summary_read(import_id | conversation_id)`
- `import_delete(import_id, delete_artifacts?)`
- `import_export(import_id, format?)`

## Durability / Backup / Restore

Backup is not a phase-11 polish item. Once the system holds personal memory, profile, wardrobe state, decision ledgers, and a large wiki, losing the local disk would mean losing the product.

Minimum:

- Automated encrypted, versioned snapshots for every local source of truth: SQLite stores, Markdown wiki files, profile documents, import/decision ledgers, controller registries, and service config.
- Restore must be tested, not assumed. `agent-backup` should include `backup_create`, `backup_status`, `backup_verify`, `backup_restore_plan`, and `backup_restore`.
- Snapshots should support local-only targets and optional R2/object-store targets. Edge backup is opt-in and client-side encrypted before upload.
- `doctor` and `ops_health()` should warn when the last successful backup or integrity check is stale.
- Deletes must propagate through backups via tombstones or retention windows; "forget/delete" cannot leave an easy-to-restore stale copy forever.

## Ingestion To Publish Trust Boundary

Wiki content is source material, not trusted instructions. The trust boundary must survive the full pipeline: web/X/email/newsletter -> wiki -> content draft -> social/blog/email publish.

Rules:

- Externally ingested sources remain untrusted even after they become wiki pages or source cards.
- Public posts, emails, repo writes, and broad sharing derived from untrusted source lineage require human review unless a specific policy says otherwise.
- Auto-publish is allowed only for content with no untrusted-source lineage or for explicitly preapproved low-risk templates.
- `agent-content` should show source lineage before publishing: wiki pages, source cards, retrieved URLs, model/prompt versions, and any risky instructions found in source text.
- Add a pre-publish injection/exfiltration scan for source-derived drafts.

## Model Policy

Do not carry over the Swift MLX lane into the first open-source extraction stack. It is valuable but adds model packaging, Metal, memory, and backend-complexity costs before the service boundary is proven.

Use a configurable hosted-model policy instead:

| Job | Default model policy | Notes |
|---|---|---|
| Wiki extraction / claim cards | Current best GPT model for structured extraction; as of 2026-06-19, start with `gpt-5.5` and medium reasoning | Optimize for JSON/structured outputs, source provenance, and consistency. |
| Cheap classification / dedupe / routing | Smaller/faster current GPT model or low reasoning setting | Keep this swappable; benchmark before over-optimizing. |
| Embeddings | OpenAI embedding v3 family, with `text-embedding-3-small` as the cheap default and `text-embedding-3-large` for higher-recall corpora | Store provider id, dimensions, and normalization policy in corpus metadata. |
| Deep Research full runs | Use the host agent's native web search first; in Codex, that means OpenAI web search + Codex subagents; in Claude, that means Claude web search + MCP | Add Brave/Perplexity only as optional extra providers. OpenAI deep-research models remain an option for standalone long-running research jobs. |
| Final synthesis / contradiction review | Current best GPT model with higher reasoning only when evals show it helps | Keep source citations and confidence labels in the output contract. |
| Channel formatting / notification rendering | Fast model or deterministic renderer first | Prefer deterministic formatting adapters where possible; do not spend a premium model on escaping Markdown. |

Rules:

- Keep model names in app config, not hard-coded in skills.
- Use evals/smoke tests per job before changing defaults.
- Record model, reasoning effort, prompt version, and extractor version in wiki job metadata.
- Add a task-effort classifier before model/tool selection. Inputs should include stakes, ambiguity, freshness/currentness, personalization need, emotional sensitivity, destructive/external-write risk, and whether durable memory/profile/wiki context is likely relevant.
- Do not default to cheap/low-reasoning paths for high-stakes, emotionally sensitive, multi-step, personalized, or source-sensitive work. Escalate model/reasoning/tool use automatically, or explain why escalation is unavailable.
- For personalized tasks, host adapters should consult `agent-profile`/`agent-memory`/`agent-llm-wiki` as appropriate before answering, then provide a small consultation receipt when it helps trust.
- Prefer hosted models until a real privacy/cost/latency need justifies adding local inference back.
- Do not mix embedding providers or dimensions inside one corpus without an explicit migration.

## Cost / Budget Policy

Always-on monitoring, wiki expansion, dream/reconcile jobs, deep research, and effort escalation can silently turn into recurring model spend. Cost control should be system-wide, not implemented one package at a time.

Requirements:

- Record provider, model, reasoning effort, input/output tokens, embedding units, tool/search calls, estimated cost, final cost, and job id for every model-backed job.
- Support daily/monthly budgets at global, package, source, and channel levels.
- Add a global kill switch and per-package pause switches surfaced in `agent-ops-ui`.
- Long jobs should expose cost estimates before they run when possible, especially deep research, corpus ingest, page expansion, and re-embedding.
- The effort router must respect budget ceilings. If a task deserves higher effort but the budget blocks escalation, the agent should say so and offer options.

MCP/status surface:

- `ops_costs_summary(period?, package?)`
- `ops_costs_jobs(filter?)`
- `ops_budget_read(scope?)`
- `ops_budget_update(scope, limit, policy?)`
- `ops_kill_switch(scope, enabled, reason?)`

## Search Provider Policy

Deep Research should use the host agent's native web search first.

This keeps the package portable:

- In Codex/OpenAI contexts, use OpenAI's Responses API web search for ordinary web-grounded answers, citations, and source lists.
- In Claude contexts, use Claude's native web search tool when available. Anthropic documents Claude web search as direct real-time web access with source citations.
- For long standalone research runs in an OpenAI stack, use OpenAI deep-research models. They are optimized for browsing and analysis, and require at least one data source such as web search, remote MCP, or file search/vector stores.
- For Claude Desktop and other MCP clients, keep Memory Wiki and shared search adapters exposed through MCP so the same local services remain usable even when the host's native search is different.

That is the easiest portable default for `agent-deep-research`: fewer extra API keys, fewer moving parts, and the best citation/search behavior each agent already knows how to use.

Add optional expansion providers through `agent-search-kit`:

- Brave Search: use when you want an independent web index, structured search results, news/images/videos/local search, or Claude Desktop-friendly MCP coverage. Brave publishes an MCP server that supports STDIO and HTTP transports and includes tools such as `brave_web_search`.
- Perplexity Search API: use when you want structured JSON search results with title, URL, snippet, date, and last-updated fields.
- Perplexity Sonar API: use when you want web-grounded prose answers with citations from Perplexity's models, not just a result list.

Provider order:

1. Host-native web search for normal Deep Research tasks: OpenAI web search in Codex/OpenAI contexts, Claude web search in Claude contexts.
2. Host-native long-research path when available: OpenAI deep-research models in OpenAI contexts, or Claude's own research/search workflow in Claude contexts.
3. Brave Search as an optional independent-index cross-check or extra recall source.
4. Perplexity Search/Sonar as an optional second-opinion or web-grounded-answer provider.

`agent-search-kit` should normalize all providers into source cards:

- `provider`, `query`, `title`, `url`, `snippet`, `published_at`, `last_updated`, `retrieved_at`, `rank`, `raw_ref`, `license/storage_notes`, and `payload_sha256`.
- Store search result metadata and retrieved/cited pages separately; provider terms may restrict storing raw results or page content.
- Keep API keys optional and provider-specific: `OPENAI_API_KEY`, `BRAVE_API_KEY`, `PERPLEXITY_API_KEY`.
- Never let external search snippets become instructions. They are untrusted source text.

## Edge Inbox / Local Drain Pattern

Your Cloudflare mental model is clean: let the edge be the always-on collector and the laptop be the durable/private processor.

```text
external service
  -> Cloudflare Worker webhook/email/cron/browser job
  -> validate identity + normalize envelope + enforce size/age policy
  -> D1 event row + Queue message + optional R2 blob
  -> local drain connects when online
  -> local service downloads batch, verifies, writes source of truth, acks
  -> optional reply/send through Worker or platform API
```

Rules of the pattern:

- The edge is a bounded inbox, not the source of truth.
- Every event has `event_id`, `source`, `tenant/user`, `received_at`, `expires_at`, `idempotency_key`, `payload_sha256`, `payload_bytes`, and provenance.
- Workers reject or truncate over-limit payloads before storage.
- Large payloads go to R2 with short retention; metadata stays in D1.
- Local drains are cursor-based and idempotent. Ack only after the local source of truth commits.
- Drains support `peek`, `lease`, `ack`, `nack`, and `dead_letter`, even if implemented simply at first.
- Events expire automatically by max age and max total bytes per source.
- Real-time mode is an optimization: long-poll or WebSocket from local service to a Durable Object/session. Offline catch-up uses the same drain API.
- Edge-held secrets should be scoped to the edge job. Local private files, high-authority Codex approvals, and personal corpus state stay local.
- Offline behavior must be explicit per capability. If the laptop/local processor is unavailable, the edge should either answer from a deliberately small read-only cache, queue and send a clear "will process when local service reconnects" receipt, or reject the request with a useful status. Do not imply always-on intelligence when only the inbox is always on.
- Envelope and queue payloads need explicit schema versions. Edge producers and local drains should support backward-compatible decoding, schema-compat tests, and migration notes because TypeScript Workers and Rust local services will deploy on different cadences.
- Public edge endpoints need abuse controls: payload caps, sender validation, rate limits, replay protection, queue-depth ceilings, dead-letter alerts, and source-level pause switches.
- Local daemons need heartbeat/liveness records. `ops_health()` and optional edge status mirrors should detect missed heartbeats and notify through configured channels.

For the wiki ingest pipeline, the edge can capture URLs, email submissions, Telegram links, RSS/webhook events, and simple rendered snapshots. It should not try to own the whole compiled wiki. The local wiki service should decide what becomes durable knowledge, run heavier extraction, apply dedupe/contradiction logic, and expose the cited result through MCP.

## Reverse Hermes / OpenClaw Model

The useful mental model is "agent added to Codex" instead of "Codex hidden inside another agent shell."

OpenClaw and Hermes both provide strong lessons, but their ownership boundary is inverted:

- OpenClaw wraps agent runtimes inside its own gateway. Its Codex harness runs embedded Codex turns through Codex app-server while OpenClaw keeps ownership of chat channels, session files, tools, approvals, media delivery, and the visible transcript mirror.
- Hermes is a broad always-on agent/gateway. It emphasizes messaging platforms, scheduled automations, memory, skills, terminal backends, and a closed learning loop that improves skills and memory from experience.
- The proposed `agent-*` family should leave Codex as the human-facing agent/runtime and add reachability, memory, event capture, and external delivery around it.

Low-hanging fruit to borrow:

- Channel vocabulary: model each integration as `platform event -> inbound context -> agent turn -> delivery receipt`, not as ad hoc bot glue.
- Core/app boundary: shared runtime owns queueing, idempotency, auth validation, retries, receipts, and generic MCP shapes; `agent-channel-kit` owns common channel safety/formatting/receipt contracts; each concrete app owns platform quirks such as Telegram threads, Discord embeds, Slack Block Kit, Gmail loop prevention, X rate limits, or email MIME parsing.
- Controller boundary: `agent-controller` owns project/thread discovery, status, summaries, natural-language routing, and creation/resume operations. Channel packages should call the controller instead of duplicating workspace routing logic.
- Explicit send outcomes: `sent`, `suppressed`, `partial_failed`, `failed`, with receipts stored durably.
- Metadata-first plugins: list capabilities, required secrets, remote endpoints, scopes, and health checks without loading the whole runtime.
- Doctor commands: every app should have `doctor`/`self-test` for OAuth callback, webhook reachability, queue drain, local Codex connection, and tool registration.
- Gateway profile supervision: local services need launchd/systemd recipes and status, but they should remain optional sidecars, not a replacement Codex runtime.
- Skills as operational memory: package runbooks, triage procedures, and platform-specific constraints as Codex skills next to MCP tools.
- Setup/migration importers: `agent-llm-wiki` can import current codex-swift wiki state; `agent-telegram` can import bot/thread mappings; `agent-x` can import OAuth/cursor state after explicit confirmation.
- Research as orchestration: keep broad investigations as disciplined multi-agent workflows instead of building a second research runtime. Codex already knows how to spawn focused subagents, wait for them, and consolidate their summaries.

The guardrail: do not recreate Hermes/OpenClaw as another top-level agent. Use their gateway discipline, but keep Codex in charge of interactive reasoning and approvals.

## Reference Agent Gap Analysis

The target product is bigger than "Codex with memory". It is Codex as the fast-moving shell for coding, research, content production, personal context, proactive monitoring, and channel reach.

User workflow picture:

- Coding stays centered in Codex because Codex/Codex App is improving faster than a side project shell can.
- Claude Desktop/mobile remains useful for personal/social conversations, so shared MCP services must work there too.
- Work output includes code, websites, blog posts, trend reports, decks, occasional video, social posts, and research.
- Monitoring should cover X, blogs, GitHub, papers, newsletters, Google Workspace, and source feeds, then build the wiki and notify through Telegram/email.
- Personal services such as wardrobe planning should live in the same agent service family, not as isolated one-offs.

Comparison:

| Reference | What it does well | What we should borrow | What we should avoid |
|---|---|---|---|
| OpenClaw | Plugin/runtime boundaries, channel ingress/egress, Codex harness, memory/wiki split, active memory, permission requests, message presentation, browser/computer-use plugins | Channel-owned adapters, delivery receipts, explicit plugin capabilities, memory/wiki separation, prompt-injection boundaries before content reaches the agent | Replacing Codex as the primary shell; generic push abstractions |
| Hermes Agent | Messaging gateway across platforms, scheduled automations, self-improving skills, memory nudges, provider/tool gateway, terminal backends, migration/doctor flows | Closed learning loop, doctor/setup commands, cron with delivery, skill improvement runbooks, platform gateway status, interrupt/redirect UX, cloud/laptop split | Building another universal agent loop when Codex already owns the main loop |
| Perplexity Comet / Computer | Browser-native assistant, cross-tab synthesis, Gmail/Calendar/browser actions, activity/history search, mobile/browser continuity | Browser workflow package, separate browser profile for agent tasks, action ledger, tab/page summarization, "operate where the work already happens" fallback when APIs are missing | Letting browser automation bypass scoped API permissions, confirmations, or origin trust checks |
| NVIDIA Project G-Assist | Local, low-latency assistant, plugin model, local telemetry/control, voice/text commands, single-PC utility | Single-binary local daemons, plugin manifests, local status/metrics, constrained tool APIs, optional voice later | Hardware-specific SLM complexity before hosted models/local service boundaries are proven |
| NVIDIA AI-Q / Deep Research blueprints | Owned/inspectable research agents, citations, shallow vs deep routing, eval harnesses, enterprise data connectors | Research workflow evals, intent routing, citation discipline, quality benchmarks, source-backed reports | Heavy enterprise stack or GPU assumptions for the local laptop path |
| GBrain | Continuous brain: ingestion, graph-aware synthesis, consolidation, digests, waking up to a smarter knowledge base | Daemonized background enrichment, gap analysis, digest loop, graph/topic relations, wiki-as-operating-memory | Postgres-first heaviness; keep SQLite local-first |
| Karpathy/LLM Wiki / agentwiki | Compiled knowledge, persistent pages, source cards, daily feed monitoring, cross-linked wiki, contradictions/staleness | Watch -> ingest -> compile -> digest, page expansion, source-backed synthesis, article/topic pages that improve over time | Treating wiki as hidden prompt context instead of inspectable files and cited tools |

Conversation export signal:

I also inspected the Claude export at `/Users/chabotc/Projects/claude-history-export/conversations.json` in aggregate. It contains 492 conversations and 5,616 messages. The useful conclusion is not any one private transcript; it is the repeated product shape:

- Personal context is core product material. Durable preferences, taste, decision criteria, health-adjacent constraints, prior choices, and "how to talk/work with me" expectations are needed across agents.
- Wardrobe and outfit planning is a flagship personal-domain use case, not a toy demo. It combines inventory, fit/sizing, weather, calendar/formality, comfort/health constraints, aesthetic taste, budget/purchase decisions, and memory of what worked.
- Decision support recurs constantly: compare options, reason from constraints, recommend, track rationale, and make future follow-ups aware of earlier choices.
- Research should be practical and source-backed: concise reports, grounded comparisons, next actions, contradiction checks, and reusable wiki/source-card output.
- Coding-agent work should preserve the codex-swift standards: root-cause debugging, tests, rendered inspection for UI, and no papering over failures.
- Claude Desktop/mobile remain important front doors. Codex can be the work shell, but the services should not assume Codex is the only conversation surface.

Product implication: add an inspectable personal profile/preference layer next to `agent-memory`. Mem0-style memory is good for compact facts and evolving personal memory, but durable operating preferences such as writing style, communication register, sizing defaults, wardrobe rules, approval friction, and recommendation criteria should be visible documents/resources the user can audit and edit.

Another important product implication: perceived under-effort is not a small UX flaw. When an agent skips memory, ignores available tools, uses too-low reasoning effort, guesses instead of checking, or stalls without saying so, it reads as inconsiderate and time-wasting. For this user profile it can also trigger rejection sensitivity. The product should treat "competence respect" as a first-class operating contract: do the obvious checks, spend enough cognition for the stakes, and explain when a source/tool/model path was unavailable.

Missing or under-specified pieces now added to the target:

1. `agent-profile`: portable personal profile and preference store.

   - Stores durable preferences as inspectable structured documents/resources, not hidden prompt state.
   - Examples: communication register, output preferences, writing/style profiles, recurring constraints, style/taste rules, sizes/settings, approval policy preferences, "ask when ambiguous" rules, and decision criteria.
   - Includes competence expectations: check relevant memory/profile/wiki first, use tools for factual/current/personalized answers, do not default to low-effort reasoning on high-stakes or emotionally sensitive tasks, and say what was consulted.
   - Tools: `profile_read`, `profile_search`, `profile_update`, `profile_review_pending`, `profile_export`, `profile_delete`.
   - Host adapters: Codex and Claude should load only the relevant profile slices for the task, with provenance and sensitivity labels.
   - Boundary: `agent-memory` remembers facts and learns from conversation; `agent-profile` is the explicit, curated operating manual.

2. `agent-conversation-import`: consentful history import.

   - Reads Claude/Codex exports and produces reviewed summaries, profile candidates, source cards, and wiki pages.
   - Never imports raw transcripts into high-authority prompt context by default.
   - Redacts or quarantines sensitive content; uses a review queue for profile/memory changes.
   - Records import ledger entries: source export, conversation id/title/date, summary version, extracted candidates, accepted/rejected decisions, and delete/export handles.

3. `agent-decision-support`: remembered reasoning, not one-off advice.

   - Store option sets, comparison tables, recommendation rationales, rejected alternatives, sizing/fit decisions, purchase/return state, and source snapshots.
   - Natural follow-ups: "why did we pick this?", "what did we reject last time?", "does this fit my rules?", "what changed since then?"
   - Shared by `agent-garderobe`, `agent-content`, `agent-workspace-context`, and research workflows.

4. `agent-content`: content production and publishing support.

   - Import `/Users/chabotc/Desktop/STYLE.md` as the canonical style profile.
   - Store style as an inspectable wiki/profile document, not personal mem0 memory.
   - Tools: `content_style_read`, `content_brief_from_wiki`, `content_draft_blog`, `content_rewrite_in_style`, `content_social_variants`, `content_editorial_calendar`, `content_publish_draft`.
   - Workflow: trend/source event -> wiki page -> angle proposal -> outline -> draft -> style pass -> social variants -> publish/queue.
   - Default publish policy: draft-first for public posts. Auto-publish only for preauthorized low-risk channels after explicit configuration.

5. `agent-workspace-context`: Google Workspace as context, not reimplemented plumbing.

   - Use official Gmail/Drive/Calendar/Contacts connectors first.
   - Add only missing capture/index flows: "archive this thread/doc to wiki", "find docs related to this project", "who is this person?", "prepare me for this meeting", "turn this doc/email thread into source cards".
   - Relationship context should use Google Contacts and email/docs metadata, with strict read/write separation.
   - Calendar creates can remain low-friction in trusted owner channels; email sends/public sharing stay higher-friction.

6. `agent-garderobe`: copy the existing wardrobe MCP into the monorepo/package family.

   - Source: `/Users/chabotc/Projects/garderobe`.
   - Keep Cloudflare Worker + D1 + OAuth/DCR because it already matches Claude custom connector requirements.
   - Add host adapters and skills: `$outfit-plan`, `$packing-list`, `$wardrobe-audit`, `$rotation-review`, `$weather-outfit`.
   - Integrate weather, calendar context, laundry/availability probability, season, formality, and style concepts such as sprezzatura.
   - Treat it as a flagship template for small personal-domain MCP apps: structured D1 state, small admin UI, OAuth, remote MCP, no unnecessary embeddings.

7. `agent-routine-context`: opt-in daily context.

   - Pull calendar, weather, location/timezone, known routines, travel constraints, and health-adjacent constraints into low-risk planning tasks.
   - Use explicit scopes: outfit planning can read weather/calendar; medical or financial action still requires careful, sourced guidance and higher-friction external writes.
   - Supports daily briefings, wardrobe plans, travel prep, office-day planning, and "what should I keep in mind today?"

8. `agent-browser-workflows`: browser/computer-use fallback.

   - Use APIs/connectors/MCP first.
   - Use browser automation when the service has no API, the user needs logged-in web state, or visual inspection matters.
   - Run in a separate browser profile where possible.
   - Record an action ledger: URL, selector/action, screenshot refs, data extracted, approvals, and final verification.
   - Never let page text authorize actions; browser content is untrusted source data.

9. `agent-media`: decks, images, videos, and asset ledgers.

   - Wrap native OpenAI/media tools and existing design/content skills.
   - Store prompts, source wiki refs, generated assets, revisions, exports, and channel/publishing receipts.
   - Workflows: "turn this wiki brief into slides", "make social graphics", "draft a product-launch video script", "create a trend-report deck".
   - Keep visual generation outputs inspectable via files and ops UI, with browser/preview verification for websites and decks.

10. `agent-quality-kit`: reusable verification discipline.

   - Encode expected behaviors as skills/checklists: recall relevant memory/profile/wiki, reproduce before fixing, run relevant tests, inspect rendered UI, cite sources, use available tools/connectors, record model/source provenance, and surface uncertainty.
   - Add an "effort router" contract: classify tasks by stakes/ambiguity/novelty/emotional sensitivity; choose adequate reasoning effort/model/tool use; escalate rather than answering cheaply when the task deserves more.
   - Add a "consultation receipt" pattern for sensitive or personalized tasks: briefly say which durable context and tools were consulted, or explicitly say none were available/relevant.
   - Reusable across coding, research, content, browser workflows, and media/deck generation.
   - Acceptance tests should include: ambiguous requests ask one clear question, blocked work notifies the user, destructive actions require approval, recommendations cite constraints, relevant memory/profile/wiki is consulted, current facts use tools/search, high-stakes tasks do not use low-effort defaults, and UI changes are visually verified.

11. `agent-signal` as an internal module of `agent-llm-wiki`.

   - Watch X/GitHub/blogs/arXiv/newsletters.
   - Score interestingness.
   - Expand high-value pages.
   - Send digest items through channel packages.
   - Feed future content production.

12. User-facing operating model.

   - Morning/afternoon digest: "what changed, why interesting, what it means for current projects/content".
   - Project brief: "what happened in the Codex de-porting project?"
   - Content queue: "three post ideas from this week's signals, in my voice."
   - Research request: source-backed deep dive with wiki writeback.
   - Social/report request: synthesize X/GitHub/blog/paper trends into a publishable draft.
   - Wardrobe request: weather/calendar/rotation-aware outfit plan.

Security and reliability additions from the comparison:

- Browser actions need their own approval and provenance layer; they are not a shortcut around API permissions.
- Public posting, email sending, repo destructive ops, and broad sharing require explicit configured policies even for trusted owner channels.
- Every proactive notification should be backed by a candidate id, source ids, digest id, and delivery receipt.
- Every background job should have timeout, budget, cancellation, retry, and "why did this run?" metadata.
- Every package should have `doctor`, `status`, replay fixtures, and fake-provider tests.
- Content generation needs source/style provenance: which wiki pages, which sources, which style profile, which model, and what changed during editing.
- Personal memory, wiki knowledge, workspace docs, and public content drafts must remain separate stores with explicit cross-links.

Natural-fit roadmap additions:

| Need | Package | Priority | Why |
|---|---|---:|---|
| Personal memory | `agent-memory` | P0 | Gives all agents durable personal context and low-friction capture. |
| Explicit personal profile | `agent-profile` | P0 | Keeps tone, preferences, recurring constraints, decision criteria, and support expectations inspectable instead of hidden. |
| Knowledge/wiki/signal | `agent-llm-wiki` | P0 | Central substrate for research, content, monitoring, and trend reports. |
| Project/thread controller | `agent-controller` | P0 | Lets Telegram/mobile ask about active Codex work and route follow-ups. |
| Safe history migration | `agent-conversation-import` | P0 | Converts Claude/Codex exports into reviewed profile, memory, wiki, and decision artifacts without raw transcript sludge. |
| Quality discipline | `agent-quality-kit` | P0 | Preserves root-cause debugging, tests, citations, visual inspection, and explicit uncertainty across packages. |
| Content/style/publishing | `agent-content` | P1 | Turns wiki/signal into blog posts, social posts, reports, slides, and drafts in the user's voice. |
| Decision support | `agent-decision-support` | P1 | Makes recommendations reusable by remembering constraints, alternatives, sources, and why a choice was made. |
| Channels | `agent-telegram`, `agent-mail`, `agent-channel-kit` | P1 | Makes the system reachable and proactive without generic push. |
| Workspace context | `agent-workspace-context` | P1 | Connects Google Docs, Gmail, Calendar, Contacts, and Drive to projects and wiki. |
| Wardrobe | `agent-garderobe` | P1 | Existing Cloudflare MCP is ready to copy into the family and is the flagship personal-domain app. |
| Routine/daily context | `agent-routine-context` | P1 | Lets calendar/weather/location/routine context support outfits, daily briefs, travel, and planning. |
| Browser workflows | `agent-browser-workflows` | P2 | Covers no-API workflows and visual tasks, but must be sandboxed and logged. |
| Media/decks/video | `agent-media` | P2 | Useful for project sites, reports, slides, social, and video, but depends on content/wiki foundation. |
| Mobile/voice front door | `agent-voice-mobile` | P2 | Useful for capture and notifications, probably built first on concrete channel packages rather than a new shell. |

## De-Portability Matrix

| codex-swift capability | Best regular-Codex home | Portability | Notes |
|---|---|---:|---|
| Personal mem0 memory | `agent-memory` using `mem0-rs` + SQLite + MCP + review skills | High | For durable personal facts/preferences such as pets, ADHD context, medications, communication preferences, and recurring task defaults. Keep separate from wiki corpora. |
| Conversation-derived profile/preferences | `agent-profile` + `agent-conversation-import` | High | Claude/Codex exports should become reviewed profile/memory/wiki/decision artifacts, not raw hidden context. |
| Decision/rationale history | `agent-decision-support` | High | Wardrobe, purchases, research choices, content angles, and technical options need reusable "why this, not that" records. |
| Memory Wiki query tools (`wiki_brief`, `wiki_compare`, `wiki_angle`, `wiki_pmfit`, hybrid search) | MCP server + plugin + skills | High | Tool boundary maps cleanly to MCP. Keep source-backed knowledge storage in sidecar. |
| Wiki ingest/watch/research/librarian/audit | Sidecar daemon or Cloudflare Worker front door + MCP tools + automations | High | Long jobs should not be Codex core. Use Cloudflare for HTTPS capture/webhooks; local daemon for heavy indexing. |
| X bookmarks/likes/following import | X MCP sidecar or Cloudflare Worker + plugin skill | High | Official X API supports bookmarks via user OAuth2 PKCE with `bookmark.read`, `tweet.read`, `users.read`. Cloudflare helps with stable HTTPS OAuth callbacks. |
| X twice-daily monitor | Sidecar daemon, Cloudflare cron, or Codex automation | High | The monitor needs durable cursors, spend caps, Telegram delivery. Cloudflare cron is a good always-on option if API credentials are remote-safe. |
| Telegram inbound channel | Sidecar bridge using Telegram Bot API + Codex SDK/app-server | Medium-high | A skill/MCP server cannot receive Telegram messages by itself. Use a daemon to receive updates and call Codex. |
| Project/thread meta-controller | `agent-controller` local service + MCP + Codex SDK/app-server | High | Needed for channel commands like "how is the de-porting project going?", "and the video project?", and "create a new project". |
| Gmail channel | Prefer official Gmail plugin for interactive use; custom sidecar if inbound/reply channel is required | Medium | Gmail push notifications exist, but require backend Pub/Sub plumbing. Polling is simpler for MVP. |
| Google Workspace tools | Prefer built-in/curated Gmail + Drive plugins; custom MCP only for missing unified operations | High | Regular Codex already has plugin/app paths. Avoid reimplementing all of Workspace unless needed. |
| Channel inbound/outbound contract | `agent-channel-kit` + concrete channel apps such as Telegram/Discord/Slack/mail | High | Replace vague push with OpenClaw-style channel boundaries: inbound, outbound, formatting, prompt-injection protection, receipts, and platform identity. |
| Cron scheduler | Codex automations | High | Replace most of this directly. Keep daemon scheduling only for non-Codex events like Telegram listener health. |
| Media generation | MCP tool or native OpenAI image tooling, packaged as plugin | Medium | Existing codex-swift ledger is useful but not core-critical. |
| Deep research / research workflows | Skill + optional custom subagents + wiki/search MCP | High | Regular Codex already supports explicit subagent workflows and custom agents. Port the discipline, not the engine. |
| Memory Wiki browsing/editing | Markdown files on disk + Obsidian | High | Keep durable wiki pages as inspectable files. Obsidian can handle browsing and manual editing. |
| Ops UI for ingest/status/logs/health | `agent-ops-ui` MCP status tools + standalone local web app | High | MCP is the primary agent interface. Use in-app browser/browser-use only for visual inspection, screenshots, and interactive UI workflows. |
| Multi-process supervisor, broker, sandbox, app-server clone | Drop | Low value to port | Regular Codex owns this. Keep only lessons, not code. |
| Swift workflow engine | Mostly drop / replace with skills + Codex subagents/automations | Medium-low | The deterministic JSC engine is cool, but maintaining it outside Codex is its own project. |

## Recommended Target Architecture

Build a service package family, backed by local drains and optional Cloudflare edge inboxes:

```text
regular Codex
  |
  +-- agent-memory plugin
  |     +-- personal memory MCP -> mem0-rs/local memory service
  |     +-- memory review/update skills
  |
  +-- agent-llm-wiki plugin
  |     +-- wiki MCP -> local wiki service
  |     +-- wiki research/ingest skills
  |
  +-- agent-x plugin
  |     +-- X MCP -> local or Cloudflare-backed X service
  |     +-- monitor/report skills
  |
  +-- agent-telegram plugin
  |     +-- status/send MCP
  |     +-- inbound bridge uses controller + Codex SDK/app-server
  |
  +-- agent-controller plugin
  |     +-- project/thread registry MCP
  |     +-- active-run status + summaries
  |     +-- natural-language project/thread router
  |
  +-- agent-ops-ui
  |     +-- local web dashboard for ingest/jobs/logs/health
  |     +-- MCP status resources + structured cards
  |
  +-- agent-channel-kit shared package
  |     +-- inbound/outbound message contract
  |     +-- prompt-injection and formatting rules
  |
  +-- agent-mail / agent-discord / agent-slack / other channels
  |
  +-- agent-deep-research plugin
        +-- deep-research skills
        +-- optional custom agents: scout, skeptic, synthesizer
        +-- uses wiki/search/source MCP tools

sidecars
  |
  +-- memd: mem0-rs personal memory, SQLite, mem0-style add/search/history
  +-- wikid: SQLite + FTS/vector storage, ingest, wiki compile, corpus search, UI API
  +-- telegramd: Telegram updates -> Codex SDK/app-server -> Telegram replies
  +-- xmonitord: cursors, X polling, spend caps, wiki writes, channel delivery

optional Cloudflare front doors
  |
  +-- app Workers: OAuth/DCR, webhooks, email handlers, remote MCP
  +-- D1/KV/DO/Queues/R2: bounded inboxes, grants, cursors, blobs
  +-- cron triggers/browser jobs: scheduled monitors and capture/render endpoints
```

Packaging shape:

- Each `agent-*` package: installable MCP server/service with app-specific docs plus host adapters for Codex skills/plugins and Claude MCP configuration.
- Optional shared `agent-runtime`: only after duplicated drain/envelope/auth code appears in at least two apps.
- Runtime standard: Rust first for local services. Use Rust for resident local daemons, MCP servers, controller services, CLIs, queue drains, and ops UI backends. Keep TypeScript as the default for Cloudflare Workers and browser/frontend code unless a package has a clear reason to use `workers-rs`.
- Storage: SQLite first. Use SQLite + FTS plus sqlite-vec/sqlite-vss or an embedded vector store for the default local path. Postgres/pgvector is too heavy for the first open-source stack and should stay an optional future migration target, not the primary install path.
- Cloudflare storage: use D1 for small relational MCP domains and cursors, KV only for lightweight metadata, Durable Objects when per-session state/coordination/stateful MCP sessions matter, Queues for edge inbox delivery, and R2 for temporary blobs/export artifacts.
- UI: keep wiki content as Markdown files for Obsidian. Build `agent-ops-ui` for operational state, not as a Codex app-server fork. It can call `wikid`, `memd`, and controller HTTP APIs.
- Automations: create Codex automations for scheduled summaries and checks; use launchd/systemd only for sidecar liveness.

Local language policy:

- Default to Rust for local services. The product goal is fast, low-latency, memory-friendly, single-binary daemons that can sit on a laptop without dragging in Node/Bun/npm runtime weight.
- Use the official Rust MCP SDK (`rmcp`) for local MCP servers where it is mature enough, with a thin protocol compatibility test suite against Codex and Claude Desktop.
- Use Rust crates for the common runtime: `agent-envelope`, `agent-channel`, `agent-search`, `agent-edge-protocol`, `agent-local`, `agent-mcp`, and `agent-config`. Generate JSON Schema/OpenAPI from Rust types for Cloudflare Workers and browser UI clients.
- Standardize local service crates on Tokio, Axum for HTTP/admin APIs, `rmcp` for MCP, `clap` for CLIs, `tracing`/OpenTelemetry for logs/spans, `serde`/`schemars` for schemas, `sqlx` or `rusqlite` for SQLite depending on async needs, and `notify` for file watching.
- Package local daemons as single binaries with subcommands such as `agent-memory serve`, `agent-memory doctor`, `agent-llm-wiki serve`, `agent-controller serve`, and installable launchd/systemd templates.
- Keep TypeScript where it genuinely fits the platform: Cloudflare Workers, existing Workers OAuth/DCR examples, small browser UIs, and generated clients. Cloudflare does support Rust via `workers-rs`, so Rust Workers are an option, but the first edge implementations can stay TypeScript if that keeps OAuth, Durable Objects, Queues, and dashboard deployment simpler.
- Avoid Python and Node/Bun as resident local service runtimes. They are fine for one-off importers, tests, frontend build tooling, or Cloudflare/browser code, but not for always-on local daemons.

Local Rust feasibility by package:

| Package | Rust local fit | Notes |
|---|---:|---|
| `agent-memory` | Excellent | `mem0-rs` is already the right core: single binary, SQLite/history, REST, procedural memory, low RSS. |
| `agent-llm-wiki` / `wikid` | High | Rust is strong for file watching, bounded fetch/decode, SQLite/FTS/vector indexes, job ledgers, and large corpus processing. Use hosted models over HTTP. |
| `agent-controller` | High | Mostly SQLite state, Codex app-server/SDK protocol calls, summaries, routing, and MCP. If official Codex SDK support is JS-only, implement the narrow app-server HTTP/SSE/JSON-RPC client in Rust or shell out only as a temporary adapter. |
| `agent-telegram` / channel daemons | High | Telegram webhooks/long-poll, formatting, queues, owner policy, and receipts map cleanly to Rust HTTP clients and SQLite. |
| `agent-x` | High | OAuth state, cursors, polling, rate/spend ledgers, and wiki writes are good Rust sidecar work. Cloudflare may still own public OAuth callbacks. |
| `agent-mail` | Medium-high | MIME parsing, loop prevention, attachment staging, and channel delivery are fine in Rust; Gmail/Google APIs are more ergonomic through generated REST clients or a small HTTP layer than through a big dynamic discovery client. |
| `agent-ops-ui` backend | High | Axum can serve the local API, static assets, and MCP-backed status views. The browser frontend may still be TypeScript/React or a generated/static UI. |
| Cloudflare edge components | Mixed | TypeScript remains the default for Workers because Cloudflare Agents/OAuth examples and ecosystem are strongest there. Rust Workers via `workers-rs` are viable for smaller endpoints once the bindings needed by the package are confirmed. |

## Capability Plans

### 1. Memory / Knowledge Boundary

Decision: keep personal memory and knowledge corpora separate.

Personal memory is mem0-shaped. It stores durable facts about the user, preferences, recurring context, and life details: "my cat is called Ophelia", "I have ADHD", "I use these medications", "I prefer minimal-friction approvals", "I dislike hidden background stalls". This should live in `agent-memory`, backed by `mem0-rs` and SQLite.

LLM Wiki is knowledge-shaped. It stores and searches source-backed corpora: 9000 pages on AI agents, 100 pages on developer relations, research briefs, source cards, contradiction notes, and imported documents. This should live in `agent-llm-wiki`, backed by its own SQLite corpus/index and Markdown wiki pages.

Rules:

- Do not put large research corpora into personal memory.
- Do not treat personal preferences as wiki knowledge unless the user deliberately writes a profile/document page.
- Retrieval should be separate by default: `memory_search` answers "what should the agent remember about me?", while `wiki_search` answers "what does this source-backed corpus say?"
- A task may use both lanes, but the response should keep their provenance distinct.
- Personal memory can be concise and lossy; wiki knowledge must be source-backed, cited, stale-aware, and inspectable.
- Store them as separate SQLite databases/corpora first. Cross-links are metadata, not shared storage.

### 2. LLM Wiki / Knowledge Corpus

Recommendation: port this as the first large knowledge package, `agent-llm-wiki`, with `codex-memory-mcp` as the local MCP server inside the app package, not as a Codex fork.

Why:

- MCP is expressly for exposing tools and context to models.
- The existing `MemoryMCP` tool shapes are already close to an MCP tool suite.
- The LLM Wiki is host-global, source-backed, corpus-oriented, and useful to other agents, not just Codex.
- Existing projects validate the idea: `llm-wiki` already ships as a Codex plugin/skill-style methodology for knowledge corpora.

MVP tools:

- `wiki_search(query, corpus?, k?)`
- `wiki_read(page_id | uri)`
- `wiki_brief(topic, depth?)`
- `wiki_compare(a, b)`
- `wiki_angle(topic, audience?)`
- `wiki_pmfit(product_or_idea)`
- `wiki_ingest(source)` as an async job launcher
- `wiki_job_status(job_id)`

MVP resources:

- `wiki://corpora`
- `wiki://corpus/{id}/index`
- `wiki://page/{id}`

MVP skills:

- `$wiki-research`: plan, query, cite, gap-check, synthesize.
- `$wiki-ingest`: add sources, run ingest, verify citations, compile pages.
- `$wiki-review`: inspect/review wiki claims, source cards, contradictions, and stale pages.

Implementation notes:

- Keep recall as tool-mediated/cited results, not invisible high-authority prompt injection.
- If auto-injection is desired, use a `SessionStart` or `UserPromptSubmit` hook that writes a small retrieval summary into the prompt path only after explicit opt-in.
- Preserve the boundary between personal memory and knowledge corpora. Built-in Codex Memories and `agent-memory` can remain enabled, but they are separate preference/identity layers, not a replacement for cited wiki knowledge.
- Start by wrapping the existing `codex-memory` CLI or `MemoryMCP` query/ingest logic, then rewrite the wiki daemon as a Rust local service once the MCP contract and storage model are proven.
- Keep SQLite as the default wiki store. Do not require Postgres for local wiki installs.

Existing project:

- [nvk/llm-wiki](https://github.com/nvk/llm-wiki) explicitly describes a portable agent wiki methodology with Codex plugin support and commands such as `/wiki:ingest`.

### 3. LLM Wiki Freshness / Librarian / Interestingness

Recommendation: make `agent-llm-wiki` a living knowledge system, not just an ingest/search tool. It should watch curated sources, detect meaningful changes cheaply, enrich wiki pages through a librarian loop, score "interestingness", and send concise digests through concrete channel packages such as `agent-telegram` and `agent-mail`.

This is the portable shape of the codex-swift intent in `llm-wiki.md`: watch feeds/repos/X/accounts, ingest only changed material, compile pages, run librarian scans, and deliver digests.

Core pipeline:

```text
watch source
  -> poll/webhook/edge capture
  -> deterministic change gate
  -> candidate event ledger
  -> cheap relevance + interestingness classifier
  -> wiki novelty check / dedupe against existing corpus
  -> source expansion plan
  -> create or expand wiki page with multi-source evidence
  -> librarian health/staleness pass
  -> digest item
  -> channel delivery intent
  -> Telegram/email formatter + delivery receipt
```

Source types:

- `github-owner` / `github-repo`: OpenAI, Anthropic-adjacent orgs, Cloudflare, Vercel, Simon Willison/simonw, people/orgs from the AI-agent watch list. Watch for new repos, releases, README changes, topic/description changes, and materially new examples. Stars/forks alone update metrics but should not trigger a synthesis or alert.
- `feed`: RSS/Atom feeds for OpenAI, Anthropic, Simon Willison, Latent Space, Interconnects, Lilian Weng, news.smol.ai, vendor blogs, and personal blogs.
- `x-account` / `x-following`: tweets/posts from selected accounts or follows, plus bookmarks/likes/imported collections where API access allows it.
- `arxiv-query`: topic or author watches for AI agents, coding agents, memory, retrieval, evals, and developer tooling.
- `email-newsletter`: inbound newsletters routed through `agent-mail` or Cloudflare Email Workers into the same candidate ledger.
- `manual-seed`: "watch this repo/blog/person/topic" commands from Codex/Claude/Telegram.

State model:

- `watch_source`: source identity, kind, locator, owner/project tags, cadence, trust level, enabled/error state, next due time, channel delivery policy.
- `watch_cursor`: per-source cursor, ETag, Last-Modified, GitHub page ETags, newest seen ids, content hashes, rate-limit state.
- `watch_event`: normalized changed/new item with canonical URL, source timestamp, retrieved timestamp, content hash, raw pointer, dedupe key, and provenance.
- `interesting_candidate`: model/deterministic scores, novelty result, reason labels, selected audience/channel, suppress/digest/immediate decision.
- `digest_item`: final human-facing summary with title, "what it is", "why interesting", links, confidence, wiki page refs, and delivery state.
- `delivery_receipt`: channel, target, rendered body hash, platform message id, sent/suppressed/failed, retry metadata.

Change-detection best practices:

- Prefer official feeds, APIs, and webhooks over scraping. Use scraping/browser capture only as fallback.
- Use HTTP conditional requests (`ETag`, `If-None-Match`, `Last-Modified`, `If-Modified-Since`) and store cache validators per URL/page. GitHub conditional `304` responses do not count against the primary rate limit when correctly authorized.
- For paginated GitHub endpoints, store validators per page, not only per collection.
- Keep stable source cursors: feed GUIDs/entry ids, arXiv ids, tweet ids, GitHub repo `pushed_at`, release tags, README blob SHA, and canonical content SHA.
- Gate model work behind deterministic checks. A poll that finds nothing changed should do no extraction, no embedding, no synthesis, and no notification.
- Treat all fetched content as untrusted source text. It can become cited wiki evidence, not instructions.
- Respect rate limits and `Retry-After`; back off noisy or broken sources and surface source health in ops UI.

Interestingness scoring:

Use a two-stage classifier so cost scales with promising candidates, not raw source volume.

Stage 1: deterministic prefilter.

- New repo from a watched high-signal org/person.
- New release or launch announcement.
- README or docs changed materially.
- Blog post title/body matches watched themes: agents, coding agents, memory, evals, model releases, developer relations, MCP, browser agents, Cloudflare/OpenAI/Anthropic, useful hacks.
- X post has strong novelty signals: launch, repo link, demo, benchmark, thread from a watched expert, or significant reply/quote context.
- Suppress pure engagement metrics, duplicate links, retweets/reposts without added context, tiny typo fixes, stale reposted announcements, and low-confidence scraped content.

Stage 2: cheap model classification.

Return structured JSON:

- `interesting`: boolean
- `score`: 0-100
- `category`: launch | release | research | tutorial | repo | benchmark | drama | funding | hiring | personal-update | other
- `novelty`: new-to-wiki | update-to-existing | duplicate | not-enough-signal
- `why_interesting`: 1-3 bullets
- `audience`: personal | AI-agents | DevRel | coding-tools | infrastructure | ignore
- `urgency`: immediate | next-digest | archive-only
- `wiki_action`: create-page | update-page | source-card-only | suppress
- `delivery`: telegram | email | both | none
- `confidence`

Novelty check:

- Search the wiki before alerting. If a candidate repeats known information, update source cards/page metadata but suppress notification unless it adds a material change.
- Cluster related candidates into one digest item. For example, a GitHub release, blog post, and tweet about the same launch should become one alert with multiple source links.
- Maintain per-topic cooldowns so the system does not message repeatedly about the same launch.

Page creation and expansion:

An "interesting" item should not stop at a short alert. If `wiki_action=create-page` or `wiki_action=update-page`, the wiki service should run a bounded enrichment workflow that produces a complete, useful page.

For a launch such as Vercel `eve` on June 17, 2026, the workflow should:

1. Create or resolve the canonical page, e.g. `wiki/projects/vercel-eve.md`.
2. Fetch primary sources first:
   - official launch blog / changelog
   - product/docs page
   - GitHub repo README, package metadata, release tags, examples
   - official X/social announcement
3. Fetch secondary context:
   - credible third-party writeups
   - posts by watched experts and relevant accounts
   - GitHub discussions/issues/PRs if they clarify maturity, adoption, limitations, or roadmap
   - related company announcements, such as Vercel Agent Stack / Connect / Sandbox if they explain the strategic context
4. Extract source cards and claims with provenance.
5. Compare against the existing wiki:
   - related projects/frameworks
   - prior Vercel agent infrastructure pages
   - OpenClaw/Hermes/Codex/Claude Code/eve-adjacent concepts
   - existing "agent framework" taxonomy
6. Write an insightful page, not a press-release summary.

Page template for launch/project pages:

- `Summary`: one paragraph explaining what it is.
- `Why it matters`: the actual novelty or strategic significance.
- `What launched`: concrete features/capabilities.
- `Architecture / design shape`: filesystem layout, runtime model, hosting/deployment, auth/connectors, channels, subagents, evals, approvals, sandboxing, etc.
- `How it compares`: relationship to OpenClaw, Hermes, Codex plugins/skills/MCP, Cloudflare Agents, LangGraph, Temporal-like durable execution, or other relevant systems.
- `Evidence`: source cards grouped by official / code / social / third-party.
- `Open questions`: unclear pricing, maturity, lock-in, missing docs, OSS governance, production limits, security model, portability.
- `Usefulness to us`: whether it changes `agent-*` design, what to borrow, what to avoid.
- `Timeline`: launch date, follow-up releases, major repo changes.
- `Related pages`: linked wiki pages and entities.

The page-expansion workflow should be idempotent. A new candidate can append a dated `What changed` section, refresh source cards, update comparisons, and add open questions without erasing prior history.

MVP tool additions:

- `wiki_expand_page(page_ref | candidate_id, depth?, source_budget?, write_policy?)`
- `wiki_source_map(topic | candidate_id, source_types?)`
- `wiki_page_gap_check(page_ref)`
- `wiki_page_update_from_sources(page_ref, sources, mode=create|append|refresh)`

Librarian responsibilities:

- Staleness scoring: source freshness, verification age, compilation age, volatility, source-chain integrity.
- Quality scoring: source count, citation coverage, confidence, contradiction density, broken links, orphan pages, missing summaries, weak "see also" links.
- Augmentation: when a new source is related to an existing page, append/update a dated "What changed" section, run source expansion when the candidate is high-value, add source cards, refresh confidence/staleness, and create open questions.
- Link repair: add backlinks and typed entity/topic links where deterministic extraction finds obvious relations.
- Contradiction handling: mark conflicting claims explicitly; do not silently overwrite history.
- Digest support: emit machine-readable changes so channel packages can render "what changed and why it matters".

Notification policy:

- Prefer batch digests by default, with immediate Telegram only for high-score launches/urgent watch targets.
- Always deliver through concrete channel packages (`agent-telegram`, `agent-mail`), not a generic push abstraction.
- A notification should include: title, one-sentence summary, why it is interesting, primary link, wiki page link, source list, confidence, and optional "mute/watch more like this" commands.
- Keep delivery idempotent: one candidate cluster, one digest item, one delivery receipt per target.
- Store rendered bodies and platform receipts so retries do not duplicate messages.
- The user can configure per-topic delivery: e.g. OpenAI/Anthropic launches -> Telegram immediately; newsletters -> daily email; X interesting posts -> daily Telegram digest unless score > 85.

State of the art / lessons from references:

- Karpathy's LLM Wiki pattern is "compiled knowledge": the agent reads sources once, maintains a persistent cross-linked wiki, flags contradictions, and accumulates synthesis instead of redoing RAG on every question.
- OpenClaw's `memory-wiki` split is the right boundary: active memory owns recall/promotion/dreaming; the wiki owns deterministic pages, claims, provenance, dashboards, and machine-readable digests.
- GBrain's main lesson is daemonization and graph-aware synthesis: ingestion, enrichment, consolidation, gap analysis, and digests run continuously enough that the user wakes up to a smarter brain. We should borrow the operating model without inheriting Postgres as the default local store.
- OpenClaw channel architecture reinforces that delivery belongs behind channel-owned adapters with shared message/delivery contracts and receipts, not generic push.
- The Swift plan's `wiki-watch`, `wiki-librarian`, `wiki-digest`, and `x monitor interesting` pieces are directionally correct. The de-ported implementation should make them Rust local daemons with SQLite ledgers, MCP tools, and channel delivery intents.

MVP tools:

- `wiki_watch_add(kind, locator, tags?, cadence?, delivery_policy?)`
- `wiki_watch_list(status?, tag?)`
- `wiki_watch_run_due(limit?, dry_run?)`
- `wiki_watch_status(source_id?)`
- `wiki_candidate_list(status?, min_score?, since?)`
- `wiki_candidate_read(candidate_id)`
- `wiki_candidate_mark(candidate_id, action)`
- `wiki_librarian_scan(scope?, tier?)`
- `wiki_librarian_report(scope?)`
- `wiki_expand_page(page_ref | candidate_id, depth?, source_budget?, write_policy?)`
- `wiki_source_map(topic | candidate_id, source_types?)`
- `wiki_page_gap_check(page_ref)`
- `wiki_digest_render(window?, audience?, min_score?)`
- `wiki_digest_deliver(digest_id, targets?)`

MVP success:

- Watching `openai`, `anthropic`, `simonw`, and a small set of RSS feeds produces no notification on no-op polls.
- A new repo/release/blog/tweet creates a candidate, scores it, dedupes it against the wiki, writes/updates a cited page or source card, and sends a concise Telegram/email item: what it is, why it matters, and links.
- A high-value launch such as Vercel `eve` creates or expands a full wiki page by pulling official blog/docs/changelog, GitHub repo metadata/README/releases, social announcements, expert commentary, and third-party coverage, then writes a useful synthesis with comparisons, open questions, and "usefulness to us".
- Source health, recent errors, suppressed candidates, librarian findings, and delivery receipts are visible through MCP and `agent-ops-ui`.

### 4. Personal Memory

Recommendation: ship personal memory as `agent-memory`, backed by `mem0-rs` and SQLite, separate from `agent-llm-wiki`. This should be mem0-shaped as a full lifecycle memory system, not just CRUD tools.

Use this for mem0-style facts and preferences:

- User facts: name, pets, family context, location preferences, durable personal details.
- Health/accessibility/preferences: ADHD context, medication names when the user chooses to store them, communication preferences, friction/approval preferences.
- Agent relationship state: "likes concise status updates", "wants no hidden approval stalls", "prefers Obsidian for markdown browsing".
- Repeated task preferences: formatting, scheduling defaults, notification preferences, trusted-owner channel behavior.

Do not use this for:

- Large research corpora.
- Imported papers, websites, bookmark archives, or source packs.
- Developer-relations and AI-agent document collections.
- Research briefs that need citations and stale/contradiction handling.

MVP tools:

- `memory_add(messages | fact, user_id?, run_id?, metadata?)`
- `memory_search(query, user_id?, k?)`
- `memory_get(memory_id)`
- `memory_update(memory_id, text | metadata)`
- `memory_delete(memory_id)`
- `memory_history(memory_id | user_id?)`
- `memory_review(filter?)`
- `memory_ingest_conversation(thread_ref | messages, scope, policy?)`
- `memory_collect_pending(scope?, status?)`
- `memory_capture_decide(messages, scope, source_context?)`
- `memory_reconcile(scope?, memory_ids?, policy?)`
- `memory_dream(scope?, window?, budget?)`
- `memory_create_procedural(messages | thread_ref, agent_id, task_ref?)`
- `memory_explain(memory_id | query)`

MVP skills:

- `$memory-review`: inspect, correct, merge, and delete personal memories.
- `$memory-capture-policy`: decide whether a conversation fact is durable personal memory, temporary context, or wiki knowledge.
- `$memory-dream`: run scheduled consolidation/reconciliation over recent memories, contradictions, repeated facts, stale preferences, and procedural summaries.
- `$memory-debug`: inspect capture hooks, recent candidates, skipped items, reconcile decisions, and extraction errors.

Lifecycle model:

```text
trusted conversation / channel / agent turn
  -> pre-turn recall hook: memory_search + scoped context card
  -> post-turn capture hook: memory_ingest_conversation
  -> candidate inbox: extracted facts, skipped reasons, sensitivity labels
  -> mem0 add/reconcile: ADD / UPDATE / DELETE / NONE with history
  -> auto-apply in trusted personal scopes + append decision ledger
  -> dream job: periodic consolidation, contradiction cleanup, procedural memory
  -> review surface: digest, corrections, merges, deletes, rollback
  -> next turn recall
```

Hook points:

- `pre_turn_recall`: before a trusted agent turn, search personal memory using the current user request, project/channel context, and recent thread summary. Return a compact, low-authority memory card with memory ids.
- `post_turn_capture`: after a trusted turn or channel message, ingest user/assistant messages through mem0 extraction. This is default-on for trusted owner contexts, with per-channel and per-project disable switches.
- `channel_capture`: Telegram/mail/future chat channels can submit normalized message batches for memory capture after identity has been server-stamped.
- `thread_summary_capture`: controller summaries can be offered to memory as candidate personal preferences or recurring task facts, while wiki/source material is routed to `agent-llm-wiki`.
- `procedural_capture`: completed workflows can create `procedural_memory` records for how an agent accomplished a task, scoped by `agent_id` and optionally project/task.
- `dream_schedule`: periodic maintenance that replays recent candidates and history, merges duplicates, reconciles contradictions, summarizes repeated patterns, and applies cleanup automatically in trusted personal scopes.
- `review_hook`: low-friction review surface for "what did you learn about me?", "forget that", "merge these", "why do you think that?", and "show recent memory changes".

Implementation notes:

- Use `mem0-rs` as the leading implementation candidate because it matches the personal-memory domain: add/search/history/update/delete over compact user facts, with embedded SQLite/vector storage and low footprint.
- Keep a Rust package layer around it for MCP, config, host adapters, tests, local sync/drain logic, and installer/doctor commands. Generate TypeScript clients/schemas only for Cloudflare Workers or browser UI pieces.
- Capture is default-on for trusted personal contexts. Unknown senders, group chats, untrusted web text, and imported corpora do not get personal-memory capture by default.
- Dream/reconcile applies changes automatically for trusted personal-memory scopes. It should not block on a review queue by default.
- Do not silently inject personal memory at high authority. Retrieve it as tool-mediated context with a small provenance note, memory ids, and a clear boundary between remembered facts and current instructions.
- Sensitive personal memories need clear review and deletion paths. Health and medication facts are useful and can be captured, but should be private, scoped personal data with obvious "show/forget/correct" flows.
- Keep the database separate from wiki corpora. Cross-link by metadata only when useful.
- Keep a candidate/decision ledger, not only final memories: captured input refs, extraction prompt version, model/provider, ADD/UPDATE/DELETE/NONE decisions, skipped reasons, sensitivity label, source channel/thread, and history ids.
- Dream jobs should be bounded, inspectable, reversible, and digest-producing. They can auto-apply merges/reconciliations according to trusted-scope policy, but they must emit a digest of changed memories, preserve history/rollback data, and never invent uncited personal facts from wiki corpora.

Existing projects:

- [mem0ai/mem0](https://github.com/mem0ai/mem0) is a large open-source memory layer with Python/Node SDKs, self-hosted server, and plugin/skills folders.
- [chrischabot/mem0-rs](https://github.com/chrischabot/mem0-rs) is a Rust mem0 port with a single-binary REST server, embedded SQLite/vector storage, and documented performance/footprint goals versus Python mem0. Treat it as the leading candidate for personal-memory implementation, subject to benchmark confirmation.
- [OpenMemory](https://mem0.ai/openmemory) presents a persistent MCP memory layer for coding agents.
- The official MCP memory server is simpler but useful as a baseline: local knowledge graph entities, relations, observations, and a `memory://knowledge-graph` resource.

### 5. X / Twitter

Recommendation: port as `x-mcp` plus an optional monitor daemon.

MVP tools:

- `x_connect_status`
- `x_import_bookmarks(max?, since?)`
- `x_import_likes(max?, since?)`
- `x_import_following(max?)`
- `x_report_popular(window, collection?)`
- `x_report_interesting(window, collection?, max_usd?)`
- `x_monitor_once(dry_run?)`

Sidecar responsibilities:

- OAuth2 Authorization Code with PKCE, token refresh, revocation.
- Cursor storage for watched accounts.
- Spend cap ledger for "interesting" reports.
- Writes into Memory Wiki via `wikid` API.
- Optional delivery through a concrete channel package such as `agent-telegram`.
- Optional Cloudflare deployment for OAuth callback stability, remote MCP access, cron scheduling, and cursor/spend state in D1/KV.

Official API constraints:

- X bookmark lookup requires an approved developer app and user access token with `bookmark.read` for OAuth2 PKCE; the X docs also list `tweet.read` and `users.read` for bookmark reads with expansions.
- X OAuth returns refresh tokens only when `offline.access` is requested.

Migration note:

- Keep `docs/X_INTEGRATION.md` as the source for your current X callback/tunnel lesson: X rejecting plain local redirect URIs is an operational constraint in your setup.
- Consider using either Codex automations or Cloudflare cron for twice-daily "run X monitor and summarize findings" while `xmonitord`/Worker owns API cursors and delivery.

### 6. Telegram

Recommendation: implement Telegram as a channel bridge backed by `agent-controller`, not as an MCP-only integration and not as a one-workspace bot.

Why:

- MCP exposes tools to Codex, but Telegram is an inbound transport. Something has to listen for updates.
- Regular Codex SDK/app-server can be the "turn runner" that the Telegram daemon calls.
- The channel needs project/thread awareness, follow-up resolution, and status summaries; that belongs in the controller, not in Telegram-specific code.
- Telegram's Bot API supports two update modes, `getUpdates` and webhooks; they are mutually exclusive, so choose one per deployment.

MVP:

- `telegramd` long-polls, maps Telegram user id/chat id to owner/non-owner and forwards normalized channel messages to the controller.
- For status questions, the controller resolves the project/thread and returns a read-only summary.
- For owner messages that continue a thread, the controller calls Codex SDK/app-server with a configured sandbox.
- For "create a new project/thread" requests, the controller follows the configured policy: trusted owner channels may create directly for low-risk local records, while repo/file/public/external side effects still use their configured approval tier.
- For non-owner messages, either refuse or run read-only/no-network prompts.
- Reply with final answer; later add streaming typing indicators and media.

Cloudflare option:

- Use a Worker as the Telegram webhook endpoint when you want public HTTPS without a tunnel.
- The Worker can validate the Telegram secret, enqueue work, and call a local bridge only when the task needs local Codex/workspace context.
- If the task is pure remote data/tool work, a Cloudflare Agent could answer directly through MCP-backed tools; that is a separate product from "Telegram drives my local Codex checkout."

Security requirements:

- Identity comes from Telegram authenticated user id, never message text.
- Owner allowlist is required; empty allowlist means no privileged actions.
- Group chats should be owner-only or per-sender isolated.
- Store channel conversation context in the controller so the same chat can switch among projects and still resolve follow-ups like "and the video project?"

### 7. Google Workspace / Gmail

Recommendation: use regular Codex's existing plugin/app ecosystem first. Build custom only where the existing Gmail/Drive/Calendar tooling does not cover a required workflow.

Regular Codex plugin docs explicitly list Gmail and Google Drive plugins as examples. That makes a wholesale reimplementation of `google_api` lower priority.

Good custom targets:

- Wiki ingestion from Drive/Docs folders.
- Gmail-to-wiki capture.
- Gmail inbound channel if you really want email as a chat transport.
- A narrow `workspace_archive_to_wiki` MCP server that reads from Google APIs and writes to Memory Wiki.
- Cloudflare-hosted OAuth/MCP front door for small Google-backed tools where public HTTPS and Dynamic Client Registration reduce local setup pain.

Gmail channel note:

- The Gmail API provides push notifications to avoid polling, but that requires backend Pub/Sub-style infrastructure. A local-first MVP can poll unread mail and use strict loop prevention.
- Default Gmail channel mode should be non-owner/read-only until send-loop defenses are proven.

### 8. Channels

Recommendation: build `agent-channel-kit` as a shared contract, then keep each platform in its own concrete package. Do not create a generic `agent-push`.

Why:

- Incoming and outgoing messages share the same trust and formatting problems.
- Platform identity must be server-stamped, not inferred from message text.
- Prompt-injection defense belongs at the channel boundary before content enters a Codex thread.
- Formatting is platform-specific: Telegram, Discord, Slack, email, and future channels all need different escaping, rich-content handling, threading, and attachment behavior.
- Delivery has platform-specific outcomes, but common receipt semantics.

`agent-channel-kit` MVP:

- Shared inbound envelope and outbound intent schemas.
- Prompt-injection screening/fencing helpers for inbound content.
- Owner/channel allowlist and per-channel policy model.
- Formatting interface: internal body -> platform-specific renderer.
- Durable send intent and delivery receipt schemas.
- Idempotency keys, retry metadata, suppression reasons, and dead-letter records.
- Test fixtures for hostile inbound messages, malformed formatting, attachment limits, and duplicate delivery.

Concrete channel app MVPs:

- `agent-telegram`: first implementation because it is already a top-priority channel.
- `agent-discord`: future package, should reuse channel kit but own Discord-specific auth, embeds, threads, guild/channel policy, and rate limits.
- `agent-slack`: future package, should reuse channel kit but own Slack app auth, channels, threads, Block Kit, app mentions, and workspace policy.
- `agent-mail`: if email is used as a channel, it should reuse channel kit but own MIME, threading, reply quoting, loop prevention, and sender verification.

Use cases:

- Telegram owner asks Codex a question and receives a formatted answer.
- X monitor emits a delivery intent to the configured Telegram channel.
- Deep Research automation sends a brief to a named channel only after policy allows it.
- A future Discord package can be added without changing the core channel contract.

Keep the codex-swift lessons, but put them in the channel layer:

- Named allowlisted targets are safer than arbitrary URLs.
- Durable send-intent-before-I/O prevents losing completed work.
- At-least-once delivery needs idempotency keys.
- SSRF handling belongs in channel/file-fetch helpers, not in a prompt.
- Prompt-injection protection must happen before inbound message text becomes agent context.

### 9. Media

Recommendation: lower priority. Use regular OpenAI image generation or a small MCP wrapper when needed.

If ported:

- `media_generate(kind, prompt, deliver_to?)` as MCP.
- Sidecar ledger for slow async providers.
- Signed local asset URLs from the sidecar UI/API, not Codex app-server.

### 10. Deep Research

Recommendation: ship this as `agent-deep-research`, primarily a skill package plus optional custom agents, using the host agent's native web search as the default research substrate. Do not build a new research daemon first.

Why:

- Codex already supports explicit subagent workflows: the main agent can spawn specialized agents in parallel, wait for results, and consolidate their summaries.
- Codex also supports custom agent TOML files with role-specific instructions, model/reasoning settings, sandbox choices, MCP servers, and skill configuration.
- OpenAI provides native web search through the Responses API, and dedicated deep-research models for comprehensive reports over web search, file search/vector stores, and remote MCP sources.
- Claude also has a native web search tool with citations, plus MCP connector support. Claude Desktop should use its native search where possible while sharing Memory Wiki/search adapters through MCP.
- Deep research is mostly a discipline: source discovery, claim extraction, contradiction hunting, synthesis, citation hygiene, and confidence labeling. Skills and custom agents are exactly the right surface for that.
- The Memory Wiki gives research a durable substrate: previous briefs, source cards, contradiction notes, and imported corpora can be exposed through MCP without making the research flow a hidden memory blob.

Package contents:

- Skill: `$deep-research` for full plan -> scout -> extract -> refute -> synthesize -> cite workflow.
- Skill: `$research-brief` for smaller one-pass research requests.
- Skill: `$research-audit` for checking an existing draft against sources and finding weak claims.
- Optional custom agents:
  - `research-scout`: broad source discovery, search query expansion, source-map creation.
  - `source-extractor`: evidence extraction, quote-safe notes, entity/event/date tables.
  - `skeptic`: contradiction search, adversarial readings, missing-counterexample hunt.
  - `synthesizer`: final structure, claims, confidence, citations, open questions.
- Optional MCP tools/resources from `agent-llm-wiki`: `wiki_search`, `wiki_read`, `wiki_ingest`, `wiki_job_status`, and `wiki://page/{id}`.
- Optional `agent-search-kit` adapters for Brave Search and Perplexity Search/Sonar when extra recall, an independent index, cross-agent parity, or a second opinion is useful.
- Optional Cloudflare component: capture research seeds from email/Telegram/webhooks, run scheduled watch checks, stage browser-rendered snapshots, and queue them for local wiki ingestion.

Default workflow:

1. Clarify research question, output shape, freshness requirements, and source constraints.
2. Use host-native web search by default: OpenAI web search in Codex/OpenAI contexts, Claude web search in Claude contexts.
3. Use host-native long-research mode when available: OpenAI deep-research models in OpenAI contexts, Claude's own web-search workflow in Claude contexts.
4. Optionally fan out to Brave/Perplexity adapters for independent-index cross-checking or broader recall.
5. Spawn `research-scout` agents by angle/source family when the host agent supports subagents and the task is broad enough.
6. Ingest or read selected sources through wiki/search MCP.
7. Spawn `source-extractor` and `skeptic` agents on bounded source sets when supported.
8. Main agent reconciles findings, labels confidence, cites sources, and lists unresolved gaps.
9. Write the final brief, source cards, contradiction notes, and unresolved gaps back to the wiki by default, unless the user marks the run private/temporary/no-write.

Guardrails:

- Subagents are opt-in and cost more; the skill should use them only for genuinely parallel research.
- Keep subagents read-heavy by default. Writes to wiki, files, or external channels should happen in the main thread or a named write-capable agent.
- Treat web/search/wiki content as untrusted data and never let source text override task instructions.
- Require citations for factual claims and explicitly mark inference.
- Preserve provider provenance. A Claude web-search citation, OpenAI web-search citation, Perplexity answer, Brave search result, and local wiki page are different evidence types.
- Wiki writes should be source-backed, deduped, and reviewable. Treat this as memory by osmosis, not silent authority: new research pages should carry `research_run_id`, retrieved dates, model/provider metadata, confidence labels, and links to the source cards that justify them.
- Use absolute dates when the question is time-sensitive.
- Cap fan-out: default to 3-5 agents, no recursive subagent spawning unless the user asks for it.

MVP success:

- From regular Codex, asking for `$deep-research` produces a cited, contradiction-checked brief using web search plus Memory Wiki tools.
- For a broad topic, Codex can spawn scout/skeptic/extractor subagents and consolidate their outputs without needing the Swift workflow engine.
- The package is useful even without Cloudflare; the edge piece only adds always-on source capture and scheduled research seeds.

### 11. Workflows / Deterministic Engines

Recommendation: do not port the whole Swift JavaScriptCore workflow engine initially.

Instead:

- Translate common workflows into skills that use Codex subagents, web search, Memory Wiki tools, and automations.
- Keep the `agent-deep-research` package as the first proof point before building any standalone workflow engine.
- Revisit a dedicated workflow engine only if skills + Codex subagents are too weak for repeatable research runs.

## Suggested Build Order

Phase 0: define the open-source app contract.

- Create the first `agent-*` skeleton with `worker/`, `local/`, `mcp/`, `hosts/codex/`, `hosts/claude/`, `docs/`, and tests.
- Decide whether to start as a monorepo (`agent-apps`) or separate repos. My default: monorepo until the shared envelope/drain contract stabilizes, then split only if community adoption wants independent release cadence.
- Keep Codex/Claude-specific setup in host adapter folders so the core package remains MCP-first and host-neutral.
- Define the event envelope, drain protocol, retention policy, and health check conventions.
- Define the Rust local runtime profile: process model, crate layout, logging/tracing, config files, SQLite access, file watching, MCP transports, generated schemas, and launchd/systemd templates.
- Define shared local config and secret storage.
- Define a stable local HTTP API between MCP servers and daemons.
- Run a Codex app-server/SDK spike before committing to `agent-controller`: prove start thread, stream events, list/read status, resume/follow-up, and failure handling from the intended host adapter. If Rust-native access is unstable or undocumented, define a tiny JS/Node shim behind a stable local HTTP/MCP contract.
- Define the minimum happy path for open-source adopters: one-command local install of `agent-profile` + `agent-memory` + `agent-backup` + `agent-ops-ui` with no Cloudflare requirement.
- Define host capability matrices for Codex, Claude Desktop, Claude Code, and generic MCP clients so unsupported hooks/skills/automations degrade visibly instead of silently no-oping.

Phase 1: Personal profile, conversation import, quality kit, and memory.

- Package `agent-profile` as an inspectable profile/preference store for communication register, output preferences, recurring constraints, taste/style rules, sizing defaults, approval/friction preferences, and decision criteria.
- Package `agent-conversation-import` with dry-run, redaction, candidates, review/apply/reject, import ledger, and export/delete flows for Claude/Codex history.
- Package `agent-quality-kit` as shared skills/contracts for root-cause debugging, reproduce-then-fix, tests, citations, visual inspection, blocked-state notification, and uncertainty surfacing.
- Package `agent-backup`, `agent-secrets`, and `agent-cost` early enough that personal data, credentials, and always-on model spend are controlled before real use.
- Add competence-respect fixtures: personalized request with available memory/profile/wiki must consult them; current factual request must use search/tooling; high-stakes or emotionally sensitive request must not choose low-effort reasoning; unavailable memory/tooling must be disclosed instead of hidden.
- Package `agent-memory` around `mem0-rs` with SQLite as the default store.
- Expose `memory_add`, `memory_search`, `memory_get`, `memory_update`, `memory_delete`, `memory_history`, and `memory_review` over MCP.
- Add lifecycle MCP tools: `memory_ingest_conversation`, `memory_collect_pending`, `memory_capture_decide`, `memory_reconcile`, `memory_dream`, `memory_create_procedural`, and `memory_explain`.
- Add pre-turn recall and post-turn capture hooks for Codex and Claude-compatible host adapters where available.
- Add a candidate/decision ledger so capture, skipped items, auto-applied reconciliation decisions, dream changes, and sensitivity labels are inspectable and reversible.
- Default dream/reconcile to auto-apply in trusted personal-memory scopes, then emit a digest and keep correction/rollback paths.
- Add `$memory-review`, `$memory-capture-policy`, `$memory-dream`, and `$memory-debug` skills.
- Add a benchmark comparing `mem0-rs` against the TypeScript port and Python upstream on representative personal-memory add/search/history workloads.
- Success: regular Codex and Claude Desktop can load explicit profile slices, import conversation history into reviewed artifacts, recall/auto-capture/review/correct/reconcile/dream personal memories, and enforce quality/blocked-state/competence-respect behavior without touching wiki corpora or raw transcripts.

Phase 2: LLM Wiki read path.

- Wrap existing `codex-memory` query/read/brief tools as MCP.
- Package `$wiki-research` and `$wiki-ingest` skills.
- Add a small corpus config and smoke test from regular Codex.
- Success: regular Codex can ask a wiki question and get cited, source-backed answers with no Swift app-server.

Phase 3: Deep Research skill path.

- Add `agent-deep-research` plugin contents: `$deep-research`, `$research-brief`, and `$research-audit`.
- Add optional custom agents for `research-scout`, `source-extractor`, `skeptic`, and `synthesizer`.
- Wire the skill to use web search plus Memory Wiki MCP tools; write source cards and final briefs to the wiki by default, with a per-run `no-write` override.
- Success: regular Codex can run a multi-subagent research brief with citations, contradictions, confidence labels, and open questions.

Phase 4: Wiki ingest jobs.

- Add async `wiki_ingest`, `wiki_compile`, `wiki_job_status`.
- Add source adapters for file, URL, RSS, GitHub, arXiv.
- Add watch/librarian/interestingness tools: `wiki_watch_run_due`, `wiki_librarian_scan`, `wiki_candidate_list`, `wiki_digest_render`, and `wiki_digest_deliver`.
- Add page expansion tools: `wiki_expand_page`, `wiki_source_map`, and `wiki_page_gap_check`, so high-value launches become complete wiki pages instead of only short notifications.
- Add corpus migration tools for embedding/model changes: `wiki_reembed`, `wiki_corpus_migration_plan`, and `wiki_corpus_compat_check`, with cost estimates and resumable jobs.
- Store wiki pages as Markdown files on disk so Obsidian can browse/edit them.
- Add MCP status resources and structured status-card tools for ingestion status, job history, logs, health checks, recent errors, and source/queue state.
- Add `agent-ops-ui` MVP that mirrors the MCP-backed ops state for human visual inspection and browser-use workflows.
- Add ops skills: `$ops-status`, `$ops-diagnose`, `$ops-ui`, `$wiki-ingest-ops`, and `$controller-status`.
- Add browser-use guidance so Codex uses MCP by default, opens the local ops UI only for visual/interactive inspection, and verifies any UI action through MCP afterward.
- Keep X out of this phase except for interface placeholders.
- Success: regular Codex can start an ingest, check status, query the new pages, expand a high-value event such as Vercel `eve` into a cited multi-source wiki page, use MCP for ops state, and optionally open the local ops UI in the in-app browser for visual inspection.

Phase 5: X.

- Port X OAuth/client/import/report to `x-mcp`, choosing local sidecar or Cloudflare Worker after testing credential and callback needs.
- Write to Memory Wiki via `wikid`.
- Add Codex automation, launchd, or Cloudflare cron for twice-daily monitor.
- Success: bookmarks/likes/following import into wiki; interesting report can be produced and spend-capped.

Phase 6: Controller, channel kit, and Telegram.

- Build `agent-controller` local service with project registry, thread registry, active-run monitor, summary index, and natural-language resolver.
- Add controller MCP tools for listing projects/threads, resolving references, reading status, creating projects, and continuing threads.
- Build `agent-channel-kit` schemas and test fixtures for inbound envelopes, outbound intents, formatting, injection fencing, and delivery receipts.
- Build `telegramd` bridge that forwards normalized messages to the controller; optionally put Cloudflare in front as webhook/OAuth/public MCP gateway.
- Connect X monitor delivery to a named Telegram channel target through the channel contract.
- Success: Telegram owner can ask "how is the de-porting project going?", follow with "and the video project?", create a new project/thread according to policy, and receive X monitor notifications through the same channel contract. Future Discord/Slack packages can reuse the controller and channel kit.

Phase 7: Google/Gmail.

- Trial official Gmail/Drive/Calendar plugins first.
- Fill only missing flows with custom MCP.
- If Gmail channel remains desirable, build read-only poller first, then add send with loop guards.
- Success: no custom Workspace reimplementation unless a concrete plugin gap exists.

Phase 8: Content/style/workspace production loop.

- Import `/Users/chabotc/Desktop/STYLE.md` as the canonical style profile for `agent-content`.
- Link `agent-content` to `agent-profile` so writing voice, register, and output preferences are editable profile artifacts rather than repeated prompt text.
- Add content tools for blog drafts, social variants, trend reports, editorial calendar, and wiki-grounded outlines.
- Add workspace capture tools that use official Google connectors first, then write selected Docs/Gmail/Drive context into wiki source cards when requested or configured.
- Add draft-first publishing workflows for blog/social/email/newsletter outputs, with explicit policy before any public post or external send.
- Success: a watched signal can become a wiki page, then a blog outline/draft/social thread in the user's style, with source citations and an editable draft artifact.

Phase 9: Personal domain app copy: Garderobe.

- Copy `/Users/chabotc/Projects/garderobe` into the monorepo as `agent-garderobe`, preserving the Cloudflare Worker + D1 + OAuth/DCR remote MCP shape.
- Copy source, migrations, docs, seed templates, and Wrangler config templates only. Do not copy `node_modules`, `.dev.vars`, deployed secrets, generated artifacts, or live tokens.
- Add host adapter docs/skills for Codex and Claude: `$outfit-plan`, `$packing-list`, `$wardrobe-audit`, `$rotation-review`, `$weather-outfit`.
- Wire `agent-garderobe` to `agent-profile`, `agent-decision-support`, and `agent-routine-context` so outfit advice can use taste rules, fit/sizing history, weather, calendar/formality, comfort constraints, rotation, and prior "what worked" decisions.
- Add wardrobe ingestion: photo/product-URL capture, vision-assisted garment cataloging, image/blob storage policy, manual correction, and provenance for item attributes.
- Add integration hooks for weather, calendar context, and channel delivery.
- Success: Codex and Claude can use the same wardrobe MCP to plan outfits based on weather, calendar/formality, availability/laundry probability, rotation, and sprezzatura preferences.

Phase 10: Browser workflows and media.

- Add `agent-browser-workflows` recipes with separate browser profile guidance, action ledgers, screenshots, and verification.
- Add `agent-media` ledger for images, slides, video scripts, generated assets, exports, and delivery receipts.
- Success: Codex can operate no-API web workflows safely, and can turn wiki/content briefs into verified sites, decks, social assets, or video scripts.

Phase 11: polish and distribution.

- Local marketplace/plugin entries for each app.
- Install script/doctor.
- End-to-end tests with fake MCP clients and mocked external APIs.
- Document operational runbooks.

## Security And Reliability Invariants

These should carry over from codex-swift:

- Do not trust inbound message text for identity. Telegram/Gmail identity must be server-stamped.
- Channel content is untrusted data. Formatting adapters must fence/quote inbound content and prevent platform text from changing tool permissions, workspace routing, or approval policy.
- Project/thread routing is advisory until confirmed. Ambiguous channel requests must ask for clarification before creating, writing, or resuming privileged work.
- Non-owner/inbound/unattended turns should be read-only/no-network by default.
- External writes require explicit approval or pre-authorized named targets/tool policies. Trusted owner channels may run low-risk preauthorized writes without one-click confirmation.
- If a task needs approval, missing auth, or a connector permission, the controller must notify the channel promptly instead of letting the agent sit blocked in the background.
- Secrets live in OS keychain or 0600 files, never checked-in TOML.
- Credential lifecycle needs an owner: token location, scopes, refresh, expiry, rotation, revocation, and failed-grant alerts should be visible through `agent-secrets`, `doctor`, and `ops_health()`.
- Sensitive personal data needs a stricter tier. Health-adjacent facts, medications, emotional triggers, private relationship context, and similar profile/memory items default to encrypted local storage, review-required extraction, no edge sync unless explicitly opted in with client-side encryption, and explicit model/provider rules before sending to hosted models.
- Memory recall is untrusted data and should be cited or fenced.
- Personal-memory dream/reconcile may auto-apply only in trusted personal scopes. Unknown senders, group chats, imported corpora, wiki pages, and untrusted web text cannot grant themselves memory-capture authority.
- Memory needs decay and anti-self-reinforcement. Facts whose only recent provenance is prior recall should not become stronger just because they were recalled; dream/reconcile should surface stale preferences and low-confidence memories for review.
- Ingested pages need source provenance and stale/contradiction handling.
- Ingested content remains untrusted through wiki and content generation. Source-derived public publishing requires provenance display, injection/exfiltration checks, and human review unless an explicit safe template policy applies.
- Deep Research subagents should be read-heavy by default; final writes and external sends should happen from the main thread or an explicitly write-capable agent.
- Long jobs need durable ledgers and resumable cursors.
- Long-running monitors need liveness monitoring. A watcher stopping silently is a user-facing failure, not merely an ops detail.
- HTTP sidecars need egress allowlists, redirect control, and IP/host SSRF defenses.
- Gmail auto-reply loops are a product risk, not a footnote.
- High-stakes, emotionally sensitive, or strongly personalized tasks require a competence preflight: consult relevant profile/memory/wiki, choose sufficient reasoning effort, use available tools for current/factual claims, and report any skipped context/tool path.
- The agent should never silently fail into a low-effort answer. If memory, search, connector, or a needed tool is unavailable, say so and continue with clearly bounded confidence.
- Delete/forget/export semantics must cross store boundaries: memory/profile/wiki links, decision ledgers, import ledgers, edge queues, backups, and source cards should report where an item existed and what was tombstoned, retained for audit, or scheduled for expiry.
- Third-party content storage needs conservative defaults: store metadata, hashes, cache validators, short snippets, and source cards by default; raw tweets/articles/newsletters/search results require per-source policy, TTL, and provider ToS notes.

## What Not To Port

- The Swift app-server clone, schema oracle, worker supervisor, broker, and WebGateway as Codex replacements.
- The Swift-specific Seatbelt sandbox/runtime unless a standalone sidecar needs macOS containment.
- The whole Google universal discovery tool unless regular Codex's existing plugins are insufficient for a named workflow.
- The whole JSC workflow engine until skills/subagents/automations have been tested and found lacking.
- A hidden prompt-injection memory layer that silently injects unreviewed wiki text at high authority.

## Decisions

- Personal memory, wiki knowledge, and other reusable tools should target all local agents where practical, with Claude Desktop as a named compatibility target.
- Claude Desktop compatibility is not full lifecycle parity. Treat Claude as a strong MCP/manual-capture client unless Anthropic exposes hooks/skills/automations comparable to Codex.
- Drop MLX/local inference for now. Use hosted OpenAI/GPT models per job, with configurable model policy and current-best defaults.
- Standardize local services on Rust-first single-binary daemons. Keep TypeScript as the default for Cloudflare Workers and browser/frontend code, with Rust Workers considered package-by-package through `workers-rs`. Use/evaluate `mem0-rs` as the Rust mem0-style personal-memory core, keep SQLite as the default store, and avoid Postgres in the first install path.
- `agent-memory` should be a full mem0-shaped lifecycle system: pre-turn recall hooks, post-turn capture hooks, conversation ingestion, candidate collection, ADD/UPDATE/DELETE/NONE reconciliation, procedural memory, scheduled dream/consolidation jobs, review/delete/correct flows, and an inspectable decision ledger. Dream/reconcile changes should auto-apply in trusted personal scopes by default, then produce a digest and remain reversible.
- Service placement: Memory Wiki is hybrid with local source of truth and optional Cloudflare capture inbox; Telegram is hybrid with Cloudflare webhook plus local Codex bridge; X is Cloudflare-eligible for OAuth/callback/cron/cursors; mail is hybrid via Cloudflare Email Worker plus local processing; Deep Research is host-native-search first with optional Cloudflare seed capture.
- Deep Research search stack: use the host agent's native search first: OpenAI web search/deep-research in Codex/OpenAI contexts, Claude web search in Claude contexts. Brave Search and Perplexity Search/Sonar are optional provider adapters through `agent-search-kit`.
- Deep Research should write source cards, final briefs, contradiction notes, and unresolved gaps back to the wiki by default, with a per-run `no-write` escape hatch.
- Start as a monorepo, likely `agent-apps`, so shared envelope, channel, search, MCP, and Cloudflare patterns can evolve together.
- Name packages `agent-*`, not `codex-app-*`. Codex-specific pieces live under host adapters; MCP-compatible services should be usable from Claude Desktop/Code and other agents without inheriting Codex branding or runtime assumptions.
- Chat channels should route through `agent-controller`, a meta-controller for projects, threads, running work, status summaries, natural-language reference resolution, follow-up context, and approved creation/resume operations.
- Google Workspace should use official Codex/connector plugins first. Custom code is only for missing flows such as Workspace-to-wiki capture, Gmail/mail as a channel, or controller-mediated channel requests. Calendar writes should be configurable: no-confirm for trusted owner channels when sufficiently specified, confirm for ambiguous/high-impact edits, and never silently stuck behind a hidden approval.
- Personal memory and LLM Wiki are separate systems: `agent-memory`/`mem0-rs` for compact personal facts and preferences; `agent-llm-wiki` for source-backed knowledge corpora. Memory Wiki pages should be Markdown files on disk and browsed/edited with Obsidian. Operational state is MCP-first, with `agent-ops-ui` as a standalone local web UI for human visual inspection of ingestion, status, history, logs, health, recent errors, and config.
- Add `agent-profile` as the explicit personal operating manual and `agent-conversation-import` as the safe bridge from Claude/Codex history into reviewed profile, memory, wiki, and decision artifacts. Do not turn conversation exports into hidden prompt sludge.
- Add `agent-backup`, `agent-secrets`, and `agent-cost` as first-wave cross-cutting services, not later polish. The system is not trustworthy without backup/restore, credential health, and spend ceilings.
- Add `agent-decision-support`, `agent-routine-context`, and `agent-quality-kit` to cover the recurring personal-use shape from the Claude export: constrained recommendations, daily context, and verified outputs.
- Define a clear policy for Codex built-in Memories when `agent-memory` is active: either disable built-in Memories for overlapping scopes or keep them in a documented non-overlapping role, so two memory systems do not conflict silently.
- The broader product target is Codex as the daily work shell, with portable services around it for coding context, wiki/research, content/style/publishing, Google Workspace, social/news/GitHub/paper monitoring, Telegram/email reachability, media/decks/video, and personal-domain MCP apps such as `agent-garderobe`.
- `/Users/chabotc/Desktop/STYLE.md` should become the canonical `agent-content` style profile. `/Users/chabotc/Projects/garderobe` should be copied into the monorepo as `agent-garderobe`, preserving its Cloudflare Worker + D1 + OAuth/DCR MCP design.

## Bottom Line

Yes, most of the functionality can move to regular Codex without maintaining a port. The clean architecture is:

- Regular Codex remains the agent UI/runtime.
- Open-source `agent-*` packages provide skills, MCP servers, local drains, and optional Cloudflare Workers.
- Cloudflare acts as a bounded edge inbox for internet-facing collection; local services remain the durable/private processors.
- Personal profile, personal memory, LLM Wiki, X, Telegram, mail, and future channels run as local sidecars or Cloudflare Workers depending on privacy, always-on, and OAuth needs.
- Codex automations replace cron.
- Official Google Workspace plugins are used before custom Google code.

The first concrete milestone should be `agent-profile` + `agent-conversation-import` + `agent-memory`, because it proves the personal operating layer with inspectable preferences, safe history migration, `mem0-rs`, SQLite, MCP, and review/delete flows. The first large knowledge milestone should be `agent-llm-wiki`, because it unlocks cited corpus search for regular Codex and gives X/Telegram/mail/deep-research somewhere durable to write. The next milestone should be `agent-deep-research`, because it proves the regular-Codex subagent path before any new workflow engine is built. After that, build `agent-garderobe` and `agent-telegram`/`agent-x` to prove both the personal-domain MCP path and the edge-inbox/local-drain model against real events.

## Source Notes

- OpenAI Codex docs: [Skills](https://developers.openai.com/codex/skills), [Plugins](https://developers.openai.com/codex/plugins), [MCP](https://developers.openai.com/codex/mcp), [Automations](https://developers.openai.com/codex/app/automations), [Subagents](https://developers.openai.com/codex/subagents), [SDK](https://developers.openai.com/codex/sdk), [App Server](https://developers.openai.com/codex/app-server).
- OpenAI Codex app docs: [In-app browser](https://developers.openai.com/codex/app/browser), [Features](https://developers.openai.com/codex/app/features).
- OpenAI API docs: [Latest model guidance](https://developers.openai.com/api/docs/guides/latest-model), [Embeddings](https://developers.openai.com/api/docs/guides/embeddings), [Web search](https://developers.openai.com/api/docs/guides/tools-web-search), [Deep research](https://developers.openai.com/api/docs/guides/deep-research).
- Anthropic docs: [Claude web search tool](https://platform.claude.com/docs/en/agents-and-tools/tool-use/web-search-tool), [Claude MCP connector](https://platform.claude.com/docs/en/agents-and-tools/mcp-connector), [Claude search results](https://platform.claude.com/docs/en/build-with-claude/search-results).
- MCP: [intro](https://modelcontextprotocol.io/docs/getting-started/intro), [tools spec](https://modelcontextprotocol.io/specification/2025-06-18/server/tools), [resources spec](https://modelcontextprotocol.io/specification/2025-06-18/server/resources), [official Rust SDK](https://github.com/modelcontextprotocol/rust-sdk).
- Cloudflare: [Remote MCP server guide](https://developers.cloudflare.com/agents/model-context-protocol/guides/remote-mcp-server/), [Cloudflare MCP overview](https://developers.cloudflare.com/agents/model-context-protocol/), [Durable Objects overview](https://developers.cloudflare.com/durable-objects/), [Queues](https://developers.cloudflare.com/queues/), [Email Service / Email Workers](https://developers.cloudflare.com/email-service/), [Browser Run](https://developers.cloudflare.com/browser-run/), [R2](https://developers.cloudflare.com/r2/), [Rust Workers support](https://developers.cloudflare.com/workers/languages/rust/), [`workers-oauth-provider`](https://github.com/cloudflare/workers-oauth-provider).
- Memory/wiki projects: [mem0](https://github.com/mem0ai/mem0), [mem0-rs](https://github.com/chrischabot/mem0-rs), [OpenMemory](https://mem0.ai/openmemory), [official MCP memory server](https://github.com/modelcontextprotocol/servers/tree/main/src/memory), [Karpathy LLM Wiki](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f), [LLM Wiki v2](https://gist.github.com/rohitg00/2067ab416f7bbe447c1977edaaa681e2), [llm-wiki](https://github.com/nvk/llm-wiki), [GBrain](https://github.com/garrytan/gbrain), [OpenClaw memory-wiki](https://docs.openclaw.ai/plugins/memory-wiki).
- Watch/freshness references: [GitHub REST API best practices](https://docs.github.com/rest/guides/best-practices-for-using-the-rest-api), [RSS/Atom feed caching best practices](https://www.ctrl.blog/entry/feed-caching.html), [X API rate limits](https://docs.x.com/x-api/fundamentals/rate-limits).
- Example launch sources: [Vercel introducing eve](https://vercel.com/blog/introducing-eve), [vercel/eve GitHub repository](https://github.com/vercel/eve), [Vercel eve docs](https://vercel.com/docs/eve), [Vercel Agent Stack](https://vercel.com/blog/agent-stack), [Vercel eve X announcement](https://x.com/vercel/status/2067180054979936413).
- Reference agents: [Perplexity Comet](https://www.perplexity.ai/comet/), [Comet getting started](https://www.perplexity.ai/comet/gettingstarted), [NVIDIA Project G-Assist](https://www.nvidia.com/en-us/software/nvidia-app/g-assist/), [NVIDIA G-Assist plugin guide](https://developer.nvidia.com/blog/building-g-assist-plugin-twitch-integration/), [NVIDIA AI-Q Blueprint](https://build.nvidia.com/nvidia/aiq), [NVIDIA AI-Q Research Assistant Blueprint](https://catalog.ngc.nvidia.com/orgs/nvidia/teams/blueprint/collections/ai-research-assistant-blueprint).
- Local inputs: `/Users/chabotc/Desktop/STYLE.md`, `/Users/chabotc/Projects/garderobe/wardrobe-mcp-design.md`, `/Users/chabotc/Projects/garderobe/README.md`, `/Users/chabotc/Projects/openclaw/docs/plugins/memory-wiki.md`, `/Users/chabotc/Projects/hermes-agent/README.md`.
- Gateway/agent references: [OpenClaw](https://github.com/openclaw/openclaw), [OpenClaw plugin architecture](https://docs.openclaw.ai/plugins/architecture), [OpenClaw Codex harness](https://docs.openclaw.ai/plugins/codex-harness), [Hermes Agent](https://github.com/NousResearch/hermes-agent).
- External APIs/tools: [Telegram Bot API](https://core.telegram.org/bots/api), [X OAuth2 PKCE](https://docs.x.com/fundamentals/authentication/oauth-2-0/user-access-token), [X bookmarks lookup](https://docs.x.com/x-api/posts/bookmarks/quickstart/bookmarks-lookup), [Gmail push notifications](https://developers.google.com/workspace/gmail/api/guides/push), [Gmail scopes](https://developers.google.com/workspace/gmail/api/auth/scopes), [Brave Search API](https://brave.com/search/api/), [Brave Search MCP server](https://github.com/brave/brave-search-mcp-server), [Perplexity Sonar API](https://docs.perplexity.ai/docs/sonar/quickstart), [Perplexity Search API](https://docs.perplexity.ai/api-reference/search-post), [Vercel Labs json-render](https://github.com/vercel-labs/json-render).
