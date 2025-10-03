use crate::camera::{CameraControls, CameraService};
use crate::functions::{handle_status, handle_update, sync_ntp, NtpRequest, STILL_CAMERA_CONTROLS_FILENAME, VIDEO_CAMERA_CONTROLS_FILENAME};
use crate::settings::{BaseSettings, Settings};
use crate::updater::restart;
use crate::utils::{AsyncClientExt, ErrorExt, ResultExt};
use config::Config;
use pyo3::Python;
use reqwest::Client;
use rumqttc::v5::mqttbytes::v5::Packet;
use rumqttc::v5::{AsyncClient, Event, EventLoop, MqttOptions};
use std::env;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// Function that does critical startup stuff that must not fail
/// and auto updates if needed, in case something later fails and auto updater in loop
/// is not reached, does not work
/// Function can panic, as there is no recovering from critical startup
pub async fn critical_startup() -> (BaseSettings, AsyncClient, EventLoop, Client, PathBuf) {
    println!("Starting up");
    // Critical startup settings
    let base_settings = Config::builder()
        .add_source(config::File::with_name("base_settings"))
        .build()
        .unwrap()
        .try_deserialize::<BaseSettings>()
        .unwrap();

    println!("Base settings: \n {:?}", base_settings);

    // Set up MQTT
    let mut mqtt_options = MqttOptions::new(
        format!("pi_zero_${}", base_settings.pi_zero_id),
        &base_settings.mqtt_url,
        base_settings.mqtt_port,
    );
    mqtt_options.set_keep_alive(Duration::from_secs(120));

    let (mqtt_client, mut mqtt_event_loop) = AsyncClient::new(mqtt_options, 100);

    // Subscribe to update topic. Other topics are not critical, as only this can fix startup problem
    mqtt_client
        .subscribe_all_individual(&base_settings.update_topic, &base_settings.pi_zero_id)
        .await
        .unwrap();

    // Http client
    let http_client = Client::new();

    // Check for updates
    let should_restart = AtomicBool::new(false);
    let current_exe = env::current_exe().unwrap();

    println!("Checking for update");
    loop {
        let event = mqtt_event_loop.poll().await.unwrap();

        // Only process incoming packets, outgoing etc. are not relevant
        let Event::Incoming(Packet::Publish(p)) = &event else {
            continue;
        };

        // We have to receive at least one update message before exiting loop
        if p.topic == &base_settings.update_topic {
            let update_result = handle_update(
                &base_settings,
                &mqtt_client,
                &http_client,
                &should_restart,
                p,
                true,
            )
            .await;
            match update_result {
                Ok(_) => {
                    break;
                }
                Err(err) => {
                    err.send_error(&base_settings, &mqtt_client, &base_settings.update_topic)
                        .await
                        .unwrap_or_default();
                    println!("Failed to update: {:?}", err);
                    // Do not break, as if update failed, then probably should update
                }
            }
        }
    }

    if should_restart.load(Ordering::Relaxed) {
        restart(&current_exe);
    }

    (
        base_settings,
        mqtt_client,
        mqtt_event_loop,
        http_client,
        current_exe,
    )
}

/// Sets up subscriptions, camera controls etc. which are less critical for startup
pub async fn startup(
    base_settings: &BaseSettings,
    mqtt_client: &AsyncClient,
) -> (Settings, CameraService) {
    let settings = Config::builder()
        .add_source(config::File::with_name("settings"))
        .build()
        .unwrap()
        .try_deserialize::<Settings>()
        .unwrap();

    println!("Settings: {:?}", settings);

    mqtt_client
        .subscribe_to_all(&base_settings, &settings)
        .await
        .unwrap();

    println!("Subscribed");

    let still_controls: Option<CameraControls> =
        read_camera_controls(&base_settings, &mqtt_client, STILL_CAMERA_CONTROLS_FILENAME).await;
    let video_controls: Option<CameraControls> =
        read_camera_controls(&base_settings, &mqtt_client, VIDEO_CAMERA_CONTROLS_FILENAME).await;

    println!("Read controls from file");

    let camera_service = Python::attach(|py| -> Result<CameraService, anyhow::Error> {
        let still_controls_pydict = match &still_controls {
            Some(v) => Some(v.to_pydict(py)?),
            None => None,
        };
        let camera_service =
            CameraService::new(py, &still_controls, &video_controls, still_controls_pydict)?;
        Ok(camera_service)
    })
    .unwrap();

    println!("Set up camera service");

    handle_status(&base_settings, &settings, &mqtt_client, &camera_service)
        .await
        .unwrap();
    sync_ntp(&base_settings, &settings, &mqtt_client, &NtpRequest::Step)
        .await
        .unwrap();

    // if photos does not exist, create it
    if !tokio::fs::metadata("photos").await.is_ok() {
        tokio::fs::create_dir_all("photos").await.unwrap();
    }

    (settings, camera_service)
}

async fn read_camera_controls(
    base_settings: &BaseSettings,
    mqtt_client: &AsyncClient,
    filename: &str,
) -> Option<CameraControls> {
    if Path::new(filename).exists() {
        let json = tokio::fs::read_to_string(filename)
            .await
            .map_err(|e| anyhow::Error::from(e))
            .send_if_err(&base_settings, &mqtt_client, "error")
            .await;
        match json {
            Ok(v) => serde_json::from_str::<CameraControls>(&v)
                .map_err(|e| anyhow::Error::from(e))
                .send_if_err(&base_settings, &mqtt_client, "error")
                .await
                .ok(),
            Err(_) => None,
        }
    } else {
        None
    }
}
