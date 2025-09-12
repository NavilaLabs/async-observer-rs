use std::sync::Arc;
use async_trait::async_trait;
use lettre::{message::Message, AsyncTransport, Tokio1Executor};
use tracing::{error, info};

use crate::Observer;

/// The `EmailObserver` sends an email containing the event data.
pub struct EmailObserver<T> {
    smtp_transport: Arc<T>,
}

impl<T> EmailObserver<T> {
    /// Creates a new `EmailObserver` instance.
    ///
    /// # Arguments
    ///
    /// * `smtp_transport` - An instance of `AsyncSmtpTransport` to send emails.
    pub fn new(
        smtp_transport: T,
    ) -> Self {
        Self {
            smtp_transport: Arc::new(smtp_transport),
        }
    }
}

#[async_trait]
impl<T: AsyncTransport + Send + Sync + 'static, D: TryInto<Message>> Observer<String> for EmailObserver<T> {
    async fn update(&self, data: &D) {
        let email = match data.try_into() {
            Ok(email) => email,
            Err(e) => {
                error!("Failed to build email message: {:?}", e);
                return;
            }
        };

        if let Err(e) = self.smtp_transport.send(email).await {
            error!("Failed to send email: {:?}", e);
        } else {
            info!("Email sent successfully to {}", self.to_email);
        }
    }
}
