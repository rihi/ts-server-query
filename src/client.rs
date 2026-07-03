use crate::command::Command;
use crate::error::ConnectionClosed;
use crate::protocol::Event;
use crate::response::Response;
use tokio::sync::{broadcast, mpsc, oneshot};

#[derive(Clone, Debug)]
/// Handle used to send ServerQuery commands and subscribe to notifications.
///
/// A `QueryClient` is cheap to clone. Clones share the same underlying
/// connection task, so all commands are sent over the connection future returned
/// by [`crate::query_connection`] or [`crate::query_connection_parts`].
pub struct QueryClient {
    commands: mpsc::Sender<Request>,
    events: broadcast::Sender<Event>,
}

impl QueryClient {
    pub(crate) fn new(
        commands: mpsc::Sender<Request>,
        events: broadcast::Sender<Event>,
    ) -> Self {
        Self { commands, events }
    }

    /// Sends a command and waits for its response.
    ///
    /// This returns [`ConnectionClosed`] if the connection future has stopped or
    /// if it is no longer able to deliver the response.
    pub async fn send(&self, command: Command) -> Result<Response, ConnectionClosed> {
        let (tx, rx) = oneshot::channel();

        self.commands
            .send(Request { command, reply: tx })
            .await
            .map_err(|_| ConnectionClosed)?;

        rx.await.map_err(|_| ConnectionClosed)
    }

    /// Subscribes to ServerQuery notification events.
    ///
    /// The returned receiver yields events parsed from lines whose command name
    /// starts with `notify`. Events are broadcast to all active subscribers.
    pub fn subscribe_events(&self) -> broadcast::Receiver<Event> {
        self.events.subscribe()
    }
}

#[derive(Debug)]
pub(crate) struct Request {
    pub(crate) command: Command,
    pub(crate) reply: oneshot::Sender<Response>,
}
