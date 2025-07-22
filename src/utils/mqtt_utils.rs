use crate::settings::BaseSettings;
use bytes::Bytes;
use rumqttc::v5::mqttbytes::QoS;
use rumqttc::v5::mqttbytes::v5::Publish;
use rumqttc::v5::{AsyncClient, ClientError};

pub trait AsyncClientExt {
    /// Subscribes to global and individual topic for receiving messages.
    /// QoS is "at least once" - the message will definitely be sent, maybe even more than once
    async fn subscribe_all_individual(
        &self,
        topic: &str,
        pi_zero_id: &str,
    ) -> Result<(), ClientError>;

    /// Subscribes to global and individual topic for receiving messages.
    async fn publish_individual<P>(
        &self,
        topic: &str,
        pi_zero_id: &str,
        payload: P,
    ) -> Result<(), ClientError>
    where
        P: Into<Bytes>;
}

// Implement the trait for the third-party type
impl AsyncClientExt for AsyncClient {
    async fn subscribe_all_individual(
        &self,
        topic: &str,
        pi_zero_id: &str,
    ) -> Result<(), ClientError> {
        self.subscribe(
            get_individual_receive_topic(topic, pi_zero_id),
            QoS::AtLeastOnce,
        )
        .await?;
        self.subscribe(
            get_column_receive_topic(topic, pi_zero_id),
            QoS::AtLeastOnce,
        )
        .await?;
        self.subscribe(
            get_row_receive_topic(topic, pi_zero_id),
            QoS::AtLeastOnce,
        )
        .await?;
        self.subscribe(topic, QoS::AtLeastOnce).await?;
        Ok(())
    }

    async fn publish_individual<P>(
        &self,
        topic: &str,
        pi_zero_id: &str,
        payload: P,
    ) -> Result<(), ClientError>
    where
        P: Into<Bytes>,
    {
        let individual_topic = get_individual_send_topic(topic, pi_zero_id);
        self.publish(individual_topic, QoS::AtLeastOnce, false, payload)
            .await
    }
}

/// Gets individual topic for message receiving from global topic and Pi Zero's id
pub fn get_individual_receive_topic(topic: &str, pi_zero_id: &str) -> String {
    format!("{}/{}", topic, pi_zero_id)
}

/// Gets column topic for message receiving from global topic and Pi Zero's id
pub fn get_column_receive_topic(topic: &str, pi_zero_id: &str) -> String {
    format!("{}/{}", topic, pi_zero_id.chars().nth(0).unwrap())
}

/// Gets row topic for message receiving from global topic and Pi Zero's id
pub fn get_row_receive_topic(topic: &str, pi_zero_id: &str) -> String {
    format!("{}/{}", topic, pi_zero_id.chars().nth(1).unwrap())
}

/// Gets individual topic for message sending from global topic and Pi Zero's id
pub fn get_individual_send_topic(topic: &str, pi_zero_id: &str) -> String {
    format!("{}/answer/{}", topic, pi_zero_id)
}

pub trait PublishExt {
    /// Checks if Publish topic matches the global or individual topic
    fn is_global_or_individual(&self, topic: &str, pi_zero_id: &str) -> bool;
}

impl PublishExt for Publish {
    fn is_global_or_individual(&self, topic: &str, pi_zero_id: &str) -> bool {
        if self.topic == topic {
            return true;
        } else {
            // Only make individual topic if global topic did not match
            let individual_topic = get_individual_receive_topic(topic, pi_zero_id);
            if self.topic == individual_topic {
                return true;
            }
        }
        false
    }
}

pub trait ErrorExt {
    async fn send_error(
        &self,
        base_settings: &BaseSettings,
        mqtt_client: &AsyncClient,
        topic: &str,
    ) -> Result<(), anyhow::Error>;
}

impl ErrorExt for anyhow::Error {
    async fn send_error(
        &self,
        base_settings: &BaseSettings,
        mqtt_client: &AsyncClient,
        topic: &str,
    ) -> Result<(), anyhow::Error> {
        let err = format!("{:?}", self);
        mqtt_client
            .publish_individual(
                topic,
                base_settings.pi_zero_id.as_str(),
                serde_json::to_string(&err)?.into_bytes(),
            )
            .await?;
        Ok(())
    }
}

pub trait ResultExt<T> {
    async fn send_if_err(
        self,
        base_settings: &BaseSettings,
        mqtt_client: &AsyncClient,
        topic: &str,
    ) -> Result<T, anyhow::Error>;
}

impl<T> ResultExt<T> for Result<T, anyhow::Error> {
    async fn send_if_err(
        self,
        base_settings: &BaseSettings,
        mqtt_client: &AsyncClient,
        topic: &str,
    ) -> Result<T, anyhow::Error> {
        if let Err(ref e) = self {
            e.send_error(base_settings, mqtt_client, topic).await?;
        }
        self
    }
}
