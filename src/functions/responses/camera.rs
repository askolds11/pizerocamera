use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum TakePictureResponse {
    PictureFailedToSchedule { message: String },
    PictureFailedToTake { message: String },
    PictureSavedOnDevice,
    PictureFailedToSave { message: String },
    PictureSent,
    PictureFailedToSend { message: String }
}