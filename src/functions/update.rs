use crate::settings::BaseSettings;
use crate::updater::update;
use crate::utils::AsyncClientExt;
use reqwest::Client;
use rumqttc::v5::mqttbytes::v5::Publish;
use rumqttc::v5::AsyncClient;
use semver::Version;
use std::sync::atomic::AtomicBool;

pub async fn handle_update(
    base_settings: &BaseSettings,
    mqtt_client: &AsyncClient,
    http_client: &Client,
    should_restart: &AtomicBool,
    publish: &Publish,
) -> Result<(), anyhow::Error> {
    let payload = str::from_utf8(&publish.payload)?;
    let new_version = Version::parse(payload)?;
    let version = env!("CARGO_PKG_VERSION");
    // this should not fail, because cargo version is guaranteed semver
    let version = Version::parse(version)?;
    println!("New version: {}, Old version: {}", new_version, version);
    if new_version > version {
        println!("Newer version available, updating");
        update(&base_settings, &http_client, &should_restart).await?;
    }

    mqtt_client
        .publish_individual(
            &base_settings.update_topic,
            &base_settings.pi_zero_id,
            "Update downloaded, restarting app",
        )
        .await?;

    Ok(())
}
