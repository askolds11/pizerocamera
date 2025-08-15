mod camera;
mod command;
mod ntp;
mod requests;
mod responses;
mod status;
mod update;

use crate::camera::CameraService;
use crate::settings::{BaseSettings, Settings};
use crate::utils::PublishExt;
use crate::utils::ResultExt;
use camera::*;
pub use camera::{STILL_CAMERA_CONTROLS_FILENAME, VIDEO_CAMERA_CONTROLS_FILENAME};
use command::*;
pub use ntp::handle_ntp;
use reqwest::Client;
use rumqttc::v5::mqttbytes::v5::Publish;
use rumqttc::v5::AsyncClient;
pub use status::handle_status;
use std::sync::atomic::AtomicBool;
pub use update::handle_update;

pub async fn handle_notification(
    base_settings: &BaseSettings,
    settings: &Settings,
    mqtt_client: &AsyncClient,
    http_client: &Client,
    should_restart: &AtomicBool,
    camera_service: &mut CameraService,
    publish: &Publish,
    wall_nanoseconds: Option<i64>,
) {
    // Handle topic
    let _ = if publish.topic_matches_pi(&settings.ntp_topic, &base_settings.pi_zero_id) {
        handle_ntp(&base_settings, &settings, &mqtt_client)
            .await
            .send_if_err(&base_settings, &mqtt_client, &settings.ntp_topic)
            .await
    } else if publish.topic_matches_pi(&settings.camera_topic, &base_settings.pi_zero_id) {
        handle_picture(
            &base_settings,
            &settings,
            &mqtt_client,
            &http_client,
            camera_service,
            &publish,
            wall_nanoseconds,
        )
        .await
        .send_if_err(&base_settings, &mqtt_client, &settings.camera_topic)
        .await
    } else if publish.topic_matches_pi(&base_settings.update_topic, &base_settings.pi_zero_id) {
        handle_update(
            &base_settings,
            &mqtt_client,
            &http_client,
            &should_restart,
            &publish,
        )
        .await
        .send_if_err(&base_settings, &mqtt_client, &base_settings.update_topic)
        .await
    } else if publish.topic_matches_pi(&settings.command_topic, &base_settings.pi_zero_id) {
        handle_command(&base_settings, &settings, &mqtt_client, &publish)
            .await
            .send_if_err(&base_settings, &mqtt_client, &settings.command_topic)
            .await
    } else if publish.topic_matches_pi(&settings.status_topic, &base_settings.pi_zero_id) {
        handle_status(&base_settings, &settings, &mqtt_client, &camera_service)
            .await
            .send_if_err(&base_settings, &mqtt_client, &settings.status_topic)
            .await
    } else {
        Err(anyhow::Error::msg("Unknown topic"))
    };
}
