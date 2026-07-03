# ts-server-query

Async TeamSpeak ServerQuery client for Tokio.

```rust
use ts_server_query::{query_connection, Command};
use tokio::net::TcpStream;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let stream = TcpStream::connect("127.0.0.1:10011").await?;
let (client, connection) = query_connection(stream);

tokio::spawn(async move {
    if let Err(error) = connection.await {
        eprintln!("ServerQuery connection failed: {error}");
    }
});

let response = client.send(Command::new("version")?).await?;
assert!(response.is_ok());
# Ok(())
# }
```

The returned connection future drives socket IO. It must be awaited or spawned for
commands and event subscriptions to make progress.

