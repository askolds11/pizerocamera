use crate::utils::SuccessWrapper;
use bytes::Bytes;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum CameraResponse {
    TakePicture {
        response: SuccessWrapper<TakePictureResponse>,
    },
    SendPicture {
        response: SuccessWrapper<SendPictureResponse>,
    },
}

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum TakePictureResponse {
    PictureFailedToSchedule {
        uuid: Uuid,
        message: String,
        #[serde(rename = "messageReceivedNanos")]
        message_received_nanos: Option<i64>,
        #[serde(rename = "waitTimeNanos")]
        wait_time_nanos: i64,
    },
    PictureTaken {
        uuid: Uuid,
        #[serde(rename = "monotonicTime")]
        monotonic_time: i64,
        #[serde(rename = "messageReceivedNanos")]
        message_received_nanos: Option<i64>,
        #[serde(rename = "waitTimeNanos")]
        wait_time_nanos: i64,
    },
    PictureFailedToTake {
        uuid: Uuid,
        message: String,
        #[serde(rename = "messageReceivedNanos")]
        message_received_nanos: Option<i64>,
        #[serde(rename = "waitTimeNanos")]
        wait_time_nanos: i64,
    },
    PictureSavedOnDevice {
        uuid: Uuid,
    },
    PictureFailedToSave {
        uuid: Uuid,
        message: String,
    },
    Failed {
        uuid: Uuid,
        message: String,
    },
}

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum SendPictureResponse {
    Failed { uuid: Uuid, message: String },
    PictureFailedToRead { uuid: Uuid, message: String },
    PictureSent { uuid: Uuid },
    PictureFailedToSend { uuid: Uuid, message: String },
}

impl CameraResponse {
    pub fn into_bytes(self) -> Result<Bytes, serde_json::error::Error> {
        serde_json::to_string(&self).map(|s| s.into())
    }
}
