use super::*;
use axum::body::to_bytes;
use std::io;

mod http_ops_controls;
mod http_ops_ui;
mod mcp_research_commerce_job;
mod mcp_sources_x;
mod mcp_tool_parity;
mod oauth_import_service;

struct BrokenPipeWriter;

impl Write for BrokenPipeWriter {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed pipe"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn test_paths(name: &str) -> AppPaths {
    AppPaths::new(std::env::temp_dir().join(format!(
        "arcwell-cli-test-{name}-{}",
        chrono::Utc::now().timestamp_nanos_opt().unwrap()
    )))
}

fn mock_base_server(body: &'static str, content_type: &'static str) -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buffer = [0_u8; 4096];
        let _ = stream.read(&mut buffer);
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: {content_type}\r\ncontent-length: {}\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).unwrap();
    });
    format!("http://{addr}")
}

fn test_http_state(name: &str, auth_token: Option<&str>) -> HttpState {
    HttpState::new(
        test_paths(name),
        auth_token.map(ToOwned::to_owned),
        8192,
        65536,
    )
    .unwrap()
}

async fn response_json(response: Response) -> (StatusCode, Value) {
    let status = response.status();
    let body = to_bytes(response.into_body(), 1_000_000).await.unwrap();
    (status, serde_json::from_slice(&body).unwrap())
}

async fn response_text(response: Response) -> (StatusCode, String) {
    let status = response.status();
    let body = to_bytes(response.into_body(), 1_000_000).await.unwrap();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

fn authed_local_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_static("Bearer local-auth-token-123"),
    );
    headers.insert(
        header::ORIGIN,
        HeaderValue::from_static("http://127.0.0.1:8787"),
    );
    headers
}

fn dead_letter_body(
    csrf_token: &str,
    idempotency_key: &str,
    edge_event_id: &str,
    reason: &str,
) -> String {
    format!(
        "csrf_token={}&idempotency_key={}&edge_event_id={}&reason={}",
        url_component(csrf_token),
        url_component(idempotency_key),
        url_component(edge_event_id),
        url_component(reason)
    )
}

fn x_bookmarks_schedule_body(
    csrf_token: &str,
    idempotency_key: &str,
    bookmark_days: i64,
    max_bookmarks: usize,
    cadence: &str,
    status: &str,
) -> String {
    format!(
        "csrf_token={}&idempotency_key={}&bookmark_days={}&max_bookmarks={}&cadence={}&status={}",
        url_component(csrf_token),
        url_component(idempotency_key),
        bookmark_days,
        max_bookmarks,
        url_component(cadence),
        url_component(status)
    )
}

fn x_bookmarks_enqueue_body(
    csrf_token: &str,
    idempotency_key: &str,
    bookmark_days: i64,
    max_bookmarks: usize,
) -> String {
    format!(
        "csrf_token={}&idempotency_key={}&bookmark_days={}&max_bookmarks={}",
        url_component(csrf_token),
        url_component(idempotency_key),
        bookmark_days,
        max_bookmarks
    )
}

fn x_watch_curation_run_body(csrf_token: &str, idempotency_key: &str, mode: &str) -> String {
    format!(
        "csrf_token={}&idempotency_key={}&mode={}",
        url_component(csrf_token),
        url_component(idempotency_key),
        url_component(mode)
    )
}

fn x_watch_curation_restore_body(
    csrf_token: &str,
    idempotency_key: &str,
    run_id: &str,
) -> String {
    format!(
        "csrf_token={}&idempotency_key={}&run_id={}",
        url_component(csrf_token),
        url_component(idempotency_key),
        url_component(run_id)
    )
}

fn knowledge_backlog_schedule_body(
    csrf_token: &str,
    idempotency_key: &str,
    max_source_cards: usize,
    min_group_size: usize,
    max_clusters: usize,
    cadence: &str,
    status: &str,
) -> String {
    format!(
        "csrf_token={}&idempotency_key={}&max_source_cards={}&min_group_size={}&max_clusters={}&cadence={}&status={}",
        url_component(csrf_token),
        url_component(idempotency_key),
        max_source_cards,
        min_group_size,
        max_clusters,
        url_component(cadence),
        url_component(status)
    )
}

fn knowledge_backlog_enqueue_body(
    csrf_token: &str,
    idempotency_key: &str,
    max_source_cards: usize,
    min_group_size: usize,
    max_clusters: usize,
) -> String {
    format!(
        "csrf_token={}&idempotency_key={}&max_source_cards={}&min_group_size={}&max_clusters={}",
        url_component(csrf_token),
        url_component(idempotency_key),
        max_source_cards,
        min_group_size,
        max_clusters
    )
}

fn knowledge_model_clusters_schedule_body(
    csrf_token: &str,
    idempotency_key: &str,
    query: &str,
    provider: &str,
    max_source_cards: usize,
    max_clusters: usize,
    cadence: &str,
    status: &str,
) -> String {
    format!(
        "csrf_token={}&idempotency_key={}&query={}&model_provider={}&model_name=&endpoint=&timeout_seconds=&max_source_cards={}&max_clusters={}&cadence={}&status={}",
        url_component(csrf_token),
        url_component(idempotency_key),
        url_component(query),
        url_component(provider),
        max_source_cards,
        max_clusters,
        url_component(cadence),
        url_component(status)
    )
}

fn knowledge_model_clusters_enqueue_body(
    csrf_token: &str,
    idempotency_key: &str,
    query: &str,
    provider: &str,
    max_source_cards: usize,
    max_clusters: usize,
) -> String {
    format!(
        "csrf_token={}&idempotency_key={}&query={}&model_provider={}&model_name=&endpoint=&timeout_seconds=&max_source_cards={}&max_clusters={}",
        url_component(csrf_token),
        url_component(idempotency_key),
        url_component(query),
        url_component(provider),
        max_source_cards,
        max_clusters
    )
}

fn knowledge_model_write_schedule_body(
    csrf_token: &str,
    idempotency_key: &str,
    cluster_id: &str,
    provider: &str,
    create_digest: bool,
    cadence: &str,
    status: &str,
) -> String {
    format!(
        "csrf_token={}&idempotency_key={}&cluster_id={}&model_provider={}&model_name=&endpoint=&timeout_seconds=&create_digest={}&cadence={}&status={}",
        url_component(csrf_token),
        url_component(idempotency_key),
        url_component(cluster_id),
        url_component(provider),
        create_digest,
        url_component(cadence),
        url_component(status)
    )
}

fn knowledge_model_write_enqueue_body(
    csrf_token: &str,
    idempotency_key: &str,
    cluster_id: &str,
    provider: &str,
    create_digest: bool,
) -> String {
    format!(
        "csrf_token={}&idempotency_key={}&cluster_id={}&model_provider={}&model_name=&endpoint=&timeout_seconds=&create_digest={}",
        url_component(csrf_token),
        url_component(idempotency_key),
        url_component(cluster_id),
        url_component(provider),
        create_digest
    )
}

fn knowledge_due_model_writes_body(
    csrf_token: &str,
    idempotency_key: &str,
    max_clusters: usize,
    provider: &str,
    create_digest: bool,
) -> String {
    format!(
        "csrf_token={}&idempotency_key={}&max_clusters={}&model_provider={}&model_name=&endpoint=&timeout_seconds=&create_digest={}",
        url_component(csrf_token),
        url_component(idempotency_key),
        max_clusters,
        url_component(provider),
        create_digest
    )
}

fn knowledge_entity_resolution_schedule_body(
    csrf_token: &str,
    idempotency_key: &str,
    max_pairs: usize,
    provider: &str,
    cadence: &str,
    status: &str,
) -> String {
    format!(
        "csrf_token={}&idempotency_key={}&model_provider={}&model_name=&endpoint=&timeout_seconds=&max_pairs={}&cadence={}&status={}",
        url_component(csrf_token),
        url_component(idempotency_key),
        url_component(provider),
        max_pairs,
        url_component(cadence),
        url_component(status)
    )
}

fn knowledge_entity_resolution_enqueue_body(
    csrf_token: &str,
    idempotency_key: &str,
    max_pairs: usize,
    provider: &str,
) -> String {
    format!(
        "csrf_token={}&idempotency_key={}&model_provider={}&model_name=&endpoint=&timeout_seconds=&max_pairs={}",
        url_component(csrf_token),
        url_component(idempotency_key),
        url_component(provider),
        max_pairs
    )
}

fn knowledge_due_clusters_body(
    csrf_token: &str,
    idempotency_key: &str,
    max_clusters: usize,
) -> String {
    format!(
        "csrf_token={}&idempotency_key={}&max_clusters={}",
        url_component(csrf_token),
        url_component(idempotency_key),
        max_clusters
    )
}

fn knowledge_cluster_promote_body(
    csrf_token: &str,
    idempotency_key: &str,
    cluster_id: &str,
    reviewer: &str,
    reason: &str,
) -> String {
    format!(
        "csrf_token={}&idempotency_key={}&cluster_id={}&reviewer={}&reason={}",
        url_component(csrf_token),
        url_component(idempotency_key),
        url_component(cluster_id),
        url_component(reviewer),
        url_component(reason)
    )
}

fn worker_run_once_body(csrf_token: &str, idempotency_key: &str, max_jobs: usize) -> String {
    format!(
        "csrf_token={}&idempotency_key={}&max_jobs={}",
        url_component(csrf_token),
        url_component(idempotency_key),
        max_jobs
    )
}
