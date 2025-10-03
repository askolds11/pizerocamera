use crate::camera::CameraControls;
use numpy::{PyArrayMethods, PyReadonlyArray1};
use pyo3::ffi::c_str;
use pyo3::prelude::{PyAnyMethods, PyDictMethods, PyModule};
use pyo3::types::{PyDict, PyTuple};
use pyo3::{Bound, IntoPyObjectExt, Py, PyAny, PyResult, Python};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum CameraMode {
    Still,
    Video,
}

pub struct CameraService {
    instance: Py<PyAny>,
    pub camera_mode: CameraMode,
    pub still_controls: Option<CameraControls>,
    pub video_controls: Option<CameraControls>,
}

impl CameraService {
    pub fn new(
        py: Python,
        still_controls: &Option<CameraControls>,
        video_controls: &Option<CameraControls>,
        still_controls_pydict: Option<Bound<PyDict>>,
    ) -> PyResult<Self> {
        println!("Rust - CameraService new");
        // Your Python code as string
        let py_code = c_str!(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/python-camera/main.py"
        )));

        // Compile the code into a Python module
        let module = PyModule::from_code(py, py_code, c_str!("main.py"), c_str!("camera_module"))?;

        // Get the class
        let class = module.getattr("CameraService")?;

        let still_controls_py = match still_controls_pydict {
            Some(v) => v.into_py_any(py)?,
            None => py.None(),
        };

        // Instantiate the class
        let instance = class.call1((still_controls_py,))?.into_py_any(py)?;

        let still_controls = still_controls.clone();
        let video_controls = video_controls.clone();

        Ok(CameraService {
            instance,
            camera_mode: CameraMode::Still,
            still_controls,
            video_controls,
        })
    }

    pub fn capture(&self, py: Python, time: u64) -> PyResult<(Vec<u8>, u16, u16, HashMap<String, String>)> {
        let result = self.instance.call_method1(py, "capture", (time, ))?;
        println!("Picture captured");
        // Returned tuple with array and metadata
        let tuple = result.downcast_bound::<PyTuple>(py)?;

        let array = tuple.get_item(0)?;
        let jpeg_bytes: PyReadonlyArray1<u8> = array.extract()?;
        let jpeg_bytes = jpeg_bytes.to_vec()?;
        println!("Bytes converted");

        let width = tuple.get_item(1)?;
        let width: u16 = width.extract()?;
        let height = tuple.get_item(2)?;
        let height: u16 = height.extract()?;

        let mut metadata: HashMap<String, String> = HashMap::new();

        // Try to convert metadata. If it doesn't work, do not return error, as it's important to get
        // images, but metadata not mandatory
        let dict = tuple.get_item(3);
        match dict {
            Ok(dict) => {
                let dict = dict.downcast::<PyDict>();
                match dict {
                    Ok(dict) => {
                        for (key, value) in dict.iter() {
                            let key_str: Option<String> = key.extract().ok();
                            if let Some(key_str) = key_str {
                                let value_str = value.str().ok();
                                if let Some(value_str) = value_str {
                                    let value_str: Option<String> = value_str.extract().ok();
                                    if let Some(value_str) = value_str {
                                        metadata.insert(key_str, value_str);
                                    }
                                }
                            }
                        }
                        println!("Metadata converted")
                    }
                    Err(e) => {
                        println!("Metadata could not be converted: {:?}", e)
                    }
                }

            }
            Err(e) => {
                println!("Metadata could not be converted: {:?}", e)
            }
        }

        Ok((jpeg_bytes, width, height, metadata))
    }

    pub fn get_sync_status(&self, py: Python) -> PyResult<(bool, i64)> {
        let result = self.instance.call_method0(py, "get_sync_status")?;
        // Returned tuple with array and metadata
        let tuple = result.downcast_bound::<PyTuple>(py)?;

        let sync_ready = tuple.get_item(0)?;
        let sync_ready: bool = sync_ready.extract()?;
        let sync_timer = tuple.get_item(1)?;
        let sync_timer: i64 = sync_timer.extract()?;

        Ok((sync_ready, sync_timer))
    }

    pub fn set_controls(&self, py: Python, controls: Bound<PyDict>) -> PyResult<()> {
        self.instance
            .call_method1(py, "set_controls", (controls,))?;
        Ok(())
    }

    pub fn get_controls_limits<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let result = self.instance.call_method0(py, "get_controls")?;
        let dict = result.downcast_bound::<PyDict>(py)?;
        Ok(dict.clone())
    }

    pub fn start_preview(
        &mut self,
        py: Python,
        video_controls_pydict: Option<Bound<PyDict>>,
    ) -> PyResult<()> {
        let video_controls_py = match video_controls_pydict {
            Some(v) => v.into_py_any(py)?,
            None => py.None(),
        };
        self.instance
            .call_method1(py, "start_preview", (video_controls_py,))?;
        self.camera_mode = CameraMode::Video;
        Ok(())
    }

    pub fn stop_preview(
        &mut self,
        py: Python,
        still_controls_pydict: Option<Bound<PyDict>>,
    ) -> PyResult<()> {
        let still_controls_py = match still_controls_pydict {
            Some(v) => v.into_py_any(py)?,
            None => py.None(),
        };
        self.instance
            .call_method1(py, "stop_preview", (still_controls_py,))?;
        self.camera_mode = CameraMode::Still;
        Ok(())
    }

    pub fn stop(&self, py: Python) -> PyResult<()> {
        self.instance.call_method0(py, "stop")?;
        Ok(())
    }
}
