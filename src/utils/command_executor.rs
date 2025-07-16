use std::process::Command;

pub fn execute_command(command_str: &str) -> Result<String, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command_str)
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}