use super::{AssetClass, AssetFilters, ExtractorConfig, OutputFormat, Path, PathBuf};

pub(super) fn parse_args() -> ExtractorConfig {
    let args: Vec<String> = std::env::args().collect();

    let mut input_dir: Option<PathBuf> = None;
    let mut output_file: Option<PathBuf> = None;
    let mut filters = AssetFilters::default();
    let mut output_format = OutputFormat::Json;
    let mut include_proofs = true;
    let mut verbose = false;

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
            "--output" => {
                if i + 1 < args.len() {
                    output_file = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    eprintln!("Error: --output requires a file path");
                    print_usage();
                    std::process::exit(1);
                }
            }
            "--class" => {
                if i + 1 < args.len() {
                    filters.class = match args[i + 1].to_lowercase().as_str() {
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
            "--serial-range" => {
                if i + 1 < args.len() {
                    match super::assets_extractor_cli_ops::parse_range(&args[i + 1]) {
                        Ok(range) => filters.serial_range = Some(range),
                        Err(e) => {
                            eprintln!("Error: Invalid serial range: {}", e);
                            std::process::exit(1);
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("Error: --serial-range requires a value (e.g., 0-99)");
                    print_usage();
                    std::process::exit(1);
                }
            }
            "--amount-min" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u64>() {
                        Ok(val) => filters.amount_min = Some(val),
                        Err(_) => {
                            eprintln!("Error: Invalid amount-min value");
                            std::process::exit(1);
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("Error: --amount-min requires a value");
                    print_usage();
                    std::process::exit(1);
                }
            }
            "--amount-max" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u64>() {
                        Ok(val) => filters.amount_max = Some(val),
                        Err(_) => {
                            eprintln!("Error: Invalid amount-max value");
                            std::process::exit(1);
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("Error: --amount-max requires a value");
                    print_usage();
                    std::process::exit(1);
                }
            }
            "--definition-id" | "--symbol" => {
                if i + 1 < args.len() {
                    filters.definition_symbol = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --symbol requires a value");
                    print_usage();
                    std::process::exit(1);
                }
            }
            "--format" => {
                if i + 1 < args.len() {
                    output_format = match args[i + 1].to_lowercase().as_str() {
                        "json" => OutputFormat::Json,
                        "bincode" | "bin" => OutputFormat::Bincode,
                        "csv" => OutputFormat::Csv,
                        _ => {
                            eprintln!("Error: Invalid format. Use: json, bincode, csv");
                            std::process::exit(1);
                        }
                    };
                    i += 2;
                } else {
                    eprintln!("Error: --format requires a value");
                    print_usage();
                    std::process::exit(1);
                }
            }
            "--no-proofs" => {
                include_proofs = false;
                i += 1;
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

    if input_dir.is_none() {
        eprintln!("Error: --input is required");
        print_usage();
        std::process::exit(1);
    }

    let input_dir = input_dir.unwrap();

    // Normalize / generate output path. If user supplied a bare filename (no parent)
    // or omitted --output, place result inside the input directory with prefix
    // `assets_extract_` and an extension matching the selected format.
    let output_path = make_output_path(&input_dir, output_file, output_format);

    ExtractorConfig {
        input_dir,
        output_file: output_path,
        filters,
        output_format,
        include_proofs,
        verbose,
    }
}

/// Create or normalize the output path so it's placed in `input_dir` with
/// prefix `assets_extract_` when the provided path is a bare filename or None.
fn make_output_path(input_dir: &Path, requested: Option<PathBuf>, format: OutputFormat) -> PathBuf {
    // determine extension for format
    let ext = match format {
        OutputFormat::Json => "json",
        OutputFormat::Bincode => "bin",
        OutputFormat::Csv => "csv",
    };

    if let Some(mut p) = requested {
        // If requested is a directory, place default file inside it
        if p.exists() && p.is_dir() {
            let file_name = format!(
                "assets_extract_{}.{}",
                input_dir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("out"),
                ext
            );
            return p.join(file_name);
        }

        // If requested has no parent (bare filename) or parent is current dir, place inside input_dir
        let use_input_dir = match p.parent() {
            None => true,
            Some(parent) => parent.as_os_str() == "" || parent == Path::new("."),
        };

        if use_input_dir {
            let fname = p.file_name().and_then(|n| n.to_str()).unwrap_or("output");
            let file_name = format!("assets_extract_{}", fname);
            let mut out = input_dir.join(file_name);
            // ensure extension matches format
            if out.extension().is_none() {
                out.set_extension(ext);
            }
            return out;
        }

        // If a full path was provided, respect it but ensure extension matches
        if p.extension().is_none() {
            p.set_extension(ext);
        }
        return p;
    }

    // No requested path: generate default in input_dir
    let file_name = format!(
        "assets_extract_{}.{}",
        input_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("out"),
        ext
    );
    input_dir.join(file_name)
}

/// Print CLI usage information
fn print_usage() {
    println!("Z00Z Genesis Assets Extractor CLI");
    println!();
    println!("USAGE:");
    println!("    assets_extractor_cli --input <DIR> --output <FILE> [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    --input <DIR>           Path to genesis output directory (required)");
    println!("    --output <FILE>         Output file path (required)");
    println!("    --class <CLASS>         Filter by asset class: coin, token, nft, void");
    println!("    --serial-range <RANGE>  Serial ID range (e.g., 0-99, 100-200)");
    println!("    --amount-min <VALUE>    Minimum amount (inclusive)");
    println!("    --amount-max <VALUE>    Maximum amount (inclusive)");
    println!("    --symbol <SYMBOL>       Filter by asset symbol (e.g., Z00Z, zUSD)");
    println!("    --format <FMT>          Output format: json, bincode, csv (default: json)");
    println!("    --no-proofs             Exclude range proofs from output");
    println!("    --verbose               Show detailed extraction progress");
    println!("    --help, -h              Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    # Extract all coins");
    println!("    cargo run --bin assets_extractor_cli -- --input genesis_devnet_20251208 --class coin --output coins.json");
    println!();
    println!("    # Extract serial range");
    println!("    cargo run --bin assets_extractor_cli -- --input genesis_devnet_20251208 --serial-range 0-99 --output first_100.bin");
    println!();
    println!("    # Extract with amount filter");
    println!("    cargo run --bin assets_extractor_cli -- --input genesis_devnet_20251208 --amount-min 10000 --output high_value.json");
    println!();
    println!("    # Extract to CSV");
    println!("    cargo run --bin assets_extractor_cli -- --input genesis_devnet_20251208 --class all --output summary.csv --format csv");
}
