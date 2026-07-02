use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Response {
    lines: Vec<String>,
    metadata: HashMap<String, String>,
}

impl Response {
    pub(crate) fn new(
        lines: Vec<String>,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self { lines, metadata }
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}
