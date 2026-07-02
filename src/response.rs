use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Response {
    lines: Vec<String>,
    params: HashMap<String, String>,
}

impl Response {
    pub(crate) fn new(
        lines: Vec<String>,
        params: HashMap<String, String>,
    ) -> Self {
        Self { lines, params }
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn params(&self) -> &HashMap<String, String> {
        &self.params
    }

    pub fn id(&self) -> Option<u32> {
        self.params.get("id")?.parse().ok()
    }

    pub fn message(&self) -> Option<&str> {
        self.params.get("msg").map(String::as_str)
    }

    pub fn is_ok(&self) -> bool {
        self.id() == Some(0)
    }
}
