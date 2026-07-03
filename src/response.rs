use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq)]
/// Response returned for a ServerQuery command.
pub struct Response {
    /// Non-status response lines received before the final `error` status line.
    pub lines: Vec<String>,

    /// Parsed key-value fields from the final `error` status line.
    pub fields: HashMap<String, String>,
}

impl Response {
    /// Returns the numeric ServerQuery status id from the final status line.
    pub fn id(&self) -> Option<u32> {
        self.fields.get("id")?.parse().ok()
    }

    /// Returns the ServerQuery status message from the final status line.
    pub fn message(&self) -> Option<&str> {
        self.fields.get("msg").map(String::as_str)
    }

    /// Returns whether the ServerQuery status id is `0`.
    pub fn is_ok(&self) -> bool {
        self.id() == Some(0)
    }
}
