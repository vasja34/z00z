//! Claim distribution policy engine.

use rand::RngCore;
use z00z_core::{Asset, AssetClass};
use z00z_utils::rng::RngCoreExt;

/// Assign assets by class policy.
pub fn assign_class_split(assets: &[Asset], assigned: &mut [Vec<Asset>]) {
    for asset in assets {
        let sym = asset.definition.symbol.to_ascii_uppercase();
        let idx = if sym == "Z00Z" || asset.definition.class == AssetClass::Coin {
            0
        } else if asset.definition.class == AssetClass::Nft {
            1
        } else {
            2
        };
        assigned[idx].push(asset.clone());
    }
}

/// Assign coins round-robin and non-coins by class policy.
pub fn assign_coin_sets(assets: &[Asset], assigned: &mut [Vec<Asset>], rng: &mut dyn RngCore) {
    let mut coins = Vec::new();
    let mut other = Vec::new();

    for asset in assets {
        if asset.definition.class == AssetClass::Coin {
            coins.push(asset.clone());
        } else {
            other.push(asset.clone());
        }
    }

    assign_class_split(&other, assigned);
    shuffle_assets(&mut coins, rng);
    for (idx, asset) in coins.into_iter().enumerate() {
        assigned[idx % 3].push(asset);
    }
}

/// Uniformly assign all assets round-robin after shuffle.
pub fn assign_uniform_all(
    assets: &mut [Asset],
    assigned: &mut [Vec<Asset>],
    rng: &mut dyn RngCore,
) {
    shuffle_assets(assets, rng);
    for (idx, asset) in assets.iter().cloned().enumerate() {
        assigned[idx % 3].push(asset);
    }
}

/// Count assigned assets in all actor buckets.
pub fn count_assigned(assigned: &[Vec<Asset>]) -> usize {
    assigned.iter().map(|row| row.len()).sum()
}

fn shuffle_assets(assets: &mut [Asset], rng: &mut dyn RngCore) {
    let len = assets.len();
    if len <= 1 {
        return;
    }
    for idx in (1..len).rev() {
        let jdx = rand_idx(rng, idx + 1);
        assets.swap(idx, jdx);
    }
}

fn rand_idx(mut rng: &mut dyn RngCore, upper: usize) -> usize {
    let mut bytes = [0u8; 8];
    rng.fill_bytes_ext(&mut bytes);
    let num = u64::from_le_bytes(bytes);
    (num % upper as u64) as usize
}

#[cfg(test)]
mod tests {
    use z00z_core::{genesis::asset_std::asset_from_dev_class, AssetClass};
    use z00z_utils::rng::MockRngProvider;

    use super::{assign_class_split, assign_uniform_all, count_assigned};

    fn mk_asset(class: AssetClass, serial_id: u32, amount: u64) -> z00z_core::Asset {
        asset_from_dev_class(class, serial_id, amount).expect("asset")
    }

    #[test]
    fn test_class_split_by_policy() {
        let assets = vec![
            mk_asset(AssetClass::Coin, 1, 10),
            mk_asset(AssetClass::Nft, 2, 1),
            mk_asset(AssetClass::Token, 3, 20),
        ];
        let mut assigned = vec![Vec::new(), Vec::new(), Vec::new()];
        assign_class_split(&assets, &mut assigned);
        assert_eq!(assigned[0].len(), 1);
        assert_eq!(assigned[1].len(), 1);
        assert_eq!(assigned[2].len(), 1);
    }

    #[test]
    fn test_uniform_assign_is_conservative() {
        let mut assets = vec![
            mk_asset(AssetClass::Coin, 1, 10),
            mk_asset(AssetClass::Nft, 2, 1),
            mk_asset(AssetClass::Token, 3, 20),
            mk_asset(AssetClass::Token, 4, 30),
        ];
        let mut assigned = vec![Vec::new(), Vec::new(), Vec::new()];
        let provider = MockRngProvider::with_u64_seed(7);
        let mut rng = provider.rng();
        assign_uniform_all(&mut assets, &mut assigned, &mut rng);
        assert_eq!(count_assigned(&assigned), 4);
    }
}
