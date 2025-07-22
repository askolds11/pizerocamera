use bytes::Bytes;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct SuccessWrapper<T: Serialize> {
    pub success: bool,
    pub value: T,
}

impl<T: Serialize> SuccessWrapper<T> {
    pub fn success(value: T) -> SuccessWrapper<T> {
        SuccessWrapper {
            success: true,
            value,
        }
    }
    pub fn failure(value: T) -> SuccessWrapper<T> {
        SuccessWrapper {
            success: false,
            value,
        }
    }

    pub fn into_bytes(self) -> Result<Bytes, serde_json::error::Error> {
        serde_json::to_string(&self).map(|s| s.into())
    }
}
