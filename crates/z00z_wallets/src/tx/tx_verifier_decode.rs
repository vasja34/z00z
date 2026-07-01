fn decode_assets_from_package(pkg: TxPackage) -> TxVerifierResult<Vec<Asset>> {
    let mut out = Vec::with_capacity(pkg.tx.outputs.len());
    for item in pkg.tx.outputs {
        out.push(item.asset_wire.to_asset().map_err(|err| {
            TxVerifierError::InvalidStructure(format!("asset decode failed: {err}"))
        })?);
    }
    Ok(out)
}

fn fee_sum_from_outputs(outputs: &[TxOutputWire]) -> TxVerifierResult<u64> {
    let mut fee_sum = 0u64;
    for output in outputs {
        if output.role != TxOutRole::Fee {
            continue;
        }
        let asset = output.asset_wire.clone().to_asset().map_err(|err| {
            TxVerifierError::InvalidStructure(format!("asset decode failed: {err}"))
        })?;
        if asset.definition.class != z00z_core::assets::AssetClass::Coin {
            return Err(TxVerifierError::VerificationFailed(
                "fee outputs must use coin class".to_string(),
            ));
        }
        fee_sum = fee_sum.checked_add(asset.amount).ok_or_else(|| {
            TxVerifierError::VerificationFailed("fee output amount overflow".to_string())
        })?;
    }
    Ok(fee_sum)
}
