use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::io;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerError {
    pub id: u32,
    pub message: String,
    pub fields: HashMap<String, String>,
}

#[derive(Debug)]
pub enum QueryError {
    Closed,
    InvalidCommand,
    Io(io::Error),
    Protocol(String),
    Server(ServerError),
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryError::Closed => f.write_str("query connection is closed"),
            QueryError::InvalidCommand => f.write_str("invalid ServerQuery command"),
            QueryError::Io(error) => write!(f, "I/O error: {error}"),
            QueryError::Protocol(message) => write!(f, "protocol error: {message}"),
            QueryError::Server(error) => {
                write!(f, "server error {}: {}", error.id, error.message)
            }
        }
    }
}

impl Error for QueryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            QueryError::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for QueryError {
    fn from(error: io::Error) -> Self {
        QueryError::Io(error)
    }
}
