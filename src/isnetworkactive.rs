use std::fs::File;
use std::{io, io::BufRead, io::BufReader};

pub fn is_network_active() -> Result<bool, io::Error> {
    Ok(
        has_non_localhost_connections("/proc/net/tcp")? ||
        has_non_localhost_connections("/proc/net/tcp6")?
    )
}

fn has_non_localhost_connections(proc_file_path: &str) -> Result<bool, io::Error> {
    let proc_file_handle = File::open(proc_file_path)?;
    let proc_file_reader = BufReader::new(proc_file_handle);

    for line in proc_file_reader.lines() {
        let line = line?;
        let fields: Vec<&str> = line.split_whitespace().collect();

        // if the line is too short or fields[3] (connection-state) is not "01" (established)
        if fields.len() < 4 || fields[3] != "01" {
            continue;
        }

        if !is_localhost_hex(fields[1]) {
            return Ok(true);
        }
    }

    Ok(false)
}

fn is_localhost_hex(address: &str) -> bool {
    let colon_index = address.find(":");
    let ip_hex_string = match colon_index {
        Some(index) => address.get(0..index),
        None => None,
    };

    match ip_hex_string {
        Some(hex) => {
            (hex.len() == 8 && hex.starts_with("7F")) ||
            (hex.len() == 32 && hex == "00000000000000000000000000000001")
        }
        None => false,
    }
}
