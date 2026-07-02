use crate::command::Command;
use crate::error::SendError;
use crate::protocol::Event;
use crate::response::Response;
use tokio::sync::{broadcast, mpsc, oneshot};

#[derive(Clone, Debug)]
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

    pub async fn send(&self, command: Command) -> Result<Response, SendError> {
        let (tx, rx) = oneshot::channel();

        self.commands
            .send(Request { command, reply: tx })
            .await
            .map_err(|_| SendError::Closed)?;

        rx.await.map_err(|_| SendError::Closed)
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<Event> {
        self.events.subscribe()
    }
}

#[derive(Debug)]
pub(crate) struct Request {
    pub(crate) command: Command,
    pub(crate) reply: oneshot::Sender<Response>,
}
