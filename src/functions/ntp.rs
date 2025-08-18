use crate::functions::requests::NtpRequest;
use crate::ntp_sync::{ntp_sync_slew, ntp_sync_step};
use crate::settings::{BaseSettings, Settings};
use crate::utils::{AsyncClientExt, SuccessWrapper};
use rumqttc::v5::mqttbytes::v5::Publish;
use rumqttc::v5::AsyncClient;

pub async fn handle_ntp(
    base_settings: &BaseSettings,
    settings: &Settings,
    mqtt_client: &AsyncClient,
    publish: &Publish,
) -> Result<(), anyhow::Error> {
    let ntp_request: NtpRequest = serde_json::from_slice(&publish.payload)?;
    sync_ntp(base_settings, settings, mqtt_client, &ntp_request).await
}

pub async fn sync_ntp(
    base_settings: &BaseSettings,
    settings: &Settings,
    mqtt_client: &AsyncClient,
    ntp_request: &NtpRequest,
) -> Result<(), anyhow::Error> {
    let ntp_result = match ntp_request {
        NtpRequest::Step => ntp_sync_step(settings),
        NtpRequest::Slew => ntp_sync_slew(settings)
    };
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
