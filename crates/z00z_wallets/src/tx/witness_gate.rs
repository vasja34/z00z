//! Core tx witness gate helpers extracted from simulator Stage-4.

use z00z_core::{
    assets::{AssetPackPlain, AssetPkgWire},
    genesis::asset_std::asset_from_dev_class,
    AssetWire,
};
use z00z_crypto::{domains::AssetIdHashDomain, DomainHasher};
use z00z_storage::settlement::{
    CheckRoot, DefinitionId, SerialId, SettlementPath, SettlementStore, StoreItem, StoreOp,
    TerminalId, TerminalLeaf,
};

use crate::{
    key::{ReceiverKeys, ReceiverSecret},
    receiver::asset_leaf_scan::receiver_scan_input,
};

use super::{
    spend_verification::{
        build_public_spend_contract, build_spend_input_proof, verify_tx_public_spend_contract,
    },
    tx_output::bind_output_wire,
    OutputBundle, SpendInputLeaf, SpendInputRef, SpendMembershipWitness, SpendProofWitness,
    TxInputWire, TxOutputWire, TxWire,
};
#[cfg(test)]
use super::{SpendProofApi, SpendProofErr};

/// Convert asset wire into spend leaf.
pub fn asset_wire_to_leaf(wire: &AssetWire) -> Result<TerminalLeaf, String> {
    let asset_id = wire_asset_id(wire)?;
    let r_pub = wire
        .r_pub
        .ok_or_else(|| "stage4: SpendWitness gate: missing input r_pub".to_string())?;
    let owner_tag = wire
        .owner_tag
        .ok_or_else(|| "stage4: SpendWitness gate: missing input owner_tag".to_string())?;
    let enc_pack = wire
        .enc_pack
        .clone()
        .ok_or_else(|| "stage4: SpendWitness gate: missing input enc_pack".to_string())?;
    let tag16 = wire
        .tag16
        .ok_or_else(|| "stage4: SpendWitness gate: missing input tag16".to_string())?;
    let range_proof = wire.range_proof.as_ref().cloned().unwrap_or_default();
    let c_amount: [u8; 32] = wire
        .commitment
        .as_bytes()
        .try_into()
        .map_err(|_| "stage4: SpendWitness gate: invalid input commitment bytes".to_string())?;

    Ok(TerminalLeaf {
        asset_id,
        serial_id: wire.serial_id,
        r_pub,
        owner_tag,
        c_amount,
        enc_pack,
        range_proof,
        tag16,
    })
}

/// Convert asset wire into the decrypt/scan leaf contract.
///
/// `receiver_scan_leaf()` consumes `TerminalLeaf.asset_id` as the decrypt-boundary
/// associated-data identifier, not as the canonical state key. Keep
/// `asset_wire_to_leaf()` for state-key work. Prefer `resolve_input_pack()` for
/// accepted spend-witness resolution so canonical `asset_id` and `leaf_ad_id`
/// stay split at the shipped wallet, scan, report, and spend-witness bridge
/// without implying repository-wide artifact closure. These are real identity-binding building blocks, but a uniform fail-closed policy is still required above this seam.
pub fn wire_decrypt_leaf(wire: &AssetWire) -> Result<TerminalLeaf, String> {
    let leaf_ad_id = wire
        .leaf_ad_id()
        .map_err(|e| format!("stage4: SpendWitness gate: missing leaf_ad_id: {e}"))?;
    let mut leaf = asset_wire_to_leaf(wire)?;
    leaf.set_terminal_id(TerminalId::new(leaf_ad_id));
    Ok(leaf)
}

fn wire_asset_id(wire: &AssetWire) -> Result<[u8; 32], String> {
    let hash = DomainHasher::<AssetIdHashDomain>::new_with_label("asset_id")
        .chain(wire.nonce)
        .chain(wire.commitment.as_bytes())
        .chain(wire.definition.id)
        .chain(wire.serial_id.to_le_bytes())
        .finalize();

    let mut asset_id = [0u8; 32];
    asset_id.copy_from_slice(&hash.as_ref()[..32]);
    Ok(asset_id)
}

/// Resolve input secret for spend witness input binding.
pub fn resolve_input_pack(recv_sec: [u8; 32], item: &AssetWire) -> Result<AssetPackPlain, String> {
    let receiver_secret = ReceiverSecret::from_bytes(recv_sec)
        .map_err(|e| format!("stage4: SpendWitness gate: bad receiver secret: {e}"))?;
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret)
        .map_err(|e| format!("stage4: SpendWitness gate: derive receiver keys failed: {e}"))?;

    let leaf_ad_id = item
        .leaf_ad_id()
        .map_err(|e| format!("stage4: SpendWitness gate: missing leaf_ad_id: {e}"))?;
    let r_pub = item
        .r_pub
        .as_ref()
        .ok_or_else(|| "stage4: SpendWitness gate: missing input r_pub".to_string())?;
    let owner_tag = item
        .owner_tag
        .as_ref()
        .ok_or_else(|| "stage4: SpendWitness gate: missing input owner_tag".to_string())?;
    let enc_pack = item
        .enc_pack
        .as_ref()
        .ok_or_else(|| "stage4: SpendWitness gate: missing input enc_pack".to_string())?;
    let tag16 = item
        .tag16
        .ok_or_else(|| "stage4: SpendWitness gate: missing input tag16".to_string())?;
    let c_amount: [u8; 32] = item
        .commitment
        .as_bytes()
        .try_into()
        .map_err(|_| "stage4: SpendWitness gate: invalid input commitment bytes".to_string())?;

    let scan = receiver_scan_input(
        &receiver_keys,
        item.serial_id,
        &leaf_ad_id,
        r_pub,
        owner_tag,
        &c_amount,
        enc_pack,
        Some(tag16),
    )
    .map_err(|e| format!("stage4: SpendWitness gate: input decrypt failed: {e}"))?;
    let pack = scan.ok_or_else(|| {
        "stage4: SpendWitness gate: input is not decryptable for sender secret".to_string()
    })?;
    let pack: AssetPackPlain = pack.opening_pack();

    if let Some(s_in) = item.secret {
        if pack.s_out != s_in {
            return Err(
                "stage4: SpendWitness gate: input pack unavailable for provided secret".to_string(),
            );
        }
    }

    Ok(pack)
}

/// Resolve input secret for spend witness input binding.
pub fn resolve_input_secret(recv_sec: [u8; 32], item: &AssetWire) -> Result<[u8; 32], String> {
    Ok(resolve_input_pack(recv_sec, item)?.s_out)
}

fn bundle_to_output_wire(output: &OutputBundle) -> Result<TxOutputWire, String> {
    let mut asset = asset_from_dev_class(output.class, output.leaf.serial_id, output.value)
        .map_err(|e| format!("stage4: SpendWitness gate: failed to build output asset: {e}"))?;
    asset.r_pub = Some(output.leaf.r_pub);
    asset.owner_tag = Some(output.leaf.owner_tag);
    asset.enc_pack = Some(output.leaf.enc_pack.clone());
    asset.tag16 = Some(output.leaf.tag16);
    let commitment = z00z_crypto::Commitment::from_bytes(&output.leaf.c_amount)
        .map_err(|e| format!("stage4: SpendWitness gate: output commitment parse failed: {e}"))?;
    asset.commitment = commitment.as_commitment().clone();
    asset.range_proof = Some(output.leaf.range_proof.clone());
    asset.owner_signature = None;

    let wire = bind_output_wire(AssetWire::from_asset(&asset), &output.leaf)
        .map_err(|e| format!("stage4: SpendWitness gate: bind output wire failed: {e}"))?;
    Ok(TxOutputWire {
        role: output.role,
        asset_wire: AssetPkgWire::from_wire(&wire),
    })
}

/// Prepare canonical public spend-proof inputs from owned selected inputs.
pub fn prepare_spend_public_inputs(
    chain_id: u32,
    recv_sec: [u8; 32],
    selected_inputs: &[AssetWire],
    tx_inputs: &[TxInputWire],
) -> Result<Vec<super::tx_wire::SpendInputProofWire>, String> {
    if selected_inputs.len() != tx_inputs.len() {
        return Err("stage4: SpendWitness gate: tx input length mismatch".to_string());
    }

    selected_inputs
        .iter()
        .zip(tx_inputs.iter())
        .map(|(item, tx_input)| {
            let pack = resolve_input_pack(recv_sec, item)?;
            let leaf = asset_wire_to_leaf(item)?;
            let leaf_ad_id = item
                .leaf_ad_id
                .ok_or_else(|| "stage4: SpendWitness gate: missing input leaf_ad_id".to_string())?;
            let input_ref = SpendInputRef {
                asset_id: leaf.asset_id,
                serial_id: item.serial_id,
            };
            if hex::encode(input_ref.asset_id) != tx_input.asset_id_hex
                || input_ref.serial_id != tx_input.serial_id
            {
                return Err("stage4: SpendWitness gate: tx input ref drift".to_string());
            }
            let input_leaf = SpendInputLeaf {
                asset_id: leaf.asset_id,
                serial_id: item.serial_id,
                leaf_ad_id,
                r_pub: leaf.r_pub,
                owner_tag: leaf.owner_tag,
                c_amt: leaf.c_amount,
            };
            build_spend_input_proof(chain_id, &input_ref, &input_leaf, &pack.s_out)
                .map_err(|e| format!("stage4: SpendWitness gate: input proof build failed: {e}"))
        })
        .collect()
}

/// Build membership witnesses for selected spend inputs from a local state view.
pub fn prepare_spend_membership_witnesses(
    selected_inputs: &[AssetWire],
    tx_inputs: &[TxInputWire],
) -> Result<(CheckRoot, Vec<SpendMembershipWitness>), String> {
    if selected_inputs.len() != tx_inputs.len() {
        return Err("stage4: SpendWitness gate: tx input length mismatch".to_string());
    }

    let items = selected_inputs
        .iter()
        .zip(tx_inputs.iter())
        .map(|(item, input)| {
            let asset_id: [u8; 32] = hex::decode(&input.asset_id_hex)
                .map_err(|_| "stage4: SpendWitness gate: invalid input asset id".to_string())?
                .try_into()
                .map_err(|_| {
                    "stage4: SpendWitness gate: invalid input asset id length".to_string()
                })?;
            let mut leaf = asset_wire_to_leaf(item)?;
            leaf.set_terminal_id(TerminalId::new(asset_id));
            let path = SettlementPath::new(
                DefinitionId::new(item.definition.id),
                SerialId::new(input.serial_id),
                TerminalId::new(asset_id),
            );
            Ok((path, leaf))
        })
        .collect::<Result<Vec<_>, String>>()?;

    let mut store = SettlementStore::try_new()
        .map_err(|err| format!("stage4: SpendWitness gate: membership store open failed: {err}"))?;
    let ops = items
        .iter()
        .map(|(path, leaf)| {
            StoreItem::new(*path, leaf.clone())
                .map(|item| StoreOp::Put(Box::new(item)))
                .map_err(|err| format!("stage4: SpendWitness gate: store item failed: {err}"))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let root = store
        .apply_settlement_ops(ops)
        .map(CheckRoot::from)
        .map_err(|err| format!("stage4: SpendWitness gate: membership store failed: {err}"))?;

    let membership = items
        .into_iter()
        .map(|(path, leaf)| {
            let proof_blob = store
                .settlement_proof_blob(&path)
                .map_err(|err| format!("stage4: SpendWitness gate: proof blob failed: {err}"))?;
            let proof_item = proof_blob.item().clone();
            let proof = proof_blob
                .encode()
                .map_err(|err| format!("stage4: SpendWitness gate: proof encode failed: {err}"))?;
            SpendMembershipWitness::new(path, leaf, proof, proof_item)
                .map_err(|err| format!("stage4: SpendWitness gate: member witness failed: {err}"))
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok((root, membership))
}

/// Verify the receiver-bound spend-witness preparation gate.
///
/// This keeps the receiver-secret-gated wallet-local seam explicit before the
/// public spend contract is built, and it does not close withholding risk before
/// publication or prove public anti-theft closure.
pub fn verify_spend_witness_gate(
    chain_id: u32,
    recv_sec: [u8; 32],
    selected_inputs: &[AssetWire],
    outputs: &[OutputBundle],
    prev_root: CheckRoot,
) -> Result<(), String> {
    let (membership_root, membership) = prepare_spend_membership_witnesses(
        selected_inputs,
        &selected_inputs
            .iter()
            .map(|item| {
                let leaf = asset_wire_to_leaf(item)?;
                Ok(TxInputWire {
                    asset_id_hex: hex::encode(leaf.asset_id),
                    serial_id: item.serial_id,
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
    )?;
    if membership_root != prev_root {
        return Err("stage4: SpendWitness gate: membership root mismatch".to_string());
    }
    verify_spend_witness_gate_membership(
        chain_id,
        recv_sec,
        selected_inputs,
        outputs,
        prev_root,
        membership,
    )
}

/// Verify the spend-witness gate using membership witnesses supplied by state resolution.
pub fn verify_spend_witness_gate_membership(
    chain_id: u32,
    recv_sec: [u8; 32],
    selected_inputs: &[AssetWire],
    outputs: &[OutputBundle],
    prev_root: CheckRoot,
    membership: Vec<SpendMembershipWitness>,
) -> Result<(), String> {
    if selected_inputs.is_empty() {
        return Err("stage4: SpendWitness gate: no selected inputs".to_string());
    }

    let tx_inputs: Vec<TxInputWire> = selected_inputs
        .iter()
        .map(|item| {
            let leaf = asset_wire_to_leaf(item)?;
            Ok(TxInputWire {
                asset_id_hex: hex::encode(leaf.asset_id),
                serial_id: item.serial_id,
            })
        })
        .collect::<Result<_, String>>()?;
    let tx_outputs: Vec<TxOutputWire> = outputs
        .iter()
        .map(bundle_to_output_wire)
        .collect::<Result<_, _>>()?;
    let proof_inputs =
        prepare_spend_public_inputs(chain_id, recv_sec, selected_inputs, &tx_inputs)?;
    let receiver_secret = ReceiverSecret::from_bytes(recv_sec)
        .map_err(|e| format!("stage4: SpendWitness gate: bad receiver secret: {e}"))?;
    let input_s_in = selected_inputs
        .iter()
        .map(|item| resolve_input_pack(recv_sec, item).map(|pack| pack.s_out))
        .collect::<Result<Vec<_>, String>>()?;
    let receiver_keys = ReceiverKeys::from_receiver_secret(
        ReceiverSecret::from_bytes(recv_sec)
            .map_err(|e| format!("stage4: SpendWitness gate: bad receiver secret: {e}"))?,
    )
    .map_err(|e| format!("stage4: SpendWitness gate: derive receiver keys failed: {e}"))?;

    let mut tx = TxWire {
        tx_type: "regular_tx".to_string(),
        inputs: tx_inputs,
        outputs: tx_outputs,
        fee: 0,
        nonce: 0,
        context: Default::default(),
        proof: Default::default(),
        auth: Default::default(),
    };
    let (proof, auth) = build_public_spend_contract(
        &receiver_keys,
        chain_id,
        1,
        "witness_gate",
        "witness_gate",
        &tx,
        prev_root,
        proof_inputs,
        SpendProofWitness {
            receiver_secret,
            input_s_in,
            membership,
        },
    )
    .map_err(|e| format!("stage4: SpendWitness gate failed: {e}"))?;
    tx.proof = proof;
    tx.auth = auth;

    verify_tx_public_spend_contract(chain_id, 1, "witness_gate", "witness_gate", &tx)
        .map_err(|e| format!("stage4: SpendWitness gate failed: {e}"))
}

#[cfg(test)]
struct SpendCs;

#[cfg(test)]
impl SpendProofApi for SpendCs {
    fn bind_root(&mut self, prev_root: CheckRoot) -> Result<(), SpendProofErr> {
        if prev_root.into_bytes() == [0u8; 32] {
            return Err(SpendProofErr::BindRoot);
        }
        Ok(())
    }

    fn prove_input(
        &mut self,
        idx: usize,
        _inp: &SpendInputRef,
        leaf: &SpendInputLeaf,
        s_in: [u8; 32],
        recv_sec: [u8; 32],
    ) -> Result<(), SpendProofErr> {
        if s_in == [0u8; 32] || recv_sec == [0u8; 32] {
            return Err(SpendProofErr::Input { idx });
        }
        if leaf.r_pub == [0u8; 32] || leaf.owner_tag == [0u8; 32] || leaf.c_amt == [0u8; 32] {
            return Err(SpendProofErr::Input { idx });
        }
        Ok(())
    }

    fn check_balance(
        &mut self,
        c_ins: &[[u8; 32]],
        c_outs: &[[u8; 32]],
    ) -> Result<(), SpendProofErr> {
        if c_ins.is_empty() || c_outs.is_empty() {
            return Err(SpendProofErr::Balance);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        asset_wire_to_leaf, prepare_spend_membership_witnesses, resolve_input_pack,
        verify_spend_witness_gate, wire_decrypt_leaf, OutputBundle, SpendCs, SpendProofApi,
        SpendProofErr, TxInputWire,
    };
    use z00z_core::{genesis::asset_std::asset_from_dev_class, AssetClass, AssetWire};
    use z00z_storage::settlement::CheckRoot;

    use crate::{
        key::{ReceiverKeys, ReceiverSecret},
        stealth::{bind_stealth_output_wire, build_card_stealth_leaf},
        tx::TxOutRole,
    };

    fn make_wire() -> AssetWire {
        let keys = ReceiverKeys::from_receiver_secret(
            ReceiverSecret::from_bytes(recv_sec()).expect("receiver secret"),
        )
        .expect("receiver keys");
        let asset = asset_from_dev_class(AssetClass::Coin, 7, 55).expect("asset");

        let card = keys.export_receiver_card().expect("card");
        let leaf = build_card_stealth_leaf(&card, asset.amount, asset.serial_id).expect("leaf");

        bind_stealth_output_wire(AssetWire::from_asset(&asset), &leaf).expect("bind wire")
    }

    fn recv_sec() -> [u8; 32] {
        [0x11u8; 32]
    }

    #[test]
    fn test_wire_uses_asset_id() {
        let wire = make_wire();
        let asset = wire.clone().to_asset().expect("asset");
        let leaf = asset_wire_to_leaf(&wire).expect("leaf");

        assert_eq!(leaf.asset_id, asset.asset_id());
        assert_eq!(leaf.serial_id, wire.serial_id);
        assert_ne!(leaf.asset_id, wire.definition.id);
    }

    #[test]
    fn test_decrypt_leaf_ad() {
        let wire = make_wire();
        let asset = wire.clone().to_asset().expect("asset");
        let leaf = wire_decrypt_leaf(&wire).expect("decrypt leaf");

        assert_eq!(leaf.asset_id, wire.leaf_ad_id.expect("leaf_ad_id"));
        assert_ne!(leaf.asset_id, asset.asset_id());
    }

    #[test]
    fn test_decrypt_needs_leaf_ad() {
        let mut wire = make_wire();
        wire.leaf_ad_id = None;

        let err = wire_decrypt_leaf(&wire).expect_err("missing leaf_ad_id must reject");
        assert!(err.contains("missing leaf_ad_id"));
    }

    #[test]
    fn test_resolve_pack_ok() {
        let wire = make_wire();

        let pack = resolve_input_pack(recv_sec(), &wire).expect("resolve pack");

        assert_ne!(pack.s_out, [0u8; 32]);
    }

    #[test]
    fn test_resolve_bad_secret() {
        let wire = make_wire();

        let err = resolve_input_pack([0x12u8; 32], &wire).expect_err("bad secret must reject");

        assert!(
            err.contains("input is not decryptable") || err.contains("input decrypt failed"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_gate_typed_root() {
        let wire = make_wire();
        let leaf = asset_wire_to_leaf(&wire).expect("leaf");
        let tx_input = TxInputWire {
            asset_id_hex: hex::encode(leaf.asset_id),
            serial_id: wire.serial_id,
        };
        let (prev_root, _membership) = prepare_spend_membership_witnesses(
            std::slice::from_ref(&wire),
            std::slice::from_ref(&tx_input),
        )
        .expect("membership root");
        let output = OutputBundle {
            receiver: "bob".to_string(),
            role: TxOutRole::Recipient,
            class: AssetClass::Coin,
            value: 55,
            leaf,
            k_dh: [7u8; 32],
            s_out: [8u8; 32],
        };

        verify_spend_witness_gate(
            3,
            recv_sec(),
            std::slice::from_ref(&wire),
            std::slice::from_ref(&output),
            prev_root,
        )
        .expect("typed root must pass");
    }

    #[test]
    fn test_spend_rejects_zero_root() {
        let mut cs = SpendCs;

        let err = cs
            .bind_root(CheckRoot::new([0u8; 32]))
            .expect_err("zero root must reject");

        assert_eq!(err, SpendProofErr::BindRoot);
    }
}
