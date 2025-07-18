use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum TakePictureResponse {
    PictureFailedToTake { message: String },
    PictureSavedOnDevice,
    PictureFailedToSave { message: String },
    PictureSent,
    PictureFailedToSend { message: String }
}