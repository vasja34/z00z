use std::path::PathBuf;

#[path = "test_inc/test_mod.rs"]
mod test_common;

use test_common::managed_test_output_root;
use z00z_crypto::{
    domains::{ReceiverIdDomain, ViewKeyDomain},
    hash::{hash_to_scalar_zk, hash_zk},
};
use z00z_utils::io::{create_dir_all, write_file};
use z00z_wallets::key::{derive_owner_handle, derive_view_secret_key, ReceiverSecret};

struct VecRow {
    id: &'static str,
    seed: &'static str,
    rid: &'static str,
    view: &'static str,
}

fn out_dir() -> PathBuf {
    managed_test_output_root("e2e13")
}

fn decode_hex32(src: &str) -> [u8; 32] {
    let raw = hex::decode(src).expect("hex decode");
    raw.try_into().expect("hex len 32")
}

fn vec_set() -> Vec<VecRow> {
    vec![
        VecRow {
            id: "vec-01",
            seed: "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20",
            rid: "ed85816553b588ba6c2e82ae0e50e66d12cd1a7a0b0c247431ae3aa129d45385",
            view: "4ee8c653c16e50a927ad9ac7fb3dd634e3abd01556b216365a5a09b6eafc0e0b",
        },
        VecRow {
            id: "vec-02",
            seed: "12131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f3031",
            rid: "4c814413ded913baad34d18f1a8bff0e64357a1782b40ba4e90c6ee7f51fff33",
            view: "cd99feef6f588ed8884acb7c5dfbebb7faac759aafb8908dc799135a0cdfa20f",
        },
        VecRow {
            id: "vec-03",
            seed: "232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f404142",
            rid: "68d8f0e23dfacda0f3d2e0c097d498714cd957cddff3df8c70629fa6b7a3a5ca",
            view: "c70d55193795b6a0e029fddac326a824c93ddb0fed0327fc7a5b870e15c24204",
        },
        VecRow {
            id: "vec-04",
            seed: "3435363738393a3b3c3d3e3f404142434445464748494a4b4c4d4e4f50515253",
            rid: "2c917d4334020b654aff1d354bf23ee617c50fefa569bee346fa3f75a572b3a7",
            view: "0a73ded13ae833dc1fafe18ecc0f85af01279d2052741868797e3e638b9e240b",
        },
        VecRow {
            id: "vec-05",
            seed: "45464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f6061626364",
            rid: "46756e6b04b3a176273c5f3de2afa7dd55edfeacae594025b4c753cc76af47a6",
            view: "b045d0c8cf23fa197bd246da72f76ed32c47af12ffa092aab1b4e634128f6003",
        },
        VecRow {
            id: "vec-06",
            seed: "565758595a5b5c5d5e5f606162636465666768696a6b6c6d6e6f707172737475",
            rid: "676dcadd270b871a4f5ca0bbbd69297ed9cc6a68d7aa8a1089e7fe19332db342",
            view: "cd549ff7b230a8325027e5333b412cef1ef6885fdae50e53b53a62a1cd27a009",
        },
        VecRow {
            id: "vec-07",
            seed: "6768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f80818283848586",
            rid: "d7f40b128110d63a98450ee50f727530f1f070d0dc74a8c27a1df551e9f30e30",
            view: "7ef2db8f15e24f1b4561ab3825e88f7225b7d50f557d1c1910419d8a7d6be007",
        },
        VecRow {
            id: "vec-08",
            seed: "78797a7b7c7d7e7f808182838485868788898a8b8c8d8e8f9091929394959697",
            rid: "f0ff8d5d65363226fead8a778c3a14620318ce4acce0257942ad987ee7b44f0c",
            view: "655bfb28a6ed6685ebd0604d2aac34bbc454d40f848490928a25f572097aa009",
        },
        VecRow {
            id: "vec-09",
            seed: "898a8b8c8d8e8f909192939495969798999a9b9c9d9e9fa0a1a2a3a4a5a6a7a8",
            rid: "e17249607d8651dd1065dc8004f1fbc2fc8ddb9a964d77a259c9ea047ad81f4f",
            view: "2624a6c3a51df8b71ae3097ed1831b395610d11037d1a118a72ae73442337208",
        },
        VecRow {
            id: "vec-10",
            seed: "9a9b9c9d9e9fa0a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9",
            rid: "d521781fa2d14231f12d6edbc80d0ca03c59e5916aec770eef5881e600f3e97f",
            view: "c63a8b40cd7587251e61bf02c6325e510bfece577f3a61c60b1ed04c616cb80a",
        },
    ]
}

#[test]
fn test_stage4_parity() {
    if cfg!(debug_assertions) {
        return;
    }

    let rows_fix = vec_set();
    assert!(rows_fix.len() >= 10, "seed set must be at least 10 vectors");

    let mut rows = Vec::new();
    let mut rep = String::from("E2E-13 parity\n");

    for row in &rows_fix {
        let seed = decode_hex32(row.seed);
        let rid_fix = decode_hex32(row.rid);
        let view_fix = decode_hex32(row.view);

        let recv = ReceiverSecret::from_bytes(seed).expect("receiver secret");

        let rid_wallet = derive_owner_handle(&recv);
        let view_wallet = derive_view_secret_key(&recv)
            .expect("view wallet")
            .to_bytes();

        let rid_base = hash_zk::<ReceiverIdDomain>("", &[&seed[..]]);
        let view_base = hash_to_scalar_zk::<ViewKeyDomain>("", &[&seed[..]])
            .expect("view base")
            .to_bytes();

        assert_eq!(rid_wallet, rid_fix, "RID wallet mismatch on {}", row.id);
        assert_eq!(view_wallet, view_fix, "VIEW wallet mismatch on {}", row.id);
        assert_eq!(rid_base, rid_fix, "RID baseline mismatch on {}", row.id);
        assert_eq!(view_base, view_fix, "VIEW baseline mismatch on {}", row.id);

        rep.push_str(&format!("id={} ok=1\n", row.id));
        rows.push(serde_json::json!({
            "id": row.id,
            "seed": hex::encode(seed),
            "rid_wallet": hex::encode(rid_wallet),
            "rid_base": hex::encode(rid_base),
            "rid_fix": row.rid,
            "view_wallet": hex::encode(view_wallet),
            "view_base": hex::encode(view_base),
            "view_fix": row.view
        }));
    }

    create_dir_all(out_dir()).expect("mkdir outputs/tests/e2e13");
    write_file(out_dir().join("e2e13_parity.txt"), rep.as_bytes()).expect("write parity report");

    let vecs = serde_json::json!({
        "test": "E2E-13",
        "vectors": rows
    });
    let vec_bytes = serde_json::to_vec_pretty(&vecs).expect("json vectors");
    write_file(out_dir().join("e2e13_vectors.json"), &vec_bytes).expect("write vectors");
}
