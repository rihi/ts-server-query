use std::collections::HashMap;
use std::io;

use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerError {
    pub id: u32,
    pub message: String,
    pub fields: HashMap<String, String>,
}

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("query connection is closed")]
    Closed,

    #[error("invalid ServerQuery command")]
    InvalidCommand,

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("protocol error: {0}")]
    Protocol(String),

    #[error("server error {}: {}", .0.id, .0.message)]
    Server(ServerError),
}
