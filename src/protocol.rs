use std::collections::HashMap;

use crate::error::{QueryError, ServerError};
use crate::response::Response;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    pub name: String,
    pub fields: HashMap<String, String>,
}

pub(crate) fn finish_response(
    current_response: &mut Vec<String>,
    error_line: &str,
) -> Result<Response, QueryError> {
    let fields = parse_fields(error_line.strip_prefix("error ").unwrap_or(error_line))?;
    let id = required_u32(&fields, "id")?;
    let message = fields.get("msg").cloned().unwrap_or_default();

    if id == 0 {
        Ok(Response::new(std::mem::take(current_response), fields))
    } else {
        current_response.clear();
        Err(QueryError::Server(ServerError {
            id,
            message,
            fields,
        }))
    }
}

pub(crate) fn parse_event(line: &str) -> Result<Event, QueryError> {
    let mut parts = line.splitn(2, ' ');
    let name = parts.next().unwrap_or_default().to_owned();
    if name == "notify" || !name.starts_with("notify") {
        return Err(QueryError::Protocol(format!(
            "invalid notification line: `{line}`"
        )));
    }

    let fields = parts.next().map(parse_fields).transpose()?.unwrap_or_default();

    Ok(Event { name, fields })
}

pub(crate) fn parse_fields(input: &str) -> Result<HashMap<String, String>, QueryError> {
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

pub(crate) fn required_string(
    fields: &HashMap<String, String>,
    key: &str,
) -> Result<String, QueryError> {
    fields
        .get(key)
        .cloned()
        .ok_or_else(|| QueryError::Protocol(format!("missing required field `{key}`")))
}

pub(crate) fn required_u64(
    fields: &HashMap<String, String>,
    key: &str,
) -> Result<u64, QueryError> {
    required_string(fields, key)?
        .parse()
        .map_err(|_| QueryError::Protocol(format!("invalid integer field `{key}`")))
}

fn required_u32(fields: &HashMap<String, String>, key: &str) -> Result<u32, QueryError> {
    required_string(fields, key)?
        .parse()
        .map_err(|_| QueryError::Protocol(format!("invalid integer field `{key}`")))
}

pub(crate) fn bool_field(fields: &HashMap<String, String>, key: &str) -> bool {
    fields.get(key).is_some_and(|value| value == "1")
}

pub fn escape(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());

    for ch in input.chars() {
        match ch {
            '\\' => escaped.push_str(r"\\"),
            '/' => escaped.push_str(r"\/"),
            ' ' => escaped.push_str(r"\s"),
            '|' => escaped.push_str(r"\p"),
            '\u{7}' => escaped.push_str(r"\a"),
            '\u{8}' => escaped.push_str(r"\b"),
            '\u{c}' => escaped.push_str(r"\f"),
            '\n' => escaped.push_str(r"\n"),
            '\r' => escaped.push_str(r"\r"),
            '\t' => escaped.push_str(r"\t"),
            '\u{b}' => escaped.push_str(r"\v"),
            _ => escaped.push(ch),
        }
    }

    escaped
}

pub fn unescape(input: &str) -> Result<String, QueryError> {
    let mut unescaped = String::with_capacity(input.len());
    let mut chars = input.chars();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            unescaped.push(ch);
            continue;
        }

        let Some(escaped) = chars.next() else {
            return Err(QueryError::Protocol("unterminated escape sequence".to_owned()));
        };

        match escaped {
            '\\' => unescaped.push('\\'),
            '/' => unescaped.push('/'),
            's' => unescaped.push(' '),
            'p' => unescaped.push('|'),
            'a' => unescaped.push('\u{7}'),
            'b' => unescaped.push('\u{8}'),
            'f' => unescaped.push('\u{c}'),
            'n' => unescaped.push('\n'),
            'r' => unescaped.push('\r'),
            't' => unescaped.push('\t'),
            'v' => unescaped.push('\u{b}'),
            other => {
                return Err(QueryError::Protocol(format!(
                    "unknown escape sequence `\\{other}`"
                )));
            }
        }
    }

    Ok(unescaped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_and_unescapes_server_query_values() {
        let value = "Lobby / Alpha | beta\nnext";
        let escaped = escape(value);

        assert_eq!(escaped, r"Lobby\s\/\sAlpha\s\p\sbeta\nnext");
        assert_eq!(unescape(&escaped).unwrap(), value);
    }

    #[test]
    fn parses_successful_response() {
        let mut lines = vec!["clid=1 client_nickname=serveradmin".to_owned()];
        let response = finish_response(&mut lines, "error id=0 msg=ok").unwrap();

        assert_eq!(response.lines(), ["clid=1 client_nickname=serveradmin"]);
        assert_eq!(response.metadata().get("msg").unwrap(), "ok");
    }

    #[test]
    fn parses_server_error() {
        let mut lines = Vec::new();
        let error =
            finish_response(&mut lines, r"error id=2568 msg=insufficient\sclient\spermissions")
                .unwrap_err();

        assert!(matches!(
            error,
            QueryError::Server(ServerError {
                id: 2568,
                message,
                ..
            }) if message == "insufficient client permissions"
        ));
    }

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
            Err(QueryError::Protocol(_))
        ));
    }
}
