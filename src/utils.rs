use crate::errors::{JabroniError, JabroniResult};

pub fn unquote(string: &str) -> JabroniResult<String> {
    const ALREADY_PARSED_MESSAGE: &str = "Attempted to unquote an already unquoted string";

    if string.len() < 2 {
        return Err(JabroniError::Parse(ALREADY_PARSED_MESSAGE.into()));
    }

    let mut string = string.chars();
    let terminator = string.next().unwrap(); //Safe because we already checked length
    if terminator != '"' && terminator != '\'' {
        return Err(JabroniError::Parse(ALREADY_PARSED_MESSAGE.into()));
    }

    let mut output = String::new();
    let mut backslash = false;

    while let Some(c) = string.next() {
        if c == '\\' && !backslash {
            backslash = true;
            continue;
        }
        if c == terminator && !backslash {
            if string.next().is_some() {
                return Err(JabroniError::Parse(
                    "While parsing string, met terminator before end of string".into(),
                ));
            }
            return Ok(output);
        }

        if backslash {
            if c == 'n' {
                output.push('\n');
            } else if c == 't' {
                output.push('\t');
            } else if c == 'r' {
                output.push('\r');
            } else if c == '\\' || c == '\'' || c == '\"' {
                output.push(c);
            } else {
                return Err(JabroniError::Parse(
                    "Found unknown escaped sequence while parsing string".into(),
                ));
            }
        } else {
            output.push(c);
        }
        backslash = false;
    }

    Err(JabroniError::Parse(
        "String parsing unexpectedly cut short".into(),
    ))
}
