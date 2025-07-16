use serde::Deserialize;

/// Settings that are required for bare minimum communication with server
#[derive(Debug, Deserialize)]
pub struct BaseSettings {
    /// Id of the Pi Zero (e.g. A0, A1, ...)
    pub pi_zero_id: String,
    /// Address of web server (updates, pictures, ...)
    pub server_url: String,
    /// Address of MQTT server
    pub mqtt_url: String,
    /// Port for MQTT server
    pub mqtt_port: u16,
    /// MQTT topic to check for updates
    pub update_topic: String,
}

/// Settings that could eventually be updated, are not critical to startup
#[derive(Debug, Deserialize)]
pub struct Settings {
    /// Address of NTP server
    pub ntp_server_url: String,
    /// MQTT topic for time sync (ntp)
    pub ntp_topic: String,
    /// MQTT topic for taking pictures
    pub camera_topic: String,
    /// MQTT topic for generic linux commands
    pub command_topic: String,
    /// MQTT topic for generic linux commands
    pub status_topic: String,
}