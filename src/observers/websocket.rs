use async_trait::async_trait;
use serde::Serialize;
use serde_json::to_string_pretty;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
#[cfg(feature = "logging")]
use tracing::{error, info};

use crate::Observer;

/// An observer that sends a message to a WebSocket client.
pub struct WebSocketObserver {
    sink: mpsc::Sender<Message>,
}

impl WebSocketObserver {
    /// Creates a new `WebSocketObserver` from the send half of a WebSocket stream.
    pub fn new(sink: mpsc::Sender<Message>) -> Self {
        Self { sink }
    }
}

#[async_trait]
impl<T: Serialize + Send + Sync + 'static> Observer<T> for WebSocketObserver {
    async fn update(&self, data: &T) {
        match to_string_pretty(data) {
            Ok(json_data) => {
                let msg = Message::Text(json_data.into());
                if let Err(e) = self.sink.send(msg).await {
                    #[cfg(feature = "logging")]
                    error!("Failed to send WebSocket message: {}", e);
                } else {
                    #[cfg(feature = "logging")]
                    info!("WebSocketObserver sent a message.");
                }
            }
            Err(e) => {
                #[cfg(feature = "logging")]
                error!("Failed to serialize data for WebSocket: {}", e);
            }
        }
    }
}
