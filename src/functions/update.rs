use crate::functions::responses::UpdateResponse;
use crate::settings::BaseSettings;
use crate::updater::update;
use crate::utils::{AsyncClientExt, SuccessWrapper};
use reqwest::Client;
use rumqttc::v5::mqttbytes::v5::Publish;
use rumqttc::v5::AsyncClient;
use semver::Version;
use std::sync::atomic::{AtomicBool, Ordering};

pub async fn handle_update(
    base_settings: &BaseSettings,
    mqtt_client: &AsyncClient,
    http_client: &Client,
    should_restart: &AtomicBool,
    publish: &Publish,
    startup: bool,
) -> Result<(), anyhow::Error> {
    // this should not fail, because cargo version is guaranteed semver
    let version = Version::parse(env!("CARGO_PKG_VERSION"));
    let version = match version {
        Ok(v) => v,
        Err(err) => {
            let update_response = UpdateResponse::Failed {
                new_version: None,
                version: None,
                message: err.to_string(),
            };
            let success_wrapper = SuccessWrapper::failure(update_response);
            if let Ok(msg) = success_wrapper.into_bytes() {
                mqtt_client
                    .publish_individual(&base_settings.update_topic, &base_settings.pi_zero_id, msg)
                    .await
                    .unwrap_or_default();
            }
            return Ok(());
        }
    };

    let new_version = str::from_utf8(&publish.payload)
        .map_err(anyhow::Error::from)
        .and_then(|p| Version::parse(p).map_err(anyhow::Error::from));
    let new_version = match new_version {
        Ok(v) => v,
        Err(err) => {
            let update_response = UpdateResponse::Failed {
                new_version: None,
                version: Some(version.to_string()),
                message: err.to_string(),
            };
            let success_wrapper = SuccessWrapper::failure(update_response);
            if let Ok(msg) = success_wrapper.into_bytes() {
                mqtt_client
                    .publish_individual(&base_settings.update_topic, &base_settings.pi_zero_id, msg)
                    .await
                    .unwrap_or_default();
            }
            return Ok(());
        }
    };
    println!("New version: {}, Old version: {}", new_version, version);
    if new_version <= version {
        // Don't say "Already updated" on startup update check
        if !startup {
            let update_response = UpdateResponse::AlreadyUpdated {
                new_version: new_version.to_string(),
                version: version.to_string(),
            };
            let success_wrapper = SuccessWrapper::success(update_response);
            if let Ok(msg) = success_wrapper.into_bytes() {
                mqtt_client
                    .publish_individual(&base_settings.update_topic, &base_settings.pi_zero_id, msg)
                    .await
                    .unwrap_or_default();
            }
        }
        return Ok(());
    }

    let update_response = UpdateResponse::DownloadingUpdate {
        new_version: new_version.to_string(),
        version: version.to_string(),
    };
    let success_wrapper = SuccessWrapper::success(update_response);

    if let Ok(msg) = success_wrapper.into_bytes() {
        mqtt_client
            .publish_individual(&base_settings.update_topic, &base_settings.pi_zero_id, msg)
            .await
            .unwrap_or_default();
    }

    println!("Newer version available, updating");
    let update_result = update(&base_settings, &http_client).await;

    match update_result {
        Ok(_) => {
            let update_response = UpdateResponse::UpdateDownloaded {
                new_version: new_version.to_string(),
                version: version.to_string(),
            };
            let success_wrapper = SuccessWrapper::success(update_response);
            if let Ok(msg) = success_wrapper.into_bytes() {
                mqtt_client
                    .publish_individual(&base_settings.update_topic, &base_settings.pi_zero_id, msg)
                    .await
                    .unwrap_or_default();
            }

            // Restart from main thread
            should_restart.store(true, Ordering::Relaxed);
        }
        Err(err) => {
            let update_response = UpdateResponse::Failed {
                new_version: Some(new_version.to_string()),
                version: Some(version.to_string()),
                message: err.to_string(),
            };
            let success_wrapper = SuccessWrapper::failure(update_response);
            if let Ok(msg) = success_wrapper.into_bytes() {
                mqtt_client
                    .publish_individual(&base_settings.update_topic, &base_settings.pi_zero_id, msg)
                    .await
                    .unwrap_or_default();
            }
        }
    }

    Ok(())
}
