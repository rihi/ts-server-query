use std::collections::VecDeque;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};
use tokio::time::timeout;

use crate::client::Request;
use crate::error::QueryError;
use crate::protocol::{finish_response, parse_event, Event};

const STARTUP_LINE_COUNT: usize = 2;
const STARTUP_LINE_TIMEOUT: Duration = Duration::from_secs(5);

pub(crate) async fn run_query_connection(
    mut stream: TcpStream,
    mut commands: mpsc::Receiver<Request>,
    events: broadcast::Sender<Event>,
) -> Result<(), QueryError> {
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut line = Vec::new();
    let mut pending = VecDeque::new();
    let mut current_response = Vec::new();
    let mut encountered_cmd_eof = false;
    let mut encountered_reader_eof = false;

    skip_startup_lines(&mut reader).await?;

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
            read = reader.read_until(b'\n', &mut line), if !encountered_reader_eof => {
                let bytes_read = read?;
                if bytes_read == 0 {
                    commands.close();
                    encountered_reader_eof = true;
                    continue;
                }

                let line_text = std::str::from_utf8(&line)
                    .map_err(|_| QueryError::Protocol("received non-UTF-8 line".to_owned()))?;
                let line_text = trim_line_end(line_text).to_owned();
                line.clear();
                let line = line_text.as_str();

                if line.starts_with("notify") {
                    let event = parse_event(line)?;
                    let _ = events.send(event);
                    continue;
                }

                if line.starts_with("error ") {
                    let response = finish_response(&mut current_response, line);
                    let reply = pending.pop_front().ok_or_else(|| {
                        QueryError::Protocol("received response without a pending request".to_owned())
                    })?;
                    let _ = reply.send(response);
                    continue;
                }

                if pending.is_empty() {
                    return Err(QueryError::Protocol(format!(
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

async fn skip_startup_lines<R>(reader: &mut BufReader<R>) -> Result<(), QueryError>
where
    R: tokio::io::AsyncRead + Unpin,
{
    for index in 0..STARTUP_LINE_COUNT {
        let mut line = String::new();
        let bytes_read = timeout(STARTUP_LINE_TIMEOUT, reader.read_line(&mut line))
            .await
            .map_err(|_| {
                QueryError::Protocol(format!(
                    "timed out waiting for startup line {}",
                    index + 1
                ))
            })??;

        if bytes_read == 0 {
            return Err(QueryError::Closed);
        }
    }

    Ok(())
}

fn trim_line_end(line: &str) -> &str {
    line.trim_end_matches(['\r', '\n'])
}
