use std::mem::size_of;
use std::time::Duration;

use z00z_wallets::receiver::{Tag16Cache, Tag16Context};

#[test]
#[ignore = "memory stress test"]
fn test_scanner_mem() {
    let mut cache = Tag16Cache::new();
    for idx in 0..10_000u32 {
        let mut req_id = [0u8; 32];
        req_id[0..4].copy_from_slice(&idx.to_le_bytes());
        cache.add_active_request(req_id);

        let tag = u16::from_le_bytes([req_id[0], req_id[1]]);
        cache.insert(
            tag,
            Tag16Context {
                k_dh: req_id,
                req_id: Some(req_id),
            },
        );
    }

    let stats = cache.stats();
    let mem_est = size_of::<Tag16Context>() * 10_000usize;
    assert_eq!(
        stats.size, 10_000,
        "cache must contain one bucket per request"
    );
    assert_eq!(
        stats.collisions, 0,
        "memory stress setup must not create collisions"
    );
    assert!(
        mem_est < 10 * 1024 * 1024,
        "cache memory estimate must stay <10MB"
    );

    let start = std::time::Instant::now();
    let mut hits = 0usize;
    for idx in 0..10_000u32 {
        let req_id = idx.to_le_bytes();
        let tag = u16::from_le_bytes([req_id[0], req_id[1]]);
        if cache.contains(tag) {
            hits = hits.saturating_add(1);
        }
    }

    assert_eq!(hits, 10_000, "all inserted cache entries must be reachable");
    assert!(
        start.elapsed() < Duration::from_secs(1),
        "lookup must stay near O(1)"
    );
}
