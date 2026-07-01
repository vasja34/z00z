use z00z_utils::rng::{MockRngProvider, RngCoreExt};
use z00z_wallets::key::{ReceiverKeys, ReceiverSecret};

fn secret_from_seed(seed: u64) -> [u8; 32] {
    let mut rng = MockRngProvider::with_u64_seed(seed).rng();
    let mut secret = [0u8; 32];
    rng.fill_bytes_ext(&mut secret);

    if secret == [0u8; 32] {
        let mut retry = MockRngProvider::with_u64_seed(seed + 1).rng();
        retry.fill_bytes_ext(&mut secret);
    }

    secret
}

#[test]
fn test_stage2_receiver_sig_valid() {
    for (seed, name) in [(42_u64, "alice"), (43_u64, "bob"), (44_u64, "charlie")] {
        let recv_secret = ReceiverSecret::from_bytes(secret_from_seed(seed))
            .expect("receiver secret from deterministic seed");
        let keys = ReceiverKeys::from_receiver_secret(recv_secret)
            .expect("derive receiver keys from receiver secret");
        let card = keys.export_receiver_card().expect("export receiver card");

        assert_ne!(
            card.signature, [0u8; 64],
            "{name}: signature must not be zero"
        );
        assert!(card.verify().is_ok(), "{name}: receiver card sig invalid");
    }
}
