mod isanyuseractive;
mod isnetworkactive;

use crate::isanyuseractive::is_any_user_active;
use crate::isnetworkactive::is_network_active;
use log::{debug, error, warn, LevelFilter};
use simple_logger::SimpleLogger;
use std::{env, error, thread, time, process::Command};

struct CLIArgs {
    verbose: bool,
    idle_minutes: u8,
}

fn parse_cli_args() -> CLIArgs {
    let mut args = CLIArgs {
        verbose: false,
        idle_minutes: 3,
    };

    let cli_args: Vec<String> = env::args().collect();
    
    // 0 = the binaries name
    let mut i = 1;
    while i < cli_args.len() {
        match cli_args[i].as_str() {
            "-v" | "--verbose" => {
                args.verbose = true;
            }
            "-i" | "--idle-minutes" => {
                if i + 1 < cli_args.len() {
                    if let Ok(minutes) = cli_args[i + 1].parse::<u8>() {
                        args.idle_minutes = minutes;
                    } else {
                        error!("Invalid value for idle minutes: {}", cli_args[i + 1]);
                        std::process::exit(1);
                    }
                    i += 1;
                }
            }
            "-h" | "--help" => {
                println!("Usage: {} [OPTIONS]", cli_args[0]);
                println!("Options:");
                println!("  -v, --verbose          Enable verbose logging (default: false)");
                println!("  -i, --idle-minutes N   Minutes to wait before suspending (default: 3)");
                println!("  -h, --help             Show this help message");
                std::process::exit(0);
            }
            _ => warn!("Unknown argument: {}", cli_args[i])
        }
        i += 1;
    }
    
    return args;
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let cli_args = parse_cli_args();

    SimpleLogger::new()
        .with_level(if cli_args.verbose {LevelFilter::Debug} else {LevelFilter::Info})
        .env()
        .init()?;

    let mut current_inactivity_counter: u8 = 0;
    let one_minute = time::Duration::from_secs(60);

    loop {
        thread::sleep(one_minute);

        if is_network_active()? || is_any_user_active()? {
            debug!("system is in use, skip suspending...");
            current_inactivity_counter = 0;
            continue;
        }
        // else system is idle

        current_inactivity_counter += 1;

        if current_inactivity_counter < cli_args.idle_minutes {
            debug!("system is inactive, counter={}", current_inactivity_counter);
            continue;
        }

        debug!("system is inactive for long period, start suspending...");
        let suspend_result = Command::new("systemctl")
            .arg("suspend")
            .status();

        match suspend_result {
            Ok(status) => {
                if !status.success() {
                    error!("error executing 'systemctl suspend', exitcode: {}", status.code().unwrap_or(-1));
                    continue;
                }
                // else we are back from suspend
                current_inactivity_counter = 0;
            }
            Err(error) => {
                error!("failed to execute 'systemctl suspend': {error:?}");
                return Err(error.into());
            },
        }
    }
}
