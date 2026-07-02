use thiserror::Error;

use crate::escaping::{escape, is_special_character};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Command {
    raw: String,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum CommandError {
    #[error("raw ServerQuery command must not contain line breaks")]
    ContainsLineBreak,

    #[error("{kind} must not be empty")]
    EmptyName { kind: &'static str },

    #[error("{kind} `{name}` contains a special character")]
    SpecialCharacter {
        kind: &'static str,
        name: String,
    },
}

impl Command {
    pub fn raw(
        command: impl Into<String>
    ) -> Result<Self, CommandError> {
        let raw = command.into();
        validate_raw(&raw)?;
        Ok(Self { raw })
    }
    
    pub fn new(
        name: impl Into<String>
    ) -> Result<Self, CommandError> {
        let name = name.into();
        validate_name(&name, "command name")?;

        Ok(Self { raw: name })
    }

    pub fn arg(
        mut self,
        name: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Result<Self, CommandError> {
        let name = name.as_ref();
        validate_name(name, "argument name")?;

        self.raw.push(' ');
        self.raw.push_str(name);
        self.raw.push('=');
        self.raw.push_str(&escape(value.as_ref()));

        Ok(self)
    }

    pub fn option(
        mut self,
        name: impl AsRef<str>
    ) -> Result<Self, CommandError> {
        let name = name.as_ref();
        validate_name(name, "option name")?;

        self.raw.push(' ');
        self.raw.push('-');
        self.raw.push_str(name);

        Ok(self)
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }
}

fn validate_raw(command: &str) -> Result<(), CommandError> {
    if command.contains('\r') || command.contains('\n') {
        return Err(CommandError::ContainsLineBreak);
    }

    Ok(())
}

fn validate_name(name: &str, kind: &'static str) -> Result<(), CommandError> {
    if name.is_empty() {
        return Err(CommandError::EmptyName { kind });
    }

    if name.chars().any(is_special_character) {
        return Err(CommandError::SpecialCharacter {
            kind,
            name: name.to_owned(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_accepts_single_line_command() {
        let command = Command::raw("channellist -topic").unwrap();

        assert_eq!(command.as_str(), "channellist -topic");
    }

    #[test]
    fn raw_allows_empty_command() {
        let command = Command::raw("").unwrap();

        assert_eq!(command.as_str(), "");
    }

    #[test]
    fn raw_rejects_multiline_command() {
        assert!(matches!(
            Command::raw("whoami\nquit"),
            Err(CommandError::ContainsLineBreak)
        ));
    }

    #[test]
    fn structured_command_escapes_arg_values() {
        let command = Command::new("clientupdate")
            .unwrap()
            .arg("client_nickname", "Query Bot | Admin")
            .unwrap()
            .option("away")
            .unwrap();

        assert_eq!(
            command.as_str(),
            r"clientupdate client_nickname=Query\sBot\s\p\sAdmin -away"
        );
    }

    #[test]
    fn structured_command_rejects_special_chars_in_names() {
        assert!(matches!(
            Command::new("client update"),
            Err(CommandError::SpecialCharacter { .. })
        ));
        assert!(matches!(
            Command::new("clientupdate").unwrap().arg("bad name", "value"),
            Err(CommandError::SpecialCharacter { .. })
        ));
        assert!(matches!(
            Command::new("clientupdate").unwrap().option("bad|option"),
            Err(CommandError::SpecialCharacter { .. })
        ));
    }
}
