use crate::ntp_sync::ntp_sync;
use crate::settings::{BaseSettings, Settings};
use crate::utils::{AsyncClientExt, SuccessWrapper};
use rumqttc::v5::AsyncClient;

pub async fn handle_ntp(
    base_settings: &BaseSettings,
    settings: &Settings,
    mqtt_client: &AsyncClient,
) -> Result<(), anyhow::Error> {
    let ntp_result = ntp_sync(settings);
    let ntp_success_wrapper = ntp_result
        .map(|x| SuccessWrapper::success(x))
        .map_err(|e| SuccessWrapper::failure(e.to_string()))
        .unwrap_or_else(|e| e);
    let ntp_json = serde_json::to_string(&ntp_success_wrapper)?;

    mqtt_client
        .publish_individual(
            settings.ntp_topic.as_str(),
            base_settings.pi_zero_id.as_str(),
            ntp_json,
        )
        .await?;

    Ok(())
}
