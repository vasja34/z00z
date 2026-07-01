use super::{Asset, RngCore, RngCoreExt};

use z00z_core::AssetClass;

#[derive(Debug, Clone, Copy)]
pub(crate) enum ClaimMode {
    ClassSplit,
    CoinSets,
    UniformAll,
}

impl ClaimMode {
    pub(crate) fn from_active(active: Option<&str>) -> Self {
        match active.unwrap_or("uniform_all") {
            "class_split" => Self::ClassSplit,
            "coin_sets" => Self::CoinSets,
            _ => Self::UniformAll,
        }
    }

    pub(crate) fn mode_str(self) -> &'static str {
        match self {
            Self::ClassSplit => "class_split",
            Self::CoinSets => "coin_sets",
            Self::UniformAll => "uniform_all",
        }
    }
}

pub(crate) fn want_half_abort(mode: Option<&str>) -> bool {
    matches!(mode, Some("half_abort"))
}

pub(crate) fn want_reject_first(mode: Option<&str>) -> bool {
    matches!(mode, Some("reject_first"))
}

pub(crate) fn want_replay_first(mode: Option<&str>) -> bool {
    matches!(mode, Some("replay_first"))
}

pub(crate) fn assign_class_split(assets: &[Asset], assigned: &mut [Vec<Asset>]) {
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

pub(crate) fn assign_coin_sets(
    assets: &[Asset],
    assigned: &mut [Vec<Asset>],
    rng: &mut dyn RngCore,
) {
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
    for (index, asset) in coins.into_iter().enumerate() {
        assigned[index % 3].push(asset);
    }
}

pub(crate) fn assign_uniform_all(
    assets: &mut [Asset],
    assigned: &mut [Vec<Asset>],
    rng: &mut dyn RngCore,
) {
    shuffle_assets(assets, rng);
    for (index, asset) in assets.iter().cloned().enumerate() {
        assigned[index % 3].push(asset);
    }
}

pub(crate) fn count_assigned(assigned: &[Vec<Asset>]) -> usize {
    assigned.iter().map(|items| items.len()).sum()
}

fn shuffle_assets(assets: &mut [Asset], rng: &mut dyn RngCore) {
    let len = assets.len();
    if len <= 1 {
        return;
    }
    for index in (1..len).rev() {
        let swap_idx = rand_idx(rng, index + 1);
        assets.swap(index, swap_idx);
    }
}

fn rand_idx(mut rng: &mut dyn RngCore, upper: usize) -> usize {
    let mut bytes = [0u8; 8];
    rng.fill_bytes_ext(&mut bytes);
    let value = u64::from_le_bytes(bytes);
    (value % upper as u64) as usize
}
