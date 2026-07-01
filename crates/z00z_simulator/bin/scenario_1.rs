use z00z_utils::logger::{Logger, StdoutLogger};

fn main() {
    if let Err(err) = run_from_args() {
        let logger = StdoutLogger;
        logger.error(&format!("sim_scenario_1.failed: {}", err));
        std::process::exit(1);
    }
}

fn run_from_args() -> Result<(), z00z_simulator::scenario_1::runner::Scenario1Err> {
    let mut args = std::env::args().skip(1);
    let mut cfg_path: Option<String> = None;
    let mut design_path: Option<String> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--config" => {
                let value = args.next().unwrap_or_else(|| {
                    eprintln!("missing value for --config");
                    std::process::exit(2);
                });
                cfg_path = Some(value);
            }
            "--design" => {
                let value = args.next().unwrap_or_else(|| {
                    eprintln!("missing value for --design");
                    std::process::exit(2);
                });
                design_path = Some(value);
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            other => {
                eprintln!("unknown argument: {other}");
                print_help();
                std::process::exit(2);
            }
        }
    }

    match (cfg_path, design_path) {
        (None, None) => z00z_simulator::scenario_1::runner::main(),
        (Some(cfg), Some(design)) => {
            let logger = StdoutLogger;
            let result = z00z_simulator::scenario_1::runner::run_with_paths(&cfg, &design)?;
            if result.is_ok() {
                logger.info(&format!(
                    "scenario_1.result: success, scenario_id={}",
                    result.scenario_id
                ));
            } else {
                logger.warn(&format!(
                    "scenario_1.result: warnings_or_failures, scenario_id={}, stages={}",
                    result.scenario_id,
                    result.stages.len()
                ));
            }
            Ok(())
        }
        _ => {
            eprintln!("--config and --design must be provided together");
            print_help();
            std::process::exit(2);
        }
    }
}

fn print_help() {
    println!("Usage: scenario_1 [--config <path> --design <path>]");
}
