use reqwest::Response;
use thiserror::Error;

#[derive(Error, Debug)]
pub struct HttpError {
    pub url: Option<String>,
    pub status: Option<u16>,
    #[source]
    pub source: anyhow::Error,
}

impl HttpError {
    pub async fn from_response(response: Response) -> Self {
        assert!(!response.status().is_success());

        HttpError {
            url: Some(response.url().as_str().into()),
            status: Some(response.status().into()),
            source: anyhow::anyhow!(
                response
                    .text()
                    .await
                    .unwrap_or("Response not success".into())
            ),
        }
    }
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Unwrap or use a placeholder for the `Option` values
        let url = self.url.as_deref().unwrap_or("Unknown URL");
        let status = self.status.map_or("Unknown".to_string(), |s| s.to_string());

        write!(
            f,
            "Http Request Error: url = {}, status = {}, source = {}",
            url, status, self.source
        )
    }
}
