//! Async TeamSpeak ServerQuery client for Tokio.
//!
//! This crate provides a small asynchronous client for the TeamSpeak
//! ServerQuery protocol. It handles command serialization, response matching,
//! startup banner handling, and ServerQuery notification events.
//!
//! # Connection model
//!
//! The API separates command submission from connection IO. [`query_connection`]
//! and [`query_connection_parts`] return two values:
//!
//! - a [`QueryClient`], used to send [`Command`] values and subscribe to events
//! - a connection future, which owns the IO object and drives all reads/writes
//!
//! The connection future must be polled for commands and subscriptions to make
//! progress. Most applications spawn it as a background task:
//!
//! ```no_run
//! use ts_server_query::{query_connection, Command};
//! use tokio::net::TcpStream;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let stream = TcpStream::connect("127.0.0.1:10011").await?;
//! let (client, connection) = query_connection(stream);
//!
//! tokio::spawn(async move {
//!     if let Err(error) = connection.await {
//!         eprintln!("ServerQuery connection failed: {error}");
//!     }
//! });
//!
//! let response = client.send(Command::new("version")?).await?;
//! assert!(response.is_ok());
//! # Ok(())
//! # }
//! ```
//!
//! Applications that want direct ownership of connection errors can also await
//! the future alongside their own work, for example with `tokio::select!`:
//!
//! ```no_run
//! use ts_server_query::{query_connection, Command};
//! use tokio::net::TcpStream;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let stream = TcpStream::connect("127.0.0.1:10011").await?;
//! let (client, connection) = query_connection(stream);
//!
//! tokio::select! {
//!     result = connection => {
//!         result?;
//!     }
//!     response = client.send(Command::new("version")?) => {
//!         assert!(response?.is_ok());
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Shutdown
//!
//! Dropping all [`QueryClient`] handles closes the command channel. Once the
//! connection future observes that closure, it shuts down the writer and exits
//! after the reader side has also ended. This means callers do not need a
//! separate close method for the common case: drop the clients, then await the
//! connection future if you need to observe final connection errors.
//!
//! # Split IO
//!
//! Use [`query_connection`] for a single bidirectional IO object such as
//! `tokio::net::TcpStream`. Use [`query_connection_parts`] when you already have
//! separate read and write halves, such as a split stream or test transport.

mod client;
mod command;
mod connection;
mod error;
mod escaping;
mod protocol;
mod response;

pub use client::QueryClient;
pub use command::{Command, CommandError};
pub use connection::{query_connection, query_connection_parts};
pub use error::{ConnectionClosed, ConnectionError};
pub use escaping::{
    escape, is_special_character, unescape, EscapeError, ESCAPE_CHARACTER,
};
pub use protocol::Event;
pub use response::Response;
