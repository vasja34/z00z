use super::{HashMap, Mutex, OnceLock, SimContext, Stage2ActorCfg};
use crate::SimActor;

pub(crate) struct ActorSpec {
    pub(crate) name: String,
    pub(crate) password: String,
    pub(crate) rng_seed: u64,
}

const DEF_ACTORS: &[(&str, &str, u64)] = &[
    ("alice", "Alice_Pass_Z00Z_42!", 42),
    ("bob", "Bob_Pass_Z00Z_43!", 43),
    ("charlie", "Charlie_Pass_Z00Z_44!", 44),
];

static ACTOR_PASS: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

fn actor_pass_store() -> &'static Mutex<HashMap<String, String>> {
    ACTOR_PASS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn set_actor_passwords(actors: &[ActorSpec]) {
    if let Ok(mut map) = actor_pass_store().lock() {
        map.clear();
        for actor in actors {
            map.insert(actor.name.to_ascii_lowercase(), actor.password.clone());
        }
    }
}

// SIMULATOR-ONLY: DO NOT MOVE TO CORE.
pub(crate) fn actor_password(actor_name: &str) -> Option<String> {
    let name = actor_name.to_ascii_lowercase();
    if let Ok(map) = actor_pass_store().lock() {
        if let Some(pass) = map.get(&name) {
            return Some(pass.clone());
        }
    }
    DEF_ACTORS
        .iter()
        .find(|item| item.0 == name)
        .map(|item| item.1.to_string())
}

pub(crate) fn actor_runtime_password(actor: &SimActor) -> Option<String> {
    actor
        .password
        .clone()
        .filter(|password| !password.is_empty())
        .or_else(|| actor_password(&actor.name))
}

pub(crate) fn cfg_actors(ctx: &SimContext) -> Vec<ActorSpec> {
    let cfg_rows = ctx
        .config
        .stage2_wallet_create
        .as_ref()
        .map(|row| row.actors.as_slice())
        .unwrap_or(&[]);
    if cfg_rows.is_empty() {
        return DEF_ACTORS
            .iter()
            .map(|item| ActorSpec {
                name: item.0.to_string(),
                password: item.1.to_string(),
                rng_seed: item.2,
            })
            .collect();
    }
    cfg_rows
        .iter()
        .map(|row: &Stage2ActorCfg| ActorSpec {
            name: row.name.to_ascii_lowercase(),
            password: row.password.clone(),
            rng_seed: row.mock_rng_seed,
        })
        .collect()
}

pub(crate) fn cfg_net(ctx: &SimContext) -> (String, String) {
    let wallet_net = "p2p".to_string();
    if let Some(cfg) = &ctx.config.stage2_wallet_create {
        return (wallet_net, cfg.wallet_chain.clone());
    }
    (wallet_net, ctx.config.chain.clone())
}
