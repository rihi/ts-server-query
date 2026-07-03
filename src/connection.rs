use std::collections::VecDeque;
use std::future::Future;
use std::time::Duration;

use tokio::io::{split, AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::sync::{broadcast, mpsc};
use tokio::time::timeout;

use crate::client::{QueryClient, Request};
use crate::error::ConnectionError;
use crate::protocol::{parse_fields, Event};
use crate::Response;

const COMMAND_BUFFER: usize = 64;
const EVENT_BUFFER: usize = 256;
const STARTUP_LINE_COUNT: usize = 2;
const STARTUP_LINE_TIMEOUT: Duration = Duration::from_secs(5);

/// Creates a client and connection future from a bidirectional Tokio IO stream.
///
/// The returned [`QueryClient`] is used to send commands. The returned future
/// owns the stream and must be awaited or spawned; it drives startup handling,
/// command writes, response reads, and event broadcasts.
///
/// Use [`query_connection_parts`] if the reader and writer have already been
/// split.
pub fn query_connection<S>(
    stream: S,
) -> (
    QueryClient,
    impl Future<Output = Result<(), ConnectionError>> + Send + 'static,
) where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let (reader, writer) = split(stream);
    query_connection_parts(reader, writer)
}

/// Creates a client and connection future from separate reader and writer
/// halves.
///
/// This is the lower-level constructor used by [`query_connection`]. It is
/// useful for already-split streams, test IO, or wrapper types that expose
/// separate read and write handles.
pub fn query_connection_parts<'a>(
    reader: impl AsyncRead + Unpin + Send + 'a,
    writer: impl AsyncWrite + Unpin + Send + 'a,
) -> (
    QueryClient,
    impl Future<Output = Result<(), ConnectionError>> + Send + 'a,
)
{
    let (commands_tx, commands_rx) = mpsc::channel(COMMAND_BUFFER);
    let (events_tx, _) = broadcast::channel(EVENT_BUFFER);

    let client = QueryClient::new(commands_tx, events_tx.clone());
    let connection = run_query_connection(reader, writer, commands_rx, events_tx);

    (client, connection)
}

pub(crate) async fn run_query_connection(
    reader: impl AsyncRead + Unpin,
    mut writer: impl AsyncWrite + Unpin,
    mut commands: mpsc::Receiver<Request>,
    events: broadcast::Sender<Event>,
) -> Result<(), ConnectionError>
{
    let mut reader = BufReader::new(reader);
    let mut pending = VecDeque::new();
    let mut encountered_cmd_eof = false;
    let mut encountered_reader_eof = false;

    skip_startup_lines(&mut reader).await?;

    let mut current_response = Vec::new();
    let mut current_line = Vec::new();
    loop {
        tokio::select! {
            command = commands.recv(), if !encountered_cmd_eof => {
                let Some(request) = command else {
                    writer.shutdown().await?;
                    encountered_cmd_eof = true;
                    continue;
                };

                writer.write_all(request.command.as_str().as_bytes()).await?;
                writer.write_all(b"\n").await?;
                writer.flush().await?;
                pending.push_back(request.reply);
            }
            read = reader.read_until(b'\r', &mut current_line), if !encountered_reader_eof => {
                let bytes_read = read?;
                if bytes_read == 0 {
                    commands.close();
                    encountered_reader_eof = true;
                    continue;
                }

                let line = std::str::from_utf8(&current_line)
                    .map_err(|_| ConnectionError::Protocol("received non-UTF-8 line".to_owned()))?;
                let line = line.trim_end_matches("\n\r").to_owned();
                current_line.clear();

                if line.starts_with("notify") {
                    let (name, rest) = line.split_once(' ').ok_or_else(|| {
                        ConnectionError::Protocol("received event without fields".to_owned())
                    })?;
                    let fields = parse_fields(rest)?;
                
                    let event = Event { 
                        name: name.to_owned(),
                        fields,
                    };
                    let _ = events.send(event);
                    continue;
                }

                if let Some(status_params) = line.strip_prefix("error ") {
                    let fields = parse_fields(status_params)?;
                    let reply = pending.pop_front().ok_or_else(|| {
                        ConnectionError::Protocol("received response without a pending request".to_owned())
                    })?;
                    
                    let response = Response { 
                        lines: std::mem::take(&mut current_response),
                        fields,
                    };
                    let _ = reply.send(response);
                    continue;
                }

                if pending.is_empty() {
                    return Err(ConnectionError::Protocol(format!(
                        "received response line without a pending request: `{line}`"
                    )));
                }

                current_response.push(line.to_owned());
            }
            else => {
                return Ok(());
            }
        }
    }
}

async fn skip_startup_lines(
    reader: &mut BufReader<impl AsyncRead + Unpin>
) -> Result<(), ConnectionError>
{
    for index in 0..STARTUP_LINE_COUNT {
        let mut line = Vec::new();
        let bytes_read = timeout(STARTUP_LINE_TIMEOUT, reader.read_until(b'\r', &mut line))
            .await
            .map_err(|_| {
                ConnectionError::Protocol(format!(
                    "timed out waiting for startup line {}",
                    index + 1
                ))
            })??;

        if bytes_read == 0 {
            return Err(ConnectionError::Closed);
        }
    }

    Ok(())
}
