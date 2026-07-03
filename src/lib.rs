//! Async TeamSpeak ServerQuery client for Tokio.
//!
//! The crate separates command submission from connection IO. Create a
//! [`QueryClient`] and a connection future with [`query_connection`] or
//! [`query_connection_parts`], then await or spawn the connection future while
//! using the client to send [`Command`] values.
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
//! The connection future drives all socket IO. Commands, responses, and event
//! subscriptions only make progress while that future is being polled.

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
