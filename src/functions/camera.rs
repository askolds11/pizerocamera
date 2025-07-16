use crate::camera::{CameraControls, CameraControlsLimit, CameraService, ControlConfig};
use crate::endpoints::get_upload_image_url;
use crate::settings::{BaseSettings, Settings};
use crate::utils::{AsyncClientExt, SuccessWrapper};
use pyo3::Python;
use reqwest::{multipart, Client};
use rumqttc::v5::mqttbytes::v5::Publish;
use rumqttc::v5::AsyncClient;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

pub const STILL_CAMERA_CONTROLS_FILENAME: &'static str = "controls_still.json";
pub const VIDEO_CAMERA_CONTROLS_FILENAME: &'static str = "controls_video.json";

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TakePicture {
    start_epoch: u64,
    uuid: Uuid,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CameraControlsResponse {
    min: CameraControlsLimit,
    max: CameraControlsLimit,
    default: CameraControlsLimit,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SetControls {
    pub control_config: ControlConfig,
    pub camera_controls: CameraControls,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum CameraRequest {
    TakePicture(TakePicture),
    SetControls(SetControls),
    GetControls,
    StartPreview,
    StopPreview,
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
            take_picture(
                base_settings,
                settings,
                mqtt_client,
                http_client,
                camera_service,
                request,
            )
            .await?
        }
        CameraRequest::SetControls(controls) => {
            set_controls(
                base_settings,
                settings,
                mqtt_client,
                camera_service,
                controls,
            )
            .await?
        }
        CameraRequest::GetControls => {
            get_controls(base_settings, settings, mqtt_client, camera_service).await?
        }
        CameraRequest::StartPreview => {
            start_preview(camera_service).await?;
        }
        CameraRequest::StopPreview => {
            stop_preview(camera_service).await?;
        }
    }

    Ok(())
}

async fn take_picture(
    base_settings: &BaseSettings,
    settings: &Settings,
    mqtt_client: &AsyncClient,
    http_client: &Client,
    camera_service: &CameraService,
    request: TakePicture,
) -> Result<(), anyhow::Error> {
    let (bytes, metadata) = Python::with_gil(|py| camera_service.capture(py).unwrap());

    let filename = format!("{}_{}.jpg", &request.uuid, &base_settings.pi_zero_id);
    // Save file first
    let mut file = File::create(&filename).await?;
    file.write_all(&bytes).await?;

    // Picture taken
    mqtt_client
        .publish_individual(
            &settings.camera_topic,
            &base_settings.pi_zero_id,
            "Picture taken new",
        )
        .await?;

    println!("Metadata: {:?}", metadata);
    let metadata_json = serde_json::to_string(&metadata)?;

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
        .post(get_upload_image_url(&base_settings.server_url)) // <-- update your URL
        .multipart(form)
        .send()
        .await?;

    let status = response.status();
    let success_wrapper = if status.is_success() {
        SuccessWrapper::success(status.as_str())
    } else {
        SuccessWrapper::failure(status.as_str())
    };

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
    let filename = match controls.control_config {
        ControlConfig::Still => STILL_CAMERA_CONTROLS_FILENAME,
        ControlConfig::Video => VIDEO_CAMERA_CONTROLS_FILENAME,
    };
    let mut file = File::create(&filename).await?;
    let bytes = serde_json::to_string(&controls.camera_controls)?.into_bytes();
    file.write_all(&bytes).await?;

    println!("Settings controls in camera service");
    // Set controls
    match controls.control_config {
        ControlConfig::Still => {
            camera_service.still_controls = Some(controls.camera_controls.clone())
        }
        ControlConfig::Video => {
            camera_service.video_controls = Some(controls.camera_controls.clone())
        }
    }

    println!("Settings controls in python");
    Python::with_gil(|py| -> Result<(), anyhow::Error> {
        camera_service.set_controls(py, controls.camera_controls.to_pydict(py)?)?;
        Ok(())
    })?;
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

async fn get_controls(
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
