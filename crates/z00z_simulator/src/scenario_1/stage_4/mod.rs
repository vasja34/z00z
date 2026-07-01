//! Scenario 1 stage 4: claim publish.
//!
//! The claim-publish files emitted here are structurally useful but weaker than later
//! spend/checkpoint semantic acceptance. They remain non-authoritative and do not
//! upgrade early artifacts into canonical checkpoint truth.

mod claim_paths;
mod publish;
mod storage_view;
mod storage_view_patch;

pub(crate) use claim_paths::{resolve_stage3_claim_pkg_file, resolve_stage3_claim_pub_file};
pub use publish::run_claim_publish;
pub(crate) use storage_view::{
    describe_store_roots, export_claim_post_view, export_post_tx_final_view, export_post_tx_view,
    export_pre_tx_view, publish_genesis_rights,
};
