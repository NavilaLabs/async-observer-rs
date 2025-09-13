use async_observer::Subject;
use async_observer::observers::{
    channel::ChannelObserver, email::EmailObserver, file_logger::FileLoggerObserver,
    http::HttpObserver, websocket::WebSocketObserver,
};
use lettre::{AsyncSmtpTransport, Tokio1Executor};
use lettre::{Message as EmailMessage, transport::smtp::SmtpTransport};
use reqwest::Client;
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;
use tokio::fs::{File, OpenOptions};
use tokio::sync::{Mutex, mpsc};
use tokio::time::Duration;
use tokio_tungstenite::tungstenite::Message;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, Serialize, Clone)]
struct EventData {
    id: u32,
    message: String,
    timestamp: String,
}

#[tokio::main]
async fn main() {
    // 1. Setup a simple tracing subscriber for logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // 2. Setup the Subject that will notify observers
    let subject = Arc::new(Subject::default());

    // 3. Setup and attach the FileLoggerObserver
    let file = Arc::new(Mutex::new(
        OpenOptions::new()
            .append(true)
            .create(true)
            .open("events.log")
            .await
            .unwrap(),
    ));
    let file_observer = Arc::new(FileLoggerObserver::new(file));
    let _file_handle = subject.attach(file_observer);

    // 4. Setup and attach the HttpObserver
    let http_client = Client::new();
    let url = "https://httpbin.org/post".to_string();
    let http_observer = Arc::new(HttpObserver::new(http_client, url));
    let _http_handle = subject.attach(http_observer);

    // 5. Setup and attach the EmailObserver
    // To avoid needing a real SMTP server, we use a mock transport.
    let mock_transport = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous("localhost")
        .port(25)
        .build();
    let email_observer = Arc::new(EmailObserver::new(
        mock_transport,
        Box::new(|data: &EventData| {
            let body = format!("New event received at {}: {}", data.timestamp, data.message);
            EmailMessage::builder()
                .from("sender@example.com".parse().unwrap())
                .to("recipient@example.com".parse().unwrap())
                .subject(format!("Event #{} Occurred!", data.id))
                .body(body.to_string())
                .unwrap()
        }),
    ));
    let _email_handle = subject.attach(email_observer);

    // 6. Setup and attach the WebSocketObserver
    let (ws_tx, mut ws_rx) = mpsc::channel::<Message>(100);
    let ws_observer = Arc::new(WebSocketObserver::new(ws_tx));
    let _ws_handle = subject.attach(ws_observer);
    tokio::spawn(async move {
        while let Some(msg) = ws_rx.recv().await {
            if let Ok(text) = msg.into_text() {
                info!("WebSocket Sink: Received '{}' message.", text);
            }
        }
    });

    // 7. Now, trigger the events
    info!("\nStarting event loop. Notifications will go to all observers.");
    for i in 1..=3 {
        let event_data = EventData {
            id: i,
            message: format!("A new user has signed up."),
            timestamp: format!("{:?}", tokio::time::Instant::now()),
        };
        subject.notify(&event_data).await;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    info!("\nAll events sent. The listeners will continue running until the application exits.");
}
