use std::io;

use thiserror::Error;

use crate::escaping::EscapeError;

#[derive(Debug, Error)]
/// Error returned by the connection future.
pub enum ConnectionError {
    /// The remote side closed before the connection could continue.
    #[error("query connection is closed")]
    Closed,

    /// Underlying IO failed.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Received data was not valid for the expected ServerQuery protocol state.
    #[error("protocol error: {0}")]
    Protocol(String),

    /// Escaped ServerQuery data could not be decoded.
    #[error("escaping error: {0}")]
    Escape(#[from] EscapeError),
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("query connection is closed")]
/// Error returned by [`crate::QueryClient`] when the connection is no longer
/// available.
pub struct ConnectionClosed;
