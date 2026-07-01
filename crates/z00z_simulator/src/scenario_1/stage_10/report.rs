use serde::Serialize;
use std::path::Path;

use z00z_utils::io::save_json;

use super::super::stage_9::bundle_lane_impl::Stage9Bridge;

#[derive(Serialize)]
struct Stage10Report {
    stage: u32,
    fragments: u32,
    amount_sum: u64,
    prev_root_hex: String,
    exec_input_id_hex: String,
    status: String,
}

pub(crate) fn write_report(
    out: &Path,
    file: &str,
    stage: u32,
    amount_sum: u64,
    bridge: &Stage9Bridge,
) -> Result<(), String> {
    let report = Stage10Report {
        stage,
        fragments: bridge.fragment_ids.len() as u32,
        amount_sum,
        prev_root_hex: bridge.prev_root_hex.clone(),
        exec_input_id_hex: bridge.exec_input_id_hex.clone(),
        status: bridge.status.clone(),
    };
    save_json(out.join(file), &report).map_err(|e| e.to_string())
}
