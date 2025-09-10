# async-observer-rs
An ergonomic, asynchronous, and thread-safe implementation of the Observer design pattern for Rust.

This crate provides a robust and flexible way to implement event-driven architectures in your async/.await applications. It is independent of any specific runtime, built on top of the standard futures crate, making it compatible with runtimes like Tokio, async-std, and others.

Features
Asynchronous: The observer's update method is async, allowing it to perform non-blocking operations.

Thread-Safe: The Subject is designed to be shared across threads and tasks using Arc and Mutex.

Ergonomic Detachment: An ObserverHandle is returned upon attachment. When this handle is dropped, the observer is automatically detached from the Subject. Explicit detachment is also supported.

Optional Logging: Uses the tracing crate for configurable logging, providing visibility into observer lifecycle events.

## Examples

```bash
cargo run --example observer --features logging
```
