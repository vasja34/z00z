use blake2::{Blake2b512, Digest};
use z00z_core::{assets::AssetPkgWire, genesis::asset_std::asset_from_dev_class};
use z00z_utils::codec::{Codec, JsonCodec};
use z00z_wallets::tx::{
    build_tx_package_digest, TxAuthWire, TxContextWire, TxInputWire, TxOutRole, TxOutputWire,
    TxProofWire, TxWire,
};

fn sample_tx() -> TxWire {
    let asset = asset_from_dev_class(z00z_core::assets::AssetClass::Coin, 1, 100).unwrap();
    let fee_asset = asset_from_dev_class(z00z_core::assets::AssetClass::Coin, 9, 1).unwrap();

    TxWire {
        tx_type: "regular_tx".to_string(),
        inputs: vec![TxInputWire {
            asset_id_hex: hex::encode([0x11u8; 32]),
            serial_id: 1,
        }],
        outputs: vec![
            TxOutputWire {
                role: TxOutRole::Recipient,
                asset_wire: AssetPkgWire::from_asset(&asset),
            },
            TxOutputWire {
                role: TxOutRole::Fee,
                asset_wire: AssetPkgWire::from_asset(&fee_asset),
            },
        ],
        fee: 1,
        nonce: 7,
        context: TxContextWire::default(),
        proof: TxProofWire::default(),
        auth: TxAuthWire::default(),
    }
}

fn test_boundary_collision_digest_reference(
    kind: &str,
    package_type: &str,
    version: u8,
    chain_id: u32,
    chain_type: &str,
    chain_name: &str,
    tx: &TxWire,
) -> String {
    let tx_json = JsonCodec.serialize(tx).unwrap();
    let mut hasher = Blake2b512::new();
    hasher.update(b"z00z.tx.pkg.digest.v1");
    hasher.update(kind.as_bytes());
    hasher.update(package_type.as_bytes());
    hasher.update([version]);
    hasher.update(chain_id.to_le_bytes());
    hasher.update(chain_type.as_bytes());
    hasher.update(chain_name.as_bytes());
    hasher.update(&tx_json);
    let digest = hasher.finalize();
    hex::encode(&digest[..32])
}

#[test]
fn test_tx_package_unframed_collision() {
    let tx = sample_tx();
    let collision_a =
        test_boundary_collision_digest_reference("ab", "c", 1, 7, "devnet", "z00z", &tx);
    let collision_b =
        test_boundary_collision_digest_reference("a", "bc", 1, 7, "devnet", "z00z", &tx);
    assert_eq!(collision_a, collision_b);

    let digest_a = build_tx_package_digest("ab", "c", 1, 7, "devnet", "z00z", &tx).unwrap();
    let digest_b = build_tx_package_digest("a", "bc", 1, 7, "devnet", "z00z", &tx).unwrap();
    assert_ne!(digest_a, digest_b);
}

#[test]
fn test_tx_package_digest_ignores_output_range_proof_bytes() {
    let tx = sample_tx();
    let digest_before =
        build_tx_package_digest("TxPackage", "regular_tx", 1, 7, "devnet", "z00z", &tx)
            .expect("digest before");
    let mut mutated = tx.clone();
    let proof = mutated.outputs[0]
        .asset_wire
        .range_proof
        .as_mut()
        .expect("sample output range proof");
    proof[0] ^= 1;

    let digest_after =
        build_tx_package_digest("TxPackage", "regular_tx", 1, 7, "devnet", "z00z", &mutated)
            .expect("digest after");

    assert_eq!(digest_before, digest_after);
}
