use async_trait::async_trait;
use tokio::sync::mpsc;
#[cfg(feature = "logging")]
use tracing::{error, info};

use crate::Observer;

/// An observer that sends data through a Tokio MPSC channel.
pub struct ChannelObserver<T> {
    sender: mpsc::Sender<T>,
}

impl<T> ChannelObserver<T> {
    /// Creates a new `ChannelObserver` from a channel sender.
    pub fn new(sender: mpsc::Sender<T>) -> Self {
        Self { sender }
    }
}

#[async_trait]
impl<T: Clone + Send + Sync + 'static> Observer<T> for ChannelObserver<T> {
    async fn update(&self, data: &T) {
        if let Err(e) = self.sender.send(data.clone()).await {
            #[cfg(feature = "logging")]
            error!("Failed to send data to channel: {}", e);
        } else {
            #[cfg(feature = "logging")]
            info!("ChannelObserver sent data to MPSC channel.");
        }
    }
}
