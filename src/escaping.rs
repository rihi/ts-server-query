use thiserror::Error;

pub const ESCAPE_CHARACTER: char = '\\';

const ESCAPES: &[EscapeSequence] = &[
    EscapeSequence { plain: '\\', escaped: '\\' },
    EscapeSequence { plain: '/', escaped: '/' },
    EscapeSequence { plain: ' ', escaped: 's' },
    EscapeSequence { plain: '|', escaped: 'p' },
    EscapeSequence { plain: '\u{7}', escaped: 'a' },
    EscapeSequence { plain: '\u{8}', escaped: 'b' },
    EscapeSequence { plain: '\u{c}', escaped: 'f' },
    EscapeSequence { plain: '\n', escaped: 'n' },
    EscapeSequence { plain: '\r', escaped: 'r' },
    EscapeSequence { plain: '\t', escaped: 't' },
    EscapeSequence { plain: '\u{b}', escaped: 'v' },
];

struct EscapeSequence {
    plain: char,
    escaped: char,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum EscapeError {
    #[error("unterminated escape sequence")]
    UnterminatedSequence,

    #[error("unknown escape sequence `\\{0}`")]
    UnknownSequence(char),
}

pub fn is_special_character(ch: char) -> bool {
    ESCAPES.iter().any(|sequence| sequence.plain == ch)
}

pub fn escape(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());

    for ch in input.chars() {
        if let Some(escaped_ch) = escaped_for_plain(ch) {
            escaped.push(ESCAPE_CHARACTER);
            escaped.push(escaped_ch);
        } else {
            escaped.push(ch);
        }
    }

    escaped
}

pub fn unescape(input: &str) -> Result<String, EscapeError> {
    let mut unescaped = String::with_capacity(input.len());
    let mut chars = input.chars();

    while let Some(ch) = chars.next() {
        if ch != ESCAPE_CHARACTER {
            unescaped.push(ch);
            continue;
        }

        let Some(escaped) = chars.next() else {
            return Err(EscapeError::UnterminatedSequence);
        };
        let Some(plain_ch) = plain_for_escaped(escaped) else {
            return Err(EscapeError::UnknownSequence(escaped));
        };
        
        unescaped.push(plain_ch);
    }

    Ok(unescaped)
}

fn escaped_for_plain(ch: char) -> Option<char> {
    ESCAPES
        .iter()
        .find(|escape| escape.plain == ch)
        .map(|escape| escape.escaped)
}

fn plain_for_escaped(ch: char) -> Option<char> {
    ESCAPES
        .iter()
        .find(|escape| escape.escaped == ch)
        .map(|escape| escape.plain)
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
    fn exposes_special_character_list() {
        assert!(is_special_character(' '));
        assert!(is_special_character('|'));
        assert!(!is_special_character('A'));
    }

    #[test]
    fn rejects_unknown_escape_sequence() {
        assert_eq!(
            unescape(r"Alice\xSmith"),
            Err(EscapeError::UnknownSequence('x'))
        );
    }
}
