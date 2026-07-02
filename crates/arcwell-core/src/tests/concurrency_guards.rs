use super::*;

// CLAIM: hot worker write paths (wiki job dedup enqueue, wiki job leasing, edge event
// leasing) must not double-lease or double-enqueue under concurrent access.
// These tests first characterize today's single-threaded behavior (it must keep
// passing unmodified), then Step 6 adds a genuine two-connection race test.
// SEVERITY: Severe because double delivery / double fetch is a direct cost and
// correctness bug across four worker entrypoints (launchd loop, CLI, ops UI, MCP).

#[test]
fn enqueue_wiki_job_dedup_returns_same_job_and_single_row() {
    let store = test_store("concurrency-guards-enqueue-dedup");
    let input = json!({ "url": "https://example.com/dedup-fixture" });

    let first = store.enqueue_wiki_job("rss_fetch", input.clone()).unwrap();
    let second = store.enqueue_wiki_job("rss_fetch", input.clone()).unwrap();

    assert_eq!(
        first.id, second.id,
        "duplicate enqueue must return the same job id"
    );

    let count: i64 = store
        .conn
        .query_row(
            "SELECT COUNT(*) FROM wiki_jobs WHERE kind = 'rss_fetch' AND input_json = ?1",
            params![serde_json::to_string(&input).unwrap()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        count, 1,
        "exactly one wiki_jobs row must exist for the deduped input"
    );
}

#[test]
fn claim_next_pending_job_does_not_release_a_live_lease() {
    let store = test_store("concurrency-guards-claim-lease");
    let enqueued = store
        .enqueue_wiki_job(
            "rss_fetch",
            json!({ "url": "https://example.com/claim-fixture" }),
        )
        .unwrap();

    let claimed = store.claim_next_pending_job().unwrap();
    let claimed = claimed.expect("expected the pending job to be claimed");
    assert_eq!(claimed.id, enqueued.id);
    assert_eq!(claimed.status, "running");

    // Immediate second claim attempt must not re-return the same job while its
    // lease (leased_until = now + 300s) is still live.
    let second_claim = store.claim_next_pending_job().unwrap();
    assert!(
        second_claim.is_none(),
        "a job with a live lease must not be claimable again, got: {second_claim:?}"
    );
}

#[test]
fn lease_edge_event_matching_does_not_release_a_live_lease() {
    let store = test_store("concurrency-guards-edge-lease");
    let enqueued = store
        .enqueue_edge_event(
            "test-source",
            "concurrency-guards-edge-fixture",
            json!({ "note": "fixture" }),
            3600,
        )
        .unwrap();

    let leased = store.lease_edge_event().unwrap();
    let leased = leased.expect("expected the pending edge event to be leased");
    assert_eq!(leased.id, enqueued.id);
    assert_eq!(leased.status, "leased");

    // Immediate second lease attempt must not re-return the same event while its
    // lease (leased_until = now + 300s) is still live.
    let second_lease = store.lease_edge_event().unwrap();
    assert!(
        second_lease.is_none(),
        "an edge event with a live lease must not be leasable again, got: {second_lease:?}"
    );
}

#[test]
fn enqueue_wiki_job_dedup_is_race_free_across_two_connections() {
    // Two independent Store::open connections against the same on-disk db,
    // racing to enqueue the identical (kind, input_json) job concurrently.
    // With busy_timeout (Step 1) + the BEGIN IMMEDIATE transaction guard
    // (Step 4), exactly one wiki_jobs row must exist afterwards.
    let paths = test_paths("concurrency-guards-race-dedup");
    paths.ensure().unwrap();
    // Pre-create the schema with a single connection so the two racing threads
    // below contend only on the enqueue_wiki_job write path, not on concurrent
    // first-time schema migration (out of scope for this plan).
    drop(Store::open(paths.clone()).unwrap());

    let input = json!({ "url": "https://example.com/race-fixture" });

    let paths_a = paths.clone();
    let input_a = input.clone();
    let handle_a = thread::spawn(move || {
        let store = Store::open(paths_a).unwrap();
        store.enqueue_wiki_job("rss_fetch", input_a).unwrap()
    });

    let paths_b = paths.clone();
    let input_b = input.clone();
    let handle_b = thread::spawn(move || {
        let store = Store::open(paths_b).unwrap();
        store.enqueue_wiki_job("rss_fetch", input_b).unwrap()
    });

    let job_a = handle_a.join().expect("connection A thread panicked");
    let job_b = handle_b.join().expect("connection B thread panicked");

    assert_eq!(
        job_a.id, job_b.id,
        "both connections must observe the same deduped job id"
    );

    let store = Store::open(paths).unwrap();
    let count: i64 = store
        .conn
        .query_row(
            "SELECT COUNT(*) FROM wiki_jobs WHERE kind = 'rss_fetch' AND input_json = ?1",
            params![serde_json::to_string(&input).unwrap()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        count, 1,
        "exactly one wiki_jobs row must exist after the race"
    );
}
