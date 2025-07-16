use crate::utils::HttpError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UpdateError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    RequestError(#[from] reqwest::Error),
    #[error(transparent)]
    HttpError(#[from] HttpError),
}
