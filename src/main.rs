mod isnetworkactive;
mod locallogger;
mod systemdbindings;

use crate::isnetworkactive::is_network_active;
use crate::locallogger::LocalLogger;
use crate::systemdbindings::{has_active_user_sessions, systemd_suspend};
use log::{debug, error, warn};
use std::{env, error, thread, time};

struct CLIArgs {
    verbose: bool,
    shutdown: bool,
    idle_minutes: u8,
}

fn parse_cli_args() -> CLIArgs {
    let mut args = CLIArgs {
        verbose: false,
        shutdown: false,
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
            "-s" | "--shutdown" => {
                args.shutdown = true;
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

    let mut logger: LocalLogger = LocalLogger{
        log_level: log::Level::Info,
    };

    if cli_args.verbose {
        logger.log_level = log::Level::Debug;
    }

    let log_level_filter = logger.log_level.to_level_filter();
    log::set_boxed_logger(Box::new(logger)).map(move |()| log::set_max_level(log_level_filter))?;

    let mut current_inactivity_counter: u8 = 0;
    let one_minute = time::Duration::from_secs(60);

    loop {
        thread::sleep(one_minute);

        if is_network_active()? || has_active_user_sessions()? {
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
        let suspend_result = systemd_suspend(cli_args.shutdown);

        match suspend_result {
            Ok(_) => {
                current_inactivity_counter = 0;
            }
            Err(error) => {
                error!("failed to execute 'systemctl suspend': {error:?}");
                return Err(error.into());
            },
        }
    }
}
