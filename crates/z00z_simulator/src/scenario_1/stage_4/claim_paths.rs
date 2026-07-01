use crate::SimContext;

pub(crate) fn resolve_stage3_claim_dir(ctx: &SimContext) -> std::path::PathBuf {
    ctx.outputs_dir.join(&ctx.config.stage3_paths().claim_dir)
}

pub(crate) fn resolve_stage3_claim_pkg_file(ctx: &SimContext) -> std::path::PathBuf {
    resolve_stage3_claim_dir(ctx).join("tx_claim_pkg.json")
}

pub(crate) fn resolve_stage3_claim_pub_file(ctx: &SimContext) -> std::path::PathBuf {
    resolve_stage3_claim_dir(ctx).join("claim_store_pub.json")
}
