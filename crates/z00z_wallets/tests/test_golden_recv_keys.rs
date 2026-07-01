use z00z_wallets::key::{ReceiverKeys, ReceiverSecret};

#[test]
fn test_golden_recv_keys() {
    let owner_hex = "a17fb65d63b430d16be307cd600652cd4fac41450b257d504b6ccf96244d551d";
    let view_sk_hex = "14ef70529859ec3146a2d101d2f1b511b586717a5f835348e72260e8e31d9b06";
    let view_pk_hex = "68011c2b81d2adab05b4c778e3460435438537982d92d0871542bee02d497d4f";
    let ident_sk_hex = "c863d6f3433188986a537bc46dfba8707b9ea8f1fe52880b4fc61b87ad83a604";
    let ident_pk_hex = "6e290fa18514363bac2600a7d1c6cae7dd4e9b727780188191ca900a81d90522";

    let recv = ReceiverSecret::from_bytes([0x11; 32]).expect("receiver secret");
    let keys = ReceiverKeys::from_receiver_secret(recv).expect("receiver keys");

    assert_eq!(hex::encode(keys.owner_handle), owner_hex);
    assert_eq!(hex::encode(keys.reveal_view_sk().as_bytes()), view_sk_hex);
    assert_eq!(hex::encode(keys.view_pk.as_bytes()), view_pk_hex);
    assert_eq!(
        hex::encode(keys.reveal_identity_sk().as_bytes()),
        ident_sk_hex
    );
    assert_eq!(hex::encode(keys.identity_pk.as_bytes()), ident_pk_hex);
}
