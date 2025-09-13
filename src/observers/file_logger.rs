use std::sync::Arc;

use crate::Observer;
use async_trait::async_trait;
use serde::Serialize;
use serde_json::to_string_pretty;
use tokio::io::AsyncWriteExt;
use tokio::{fs::File, sync::Mutex};
#[cfg(feature = "logging")]
use tracing::{error, info};

/// An observer that writes received data to a file.
pub struct FileLoggerObserver {
    file: Arc<Mutex<File>>,
}

impl FileLoggerObserver {
    /// Creates a new `FileLoggerObserver` from a pre-opened file.
    pub fn new(file: Arc<Mutex<File>>) -> Self {
        Self { file }
    }
}
#[async_trait]
impl<T: Serialize + Send + Sync + 'static> Observer<T> for FileLoggerObserver {
    async fn update(&self, data: &T) {
        let mut file = self.file.lock().await;
        match to_string_pretty(data) {
            Ok(json_data) => {
                if let Err(e) = file.write_all(json_data.as_bytes()).await {
                    #[cfg(feature = "logging")]
                    error!("Failed to write to file: {}", e);
                } else if let Err(e) = file.write_all(b"\n").await {
                    #[cfg(feature = "logging")]
                    error!("Failed to write newline to file: {}", e);
                } else {
                    #[cfg(feature = "logging")]
                    info!("FileLoggerObserver wrote message to file.");
                }
            }
            Err(e) => {
                #[cfg(feature = "logging")]
                error!("Failed to serialize data for file logging: {}", e);
            }
        }
    }
}
