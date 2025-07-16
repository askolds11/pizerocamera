/// Url for downloading update
pub fn get_download_update_url(base_url: &str) -> String {
    format!("{}/downloadupdate", base_url)
}

pub fn get_upload_image_url(base_url: &str) -> String {
    format!("{}/uploadimage", base_url)
}
