use super::UpdateError;
use crate::endpoints::get_download_update_url;
use crate::settings::BaseSettings;
use crate::utils::HttpError;
use reqwest::Client;
use self_replace::self_replace;
use std::convert::From;
use std::fs::File;
use std::io::copy;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{env, fs};

const TEMP_DIR: &str = "self_update_";
const DOWNLOADED_FILE: &str = "pizerocamera_exec";

pub async fn update(
    base_settings: &BaseSettings,
    http_client: &Client,
    restart: &AtomicBool,
) -> Result<(), UpdateError> {
    // Temporary directory for download
    // ./self_update_RANDOMID
    let tmp_dir = tempfile::Builder::new()
        .prefix(TEMP_DIR)
        .tempdir_in(env::current_dir()?)?;
    // Downloaded file name
    let tmp_file_path = tmp_dir.path().join(DOWNLOADED_FILE);

    // Download file
    let response = http_client
        .get(get_download_update_url(&base_settings.server_url))
        .send()
        .await?;

    // If not succesfully downloaded, return error
    if !response.status().is_success() {
        return Err(UpdateError::from(HttpError::from_response(response).await));
    }

    // Save to temp file
    let mut dest = File::create(&tmp_file_path)?;
    let bytes = response.bytes().await?;
    let mut content = bytes.as_ref();
    copy(&mut content, &mut dest)?;

    // Replace executable
    self_replace(tmp_file_path)?;
    fs::remove_dir_all(&tmp_dir)?;

    // Restart from main thread
    restart.store(true, Ordering::Relaxed);

    Ok(())
}

/// Restart the program. Does not work in a spawned task.
pub fn restart() {
    println!("Restarting");
    // Ok to unwrap, as worst case crashes and systemd restarts
    let current_exe = env::current_exe().unwrap();

    use std::os::unix::process::CommandExt;
    let e = Command::new(current_exe).exec();

    // If exec() fails for some reason, exit the process manually
    eprintln!("Exec failed, exiting. Error: \n {}", e);
    std::process::exit(1);
}
