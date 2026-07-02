use super::*;

pub(crate) fn canonical_source_url(raw: &str) -> Result<String> {
    let mut url = validate_public_http_url(raw)?;
    url.set_fragment(None);
    let scheme = url.scheme().to_ascii_lowercase();
    let host = url
        .host_str()
        .map(str::to_ascii_lowercase)
        .context("URL must include a host")?;
    url.set_scheme(&scheme)
        .map_err(|_| anyhow::anyhow!("invalid URL scheme"))?;
    url.set_host(Some(&host))?;
    if (url.scheme() == "https" && url.port() == Some(443))
        || (url.scheme() == "http" && url.port() == Some(80))
    {
        url.set_port(None)
            .map_err(|_| anyhow::anyhow!("invalid URL port"))?;
    }
    Ok(url.to_string())
}

const URL_INGEST_MAX_BYTES: u64 = 1_000_000;
const URL_INGEST_MAX_REDIRECTS: usize = 5;

#[derive(Debug, Clone)]
pub(crate) struct ResearchUrlIngestContext {
    pub(crate) run_id: String,
    pub(crate) host_search_id: Option<String>,
    pub(crate) host_search_result: Option<ResearchHostSearchResult>,
    pub(crate) source_family: String,
    pub(crate) source_type: String,
}

#[derive(Debug)]
pub(crate) struct UrlIngestDocument {
    pub(crate) requested_url: String,
    pub(crate) final_url: String,
    pub(crate) canonical_url: String,
    pub(crate) content_type: String,
    pub(crate) byte_len: usize,
    pub(crate) title: String,
    pub(crate) readable_text: String,
    pub(crate) source_excerpt: String,
    pub(crate) extraction_method: String,
    pub(crate) robots_meta: Option<String>,
    pub(crate) robots_noindex: bool,
    pub(crate) robots_nofollow: bool,
    pub(crate) crawl_rate_policy: String,
    pub(crate) captured_at: Option<String>,
    pub(crate) browser: Option<String>,
    pub(crate) screenshot_path: Option<String>,
}

pub(crate) fn research_url_ingest_claim_text(readable_text: &str) -> Option<String> {
    for sentence in readable_text
        .split_terminator(['.', '!', '?', '\n'])
        .map(str::trim)
        .filter(|sentence| sentence.len() >= 40)
    {
        let lower = sentence.to_ascii_lowercase();
        if contains_prompt_injection_text(&lower) {
            continue;
        }
        let mut text = excerpt(sentence, 900);
        if !matches!(text.chars().last(), Some('.') | Some('!') | Some('?')) {
            text.push('.');
        }
        return Some(text);
    }
    let fallback = excerpt(readable_text.trim(), 900);
    if fallback.len() >= 40 && !contains_prompt_injection_text(&fallback.to_ascii_lowercase()) {
        Some(fallback)
    } else {
        None
    }
}

pub(crate) fn fetch_url_ingest_document(url: Url) -> Result<UrlIngestDocument> {
    let requested_url = canonical_source_url(url.as_str())?;
    let client = Client::builder()
        .timeout(Duration::from_secs(20))
        .redirect(Policy::none())
        .build()?;
    let mut current = url;
    for redirect_count in 0..=URL_INGEST_MAX_REDIRECTS {
        let mut response = client
            .get(current.clone())
            .header(ACCEPT, "text/html, text/markdown, text/plain")
            .header("user-agent", "arcwell/0.1")
            .send()
            .context("url ingest request failed")?;
        if response.status().is_redirection() {
            if redirect_count == URL_INGEST_MAX_REDIRECTS {
                bail!("url ingest exceeded redirect limit");
            }
            let location = response
                .headers()
                .get(LOCATION)
                .and_then(|value| value.to_str().ok())
                .context("url ingest redirect missing Location header")?;
            current = validate_redirect_fetch_url(&current, location)?;
            continue;
        }
        response = response
            .error_for_status()
            .context("url ingest returned an error status")?;
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(|value| {
                value
                    .split(';')
                    .next()
                    .unwrap_or(value)
                    .trim()
                    .to_ascii_lowercase()
            })
            .context("url ingest response missing content-type")?;
        if !is_allowed_url_ingest_content_type(&content_type) {
            bail!("url ingest rejected content-type: {content_type}");
        }
        if let Some(length) = response
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok())
            && length > URL_INGEST_MAX_BYTES
        {
            bail!("url body is too large");
        }
        let mut bytes = Vec::new();
        let mut limited = response.take(URL_INGEST_MAX_BYTES + 1);
        limited
            .read_to_end(&mut bytes)
            .context("reading url ingest response")?;
        if bytes.len() as u64 > URL_INGEST_MAX_BYTES {
            bail!("url body is too large");
        }
        let body = String::from_utf8(bytes).context("url ingest returned invalid utf-8 text")?;
        let extraction = if content_type.contains("html") {
            html_to_readable_text(&body)
        } else {
            ReadableHtmlExtraction {
                text: normalize_readable_text(&body),
                method: "plain-text".to_string(),
            }
        };
        let readable_text = extraction.text;
        if readable_text.trim().is_empty() {
            bail!("url ingest did not contain readable text");
        }
        let title = html_title(&body)
            .or_else(|| markdown_title(&readable_text))
            .unwrap_or_else(|| current.to_string());
        let final_url = current.to_string();
        let canonical_url = if content_type.contains("html") {
            html_canonical_link(&body, &current)
                .and_then(|url| canonical_source_url(url.as_str()).ok())
                .unwrap_or_else(|| canonical_source_url(&final_url).expect("final URL validated"))
        } else {
            canonical_source_url(&final_url)?
        };
        let robots_meta = if content_type.contains("html") {
            html_meta_robots(&body)
        } else {
            None
        };
        let robots_tokens = robots_meta
            .as_deref()
            .map(parse_robots_directives)
            .unwrap_or_default();
        let robots_noindex = robots_tokens.contains("noindex");
        let robots_nofollow = robots_tokens.contains("nofollow");
        return Ok(UrlIngestDocument {
            requested_url,
            final_url,
            canonical_url,
            content_type,
            byte_len: body.len(),
            title: excerpt(&title, 200),
            readable_text,
            source_excerpt: excerpt(&body, 20_000),
            extraction_method: extraction.method,
            robots_meta,
            robots_noindex,
            robots_nofollow,
            crawl_rate_policy:
                "single manual fetch; scheduled pollers use source-health next_run_at backoff"
                    .to_string(),
            captured_at: None,
            browser: None,
            screenshot_path: None,
        });
    }
    unreachable!("redirect loop returns or bails")
}

pub(crate) fn validate_redirect_fetch_url(base: &Url, location: &str) -> Result<Url> {
    let next = base
        .join(location)
        .context("url ingest redirect was invalid")?;
    validate_fetch_url(next.as_str())
}

pub(crate) fn is_allowed_url_ingest_content_type(content_type: &str) -> bool {
    matches!(
        content_type,
        "text/html" | "application/xhtml+xml" | "text/plain" | "text/markdown"
    )
}

pub(crate) fn validate_rendered_page_snapshot_input(
    input: &RenderedPageSnapshotInput,
) -> Result<()> {
    validate_fetch_url(&input.requested_url)?;
    if let Some(final_url) = input.final_url.as_deref() {
        validate_fetch_url(final_url)?;
    }
    let has_html = input
        .rendered_html
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty());
    let has_text = input
        .rendered_text
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty());
    if !has_html && !has_text {
        bail!("rendered page snapshot requires rendered_html or rendered_text");
    }
    if let Some(title) = input.title.as_deref()
        && title.trim().is_empty()
    {
        bail!("rendered page snapshot title cannot be empty");
    }
    if input.title.as_ref().is_some_and(|value| value.len() > 500) {
        bail!("rendered page snapshot title is too long");
    }
    if input
        .rendered_html
        .as_ref()
        .is_some_and(|value| value.len() as u64 > URL_INGEST_MAX_BYTES)
    {
        bail!("rendered page snapshot html is too large");
    }
    if input
        .rendered_text
        .as_ref()
        .is_some_and(|value| value.len() > 200_000)
    {
        bail!("rendered page snapshot text is too large");
    }
    if let Some(captured_at) = input.captured_at.as_deref() {
        DateTime::parse_from_rfc3339(captured_at)
            .with_context(|| format!("invalid rendered page captured_at: {captured_at}"))?;
    }
    if let Some(browser) = input.browser.as_deref() {
        let browser = browser.trim();
        if browser.is_empty() {
            bail!("rendered page browser cannot be empty");
        }
        if browser.len() > 120 || browser.chars().any(char::is_control) {
            bail!("rendered page browser is invalid");
        }
    }
    if let Some(path) = input.screenshot_path.as_deref() {
        if path.trim().is_empty() {
            bail!("rendered page screenshot_path cannot be empty");
        }
        if path.len() > 2_000 || path.contains('\0') || path.split('/').any(|part| part == "..") {
            bail!("rendered page screenshot_path is invalid");
        }
    }
    Ok(())
}

pub(crate) fn rendered_page_snapshot_document(
    input: &RenderedPageSnapshotInput,
) -> Result<UrlIngestDocument> {
    validate_rendered_page_snapshot_input(input)?;
    let requested = validate_fetch_url(&input.requested_url)?;
    let final_url = input.final_url.as_deref().unwrap_or(&input.requested_url);
    let final_url = validate_fetch_url(final_url)?;
    let requested_url = canonical_source_url(requested.as_str())?;
    let final_url_string = final_url.to_string();
    let html = input.rendered_html.as_deref();
    let text = input.rendered_text.as_deref();
    let extraction = if let Some(text) = text.filter(|value| !value.trim().is_empty()) {
        ReadableHtmlExtraction {
            text: normalize_readable_text(text),
            method: "host-browser-rendered-text".to_string(),
        }
    } else if let Some(html) = html {
        let mut extraction = html_to_readable_text(html);
        extraction.method = format!("host-browser-rendered-{}", extraction.method);
        extraction
    } else {
        unreachable!("validated snapshot has html or text")
    };
    if extraction.text.trim().is_empty() {
        bail!("rendered page snapshot did not contain readable text");
    }
    let title = input
        .title
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| html.and_then(html_title))
        .or_else(|| markdown_title(&extraction.text))
        .unwrap_or_else(|| final_url_string.clone());
    let canonical_url = if let Some(html) = html {
        html_canonical_link(html, &final_url)
            .and_then(|url| canonical_source_url(url.as_str()).ok())
            .unwrap_or_else(|| {
                canonical_source_url(&final_url_string).expect("final URL already validated")
            })
    } else {
        canonical_source_url(&final_url_string)?
    };
    let robots_meta = html.and_then(html_meta_robots);
    let robots_tokens = robots_meta
        .as_deref()
        .map(parse_robots_directives)
        .unwrap_or_default();
    let source = html.or(text).unwrap_or_default();
    Ok(UrlIngestDocument {
        requested_url,
        final_url: final_url_string,
        canonical_url,
        content_type: if html.is_some() {
            "text/html".to_string()
        } else {
            "text/plain".to_string()
        },
        byte_len: source.len(),
        title: excerpt(&title, 200),
        readable_text: extraction.text,
        source_excerpt: excerpt(source, 20_000),
        extraction_method: extraction.method,
        robots_meta,
        robots_noindex: robots_tokens.contains("noindex"),
        robots_nofollow: robots_tokens.contains("nofollow"),
        crawl_rate_policy:
            "host-supplied rendered snapshot; Arcwell performed no browser or network fetch"
                .to_string(),
        captured_at: input.captured_at.clone(),
        browser: input.browser.clone(),
        screenshot_path: input.screenshot_path.clone(),
    })
}

pub(crate) fn render_url_ingest_page(doc: &UrlIngestDocument) -> String {
    let mut markdown = String::new();
    markdown.push_str(&format!(
        "# {}\n\n",
        escape_untrusted_markdown_text(&doc.title)
    ));
    markdown.push_str(untrusted_evidence_notice("Retrieved URL content below"));
    markdown.push_str("## Provenance\n\n");
    markdown.push_str(&format!("- Requested URL: <{}>\n", doc.requested_url));
    markdown.push_str(&format!("- Final URL: <{}>\n", doc.final_url));
    markdown.push_str(&format!("- Canonical URL: <{}>\n", doc.canonical_url));
    markdown.push_str(&format!("- Content-Type: `{}`\n", doc.content_type));
    markdown.push_str(&format!("- Bytes read: `{}`\n", doc.byte_len));
    markdown.push_str(&format!(
        "- Extraction method: `{}`\n",
        doc.extraction_method
    ));
    if let Some(captured_at) = &doc.captured_at {
        markdown.push_str(&format!(
            "- Captured at: `{}`\n",
            escape_untrusted_markdown_text(captured_at)
        ));
    }
    if let Some(browser) = &doc.browser {
        markdown.push_str(&format!(
            "- Browser: `{}`\n",
            escape_untrusted_markdown_text(browser)
        ));
    }
    if let Some(screenshot_path) = &doc.screenshot_path {
        markdown.push_str(&format!(
            "- Screenshot path: `{}`\n",
            escape_untrusted_markdown_text(screenshot_path)
        ));
    }
    if let Some(robots_meta) = &doc.robots_meta {
        markdown.push_str(&format!(
            "- Robots meta: `{}`\n",
            escape_untrusted_markdown_text(robots_meta)
        ));
    } else {
        markdown.push_str("- Robots meta: `not declared in fetched document`\n");
    }
    markdown.push_str(&format!("- Robots noindex: `{}`\n", doc.robots_noindex));
    markdown.push_str(&format!("- Robots nofollow: `{}`\n", doc.robots_nofollow));
    markdown.push_str(&format!(
        "- Crawl-rate policy: `{}`\n\n",
        escape_untrusted_markdown_text(&doc.crawl_rate_policy)
    ));
    markdown.push_str("## Readable Text\n\n");
    markdown.push_str(&escape_untrusted_markdown_text(&doc.readable_text));
    markdown.push_str("\n\n## Escaped Source Excerpt\n\n```text\n");
    markdown.push_str(&escape_html_fragment(&doc.source_excerpt));
    markdown.push_str("\n```\n");
    markdown
}

pub(crate) fn html_title(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let start = lower.find("<title")?;
    let after_tag = lower[start..].find('>')? + start + 1;
    let end = lower[after_tag..].find("</title>")? + after_tag;
    let title = html[after_tag..end].trim();
    if title.is_empty() {
        None
    } else {
        Some(html_unescape_basic(title))
    }
}

#[derive(Debug)]
pub(crate) struct ReadableHtmlExtraction {
    pub(crate) text: String,
    pub(crate) method: String,
}

pub(crate) fn html_to_readable_text(html: &str) -> ReadableHtmlExtraction {
    let cleaned = strip_non_content_html_blocks(html);
    for (element, method) in [
        ("article", "html-article"),
        ("main", "html-main"),
        ("body", "html-body"),
    ] {
        if let Some(fragment) = first_html_element_block(&cleaned, element) {
            let text = html_fragment_to_text(&fragment);
            if text.len() >= 40 {
                return ReadableHtmlExtraction {
                    text,
                    method: method.to_string(),
                };
            }
        }
    }
    ReadableHtmlExtraction {
        text: html_fragment_to_text(&cleaned),
        method: "html-document".to_string(),
    }
}

pub(crate) fn strip_non_content_html_blocks(html: &str) -> String {
    [
        "script", "style", "noscript", "svg", "nav", "header", "footer", "aside", "form",
    ]
    .iter()
    .fold(html.to_string(), |content, element| {
        strip_html_element_blocks(&content, element)
    })
}

pub(crate) fn first_html_element_block(html: &str, element: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let open = format!("<{element}");
    let start = lower.find(&open)?;
    let after_tag = lower[start..].find('>')? + start + 1;
    let close = format!("</{element}>");
    let end = lower[after_tag..].find(&close)? + after_tag;
    Some(html[after_tag..end].to_string())
}

pub(crate) fn html_fragment_to_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => {
                in_tag = true;
                out.push(' ');
            }
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    normalize_readable_text(&html_unescape_basic(&out))
}

pub(crate) fn html_canonical_link(html: &str, base: &Url) -> Option<Url> {
    for tag in html_start_tags(html, "link") {
        let Some(rel) = html_attr_value(&tag, "rel") else {
            continue;
        };
        if !rel
            .split_whitespace()
            .any(|item| item.eq_ignore_ascii_case("canonical"))
        {
            continue;
        }
        let Some(href) = html_attr_value(&tag, "href") else {
            continue;
        };
        if let Ok(url) = base.join(&href)
            && validate_public_http_url(url.as_str()).is_ok()
            && (!is_blocked_fetch_host(&url) || url.host_str() == base.host_str())
        {
            return Some(url);
        }
    }
    None
}

pub(crate) fn html_meta_robots(html: &str) -> Option<String> {
    for tag in html_start_tags(html, "meta") {
        let name = html_attr_value(&tag, "name").unwrap_or_default();
        let property = html_attr_value(&tag, "property").unwrap_or_default();
        if !name.eq_ignore_ascii_case("robots") && !property.eq_ignore_ascii_case("robots") {
            continue;
        }
        let content = html_attr_value(&tag, "content")?;
        if !content.trim().is_empty() {
            return Some(excerpt(&content, 500));
        }
    }
    None
}

pub(crate) fn parse_robots_directives(content: &str) -> BTreeSet<String> {
    content
        .split([',', ';'])
        .filter_map(|token| {
            let token = token.trim().to_ascii_lowercase();
            if token.is_empty() { None } else { Some(token) }
        })
        .collect()
}

pub(crate) fn html_start_tags(html: &str, element: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let mut remaining = html;
    let open = format!("<{element}");
    loop {
        let lower = remaining.to_ascii_lowercase();
        let Some(start) = lower.find(&open) else {
            break;
        };
        let Some(end_offset) = lower[start..].find('>') else {
            break;
        };
        let end = start + end_offset + 1;
        tags.push(remaining[start..end].to_string());
        remaining = &remaining[end..];
    }
    tags
}

pub(crate) fn html_attr_value(tag: &str, attr: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let mut cursor = 0;
    let attr_lower = attr.to_ascii_lowercase();
    while let Some(offset) = lower[cursor..].find(&attr_lower) {
        let start = cursor + offset;
        let before_ok = start == 0
            || lower
                .as_bytes()
                .get(start.wrapping_sub(1))
                .is_some_and(|ch| ch.is_ascii_whitespace() || matches!(*ch, b'<' | b'/'));
        let after_attr = start + attr_lower.len();
        let after = lower.as_bytes().get(after_attr)?;
        if !before_ok || !after.is_ascii_whitespace() && *after != b'=' {
            cursor = after_attr;
            continue;
        }
        let rest = &tag[after_attr..];
        let rest_trimmed = rest.trim_start();
        if !rest_trimmed.starts_with('=') {
            cursor = after_attr;
            continue;
        }
        let value = rest_trimmed[1..].trim_start();
        let mut chars = value.chars();
        let quote = chars.next()?;
        if quote == '"' || quote == '\'' {
            let body = &value[quote.len_utf8()..];
            let end = body.find(quote)?;
            return Some(html_unescape_basic(&body[..end]));
        }
        let end = value
            .find(|ch: char| ch.is_whitespace() || ch == '>')
            .unwrap_or(value.len());
        return Some(html_unescape_basic(&value[..end]));
    }
    None
}

pub(crate) fn strip_html_element_blocks(html: &str, element: &str) -> String {
    let mut remaining = html;
    let mut out = String::with_capacity(html.len());
    let open = format!("<{element}");
    let close = format!("</{element}>");
    loop {
        let lower = remaining.to_ascii_lowercase();
        let Some(start) = lower.find(&open) else {
            out.push_str(remaining);
            break;
        };
        out.push_str(&remaining[..start]);
        let Some(end_offset) = lower[start..].find(&close) else {
            break;
        };
        let end = start + end_offset + close.len();
        remaining = &remaining[end..];
    }
    out
}

pub(crate) fn normalize_readable_text(text: &str) -> String {
    let mut out = String::new();
    let mut last_space = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            if !last_space {
                out.push(' ');
                last_space = true;
            }
        } else {
            out.push(ch);
            last_space = false;
        }
    }
    out.trim().to_string()
}

pub(crate) fn html_unescape_basic(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

pub(crate) fn escape_html_fragment(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub(crate) fn render_typed_source_card(
    input: &SourceCardInput,
    retrieved_at: &str,
) -> Result<String> {
    let mut markdown = String::new();
    markdown.push_str(&format!(
        "# Source Card: {}\n\n",
        escape_untrusted_markdown_text(&input.title)
    ));
    markdown.push_str(untrusted_evidence_notice("Source text and claims below"));
    markdown.push_str(&format!("- URL: <{}>\n", input.url));
    markdown.push_str(&format!("- Source type: `{}`\n", input.source_type));
    markdown.push_str(&format!("- Provider: `{}`\n", input.provider));
    markdown.push_str(&format!(
        "- Source-card schema: `v{}`\n",
        input
            .metadata
            .get("schema_version")
            .and_then(Value::as_u64)
            .unwrap_or(SOURCE_CARD_SCHEMA_VERSION)
    ));
    markdown.push_str(&format!(
        "- Evidence role: `{}`\n",
        input
            .metadata
            .get("source_role")
            .and_then(Value::as_str)
            .unwrap_or("secondary")
    ));
    markdown.push_str(&format!(
        "- Trust level: `{}`\n",
        input
            .metadata
            .get("trust_level")
            .and_then(Value::as_str)
            .unwrap_or("medium")
    ));
    markdown.push_str(&format!(
        "- Reliability score: `{:.2}`\n",
        input
            .metadata
            .get("reliability_score")
            .and_then(Value::as_f64)
            .unwrap_or(0.5)
    ));
    markdown.push_str(&format!(
        "- Provenance strength: `{}`\n",
        input
            .metadata
            .get("provenance_strength")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    ));
    if let Some(owner) = input.metadata.get("source_owner").and_then(Value::as_str) {
        markdown.push_str(&format!(
            "- Source owner: `{}`\n",
            escape_untrusted_markdown_text(owner)
        ));
    }
    if let Some(policy) = input
        .metadata
        .get("crawl_rate_policy")
        .and_then(Value::as_str)
    {
        markdown.push_str(&format!(
            "- Crawl-rate policy: `{}`\n",
            escape_untrusted_markdown_text(policy)
        ));
    }
    markdown.push_str(&format!("- Retrieved: `{retrieved_at}`\n\n"));
    markdown.push_str("## Summary\n\n");
    markdown.push_str(&escape_untrusted_markdown_text(&input.summary));
    markdown.push_str("\n\n## Claims\n\n");
    if input.claims.is_empty() {
        markdown.push_str("- No claims extracted yet.\n");
    } else {
        for claim in &input.claims {
            markdown.push_str(&format!(
                "- [{} {:.2}] {}\n",
                claim.kind,
                claim.confidence,
                escape_untrusted_markdown_text(&claim.claim)
            ));
        }
    }
    if input.metadata != Value::Null {
        let flags = source_card_metadata_strings(&input.metadata, "quality_flags");
        if !flags.is_empty() {
            markdown.push_str("\n## Audit Flags\n\n");
            for flag in flags {
                markdown.push_str(&format!("- `{flag}`\n"));
            }
        }
        markdown.push_str("\n## Metadata\n\n");
        markdown.push_str(&render_untrusted_json_code_block(&input.metadata)?);
    }
    Ok(markdown)
}

pub(crate) fn render_expanded_wiki_page(
    topic: &str,
    source_cards: &[SourceCard],
    pages: &[WikiPageSummary],
) -> Result<String> {
    let mut markdown = String::new();
    markdown.push_str(&format!(
        "# Expanded: {}\n\n",
        escape_untrusted_markdown_text(topic)
    ));
    markdown.push_str(&format!("Generated: {}\n\n", now()));
    markdown.push_str(
        "> Generated page: use this as a draft synthesis only. It is not primary evidence; cite the source cards and source links below.\n\n",
    );
    markdown.push_str("## Summary\n\n");
    if source_cards.is_empty() && pages.is_empty() {
        markdown.push_str("No local source cards or wiki pages matched this topic yet.\n\n");
    } else {
        markdown.push_str("This page is an expansion scaffold generated from local source cards and wiki pages. Treat it as a draft until audited.\n\n");
    }
    markdown.push_str("## Source Cards\n\n");
    if source_cards.is_empty() {
        markdown.push_str("- None found.\n");
    } else {
        for card in source_cards {
            markdown.push_str(&format!(
                "- `{}` [{}]({}) via `{}`\n",
                card.id,
                escape_markdown_link_text(&card.title),
                card.url,
                card.provider
            ));
            for claim in card.claims.iter().take(5) {
                markdown.push_str(&format!(
                    "  - [{} {:.2}] {}\n",
                    claim.kind,
                    claim.confidence,
                    escape_untrusted_markdown_text(&claim.claim)
                ));
            }
        }
    }
    let mut audit_findings = Vec::new();
    for card in source_cards {
        audit_findings.extend(audit_source_card(card));
    }
    audit_findings.extend(detect_source_contradictions(source_cards));
    markdown.push_str("\n## Evidence Audit\n\n");
    if audit_findings.is_empty() {
        markdown.push_str("- No local audit findings for selected source cards.\n");
    } else {
        for finding in &audit_findings {
            markdown.push_str(&format!(
                "- `{}` `{}` {}\n",
                finding.severity,
                finding.code,
                escape_untrusted_markdown_text(&finding.message)
            ));
        }
    }
    markdown.push_str("\n## Related Wiki Pages\n\n");
    if pages.is_empty() {
        markdown.push_str("- None found.\n");
    } else {
        for page in pages {
            markdown.push_str(&format!(
                "- `{}`: {}\n",
                page.id,
                escape_untrusted_markdown_text(&page.title)
            ));
        }
    }
    markdown.push_str("\n## Gaps\n\n");
    markdown
        .push_str("- Check primary sources and current web search before using this externally.\n");
    markdown.push_str("- Add contradiction notes and dated source cards for new claims.\n");
    Ok(markdown)
}

pub(crate) fn render_x_report(
    query: Option<&str>,
    items: &[XItem],
    links: &[XReportLink],
) -> String {
    let mut markdown = String::new();
    markdown.push_str("# X Import Report\n\n");
    markdown.push_str(&format!("Generated: {}\n\n", now()));
    if let Some(query) = query {
        markdown.push_str(&format!(
            "Query: `{}`\n\n",
            escape_untrusted_markdown_text(query)
        ));
    }
    markdown.push_str(untrusted_evidence_notice("Source text and claims below"));
    markdown.push_str(&format!("Items: {}\n\n", items.len()));
    markdown.push_str("## Items\n\n");
    if items.is_empty() {
        markdown.push_str("- No matching X items.\n");
    } else {
        let mut links_by_tweet: BTreeMap<String, Vec<&XReportLink>> = BTreeMap::new();
        for link in links {
            links_by_tweet
                .entry(link.tweet_x_id.clone())
                .or_default()
                .push(link);
        }
        for item in items {
            markdown.push_str(&format!(
                "- [{}]({}) by `@{}`\n  - Source: {}\n  - Stats: {}\n  - {}\n",
                item.x_id,
                item.url,
                escape_untrusted_markdown_text(&item.author),
                escape_untrusted_markdown_text(&x_sources_summary(item)),
                escape_untrusted_markdown_text(&x_metrics_summary(&item.metrics)),
                escape_untrusted_markdown_text(&item.text)
            ));
            if let Some(item_links) = links_by_tweet.get(&item.x_id) {
                markdown.push_str("  - Links:\n");
                for link in item_links {
                    let label = link.display_url.as_deref().unwrap_or(link.url.as_str());
                    let mut details = vec![
                        format!("source `{}`", escape_untrusted_markdown_text(&link.source)),
                        format!(
                            "expansion `{}`",
                            escape_untrusted_markdown_text(&link.expansion_status)
                        ),
                    ];
                    if let Some(wiki_page_id) = &link.wiki_page_id {
                        details.push(format!(
                            "wiki `{}`",
                            escape_untrusted_markdown_text(wiki_page_id)
                        ));
                    }
                    if let Some(final_url) = &link.final_url {
                        details.push(format!(
                            "final {}",
                            escape_untrusted_markdown_text(final_url)
                        ));
                    }
                    if let Some(error) = &link.last_error {
                        details.push(format!(
                            "error {}",
                            escape_untrusted_markdown_text(&excerpt(error, 160))
                        ));
                    }
                    markdown.push_str(&format!(
                        "    - [{}]({}) - {}\n",
                        escape_untrusted_markdown_text(label),
                        escape_untrusted_markdown_text(&link.url),
                        details.join("; ")
                    ));
                }
            }
        }
    }
    markdown
}

pub(crate) fn render_x_research_brief(
    query: &str,
    generated_at: &str,
    items: &[XResearchBriefItem],
) -> String {
    let mut markdown = String::new();
    markdown.push_str("# X Research Brief\n\n");
    markdown.push_str(&format!("Generated: `{generated_at}`\n\n"));
    markdown.push_str(&format!(
        "Query: `{}`\n\n",
        escape_untrusted_markdown_text(query)
    ));
    markdown.push_str(untrusted_evidence_notice("Source text and claims below"));
    markdown.push_str(
        "> Scope: local-only brief over already-imported X tweets. No browser, provider, live thread lookup, model synthesis, or durable writes were performed.\n\n",
    );
    markdown.push_str(&format!("Items: {}\n\n", items.len()));
    markdown.push_str("## Evidence\n\n");
    for item in items {
        markdown.push_str(&format!(
            "- Tweet `{}` by `@{}`: [{}]({})\n",
            escape_untrusted_markdown_text(&item.x_id),
            escape_untrusted_markdown_text(&item.author),
            escape_markdown_link_text(&item.url),
            item.url
        ));
        markdown.push_str(&format!(
            "  - Source card: `{}`\n",
            escape_untrusted_markdown_text(&item.source_card_id)
        ));
        if let Some(wiki_page_id) = &item.wiki_page_id {
            markdown.push_str(&format!(
                "  - Wiki page: `{}`\n",
                escape_untrusted_markdown_text(wiki_page_id)
            ));
        }
        if let Some(created_at) = &item.created_at {
            markdown.push_str(&format!(
                "  - Created: `{}`\n",
                escape_untrusted_markdown_text(created_at)
            ));
        }
        markdown.push_str(&format!(
            "  - Quote: > {}\n",
            escape_untrusted_markdown_text(&item.quote)
        ));
        if !item.thread_context.is_empty() {
            markdown.push_str("  - Local thread context:\n");
            for thread_tweet in &item.thread_context {
                markdown.push_str(&format!(
                    "    - `{}` relation `{}` depth {} source-card `{}`; quote: > {}\n",
                    escape_untrusted_markdown_text(&thread_tweet.x_id),
                    escape_untrusted_markdown_text(&thread_tweet.relation_to_root),
                    thread_tweet.depth,
                    escape_untrusted_markdown_text(&thread_tweet.source_card_id),
                    escape_untrusted_markdown_text(&thread_tweet.quote)
                ));
            }
        }
    }
    markdown.push_str("\n## Gaps\n\n");
    markdown.push_str("- This is not a completed deep-research report.\n");
    markdown.push_str("- Missing local thread context is not fetched live in this mode.\n");
    markdown.push_str(
        "- Claims must be promoted into source-card-backed research artifacts before external use.\n",
    );
    markdown
}

pub(crate) fn x_sources_summary(item: &XItem) -> String {
    let sources = item
        .sources
        .iter()
        .map(|source| match &source.source_detail {
            Some(detail) if !detail.is_empty() => {
                format!("{} ({detail})", source.source_kind)
            }
            _ => source.source_kind.clone(),
        })
        .collect::<Vec<_>>();
    if sources.is_empty() {
        "unknown".to_string()
    } else {
        sources.join(", ")
    }
}

pub(crate) fn x_metrics_summary(metrics: &Value) -> String {
    let Some(object) = metrics.as_object() else {
        return "none recorded".to_string();
    };
    let mut parts = Vec::new();
    for key in [
        "like_count",
        "reply_count",
        "retweet_count",
        "quote_count",
        "bookmark_count",
        "impression_count",
    ] {
        if let Some(value) = object.get(key).and_then(Value::as_i64) {
            parts.push(format!("{key}={value}"));
        }
    }
    if parts.is_empty() {
        "none recorded".to_string()
    } else {
        parts.join(", ")
    }
}
