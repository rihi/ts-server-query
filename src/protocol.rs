use std::collections::HashMap;

use crate::error::ConnectionError;
use crate::escaping::unescape;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    pub name: String,
    pub fields: HashMap<String, String>,
}

pub(crate) fn parse_event(line: &str) -> Result<Event, ConnectionError> {
    let mut parts = line.splitn(2, ' ');
    let name = parts.next().unwrap_or_default().to_owned();
    if name == "notify" || !name.starts_with("notify") {
        return Err(ConnectionError::Protocol(format!(
            "invalid notification line: `{line}`"
        )));
    }

    let fields = parts.next().map(parse_fields).transpose()?.unwrap_or_default();

    Ok(Event { name, fields })
}

pub(crate) fn parse_fields(input: &str) -> Result<HashMap<String, String>, ConnectionError> {
    let mut fields = HashMap::new();

    for field in input.split_whitespace() {
        let Some((key, value)) = field.split_once('=') else {
            fields.insert(unescape(field)?, String::new());
            continue;
        };

        fields.insert(unescape(key)?, unescape(value)?);
    }

    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_notification_event() {
        let event =
            parse_event(r"notifycliententerview clid=7 client_nickname=Alice\sSmith").unwrap();

        assert_eq!(event.name, "notifycliententerview");
        assert_eq!(event.fields.get("clid").unwrap(), "7");
        assert_eq!(event.fields.get("client_nickname").unwrap(), "Alice Smith");
    }

    #[test]
    fn rejects_malformed_notification_event() {
        assert!(matches!(
            parse_event(r"notifycliententerview client_nickname=Alice\xSmith"),
            Err(ConnectionError::Escape(_))
        ));
    }
}
