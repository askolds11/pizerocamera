use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all_fields = "camelCase")]
pub enum UpdateResponse {
    DownloadingUpdate {
        new_version: String,
        version: String,
    },
    UpdateDownloaded {
        new_version: String,
        version: String,
    },
    AlreadyUpdated {
        new_version: String,
        version: String,
    },
    Failed {
        new_version: Option<String>,
        version: Option<String>,
        message: String,
    },
}
