use crate::camera::{CameraMode, CameraService};
use crate::settings::{BaseSettings, Settings};
use crate::utils::{AsyncClientExt, SuccessWrapper, execute_command};
use rumqttc::v5::AsyncClient;
use serde::Serialize;

pub async fn handle_status(
    base_settings: &BaseSettings,
    settings: &Settings,
    mqtt_client: &AsyncClient,
    camera_service: &CameraService,
) -> Result<(), anyhow::Error> {
    let version = env!("CARGO_PKG_VERSION").to_string();
    let ip_address = execute_command("hostname -I | awk '{print $1}'")
        .ok()
        .map(|v| v.trim().to_string());
    let camera_mode = (&camera_service.camera_mode).clone();

    let status = Status {
        version,
        ip_address,
        camera_mode,
    };

    let status_msg = SuccessWrapper {
        success: true,
        value: &status,
    };

    let json = serde_json::to_string(&status_msg)?;

    mqtt_client
        .publish_individual(&settings.status_topic, &base_settings.pi_zero_id, json)
        .await?;

    Ok(())
}

#[derive(Serialize)]
pub struct Status {
    version: String,
    ip_address: Option<String>,
    camera_mode: CameraMode,
}
