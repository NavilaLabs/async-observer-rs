use async_trait::async_trait;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
#[cfg(feature = "logging")]
use tracing::{error, info};

use crate::Observer;

/// An observer that sends an email using a provided SMTP transport and message builder.
pub struct EmailObserver<T> {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    message_builder: Box<dyn Fn(&T) -> Message + Send + Sync>,
}

impl<T> EmailObserver<T> {
    /// Creates a new `EmailObserver` with a configured `SmtpTransport` and a message builder closure.
    ///
    /// The `message_builder` closure takes a reference to the event data and should return
    /// a fully constructed `Message`.
    pub fn new(
        transport: AsyncSmtpTransport<Tokio1Executor>,
        message_builder: Box<dyn Fn(&T) -> Message + Send + Sync>,
    ) -> Self {
        Self {
            transport,
            message_builder,
        }
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> Observer<T> for EmailObserver<T> {
    async fn update(&self, data: &T) {
        let email = (self.message_builder)(data);
        match self.transport.send(email).await {
            Ok(_) => {
                #[cfg(feature = "logging")]
                info!("Email sent successfully.");
            }
            Err(e) => {
                #[cfg(feature = "logging")]
                error!("Failed to send email: {}", e);
            }
        }
    }
}
