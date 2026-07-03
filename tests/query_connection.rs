use std::error::Error;
use std::time::Duration;

use tokio::io::{duplex, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::time::timeout;
use ts_server_query::{query_connection_parts, Command};

const TEST_TIMEOUT: Duration = Duration::from_secs(1);

#[tokio::test]
async fn queries_response_from_split_stream() -> Result<(), Box<dyn Error>> {
    let (client_stream, mut server_stream) = duplex(1024);
    let (reader, writer) = tokio::io::split(client_stream);
    let (client, connection) = query_connection_parts(reader, writer);
    let connection = tokio::spawn(connection);

    let server = tokio::spawn(async move {
        server_stream
            .write_all(b"TS3\n\rWelcome to the TeamSpeak 3 ServerQuery interface\n\r")
            .await?;

        let mut server_reader = BufReader::new(server_stream);
        let mut command = Vec::new();
        server_reader.read_until(b'\n', &mut command).await?;
        assert_eq!(command, b"version\n");

        let mut server_stream = server_reader.into_inner();
        server_stream
            .write_all(b"error id=0 msg=ok\n\r")
            .await?;

        Ok::<_, std::io::Error>(())
    });

    let response = timeout(TEST_TIMEOUT, client.send(Command::new("version")?)).await??;
    assert!(response.is_ok());
    assert!(response.lines.is_empty());

    drop(client);
    timeout(TEST_TIMEOUT, server).await???;
    timeout(TEST_TIMEOUT, connection).await???;

    Ok(())
}
