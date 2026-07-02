use std::collections::HashMap;

use crate::error::ConnectionError;
use crate::escaping::unescape;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    pub name: String,
    pub fields: HashMap<String, String>,
}

pub(crate) fn parse_fields(input: &str) -> Result<HashMap<String, String>, ConnectionError> {
    let mut fields = HashMap::new();

    for field in input.split_whitespace() {
        let Some((key, value)) = field.split_once('=') else {
            return Err(ConnectionError::Protocol(format!(
                "field is missing `=`: `{field}`"
            )));
        };

        let key = unescape(key)?;
        if key.is_empty() {
            return Err(ConnectionError::Protocol(
                "field key must not be empty".to_owned(),
            ));
        }

        fields.insert(key, unescape(value)?);
    }

    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_field_without_key_value_separator() {
        assert!(matches!(
            parse_fields("clid=7 malformed"),
            Err(ConnectionError::Protocol(_))
        ));
    }

    #[test]
    fn rejects_field_with_empty_key() {
        assert!(matches!(
            parse_fields("=value"),
            Err(ConnectionError::Protocol(_))
        ));
    }
}
