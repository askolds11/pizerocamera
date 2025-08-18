use crate::settings::Settings;
use crate::utils::execute_command;

pub fn ntp_sync_step(settings: &Settings) -> Result<String, anyhow::Error> {
    // TODO: Use sntp
    execute_command(format!("sudo ntpdate -b {}", settings.ntp_server_url).as_str()).map_err(anyhow::Error::msg)
}

pub fn ntp_sync_slew(settings: &Settings) -> Result<String, anyhow::Error> {
    // TODO: Use sntp
    execute_command(format!("sudo ntpdate -B {}", settings.ntp_server_url).as_str()).map_err(anyhow::Error::msg)
}
