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
