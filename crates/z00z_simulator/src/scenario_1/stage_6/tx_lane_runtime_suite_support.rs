use super::{decode_output_pack, to_tx_output_wires};
use crate::config::{
    Stage4ListFilterCfg, Stage4OutputsCfg, Stage4PathsCfg, Stage4RpcCfg, Stage4SelectionCfg,
    Stage4TransactionCfg, Stage4TxPrepareCfg,
};
use crate::scenario_1::stage_6::make_output_with_blind;
use z00z_core::{genesis::asset_std::asset_from_dev_class, AssetClass, AssetWire};
use z00z_crypto::Z00ZScalar;
use z00z_wallets::{
    key::{ReceiverKeys as CoreKeys, ReceiverSecret},
    receiver::ReceiverCard,
    stealth::build_output_bundle as core_build_output_bundle,
    tx::OutputBundle,
};

pub(super) fn mk_cfg(mode: &str, fraction: Option<f64>, amount: Option<u64>) -> Stage4TxPrepareCfg {
    Stage4TxPrepareCfg {
        enabled: true,
        sender_actor: "alice".to_string(),
        receiver_actor: "bob".to_string(),
        paths: Stage4PathsCfg {
            outputs_dir: "out".to_string(),
            logs_dir: "logs".to_string(),
            transactions_dir: "tx".to_string(),
            wallets_dir: "wallets".to_string(),
            tx_pkg_file: "tx/tx.json".to_string(),
            snapshot_file: "snap.json".to_string(),
            logger_file: "logs/logger.json".to_string(),
            rpc_logger_file: "logs/rpc.json".to_string(),
            alice_keys_file: "keys/alice.json".to_string(),
            bob_keys_file: "keys/bob.json".to_string(),
            wallets_state_before_file: None,
            wallets_state_after_file: None,
            wallets_state_diff_file: None,
            wallets_state_report_md_file: None,
            wallets_state_report_xlsx_file: None,
        },
        transaction: Stage4TransactionCfg {
            class: "Coin".to_string(),
            symbol: "Z00Z".to_string(),
            mode: mode.to_string(),
            input_assets_selection: Stage4SelectionCfg::default(),
            outputs: Stage4OutputsCfg::default(),
            fee_sink: crate::config::Stage4FeeSinkCfg::default(),
            amount,
            fraction,
        },
        rpc: Stage4RpcCfg {
            transport: "logged_local".to_string(),
            unlock_method: "wallet.session.unlock_wallet".to_string(),
            lock_method: "wallet.session.lock_wallet".to_string(),
            list_assets_method: "wallet.asset.list_assets".to_string(),
            import_asset_method: "wallet.asset.import_asset".to_string(),
            build_transaction_method: "wallet.tx.build_transaction".to_string(),
            list_limit: 50,
            list_filter: Stage4ListFilterCfg {
                asset_class: None,
                min_balance: None,
            },
        },
    }
}

pub(super) fn pick_cfg() -> Stage4TxPrepareCfg {
    let mut cfg = mk_cfg("fraction", Some(1.0), None);
    cfg.transaction
        .input_assets_selection
        .distinct_serial_ids_min = 3;
    cfg.transaction
        .input_assets_selection
        .distinct_serial_ids_target = 3;
    cfg.transaction
        .input_assets_selection
        .distinct_serial_ids_max = 3;
    cfg
}

fn rebuild_def(
    definition: &z00z_core::AssetDefinition,
    symbol: &str,
) -> z00z_core::AssetDefinition {
    z00z_core::AssetDefinition::new(
        [0u8; 32],
        definition.class,
        definition.name.clone(),
        symbol.to_string(),
        definition.decimals,
        definition.serials,
        definition.nominal,
        definition.domain_name.clone(),
        definition.version,
        definition.crypto_version,
        definition.policy_flags,
        definition.metadata.clone(),
    )
    .expect("canonical test definition")
}

pub(super) fn mk_wire(serial_id: u32, amount: u64, symbol: &str) -> AssetWire {
    let mut asset = asset_from_dev_class(AssetClass::Coin, 0, amount).expect("std asset");
    let def = rebuild_def(asset.definition.as_ref(), symbol);
    asset.definition = std::sync::Arc::new(def);
    asset.serial_id = serial_id;
    AssetWire::from_asset(&asset)
}

pub(super) fn mk_pick_wire(serial_id: u32, amount: u64, symbol: &str) -> AssetWire {
    let card = mk_card(7);
    let out = core_build_output_bundle(
        "alice".to_string(),
        z00z_wallets::tx::TxOutRole::Change,
        AssetClass::Coin,
        &card,
        amount,
        serial_id,
    )
    .expect("output bundle");
    let mut wire = input_from_out(&out);
    wire.definition = rebuild_def(&wire.definition, symbol);
    wire
}

pub(super) fn mk_card(seed: u8) -> ReceiverCard {
    let recv = ReceiverSecret::from_bytes([seed; 32]).expect("receiver secret");
    CoreKeys::from_receiver_secret(recv)
        .expect("receiver keys")
        .export_receiver_card()
        .expect("receiver card")
}

pub(super) fn mk_out_with_serial(serial_id: u32) -> OutputBundle {
    core_build_output_bundle(
        "bob".to_string(),
        z00z_wallets::tx::TxOutRole::Recipient,
        AssetClass::Coin,
        &mk_card(7),
        55,
        serial_id,
    )
    .expect("output bundle")
}

pub(super) fn mk_out() -> OutputBundle {
    mk_out_with_serial(3)
}

pub(super) fn mk_balanced_out_from_input(input: &OutputBundle, serial_id: u32) -> OutputBundle {
    let pack = decode_output_pack(input).expect("decode pack");
    let blinding = Z00ZScalar::try_from_bytes(pack.blinding).expect("blinding scalar");
    make_output_with_blind(
        "bob".to_string(),
        z00z_wallets::tx::TxOutRole::Recipient,
        AssetClass::Coin,
        &mk_card(7),
        input.value,
        serial_id,
        None,
        0,
        blinding,
    )
    .expect("balanced output bundle")
}

pub(super) fn input_from_out(out: &OutputBundle) -> AssetWire {
    let mut wire = to_tx_output_wires(std::slice::from_ref(out)).expect("tx output wire")[0]
        .asset_wire
        .clone()
        .to_wire()
        .expect("asset wire");
    wire.secret = Some(out.s_out);
    wire
}

pub(super) fn input_asset_id_hex(wire: &AssetWire) -> String {
    hex::encode(
        z00z_wallets::tx::asset_wire_to_leaf(wire)
            .expect("input leaf")
            .asset_id,
    )
}
