use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum NtpRequest {
    Step,
    Slew,
}