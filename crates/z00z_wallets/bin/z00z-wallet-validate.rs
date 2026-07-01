#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use std::path::PathBuf;

    use z00z_wallets::db::validate_wallet_file_codes;

    const USAGE: &str = "Usage: z00z-wallet-validate <path-to-wallet.wlt>";

    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 || matches!(args.get(1).map(String::as_str), Some("-h" | "--help")) {
        println!("{USAGE}");
        return;
    }

    if args.len() != 2 {
        eprintln!("{USAGE}");
        std::process::exit(2);
    }

    let path = PathBuf::from(&args[1]);
    match validate_wallet_file_codes(&path) {
        Ok(diags) => {
            if diags.is_empty() {
                return;
            }

            for code in diags {
                println!("{code}");
            }
            std::process::exit(1);
        }
        Err(err) => {
            eprintln!("Error: {err}");
            std::process::exit(3);
        }
    }
}
