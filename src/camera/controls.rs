use pyo3::types::{PyAnyMethods, PyDict, PyDictMethods, PyTuple};
use pyo3::{Bound, FromPyObject, PyAny, Python};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ControlConfig {
    Still,
    Video
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FrameDurationLimits {
    pub min: u64,
    pub max: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ColourGain {
    pub red: f32,
    pub blue: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScalerCrop {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CameraControls {
    pub ae_constraint_mode: Option<u8>,
    pub ae_enable: Option<bool>,
    pub ae_exposure_mode: Option<u8>,
    pub ae_flicker_mode: Option<u8>,
    pub ae_flicker_period: Option<i64>,
    pub ae_metering_mode: Option<u8>,
    pub analogue_gain: Option<f32>,
    pub analogue_gain_mode: Option<u8>,
    pub awb_enable: Option<bool>,
    pub awb_mode: Option<u8>,
    pub brightness: Option<f32>,
    pub colour_gains: Option<ColourGain>,
    pub colour_temperature: Option<i64>,
    pub contrast: Option<f32>,
    pub cnn_enable_input_tensor: Option<bool>,
    pub exposure_time: Option<i64>,
    pub exposure_time_mode: Option<u8>,
    pub exposure_value: Option<f32>,
    pub frame_duration_limits: Option<FrameDurationLimits>,
    pub hdr_mode: Option<u8>,
    pub noise_reduction_mode: Option<u8>,
    pub saturation: Option<f32>,
    pub scaler_crop: Option<ScalerCrop>,
    pub sharpness: Option<f32>,
    pub sync_mode: Option<u8>,
    pub sync_frames: Option<i64>,
    pub stats_output_enable: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CameraControlsLimit {
    pub ae_constraint_mode: Option<u8>,
    pub ae_enable: Option<bool>,
    pub ae_exposure_mode: Option<u8>,
    pub ae_flicker_mode: Option<u8>,
    pub ae_flicker_period: Option<i64>,
    pub ae_metering_mode: Option<u8>,
    pub analogue_gain: Option<f32>,
    pub analogue_gain_mode: Option<u8>,
    pub awb_enable: Option<bool>,
    pub awb_mode: Option<u8>,
    pub brightness: Option<f32>,
    pub colour_gains: Option<f32>,
    pub colour_temperature: Option<i64>,
    pub contrast: Option<f32>,
    pub cnn_enable_input_tensor: Option<bool>,
    pub exposure_time: Option<i64>,
    pub exposure_time_mode: Option<u8>,
    pub exposure_value: Option<f32>,
    pub frame_duration_limits: Option<i64>,
    pub hdr_mode: Option<u8>,
    pub noise_reduction_mode: Option<u8>,
    pub saturation: Option<f32>,
    pub scaler_crop: Option<ScalerCrop>,
    pub sharpness: Option<f32>,
    pub sync_mode: Option<u8>,
    pub sync_frames: Option<i64>,
    pub stats_output_enable: Option<bool>,
}

impl CameraControls {
    pub fn to_pydict<'py>(&self, py: Python<'py>) -> Result<Bound<'py, PyDict>, anyhow::Error> {
        let dict = PyDict::new(py);

        if let Some(v) = self.contrast {
            dict.set_item("Contrast", v)?;
        }
        if let Some(v) = self.ae_enable {
            dict.set_item("AeEnable", v)?;
        }
        if let Some(v) = self.sharpness {
            dict.set_item("Sharpness", v)?;
        }
        if let Some(v) = self.noise_reduction_mode {
            dict.set_item("NoiseReductionMode", v)?;
        }
        if let Some(v) = self.ae_flicker_period {
            dict.set_item("AeFlickerPeriod", v)?;
        }
        if let Some(v) = self.analogue_gain {
            dict.set_item("AnalogueGain", v)?;
        }
        if let Some(v) = self.colour_temperature {
            dict.set_item("ColourTemperature", v)?;
        }
        if let Some(v) = self.ae_flicker_mode {
            dict.set_item("AeFlickerMode", v)?;
        }
        if let Some(v) = self.exposure_time_mode {
            dict.set_item("ExposureTimeMode", v)?;
        }
        if let Some(ColourGain { red, blue }) = self.colour_gains {
            dict.set_item("ColourGains", (red, blue))?;
        }
        if let Some(v) = self.ae_exposure_mode {
            dict.set_item("AeExposureMode", v)?;
        }
        if let Some(v) = self.exposure_value {
            dict.set_item("ExposureValue", v)?;
        }
        if let Some(v) = self.ae_constraint_mode {
            dict.set_item("AeConstraintMode", v)?;
        }
        if let Some(v) = self.brightness {
            dict.set_item("Brightness", v)?;
        }
        if let Some(v) = self.awb_mode {
            dict.set_item("AwbMode", v)?;
        }
        if let Some(v) = self.cnn_enable_input_tensor {
            dict.set_item("CnnEnableInputTensor", v)?;
        }
        if let Some(v) = self.sync_frames {
            dict.set_item("SyncFrames", v)?;
        }
        if let Some(v) = self.saturation {
            dict.set_item("Saturation", v)?;
        }
        if let Some(v) = self.stats_output_enable {
            dict.set_item("StatsOutputEnable", v)?;
        }
        if let Some(v) = self.sync_mode {
            dict.set_item("SyncMode", v)?;
        }
        if let Some(FrameDurationLimits { min, max }) = self.frame_duration_limits {
            dict.set_item("FrameDurationLimits", (min, max))?;
        }
        if let Some(v) = self.exposure_time {
            dict.set_item("ExposureTime", v)?;
        }
        if let Some(v) = self.ae_metering_mode {
            dict.set_item("AeMeteringMode", v)?;
        }
        if let Some(ScalerCrop {
            x,
            y,
            width,
            height,
        }) = self.scaler_crop
        {
            dict.set_item("ScalerCrop", (x, y, width, height))?;
        }
        if let Some(v) = self.awb_enable {
            dict.set_item("AwbEnable", v)?;
        }
        if let Some(v) = self.hdr_mode {
            dict.set_item("HdrMode", v)?;
        }
        if let Some(v) = self.analogue_gain_mode {
            dict.set_item("AnalogueGainMode", v)?;
        }

        Ok(dict)
    }
}

impl CameraControlsLimit {
    fn extract_option<'py, T>(val: Bound<'py, PyAny>) -> Result<Option<T>, anyhow::Error>
    where
        T: FromPyObject<'py>,
    {
        if val.is_none() {
            Ok(None)
        } else {
            Ok(Some(val.extract()?))
        }
    }

    pub fn from_control_triplets<'py>(
        dict: Bound<'py, PyDict>,
    ) -> Result<
        (
            CameraControlsLimit,
            CameraControlsLimit,
            CameraControlsLimit,
        ),
        anyhow::Error,
    > {
        let mut controls_min = CameraControlsLimit::default();
        let mut controls_max = CameraControlsLimit::default();
        let mut controls_def = CameraControlsLimit::default();

        let get_triplet = |obj: Bound<'py, PyAny>| -> Result<
            (Bound<'py, PyAny>, Bound<'py, PyAny>, Bound<'py, PyAny>),
            anyhow::Error,
        > {
            let tup = obj
                .downcast::<PyTuple>()
                .map_err(|e| anyhow::anyhow!("Expected a 3-tuple: {}", e))?;
            if tup.len()? != 3 {
                anyhow::bail!("Expected 3-tuple");
            }
            Ok((tup.get_item(0)?, tup.get_item(1)?, tup.get_item(2)?))
        };

        if let Some(obj) = dict.get_item("Contrast")? {
            let (min, max, def) = get_triplet(obj)?;
            controls_min.contrast = Self::extract_option(min)?;
            controls_max.contrast = Self::extract_option(max)?;
            controls_def.contrast = Self::extract_option(def)?;
        }
        if let Some(obj) = dict.get_item("AeEnable")? {
            let (min, max, def) = get_triplet(obj)?;
            controls_min.ae_enable = Self::extract_option(min)?;
            controls_max.ae_enable = Self::extract_option(max)?;
            controls_def.ae_enable = Self::extract_option(def)?;
        }
        if let Some(obj) = dict.get_item("Sharpness")? {
            let (min, max, def) = get_triplet(obj)?;
            controls_min.sharpness = Self::extract_option(min)?;
            controls_max.sharpness = Self::extract_option(max)?;
            controls_def.sharpness = Self::extract_option(def)?;
        }
        if let Some(obj) = dict.get_item("NoiseReductionMode")? {
            let (min, max, def) = get_triplet(obj)?;
            controls_min.noise_reduction_mode = Self::extract_option(min)?;
            controls_max.noise_reduction_mode = Self::extract_option(max)?;
            controls_def.noise_reduction_mode = Self::extract_option(def)?;
        }
        if let Some(obj) = dict.get_item("AeFlickerPeriod")? {
            let (min, max, def) = get_triplet(obj)?;
            controls_min.ae_flicker_period = Self::extract_option(min)?;
            controls_max.ae_flicker_period = Self::extract_option(max)?;
            controls_def.ae_flicker_period = Self::extract_option(def)?;
        }
        if let Some(obj) = dict.get_item("AnalogueGain")? {
            let (min, max, def) = get_triplet(obj)?;
            controls_min.analogue_gain = Self::extract_option(min)?;
            controls_max.analogue_gain = Self::extract_option(max)?;
            controls_def.analogue_gain = Self::extract_option(def)?;
        }
        if let Some(obj) = dict.get_item("ColourTemperature")? {
            let (min, max, def) = get_triplet(obj)?;
            controls_min.colour_temperature = Self::extract_option(min)?;
            controls_max.colour_temperature = Self::extract_option(max)?;
            controls_def.colour_temperature = Self::extract_option(def)?;
        }
        if let Some(obj) = dict.get_item("AeFlickerMode")? {
            let (min, max, def) = get_triplet(obj)?;
            controls_min.ae_flicker_mode = Self::extract_option(min)?;
            controls_max.ae_flicker_mode = Self::extract_option(max)?;
            controls_def.ae_flicker_mode = Self::extract_option(def)?;
        }
        if let Some(obj) = dict.get_item("ExposureTimeMode")? {
            let (min, max, def) = get_triplet(obj)?;
            controls_min.exposure_time_mode = Self::extract_option(min)?;
            controls_max.exposure_time_mode = Self::extract_option(max)?;
            controls_def.exposure_time_mode = Self::extract_option(def)?;
        }
        if let Some(obj) = dict.get_item("ColourGains")? {
            let (min, max, def) = get_triplet(obj)?;
            controls_min.colour_gains = Self::extract_option(min)?;
            controls_max.colour_gains = Self::extract_option(max)?;
            controls_def.colour_gains = Self::extract_option(def)?;
        }
        if let Some(obj) = dict.get_item("AeExposureMode")? {
            let (min, max, def) = get_triplet(obj)?;
            controls_min.ae_exposure_mode = Self::extract_option(min)?;
            controls_max.ae_exposure_mode = Self::extract_option(max)?;
            controls_def.ae_exposure_mode = Self::extract_option(def)?;
        }
        if let Some(obj) = dict.get_item("ExposureValue")? {
            let (min, max, def) = get_triplet(obj)?;
            controls_min.exposure_value = Self::extract_option(min)?;
            controls_max.exposure_value = Self::extract_option(max)?;
            controls_def.exposure_value = Self::extract_option(def)?;
        }
        if let Some(obj) = dict.get_item("AeConstraintMode")? {
            let (min, max, def) = get_triplet(obj)?;
            controls_min.ae_constraint_mode = Self::extract_option(min)?;
            controls_max.ae_constraint_mode = Self::extract_option(max)?;
            controls_def.ae_constraint_mode = Self::extract_option(def)?;
        }
        if let Some(obj) = dict.get_item("Brightness")? {
            let (min, max, def) = get_triplet(obj)?;
            controls_min.brightness = Self::extract_option(min)?;
            controls_max.brightness = Self::extract_option(max)?;
            controls_def.brightness = Self::extract_option(def)?;
        }
        if let Some(obj) = dict.get_item("AwbMode")? {
            let (min, max, def) = get_triplet(obj)?;
            controls_min.awb_mode = Self::extract_option(min)?;
            controls_max.awb_mode = Self::extract_option(max)?;
            controls_def.awb_mode = Self::extract_option(def)?;
        }
        if let Some(obj) = dict.get_item("CnnEnableInputTensor")? {
            let (min, max, def) = get_triplet(obj)?;
            controls_min.cnn_enable_input_tensor = Self::extract_option(min)?;
            controls_max.cnn_enable_input_tensor = Self::extract_option(max)?;
            controls_def.cnn_enable_input_tensor = Self::extract_option(def)?;
        }
        if let Some(obj) = dict.get_item("SyncFrames")? {
            let (min, max, def) = get_triplet(obj)?;
            controls_min.sync_frames = Self::extract_option(min)?;
            controls_max.sync_frames = Self::extract_option(max)?;
            controls_def.sync_frames = Self::extract_option(def)?;
        }
        if let Some(obj) = dict.get_item("Saturation")? {
            let (min, max, def) = get_triplet(obj)?;
            controls_min.saturation = Self::extract_option(min)?;
            controls_max.saturation = Self::extract_option(max)?;
            controls_def.saturation = Self::extract_option(def)?;
        }
        if let Some(obj) = dict.get_item("StatsOutputEnable")? {
            let (min, max, def) = get_triplet(obj)?;
            controls_min.stats_output_enable = Self::extract_option(min)?;
            controls_max.stats_output_enable = Self::extract_option(max)?;
            controls_def.stats_output_enable = Self::extract_option(def)?;
        }
        if let Some(obj) = dict.get_item("SyncMode")? {
            let (min, max, def) = get_triplet(obj)?;
            controls_min.sync_mode = Self::extract_option(min)?;
            controls_max.sync_mode = Self::extract_option(max)?;
            controls_def.sync_mode = Self::extract_option(def)?;
        }
        if let Some(obj) = dict.get_item("FrameDurationLimits")? {
            let (min, max, def) = get_triplet(obj)?;
            controls_min.frame_duration_limits = Self::extract_option(min)?;
            controls_max.frame_duration_limits = Self::extract_option(max)?;
            controls_def.frame_duration_limits = Self::extract_option(def)?;
        }
        if let Some(obj) = dict.get_item("ExposureTime")? {
            let (min, max, def) = get_triplet(obj)?;
            controls_min.exposure_time = Self::extract_option(min)?;
            controls_max.exposure_time = Self::extract_option(max)?;
            controls_def.exposure_time = Self::extract_option(def)?;
        }
        if let Some(obj) = dict.get_item("AeMeteringMode")? {
            let (min, max, def) = get_triplet(obj)?;
            controls_min.ae_metering_mode = Self::extract_option(min)?;
            controls_max.ae_metering_mode = Self::extract_option(max)?;
            controls_def.ae_metering_mode = Self::extract_option(def)?;
        }
        if let Some(obj) = dict.get_item("ScalerCrop")? {
            let (min, max, def) = get_triplet(obj)?;
            let parse = |val: Bound<'py, PyAny>| -> Result<Option<ScalerCrop>, anyhow::Error> {
                if val.is_none() {
                    return Ok(None);
                }
                let tup = val
                    .downcast::<PyTuple>()
                    .map_err(|e| anyhow::anyhow!("Expected a 3-tuple: {}", e))?;
                Ok(Some(ScalerCrop {
                    x: tup.get_item(0)?.extract()?,
                    y: tup.get_item(1)?.extract()?,
                    width: tup.get_item(2)?.extract()?,
                    height: tup.get_item(3)?.extract()?,
                }))
            };
            controls_min.scaler_crop = parse(min)?;
            controls_max.scaler_crop = parse(max)?;
            controls_def.scaler_crop = parse(def)?;
        }
        if let Some(obj) = dict.get_item("AwbEnable")? {
            let (min, max, def) = get_triplet(obj)?;
            controls_min.awb_enable = Self::extract_option(min)?;
            controls_max.awb_enable = Self::extract_option(max)?;
            controls_def.awb_enable = Self::extract_option(def)?;
        }
        if let Some(obj) = dict.get_item("HdrMode")? {
            let (min, max, def) = get_triplet(obj)?;
            controls_min.hdr_mode = Self::extract_option(min)?;
            controls_max.hdr_mode = Self::extract_option(max)?;
            controls_def.hdr_mode = Self::extract_option(def)?;
        }
        if let Some(obj) = dict.get_item("AnalogueGainMode")? {
            let (min, max, def) = get_triplet(obj)?;
            controls_min.analogue_gain_mode = Self::extract_option(min)?;
            controls_max.analogue_gain_mode = Self::extract_option(max)?;
            controls_def.analogue_gain_mode = Self::extract_option(def)?;
        }

        Ok((controls_min, controls_max, controls_def))
    }
}
