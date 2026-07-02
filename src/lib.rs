use std::future::Future;

use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};

mod channel;
mod client;
mod command;
mod connection;
mod error;
mod escaping;
mod protocol;
mod response;

pub use channel::{parse_channel_list, Channel, ChannelFlags};
pub use client::QueryClient;
pub use command::{Command, CommandError};
pub use error::{QueryError, ServerError};
pub use escaping::{
    escape, is_special_character, unescape, EscapeError, ESCAPE_CHARACTER,
};
pub use protocol::Event;
pub use response::Response;

const COMMAND_BUFFER: usize = 64;
const EVENT_BUFFER: usize = 256;

pub fn query_connection(
    stream: TcpStream,
) -> (
    QueryClient,
    impl Future<Output = Result<(), QueryError>> + Send + 'static,
) {
    let (commands_tx, commands_rx) = mpsc::channel(COMMAND_BUFFER);
    let (events_tx, _) = broadcast::channel(EVENT_BUFFER);

    let client = QueryClient::new(commands_tx, events_tx.clone());
    let connection = connection::run_query_connection(stream, commands_rx, events_tx);

    (client, connection)
}
