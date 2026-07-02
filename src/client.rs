use tokio::sync::{broadcast, mpsc, oneshot};

use crate::channel::{parse_channel_list, Channel};
use crate::command::Command;
use crate::error::QueryError;
use crate::protocol::Event;
use crate::response::Response;

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

    pub async fn send(&self, command: Command) -> Result<Response, QueryError> {
        let (tx, rx) = oneshot::channel();

        self.commands
            .send(Request { command, reply: tx })
            .await
            .map_err(|_| QueryError::Closed)?;

        rx.await.map_err(|_| QueryError::Closed)?
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<Event> {
        self.events.subscribe()
    }

    pub async fn channel_list(&self) -> Result<Vec<Channel>, QueryError> {
        let response = self
            .send(Command::raw("channellist -topic -flags")?)
            .await?;
        parse_channel_list(&response)
    }
}

#[derive(Debug)]
pub(crate) struct Request {
    pub(crate) command: Command,
    pub(crate) reply: oneshot::Sender<Result<Response, QueryError>>,
}
