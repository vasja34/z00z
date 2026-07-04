#![allow(dead_code)]

use std::{fs, path::Path};

use z00z_aggregators::{AggregatorId, BatchRoute, ShardId, ShardPlacement, ShardRouteTable};
use z00z_rollup_node::NodeConfig;
use z00z_utils::io;

#[path = "../../../z00z_rollup_node/tests/support/test_hjmt_home.rs"]
mod hjmt_test_home;

pub use hjmt_test_home::TestAgg;

use hjmt_test_home::{agg, write_hjmt_home};

pub fn write_home(home: &Path, routing_generation: u64, aggs: &[TestAgg]) {
    write_hjmt_home(home, routing_generation, aggs);
}

pub fn canonical_five_by_seven(base_port: u16) -> Vec<TestAgg> {
    vec![
        agg(0, base_port, &[(0, &[1, 2]), (5, &[2, 4])]),
        agg(1, base_port + 1, &[(1, &[0, 3])]),
        agg(2, base_port + 2, &[(2, &[0, 4])]),
        agg(3, base_port + 3, &[(3, &[1, 4])]),
        agg(4, base_port + 4, &[(4, &[2, 3]), (6, &[0, 1])]),
    ]
}

pub fn secondary_join_six_by_seven(base_port: u16) -> Vec<TestAgg> {
    vec![
        agg(0, base_port, &[(0, &[1, 5]), (5, &[2, 4])]),
        agg(1, base_port + 1, &[(1, &[0, 3])]),
        agg(2, base_port + 2, &[(2, &[0, 4])]),
        agg(3, base_port + 3, &[(3, &[1, 4])]),
        agg(4, base_port + 4, &[(4, &[2, 3]), (6, &[0, 5])]),
        agg(5, base_port + 5, &[]),
    ]
}

pub fn owner_join_six_by_seven(base_port: u16) -> Vec<TestAgg> {
    vec![
        agg(0, base_port, &[(5, &[2, 4])]),
        agg(1, base_port + 1, &[(1, &[0, 3])]),
        agg(2, base_port + 2, &[(2, &[0, 4])]),
        agg(3, base_port + 3, &[(3, &[1, 4])]),
        agg(4, base_port + 4, &[(4, &[2, 3]), (6, &[0, 1])]),
        agg(5, base_port + 5, &[(0, &[0, 1])]),
    ]
}

pub fn remaining_transfer_five_by_seven(base_port: u16) -> Vec<TestAgg> {
    vec![
        agg(0, base_port, &[(5, &[2, 4])]),
        agg(1, base_port + 1, &[(0, &[0, 2]), (1, &[0, 3])]),
        agg(2, base_port + 2, &[(2, &[0, 4])]),
        agg(3, base_port + 3, &[(3, &[1, 4])]),
        agg(4, base_port + 4, &[(4, &[2, 3]), (6, &[0, 1])]),
    ]
}

pub fn new_transfer_six_by_seven(base_port: u16) -> Vec<TestAgg> {
    owner_join_six_by_seven(base_port)
}

pub fn staged_three_by_seven(base_port: u16) -> Vec<TestAgg> {
    vec![
        agg(0, base_port, &[(0, &[1, 5]), (1, &[1, 5]), (2, &[1, 5])]),
        agg(1, base_port + 1, &[(3, &[0, 5]), (4, &[0, 5])]),
        agg(5, base_port + 5, &[(5, &[0, 1]), (6, &[0, 1])]),
    ]
}

pub fn staged_two_by_seven(base_port: u16) -> Vec<TestAgg> {
    vec![
        agg(0, base_port, &[(0, &[1]), (1, &[1]), (2, &[1]), (5, &[1])]),
        agg(1, base_port + 1, &[(3, &[0]), (4, &[0]), (6, &[0])]),
    ]
}

pub fn staged_five_by_seven(base_port: u16) -> Vec<TestAgg> {
    canonical_five_by_seven(base_port)
}

pub fn load_cfg(home: &Path) -> NodeConfig {
    NodeConfig::from_hjmt_home(home).expect("load hjmt home")
}

pub fn placement_row(cfg: &NodeConfig, shard_id: u16, generation: u64) -> ShardPlacement {
    cfg.placement_table()
        .expect("placement table")
        .placement(BatchRoute {
            shard_id: ShardId::new(shard_id),
            routing_generation: generation,
        })
        .expect("placement row")
        .clone()
}

pub fn read_route_table(home: &Path) -> ShardRouteTable {
    let bytes = io::read_file(home.join("shard_route_tables/route-table-v1.canon.hex"))
        .expect("read route table");
    let hex = String::from_utf8(bytes).expect("route table utf8");
    let raw = hex::decode(hex.trim()).expect("route table hex");
    ShardRouteTable::from_canon(&raw).expect("route table")
}

pub fn set_activation_checkpoint(home: &Path, checkpoint: u64) {
    rewrite_route_table(home, |table| table.activation_checkpoint = checkpoint);
}

pub fn bind_previous_generation(home: &Path, prev: &ShardRouteTable) {
    rewrite_route_table(home, |table| {
        table.previous_generation_digest = Some(prev.digest())
    });
}

pub fn primary_id(cfg: &NodeConfig, shard_id: u16, generation: u64) -> AggregatorId {
    placement_row(cfg, shard_id, generation).primary_id
}

pub fn shard_ids(home: &Path) -> Vec<u16> {
    read_route_table(home)
        .shard_set
        .iter()
        .map(|shard_id| shard_id.as_u16())
        .collect()
}

fn rewrite_route_table(home: &Path, mutate: impl FnOnce(&mut ShardRouteTable)) {
    let mut table = read_route_table(home);
    let old_digest = hex::encode(table.digest().as_bytes());
    mutate(&mut table);
    table.validate().expect("route table stays valid");

    io::write_file(
        home.join("shard_route_tables/route-table-v1.canon.hex"),
        hex::encode(table.canonical_bytes()).as_bytes(),
    )
    .expect("write route table");

    let new_digest = hex::encode(table.digest().as_bytes());
    replace_digest(
        &home.join("planner/planner-config.yaml"),
        &old_digest,
        &new_digest,
    );

    for entry in fs::read_dir(home.join("aggregators")).expect("aggregators dir") {
        let path = entry
            .expect("aggregator entry")
            .path()
            .join("aggregator-config.yaml");
        replace_digest(&path, &old_digest, &new_digest);
    }
}

fn replace_digest(path: &Path, old_digest: &str, new_digest: &str) {
    let body = String::from_utf8(io::read_file(path).expect("read cfg")).expect("cfg utf8");
    let body = body.replace(old_digest, new_digest);
    io::write_file(path, body.as_bytes()).expect("write cfg");
}
