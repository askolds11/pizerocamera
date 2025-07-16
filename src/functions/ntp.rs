use crate::ntp_sync::ntp_sync;
use crate::settings::{BaseSettings, Settings};
use crate::utils::{AsyncClientExt, SuccessWrapper};
use rumqttc::v5::AsyncClient;

pub async fn handle_ntp(
    base_settings: &BaseSettings,
    settings: &Settings,
    mqtt_client: &AsyncClient,
) -> Result<(), anyhow::Error> {
    let result = ntp_sync(settings)?;

    let result_message = SuccessWrapper {
        success: true,
        value: result,
    };

    let json = serde_json::to_string(&result_message)?;

    mqtt_client
        .publish_individual(
            settings.ntp_topic.as_str(),
            base_settings.pi_zero_id.as_str(),
            json,
        )
        .await?;

    Ok(())
}
