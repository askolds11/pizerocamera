use std::env;
use crate::camera::{CameraControls, CameraService};
use crate::functions::{STILL_CAMERA_CONTROLS_FILENAME, VIDEO_CAMERA_CONTROLS_FILENAME};
use crate::ntp_sync::ntp_sync;
use crate::settings::{BaseSettings, Settings};
use crate::updater::{restart, update};
use crate::utils::{AsyncClientExt, ResultExt, SuccessWrapper};
use config::Config;
use pyo3::Python;
use reqwest::Client;
use rumqttc::v5::mqttbytes::v5::Packet;
use rumqttc::v5::{AsyncClient, Event, EventLoop, MqttOptions};
use semver::Version;
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
    let mut mqttoptions = MqttOptions::new(
        format!("pi_zero_${}", base_settings.pi_zero_id),
        &base_settings.mqtt_url,
        base_settings.mqtt_port,
    );
    mqttoptions.set_keep_alive(Duration::from_secs(120));

    let (mqtt_client, mut mqtt_eventloop) = AsyncClient::new(mqttoptions, 10);

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
        let event = mqtt_eventloop.poll().await.unwrap();

        // Only process incoming packets, outgoing etc. are not relevant
        let Event::Incoming(Packet::Publish(p)) = &event else {
            continue;
        };

        // We have to receive at least one update message before exiting loop
        if p.topic == &base_settings.update_topic {
            let payload = str::from_utf8(&p.payload).unwrap();
            let new_version = Version::parse(payload);
            match new_version {
                Ok(new_version) => {
                    let version = env!("CARGO_PKG_VERSION");
                    // this should not fail, because cargo version is guaranteed semver
                    let version = Version::parse(version).unwrap();
                    println!("New version: {}, Old version: {}", new_version, version);
                    if new_version > version {
                        println!("Newer version available, updating");
                        let update_result =
                            update(&base_settings, &http_client, &should_restart).await;

                        if let Err(err) = &update_result {
                            // TODO: Log
                            println!("Failed to update: {}", err);
                        };
                    }
                    // Version check completed
                    break;
                }
                // couldn't parse, log it
                Err(_) => {
                    println!("Failed to parse new version");
                    // TODO: Log
                    // Do not break, as if can't parse new version, then probably should update
                }
            }
        }
    }

    if should_restart.load(Ordering::Relaxed) {
        restart(&current_exe);
    }

    (base_settings, mqtt_client, mqtt_eventloop, http_client, current_exe)
}

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

    // NTP
    mqtt_client
        .subscribe_all_individual(
            settings.ntp_topic.as_str(),
            base_settings.pi_zero_id.as_str(),
        )
        .await
        .unwrap();
    // Taking pictures
    mqtt_client
        .subscribe_all_individual(
            settings.camera_topic.as_str(),
            base_settings.pi_zero_id.as_str(),
        )
        .await
        .unwrap();
    // Linux commands
    mqtt_client
        .subscribe_all_individual(
            settings.command_topic.as_str(),
            base_settings.pi_zero_id.as_str(),
        )
        .await
        .unwrap();
    // Status
    mqtt_client
        .subscribe_all_individual(
            settings.status_topic.as_str(),
            base_settings.pi_zero_id.as_str(),
        )
        .await
        .unwrap();

    let still_controls: Option<CameraControls> =
        read_camera_controls(&base_settings, &mqtt_client, STILL_CAMERA_CONTROLS_FILENAME).await;
    let video_controls: Option<CameraControls> =
        read_camera_controls(&base_settings, &mqtt_client, VIDEO_CAMERA_CONTROLS_FILENAME).await;

    let camera_service = Python::with_gil(|py| -> Result<CameraService, anyhow::Error> {
        let still_controls_pydict = match &still_controls {
            Some(v) => Some(v.to_pydict(py)?),
            None => None,
        };
        let camera_service =
            CameraService::new(py, &still_controls, &video_controls, still_controls_pydict)?;
        Ok(camera_service)
    })
    .unwrap();

    let ntp_result = ntp_sync(&settings);
    let ntp_json = match ntp_result {
        Ok(v) => serde_json::to_string(&SuccessWrapper::success(v)),
        Err(e) => {
            println!("{}", e);
            serde_json::to_string(&SuccessWrapper::failure(e.to_string()))
        }
    }
    .unwrap();

    mqtt_client
        .publish_individual(
            settings.ntp_topic.as_str(),
            base_settings.pi_zero_id.as_str(),
            ntp_json.into_bytes(),
        )
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
