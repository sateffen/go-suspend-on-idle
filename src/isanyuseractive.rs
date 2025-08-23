use std::{io, process::Command};

pub fn is_any_user_active() -> Result<bool, io::Error> {
    let loginctl_output = Command::new("loginctl")
        .arg("list-sessions")
        .arg("--no-legend")
        .output()?;

    Ok(loginctl_output.status.success() && !String::from_utf8_lossy(&loginctl_output.stdout).trim().is_empty())
}
