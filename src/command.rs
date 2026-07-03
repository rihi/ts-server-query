use thiserror::Error;

use crate::escaping::{escape, is_special_character};

#[derive(Clone, Debug, Eq, PartialEq)]
/// ServerQuery command text.
///
/// Use [`Command::new`] for structured commands with escaped argument values,
/// or [`Command::raw`] when you already have a complete single-line command.
pub struct Command {
    raw: String,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
/// Error returned when command construction would produce invalid ServerQuery
/// command text.
pub enum CommandError {
    /// Raw command text contains `\r` or `\n`.
    #[error("raw ServerQuery command must not contain line breaks")]
    ContainsLineBreak,

    /// A command, argument, or option name is empty.
    #[error("{kind} must not be empty")]
    EmptyName { kind: &'static str },

    /// A command, argument, or option name contains a character that must be
    /// escaped in ServerQuery values.
    #[error("{kind} `{name}` contains a special character")]
    SpecialCharacter {
        kind: &'static str,
        name: String,
    },
}

impl Command {
    /// Creates a command from complete raw ServerQuery text.
    ///
    /// This does not escape or validate command syntax beyond rejecting line
    /// breaks. Prefer [`Command::new`] unless callers provide the entire command
    /// text themselves.
    pub fn raw(
        command: impl Into<String>
    ) -> Result<Self, CommandError> {
        let raw = command.into();
        validate_raw(&raw)?;
        Ok(Self { raw })
    }
    
    /// Starts a structured command with the given command name.
    ///
    /// The name must not be empty and must not contain ServerQuery special
    /// characters such as spaces or `|`.
    pub fn new(
        name: impl Into<String>
    ) -> Result<Self, CommandError> {
        let name = name.into();
        validate_name(&name, "command name")?;

        Ok(Self { raw: name })
    }

    /// Adds an escaped `name=value` argument.
    ///
    /// The argument name is validated as plain ServerQuery identifier text. The
    /// value is escaped with [`crate::escape`].
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

    /// Adds an option in `-name` form.
    ///
    /// The option name must not include the leading `-`.
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

    /// Returns the encoded command line without a trailing newline.
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
