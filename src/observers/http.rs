use async_trait::async_trait;
use reqwest::Client;
use serde::Serialize;
#[cfg(feature = "logging")]
use tracing::{error, info};

use crate::Observer;

/// An observer that sends an HTTP POST request with the data.
pub struct HttpObserver {
    client: Client,
    url: String,
}

impl HttpObserver {
    /// Creates a new `HttpObserver` with a `reqwest` client and a URL.
    pub fn new(client: Client, url: String) -> Self {
        Self { client, url }
    }
}

#[async_trait]
impl<T: Serialize + Send + Sync + 'static> Observer<T> for HttpObserver {
    async fn update(&self, data: &T) {
        match self.client.post(&self.url).json(data).send().await {
            Ok(response) => {
                #[cfg(feature = "logging")]
                info!(
                    "HTTP POST to {} succeeded with status: {}",
                    self.url,
                    response.status()
                );
            }
            Err(e) => {
                #[cfg(feature = "logging")]
                error!("HTTP POST to {} failed: {}", self.url, e);
            }
        }
    }
}
