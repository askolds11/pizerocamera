use crate::settings::{BaseSettings, Settings};
use crate::utils::{execute_command, AsyncClientExt};
use rumqttc::v5::mqttbytes::v5::Publish;
use rumqttc::v5::AsyncClient;

pub async fn handle_command(
    base_settings: &BaseSettings,
    settings: &Settings,
    mqtt_client: &AsyncClient,
    publish: &Publish,
) -> Result<(), anyhow::Error> {
    let payload = str::from_utf8(&publish.payload)?;
    let result = execute_command(payload);

    let result_message = match result {
        Ok(v) => {
            format!("OK: {}", v)
        }
        Err(e) => {
            format!("ERR: {}", e)
        }
    };

    mqtt_client
        .publish_individual(
            settings.command_topic.as_str(),
            base_settings.pi_zero_id.as_str(),
            result_message,
        )
        .await?;
    Ok(())
}