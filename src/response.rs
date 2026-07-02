use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Response {
    pub lines: Vec<String>,
    pub fields: HashMap<String, String>,
}

impl Response {
    pub fn id(&self) -> Option<u32> {
        self.fields.get("id")?.parse().ok()
    }

    pub fn message(&self) -> Option<&str> {
        self.fields.get("msg").map(String::as_str)
    }

    pub fn is_ok(&self) -> bool {
        self.id() == Some(0)
    }
}
