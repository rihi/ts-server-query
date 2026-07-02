use crate::error::QueryError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Command {
    raw: String,
}

impl Command {
    pub fn new(command: impl Into<String>) -> Result<Self, QueryError> {
        let raw = command.into();
        validate_raw(&raw)?;

        Ok(Self { raw })
    }

    pub fn raw(command: impl Into<String>) -> Result<Self, QueryError> {
        Self::new(command)
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }
}

impl TryFrom<String> for Command {
    type Error = QueryError;

    fn try_from(command: String) -> Result<Self, Self::Error> {
        Self::new(command)
    }
}

impl TryFrom<&str> for Command {
    type Error = QueryError;

    fn try_from(command: &str) -> Result<Self, Self::Error> {
        Self::new(command)
    }
}

fn validate_raw(command: &str) -> Result<(), QueryError> {
    if command.contains('\r') || command.contains('\n') {
        return Err(QueryError::InvalidCommand);
    }

    if command.trim().is_empty() {
        return Err(QueryError::InvalidCommand);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_single_line_command() {
        let command = Command::raw("channellist -topic").unwrap();

        assert_eq!(command.as_str(), "channellist -topic");
    }

    #[test]
    fn rejects_empty_command() {
        assert!(matches!(
            Command::raw("   "),
            Err(QueryError::InvalidCommand)
        ));
    }

    #[test]
    fn rejects_multiline_command() {
        assert!(matches!(
            Command::raw("whoami\nquit"),
            Err(QueryError::InvalidCommand)
        ));
    }
}
