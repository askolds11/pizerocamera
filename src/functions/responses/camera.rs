use bytes::Bytes;
use serde::Serialize;
use crate::utils::SuccessWrapper;

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum CameraResponse {
    TakePicture { response: SuccessWrapper<TakePictureResponse> },
    SendPicture { response: SuccessWrapper<SendPictureResponse> },
}

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum TakePictureResponse {
    PictureFailedToSchedule { message: String },
    PictureTaken,
    PictureFailedToTake { message: String },
    PictureSavedOnDevice,
    PictureFailedToSave { message: String },
    Failed { message: String }
}

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum SendPictureResponse {
    Failed { message: String },
    PictureFailedToRead { message: String },
    PictureSent,
    PictureFailedToSend { message: String },
}

impl CameraResponse {
    pub fn into_bytes(self) -> Result<Bytes, serde_json::error::Error> {
        serde_json::to_string(&self).map(|s| s.into())
    }
}