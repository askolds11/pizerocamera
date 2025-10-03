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
    SyncStatus {
        response: SuccessWrapper<SyncStatusResponse>,
    }
}

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all_fields = "camelCase")]
pub enum TakePictureResponse {
    PictureFailedToSchedule {
        uuid: Uuid,
        message: String,
        message_received_nanos: Option<i64>,
        wait_time_nanos: i64,
    },
    PictureTaken {
        uuid: Uuid,
        monotonic_time: i64,
        message_received_nanos: Option<i64>,
        wait_time_nanos: i64,
    },
    PictureFailedToTake {
        uuid: Uuid,
        message: String,
        message_received_nanos: Option<i64>,
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
#[serde(rename_all_fields = "camelCase")]
pub enum SendPictureResponse {
    Failed { uuid: Uuid, message: String },
    PictureFailedToRead { uuid: Uuid, message: String },
    PictureSent { uuid: Uuid },
    PictureFailedToSend { uuid: Uuid, message: String },
}

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all_fields = "camelCase")]
pub enum SyncStatusResponse {
    Failed { message: String },
    Success { sync_ready: bool, sync_timing: i64 },
}

impl CameraResponse {
    pub fn into_bytes(self) -> Result<Bytes, serde_json::error::Error> {
        serde_json::to_string(&self).map(|s| s.into())
    }
}
