use crate::camera::{CameraControls, CameraMode};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TakePicture {
    pub start_epoch: u64,
    pub uuid: Uuid,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetControls {
    pub camera_mode: CameraMode,
    pub camera_controls: CameraControls,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum CameraRequest {
    TakePicture(TakePicture),
    SetControls(SetControls),
    GetControls(CameraMode),
    GetControlLimits,
    StartPreview,
    StopPreview,
}