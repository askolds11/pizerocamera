use crate::settings::Settings;
use crate::utils::execute_command;

pub fn ntp_sync(settings: &Settings) -> Result<String, anyhow::Error> {
    // TODO: Use sntp
    execute_command(format!("sudo -B ntpdate {}", settings.ntp_server_url).as_str()).map_err(anyhow::Error::msg)
}
