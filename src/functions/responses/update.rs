use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all_fields = "camelCase")]
pub enum UpdateResponse {
    DownloadingUpdate {
        // #[serde(rename = "newVersion")]
        new_version: String,
        version: String,
    },
    UpdateDownloaded {
        // #[serde(rename = "newVersion")]
        new_version: String,
        version: String,
    },
    Failed {
        // #[serde(rename = "newVersion")]
        new_version: Option<String>,
        version: Option<String>,
        message: String,
    },
}
