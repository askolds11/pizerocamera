use crate::camera::{CameraControlsLimit, CameraMode, CameraService};
use crate::endpoints::get_upload_image_url;
use crate::functions::requests::{CameraRequest, SendPicture, SetControls, TakePicture};
use crate::functions::responses::{CameraResponse, SendPictureResponse, TakePictureResponse};
use crate::settings::{BaseSettings, Settings};
use crate::utils::{AsyncClientExt, SuccessWrapper};
use jpeg_encoder::{ColorType, Encoder};
use nix::sys::time::TimeValLike;
use pyo3::{PyResult, Python};
use reqwest::{Client, multipart};
use rumqttc::v5::AsyncClient;
use rumqttc::v5::mqttbytes::v5::Publish;
use serde::Serialize;
use std::collections::HashMap;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

pub const STILL_CAMERA_CONTROLS_FILENAME: &'static str = "controls_still.json";
pub const VIDEO_CAMERA_CONTROLS_FILENAME: &'static str = "controls_video.json";

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CameraControlsResponse {
    min: CameraControlsLimit,
    max: CameraControlsLimit,
    default: CameraControlsLimit,
}

pub async fn handle_picture(
    base_settings: &BaseSettings,
    settings: &Settings,
    mqtt_client: &AsyncClient,
    http_client: &Client,
    camera_service: &mut CameraService,
    publish: &Publish,
) -> Result<(), anyhow::Error> {
    let camera_request: CameraRequest = serde_json::from_slice(&publish.payload)?;

    match camera_request {
        CameraRequest::TakePicture(request) => {
            let res = take_picture(
                base_settings,
                settings,
                mqtt_client,
                camera_service,
                &request,
            )
            .await;

            if let Err(err) = res {
                println!("Error while taking picture: {:?}", err);
                let err = TakePictureResponse::Failed {
                    uuid: request.uuid,
                    message: err.to_string(),
                };
                let success_wrapper = SuccessWrapper::failure(err);
                let response = CameraResponse::TakePicture {
                    response: success_wrapper,
                };

                mqtt_client
                    .publish_individual(
                        &settings.camera_topic,
                        &base_settings.pi_zero_id,
                        response.into_bytes()?,
                    )
                    .await
                    .unwrap_or_default();
            }
        }
        CameraRequest::SendPicture(request) => {
            let res =
                send_picture(base_settings, settings, mqtt_client, http_client, &request).await;

            if let Err(err) = res {
                println!("Error while taking picture: {:?}", err);
                let err = SendPictureResponse::Failed {
                    uuid: request.uuid,
                    message: err.to_string(),
                };
                let success_wrapper = SuccessWrapper::failure(err);
                let response = CameraResponse::SendPicture {
                    response: success_wrapper,
                };

                mqtt_client
                    .publish_individual(
                        &settings.camera_topic,
                        &base_settings.pi_zero_id,
                        response.into_bytes()?,
                    )
                    .await
                    .unwrap_or_default();
            }
        }
        CameraRequest::SetControls(controls) => {
            set_controls(
                base_settings,
                settings,
                mqtt_client,
                camera_service,
                controls,
            )
            .await?;
        }
        CameraRequest::GetControlLimits => {
            get_control_limits(base_settings, settings, mqtt_client, camera_service).await?;
        }
        CameraRequest::StartPreview => {
            start_preview(camera_service).await?;
        }
        CameraRequest::StopPreview => {
            stop_preview(camera_service).await?;
        }
        CameraRequest::GetControls(_) => {
            get_controls(base_settings, settings, mqtt_client, camera_service).await?;
        }
    }

    Ok(())
}

async fn take_picture(
    base_settings: &BaseSettings,
    settings: &Settings,
    mqtt_client: &AsyncClient,
    camera_service: &CameraService,
    request: &TakePicture,
) -> Result<(), anyhow::Error> {
    // todo: proper error
    // calculate time between current time and picture time
    let wall_time = nix::time::clock_gettime(nix::time::ClockId::CLOCK_REALTIME)?;
    let monotonic_time = nix::time::clock_gettime(nix::time::ClockId::CLOCK_MONOTONIC)?;
    let wall_nanoseconds = wall_time.num_nanoseconds();
    let wait_time = request.picture_epoch as i64 * 1000000 - wall_nanoseconds;
    // return error, if wait time is negative
    if wait_time < 0 {
        let err = TakePictureResponse::PictureFailedToSchedule {
            uuid: request.uuid,
            message: format!(
                "Current time: {}, picture time: {}, late by {} ns",
                wall_nanoseconds, request.picture_epoch, wait_time
            ),
        };
        let success_wrapper = SuccessWrapper::failure(err);
        let response = CameraResponse::TakePicture {
            response: success_wrapper,
        };

        mqtt_client
            .publish_individual(
                &settings.camera_topic,
                &base_settings.pi_zero_id,
                response.into_bytes()?,
            )
            .await?;

        // Return ok, as error handled in this function
        return Ok(());
    }
    // add wait time to monotonic time
    let monotonic_nanoseconds = monotonic_time.num_nanoseconds();
    let monotonic_nanoseconds_future = monotonic_nanoseconds + wait_time;

    let pic = take_picture_take(camera_service, monotonic_nanoseconds_future as u64).await;
    let (bytes, width, height, metadata) = match pic {
        Ok(pic) => {
            // Send that taken successfully
            let picture_taken = TakePictureResponse::PictureTaken {
                uuid: request.uuid,
            };
            // It's ok if it fails, we will still try to save/send
            let success_wrapper = SuccessWrapper::success(picture_taken);
            let response = CameraResponse::TakePicture {
                response: success_wrapper,
            }
            .into_bytes()
            .ok();
            if let Some(picture_saved) = response {
                mqtt_client
                    .publish_individual(
                        &settings.camera_topic,
                        &base_settings.pi_zero_id,
                        picture_saved,
                    )
                    .await
                    .unwrap_or_default();
            }
            pic
        }
        Err(e) => {
            let err = TakePictureResponse::PictureFailedToTake {
                uuid: request.uuid,
                message: e.to_string(),
            };
            let success_wrapper = SuccessWrapper::failure(err);
            let response = CameraResponse::TakePicture {
                response: success_wrapper,
            };

            mqtt_client
                .publish_individual(
                    &settings.camera_topic,
                    &base_settings.pi_zero_id,
                    response.into_bytes()?,
                )
                .await?;

            // Return ok, as error handled in this function
            return Ok(());
        }
    };

    let mut jpeg_buf = Vec::new();
    let encoder = Encoder::new(&mut jpeg_buf, 95);
    encoder.encode(&bytes, width, height, ColorType::Rgb)?;

    let save_result = take_picture_save(&base_settings, &request, &jpeg_buf, &metadata).await;

    match save_result {
        Ok(res) => res,
        Err(e) => {
            let err = TakePictureResponse::PictureFailedToSave {
                uuid: request.uuid,
                message: e.to_string(),
            };
            let success_wrapper = SuccessWrapper::failure(err);
            let response = CameraResponse::TakePicture {
                response: success_wrapper,
            };

            mqtt_client
                .publish_individual(
                    &settings.camera_topic,
                    &base_settings.pi_zero_id,
                    response.into_bytes()?,
                )
                .await?;

            // Return ok, as error handled in this function
            return Ok(());
        }
    };

    // Log that saved successfully
    let picture_saved = TakePictureResponse::PictureSavedOnDevice { uuid: request.uuid };
    // It's ok if it fails, we will still try to send image
    let success_wrapper = SuccessWrapper::success(picture_saved);
    let response = CameraResponse::TakePicture {
        response: success_wrapper,
    }
    .into_bytes()
    .ok();
    if let Some(picture_saved) = response {
        mqtt_client
            .publish_individual(
                &settings.camera_topic,
                &base_settings.pi_zero_id,
                picture_saved,
            )
            .await
            .unwrap_or_default();
    }

    Ok(())
}

async fn send_picture(
    base_settings: &BaseSettings,
    settings: &Settings,
    mqtt_client: &AsyncClient,
    http_client: &Client,
    request: &SendPicture,
) -> Result<(), anyhow::Error> {
    let filename = get_filename(&request.uuid, &base_settings.pi_zero_id);
    let file_path = get_photos_path(&filename);
    let filename_metadata = get_metadata_filename(&request.uuid, &base_settings.pi_zero_id);
    let metadata_path = get_photos_path(&filename_metadata);

    // Read pic
    let bytes = fs::read(file_path).await;
    let bytes = match bytes {
        Ok(bytes) => bytes,
        Err(e) => {
            let err = SendPictureResponse::PictureFailedToRead {
                uuid: request.uuid,
                message: e.to_string(),
            };
            let success_wrapper = SuccessWrapper::failure(err);
            let response = CameraResponse::SendPicture {
                response: success_wrapper,
            };

            mqtt_client
                .publish_individual(
                    &settings.camera_topic,
                    &base_settings.pi_zero_id,
                    response.into_bytes()?,
                )
                .await?;

            // Return ok, as error handled in this function
            return Ok(());
        }
    };

    let metadata_json = fs::read_to_string(metadata_path)
        .await
        .unwrap_or("{}".to_string());

    let send_result = take_picture_send(
        &base_settings,
        &request,
        &http_client,
        bytes,
        filename,
        metadata_json,
    )
    .await;

    match send_result {
        Ok(_) => {}
        Err(e) => {
            let err = SendPictureResponse::PictureFailedToSend {
                uuid: request.uuid,
                message: e.to_string(),
            };
            let success_wrapper = SuccessWrapper::failure(err);
            let response = CameraResponse::SendPicture {
                response: success_wrapper,
            };

            mqtt_client
                .publish_individual(
                    &settings.camera_topic,
                    &base_settings.pi_zero_id,
                    response.into_bytes()?,
                )
                .await?;

            // Return ok, as error handled in this function
            return Ok(());
        }
    }

    let picture_sent = SendPictureResponse::PictureSent { uuid: request.uuid };
    let picture_sent = SuccessWrapper::success(picture_sent);
    let response = CameraResponse::SendPicture {
        response: picture_sent,
    }
    .into_bytes()
    .ok();
    if let Some(picture_sent) = response {
        mqtt_client
            .publish_individual(
                &settings.camera_topic,
                &base_settings.pi_zero_id,
                picture_sent,
            )
            .await
            .unwrap_or_default();
    }

    Ok(())
}

/// Take picture - 1. take pic
async fn take_picture_take(
    camera_service: &CameraService,
    time: u64,
) -> Result<(Vec<u8>, u16, u16, HashMap<String, String>), anyhow::Error> {
    let picture_result = Python::with_gil(
        |py| -> PyResult<(Vec<u8>, u16, u16, HashMap<String, String>)> {
            camera_service.capture(py, time)
        },
    )?;

    Ok(picture_result)
}

/// Take picture - 2. save pic
/// Returns filename and metadata json (can be empty, if fails)
/// Returns error only if saving file fails
async fn take_picture_save(
    base_settings: &BaseSettings,
    request: &TakePicture,
    bytes: &Vec<u8>,
    metadata: &HashMap<String, String>,
) -> Result<(String, String), anyhow::Error> {
    let filename = format!("{}_{}.jpg", &request.uuid, &base_settings.pi_zero_id);
    let filename_with_path = format!("photos/{}", filename);
    // Save file first
    let mut file = File::create(&filename_with_path).await?;
    file.write_all(&bytes).await?;

    println!("Metadata: {:?}", metadata);
    let metadata_json = serde_json::to_string(&metadata).unwrap_or("{}".to_string());

    let filename_metadata = format!(
        "photos/{}_{}_metadata.json",
        &request.uuid, &base_settings.pi_zero_id
    );
    let metadata_file = File::create(&filename_metadata).await;
    match metadata_file {
        Ok(mut metadata_file) => {
            metadata_file
                .write_all(metadata_json.as_bytes())
                .await
                .unwrap_or_default();
        }
        Err(e) => {
            println!("Failed to create metadata file: {:?}", e)
        }
    }

    Ok((filename, metadata_json))
}

fn get_filename(uuid: &Uuid, pi_zero_id: &str) -> String {
    format!("{}_{}.jpg", &uuid, &pi_zero_id)
}

fn get_metadata_filename(uuid: &Uuid, pi_zero_id: &str) -> String {
    format!("{}_{}_metadata.json", &uuid, &pi_zero_id)
}

fn get_photos_path(filename: &str) -> String {
    format!("photos/{}", &filename)
}

/// Take picture - 3. send pic
async fn take_picture_send(
    base_settings: &BaseSettings,
    request: &SendPicture,
    http_client: &Client,
    bytes: Vec<u8>,
    filename: String,
    metadata_json: String,
) -> Result<(), anyhow::Error> {
    let uuid = &request.uuid.simple();
    let form = multipart::Form::new()
        .part(
            "image",
            multipart::Part::bytes(bytes)
                .file_name(filename)
                .mime_str("application/octet-stream")?,
        )
        .text("metadata", metadata_json)
        .text("uuid", uuid.to_string());

    let response = http_client
        .post(get_upload_image_url(&base_settings.server_url))
        .multipart(form)
        .send()
        .await?;

    let status = response.status();

    if status.is_success() {
        Ok(())
    } else {
        Err(anyhow::Error::msg(status.to_string()))
    }
}

async fn start_preview(camera_service: &mut CameraService) -> Result<(), anyhow::Error> {
    Python::with_gil(|py| -> Result<(), anyhow::Error> {
        let video_controls_pydict = match &camera_service.video_controls {
            Some(v) => Some(v.to_pydict(py)?),
            None => None,
        };
        camera_service.start_preview(py, video_controls_pydict)?;
        Ok(())
    })?;
    Ok(())
}

async fn stop_preview(camera_service: &mut CameraService) -> Result<(), anyhow::Error> {
    Python::with_gil(|py| -> Result<(), anyhow::Error> {
        let still_controls_pydict = match &camera_service.still_controls {
            Some(v) => Some(v.to_pydict(py)?),
            None => None,
        };
        camera_service.stop_preview(py, still_controls_pydict)?;
        Ok(())
    })?;
    Ok(())
}

async fn set_controls(
    base_settings: &BaseSettings,
    settings: &Settings,
    mqtt_client: &AsyncClient,
    camera_service: &mut CameraService,
    controls: SetControls,
) -> Result<(), anyhow::Error> {
    println!("Writing file");
    // Save controls
    let filename = match controls.camera_mode {
        CameraMode::Still => STILL_CAMERA_CONTROLS_FILENAME,
        CameraMode::Video => VIDEO_CAMERA_CONTROLS_FILENAME,
    };
    let mut file = File::create(&filename).await?;
    let bytes = serde_json::to_string(&controls.camera_controls)?.into_bytes();
    file.write_all(&bytes).await?;

    println!("Settings controls in camera service");
    // Set controls
    match controls.camera_mode {
        CameraMode::Still => camera_service.still_controls = Some(controls.camera_controls.clone()),
        CameraMode::Video => camera_service.video_controls = Some(controls.camera_controls.clone()),
    }

    // Only set config if config matches
    if controls.camera_mode == camera_service.camera_mode {
        println!("Settings controls in python");
        Python::with_gil(|py| -> Result<(), anyhow::Error> {
            camera_service.set_controls(py, controls.camera_controls.to_pydict(py)?)?;
            Ok(())
        })?;
    }

    let success_wrapper = SuccessWrapper::success("");

    // Picture taken
    mqtt_client
        .publish_individual(
            &settings.camera_topic,
            &base_settings.pi_zero_id,
            success_wrapper.into_bytes()?,
        )
        .await?;

    Ok(())
}

async fn get_control_limits(
    base_settings: &BaseSettings,
    settings: &Settings,
    mqtt_client: &AsyncClient,
    camera_service: &CameraService,
) -> Result<(), anyhow::Error> {
    let (min, max, default) = Python::with_gil(
        |py| -> Result<
            (
                CameraControlsLimit,
                CameraControlsLimit,
                CameraControlsLimit,
            ),
            anyhow::Error,
        > {
            let pydict = camera_service.get_controls_limits(py)?;
            let controls = CameraControlsLimit::from_control_triplets(pydict)?;
            Ok(controls)
        },
    )?;
    let controls_response = CameraControlsResponse { min, max, default };
    let success_wrapper = SuccessWrapper::success(controls_response);

    // Picture taken
    mqtt_client
        .publish_individual(
            &settings.camera_topic,
            &base_settings.pi_zero_id,
            success_wrapper.into_bytes()?,
        )
        .await?;
    Ok(())
}

async fn get_controls(
    base_settings: &BaseSettings,
    settings: &Settings,
    mqtt_client: &AsyncClient,
    camera_service: &CameraService,
) -> Result<(), anyhow::Error> {
    let controls = match camera_service.camera_mode {
        CameraMode::Still => &camera_service.still_controls,
        CameraMode::Video => &camera_service.video_controls,
    };

    let success_wrapper = SuccessWrapper::success(controls);

    // Picture taken
    mqtt_client
        .publish_individual(
            &settings.camera_topic,
            &base_settings.pi_zero_id,
            success_wrapper.into_bytes()?,
        )
        .await?;
    Ok(())
}
