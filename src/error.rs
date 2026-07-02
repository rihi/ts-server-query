use std::io;

use thiserror::Error;

use crate::escaping::EscapeError;

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("query connection is closed")]
    Closed,

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("protocol error: {0}")]
    Protocol(String),

    #[error("escaping error: {0}")]
    Escape(#[from] EscapeError),
}
