use serde::Serialize;

#[derive(Serialize, Debug)]
pub enum TakePictureResponse {
    PictureFailedToTake { message: String },
    PictureSavedOnDevice,
    PictureFailedToSave { message: String },
    PictureSent,
    PictureFailedToSend { message: String }
}