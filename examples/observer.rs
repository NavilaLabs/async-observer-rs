use async_observer::{Observer, Subject};
use std::sync::Arc;
use tokio::time::{self, Duration};
use async_trait::async_trait;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

/// A concrete observer that logs a message.
struct LoggerObserver;

#[async_trait]
impl Observer<String> for LoggerObserver {
    async fn update(&self, data: &String) {
        info!("[Logger] Received message: \"{}\"", data);
    }
}

/// A concrete observer that simulates a delayed operation.
struct DelayedObserver;

#[async_trait]
impl Observer<String> for DelayedObserver {
    async fn update(&self, data: &String) {
        info!("[Delayed] Starting delayed update for: \"{}\"", data);
        time::sleep(Duration::from_millis(1500)).await;
        info!("[Delayed] Finished delayed update.");
    }
}

/// A concrete observer to be explicitly detached.
struct ExplicitObserver;

#[async_trait]
impl Observer<String> for ExplicitObserver {
    async fn update(&self, data: &String) {
        info!("[Explicit] Received message: \"{}\"", data);
    }
}

#[tokio::main]
async fn main() {
    // Initialize the tracing subscriber to log at or above the INFO level.
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    // Create a new async subject.
    let subject = Arc::new(Subject::default());

    // Create and attach observers.
    let logger_observer = Arc::new(LoggerObserver);
    let delayed_observer = Arc::new(DelayedObserver);
    let explicit_observer = Arc::new(ExplicitObserver);

    info!("Attaching three observers...");
    let logger_handle = subject.attach(logger_observer);
    let _delayed_handle = subject.attach(delayed_observer);
    let explicit_handle = subject.attach(explicit_observer);

    info!("Notifying all observers. All three should receive the event.");
    subject.notify(&String::from("First event")).await;

    // Explicitly detach the explicit observer using its handle.
    info!("\nExplicitly detaching the explicit observer...");
    subject.detach(explicit_handle);

    info!("\nNotifying again. Only the logger and delayed observers should receive the event.");
    subject.notify(&String::from("Second event")).await;
    
    // The `logger_handle` is now dropped by assigning `_` to it.
    // This will trigger the `Drop` implementation and automatically detach the observer.
    info!("\nDropping the logger_handle to trigger automatic detachment.");
    drop(logger_handle);

    info!("\nNotifying a final time. Only the delayed observer should receive the event.");
    subject.notify(&String::from("Third event")).await;

    // The `delayed_handle` will be dropped when the main function ends, detaching the last observer.
    info!("\nExample complete. The remaining observer will be automatically detached.");
}
