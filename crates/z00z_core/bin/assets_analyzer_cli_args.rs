use super::{AnalyzerConfig, AssetClass, PathBuf};

pub(super) fn parse_args() -> AnalyzerConfig {
    let args: Vec<String> = std::env::args().collect();

    let mut input_dir: Option<PathBuf> = None;
    let mut filter_class: Option<AssetClass> = None;
    let mut verbose = false;
    let mut input_files: Vec<PathBuf> = Vec::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => {
                if i + 1 < args.len() {
                    input_dir = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    eprintln!("Error: --input requires a directory path");
                    print_usage();
                    std::process::exit(1);
                }
            }
            "--file" | "-f" => {
                if i + 1 < args.len() {
                    input_files.push(PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    eprintln!("Error: --file requires a file path");
                    print_usage();
                    std::process::exit(1);
                }
            }
            "--class" => {
                if i + 1 < args.len() {
                    filter_class = match args[i + 1].to_lowercase().as_str() {
                        "coin" => Some(AssetClass::Coin),
                        "token" => Some(AssetClass::Token),
                        "nft" => Some(AssetClass::Nft),
                        "void" => Some(AssetClass::Void),
                        _ => {
                            eprintln!("Error: Invalid class. Use: coin, token, nft, void");
                            std::process::exit(1);
                        }
                    };
                    i += 2;
                } else {
                    eprintln!("Error: --class requires a value");
                    print_usage();
                    std::process::exit(1);
                }
            }
            "--verbose" => {
                verbose = true;
                i += 1;
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            _ => {
                eprintln!("Error: Unknown argument: {}", args[i]);
                print_usage();
                std::process::exit(1);
            }
        }
    }

    if input_dir.is_none() && input_files.is_empty() {
        eprintln!("Error: --input or --file is required");
        print_usage();
        std::process::exit(1);
    }

    AnalyzerConfig {
        input_dir,
        input_files,
        filter_class,
        verbose,
    }
}

/// Print CLI usage information
fn print_usage() {
    println!("Z00Z Genesis Assets Analyzer CLI");
    println!();
    println!("USAGE:");
    println!("    assets_analyzer_cli --input <DIR> [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    --input <DIR>      Path to genesis output directory (required)");
    println!(
        "    --file, -f <FILE>  Analyze specific file(s) (bin or json). Can be used multiple times"
    );
    println!("    --class <CLASS>    Filter by asset class: coin, token, nft, void");
    println!("    --verbose          Show detailed serial ID distribution");
    println!("    --help, -h         Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    # Analyze entire genesis directory");
    println!("    cargo run --bin assets_analyzer_cli -- --input crates/z00z_core/outputs/genesis/genesis_devnet_20251208");
    println!();
    println!("    # Analyze only coins");
    println!("    cargo run --bin assets_analyzer_cli -- --input crates/z00z_core/outputs/genesis/genesis_devnet_20251208 --class coin");
    println!();
    println!("    # Show detailed serial distribution");
    println!("    cargo run --bin assets_analyzer_cli -- --input crates/z00z_core/outputs/genesis/genesis_devnet_20251208 --verbose");
}
