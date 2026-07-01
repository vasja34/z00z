use std::sync::Arc;

use jsonrpsee::{core::RpcResult, types::ErrorObjectOwned};
use z00z_core::assets::{encode_asset_pkg_json, AssetPkgWire};
use z00z_utils::{
    codec::{Codec, JsonCodec},
    logger::{Logger, TracingLogger},
    time::TimeProvider,
};

use crate::{
    persistence::tx::TxStorageImpl,
    receiver::{ReceiveReject, ScanResult, StealthOutputScanner},
    rpc::{
        error_mapping::{map_wallet_error_to_rpc, runtime_tx_error_response},
        types::{
            common::PersistWalletId,
            tx::{RuntimeTxErrorCode, RuntimeTxLifecycle, RuntimeVerifyTxPkgOut},
            wallet::SessionToken,
        },
    },
    services::WalletService,
    tx::{TxOutputWire, TxPackage},
    wallet::policy::{utc_day_window_ms, PolicyImpl, PolicySpendContext},
};

use super::tx_rpc_broadcast::try_parse_tx_bytes;

#[derive(Clone)]
pub(crate) struct TimeProviderRef(pub(crate) Arc<dyn TimeProvider>);

impl TimeProvider for TimeProviderRef {
    fn now(&self) -> std::time::SystemTime {
        self.0.now()
    }
}

pub(crate) fn parse_asset_id_hex(asset_id: Option<String>) -> Result<[u8; 32], ErrorObjectOwned> {
    if let Some(hex) = asset_id {
        let bytes = z00z_crypto::expert::encoding::from_hex(&hex).map_err(|_| {
            ErrorObjectOwned::owned(-32602, "Invalid asset_id hex".to_string(), None::<()>)
        })?;
        if bytes.len() != 32 {
            return Err(ErrorObjectOwned::owned(
                -32602,
                "Invalid asset_id: expected 32 bytes".to_string(),
                None::<()>,
            ));
        }

        let mut out = [0u8; 32];
        out.copy_from_slice(&bytes);
        Ok(out)
    } else {
        Ok([0u8; 32])
    }
}

pub(crate) async fn validate_policy(
    service: &WalletService,
    time_provider: &Arc<dyn TimeProvider>,
    wallet_id: &PersistWalletId,
    asset_id: [u8; 32],
    recipient: &str,
    amount: u64,
) -> RpcResult<()> {
    let settings = service
        .get_wallet_settings(wallet_id)
        .await
        .map_err(map_wallet_error_to_rpc)?;

    if let Some(rules) = settings.policy_rules {
        let history_path = service.wallet_history_jsonl_path(wallet_id);
        let store = TxStorageImpl::new(&history_path, TimeProviderRef(time_provider.clone()));
        let context = load_policy_spend_context(&store, time_provider, asset_id)?;
        let policy = PolicyImpl::new(rules, TimeProviderRef(time_provider.clone()));
        policy
            .validate_spend_definition_with_context(&asset_id, amount, recipient, &context)
            .map_err(|error| {
                ErrorObjectOwned::owned(-32602, format!("Policy violation: {error}"), None::<()>)
            })?;
    }

    Ok(())
}

pub(crate) fn load_policy_spend_context<T: TimeProvider>(
    store: &TxStorageImpl<T>,
    time_provider: &Arc<dyn TimeProvider>,
    asset_id: [u8; 32],
) -> RpcResult<PolicySpendContext> {
    let now_ms = time_provider.try_unix_timestamp_ms().map_err(|error| {
        ErrorObjectOwned::owned(
            -32603,
            format!("Policy clock unavailable: {error}"),
            None::<()>,
        )
    })?;
    let (day_start_ms, day_end_ms) = utc_day_window_ms(now_ms);
    let window = store
        .policy_spend_window(asset_id, day_start_ms, day_end_ms)
        .map_err(|error| {
            ErrorObjectOwned::owned(
                -32603,
                format!("Policy tx history unavailable: {error}"),
                None::<()>,
            )
        })?;
    Ok(window.into())
}

pub(crate) async fn verify_session(
    service: &WalletService,
    session: &SessionToken,
) -> RpcResult<()> {
    service
        .check_auto_lock()
        .await
        .map_err(map_wallet_error_to_rpc)?;
    service
        .verify_session(session)
        .await
        .map(|_| ())
        .map_err(map_wallet_error_to_rpc)
}

pub(crate) fn parse_tx_pkg(tx_data: &str) -> RpcResult<(Vec<u8>, TxPackage)> {
    if tx_data.trim().is_empty() {
        return Err(runtime_tx_error_response(
            -32602,
            "Invalid tx_data: must not be empty".to_string(),
            vec![RuntimeTxErrorCode::InvalidEncoding],
            Some(RuntimeTxLifecycle::Failed),
        ));
    }

    let tx_bytes = try_parse_tx_bytes(tx_data).map_err(|message| {
        runtime_tx_error_response(
            -32602,
            message,
            vec![RuntimeTxErrorCode::InvalidEncoding],
            Some(RuntimeTxLifecycle::Failed),
        )
    })?;
    let pkg = JsonCodec.deserialize(&tx_bytes).map_err(|error| {
        runtime_tx_error_response(
            -32602,
            format!("Invalid tx package: {error}"),
            vec![RuntimeTxErrorCode::InvalidPackage],
            Some(RuntimeTxLifecycle::Failed),
        )
    })?;
    Ok((tx_bytes, pkg))
}

pub(crate) async fn scan_pkg_outputs(
    service: &WalletService,
    wallet_id: &PersistWalletId,
    pkg: &TxPackage,
) -> RpcResult<Vec<RuntimeVerifyTxPkgOut>> {
    let recv_keys = service
        .receiver_keys(wallet_id)
        .await
        .map_err(map_wallet_error_to_rpc)?;
    let scanner = StealthOutputScanner::from_keys(&recv_keys);
    let mut owned_outputs = Vec::new();

    for output in &pkg.tx.outputs {
        if let Some(owned) = build_owned_out(output, &scanner)? {
            owned_outputs.push(owned);
        }
    }

    Ok(owned_outputs)
}

pub(crate) fn build_owned_out(
    output: &TxOutputWire,
    scanner: &StealthOutputScanner,
) -> RpcResult<Option<RuntimeVerifyTxPkgOut>> {
    let asset = match output.asset_wire.clone().to_asset() {
        Ok(asset) => asset,
        Err(error) => {
            Logger::warn(
                &TracingLogger,
                &format!(
                    "action=tx_scan_reject reason={} detail={} tx output asset rejected before scanner",
                    ReceiveReject::InvalidInput.log_code(),
                    error
                ),
            );
            return Ok(None);
        }
    };

    let report = match scanner.scan_report(&asset) {
        Ok(report) => report,
        Err(reason) => {
            Logger::warn(
                &TracingLogger,
                &format!(
                    "action=tx_scan_reject reason={} tx output asset rejected before scanner report",
                    reason.log_code()
                ),
            );
            return Ok(None);
        }
    };

    if report.status != crate::receiver::ReceiveStatus::Detected {
        let reason = report.reject.unwrap_or(ReceiveReject::InvalidProof);
        Logger::debug(
            &TracingLogger,
            &format!(
                "action=tx_scan_reject reason={} tx output did not produce owned receive result",
                reason.log_code()
            ),
        );
        return Ok(None);
    }

    let ScanResult::Mine { wallet_output } = scanner.scan_leaf(&asset) else {
        Logger::warn(
            &TracingLogger,
            &format!(
                "action=tx_scan_reject reason={} tx output report detected but owned output reconstruction failed",
                ReceiveReject::RuntimeFail.log_code()
            ),
        );
        return Ok(None);
    };

    let wire = output.asset_wire.clone().to_wire().map_err(|error| {
        ErrorObjectOwned::owned(
            -32602,
            format!("Invalid tx output wire: {error}"),
            None::<()>,
        )
    })?;
    let asset_data = encode_asset_dto(&wire)?;

    Ok(Some(RuntimeVerifyTxPkgOut {
        asset_id_hex: hex::encode(wallet_output.asset_id),
        serial_id: wallet_output.serial_id,
        amount: wallet_output.amount,
        can_spend: wallet_output.asset_secret.is_present() && wallet_output.blinding.is_present(),
        asset_data,
    }))
}

pub(crate) fn encode_asset_dto(wire: &z00z_core::assets::AssetWire) -> RpcResult<String> {
    let dto = AssetPkgWire::from_wire(wire);
    let bytes = encode_asset_pkg_json(&dto).map_err(|error| {
        ErrorObjectOwned::owned(
            -32603,
            format!("Asset serialization failed: {error}"),
            None::<()>,
        )
    })?;
    String::from_utf8(bytes).map_err(|error| {
        ErrorObjectOwned::owned(
            -32603,
            format!("Asset encoding failed: {error}"),
            None::<()>,
        )
    })
}

pub(crate) fn is_import_ready(status: &str) -> bool {
    matches!(
        status.to_ascii_lowercase().as_str(),
        "admitted" | "confirmed" | "verified"
    )
}

pub(crate) fn lifecycle_from_package_status(status: &str) -> RuntimeTxLifecycle {
    match status.to_ascii_lowercase().as_str() {
        "confirmed" => RuntimeTxLifecycle::Confirmed,
        "verified" | "admitted" => RuntimeTxLifecycle::Admitted,
        "submitted" => RuntimeTxLifecycle::Submitted,
        _ => RuntimeTxLifecycle::Created,
    }
}

pub(crate) fn package_status_from_lifecycle(lifecycle: RuntimeTxLifecycle) -> &'static str {
    match lifecycle {
        RuntimeTxLifecycle::Confirmed => "confirmed",
        RuntimeTxLifecycle::Admitted
        | RuntimeTxLifecycle::Imported
        | RuntimeTxLifecycle::Exported => "admitted",
        RuntimeTxLifecycle::Submitted => "submitted",
        _ => "created",
    }
}
