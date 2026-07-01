use z00z_crypto::expert::encoding::to_hex;
use z00z_crypto::{domains::ViewKeyDomain, hash::hash_to_scalar_zk};

fn print_view(name: &str, secret: [u8; 32]) {
    let scalar = hash_to_scalar_zk::<ViewKeyDomain>("", &[&secret]).expect("hash_to_scalar_zk");
    println!("{}={}", name, to_hex(&scalar.to_bytes()));
}

fn main() {
    print_view("ZERO_VIEW_SK", [0x00; 32]);
    print_view("ALICE_VIEW_SK", [0x11; 32]);
    print_view("BOB_VIEW_SK", [0x22; 32]);
    print_view("MAX_VIEW_SK", [0xFF; 32]);
}
