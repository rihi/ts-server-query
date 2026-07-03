# Ts Server Query

Minimal async TeamSpeak ServerQuery client for Tokio.

`ts-server-query` is designed to fit into different kinds of async applications. It does not assume whether you are building a bot, admin tool, or background service; it just provides the connection, command, event, and error-handling pieces.

Dropping all client handles gracefully shuts down the connection future.

```rust
use ts_server_query::{query_connection, Command};
use tokio::net::TcpStream;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let stream = TcpStream::connect("127.0.0.1:10011").await?;
let (client, connection) = query_connection(stream);

let connection = tokio::spawn(async move {
    if let Err(error) = connection.await {
        eprintln!("ServerQuery connection failed: {error}");
    }
});

let response = client.send(Command::new("version")?).await?;
assert!(response.is_ok());

drop(client);
connection.await??;
# Ok(())
# }
```
