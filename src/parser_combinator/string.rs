use super::core::{ParseError, Parser, ParserState};

pub fn literal<'a>(expected: String) -> Parser<'a, str, String> {
    Parser::new(move |input: &str| match input.get(0..expected.len()) {
        Some(next) if next == expected => Ok(ParserState {
            index: expected.len(),
            result: expected.clone(),
        }),
        _ => Err(ParseError::new(format!(
            "Could not match literal: \"{}\"",
            expected
        ))),
    })
}

// pub fn literal_<'a>(expected: &'a str) -> impl Parser<str, String> {
//     move |input: &'a str| match input.get(0..expected.len()) {
//         Some(next) if next == expected => Ok(ParserState {
//             index: expected.len(),
//             result: expected.to_string(),
//         }),
//         _ => Err(ParseError::new(format!("Could not match literal: \"{}\"", expected))),
//     }
// }

pub fn character<'a>(c: char) -> Parser<'a, str, String> {
    Parser::new(move |input: &str| match input.chars().next() {
        Some(ch) if ch == c => Ok(ParserState {
            index: 1,
            result: c.to_string(),
        }),
        Some(ch) => Err(ParseError::new(format!("Expected '{}' found '{}'", c, ch))),
        None => Err(ParseError::new("Unexpected end of line".to_string())),
    })
}

pub fn hexadecimal<'a>() -> Parser<'a, str, String> {
    Parser::new(|input: &str| match input.chars().next() {
        Some(c) if c.is_ascii_hexdigit() => Ok(ParserState {
            index: 1,
            result: c,
        }),
        _ => Err(ParseError::new("Not a hex digit".to_string())),
    })
    .one_or_more()
    .map(|v| v.iter().collect())
}

pub fn alphabetic<'a>() -> Parser<'a, str, String> {
    Parser::new(|input: &str| match input.chars().next() {
        Some(c) if c.is_alphabetic() => Ok(ParserState {
            index: 1,
            result: c,
        }),
        _ => Err(ParseError::new("Not an alphabetic character".to_string())),
    })
    .one_or_more()
    .map(|v| v.iter().collect())
}

pub fn upper_or_lower<'a>(s: String) -> Parser<'a, str, String> {
    Parser::one_of(vec![literal(s.to_lowercase()), literal(s.to_uppercase())])
        .map(move |_| s.clone())
}

#[cfg(test)]
mod tests {
    use super::{literal, upper_or_lower, ParseError, ParserState};

    #[test]
    fn literal_parser() {
        let parse_joe = literal(String::from("Hello Joe!"));
        assert_eq!(
            Ok(ParserState {
                index: 10,
                result: String::from("Hello Joe!")
            }),
            parse_joe.parse("Hello Joe!")
        );
        assert_eq!(
            Ok(ParserState {
                index: 10,
                result: String::from("Hello Joe!")
            }),
            parse_joe.parse("Hello Joe! Hello Robert!")
        );
        assert_eq!(
            Err(ParseError::new(String::from(
                "Could not match literal: \"Hello Joe!\""
            ))),
            parse_joe.parse("Hello Mike!")
        );
    }

    #[test]
    fn upper_or_lower_test() {
        let parse_joe = upper_or_lower(String::from("joe!"));
        assert_eq!(
            Ok(ParserState {
                index: 4,
                result: String::from("joe!")
            }),
            parse_joe.parse("JOE!")
        );
        assert_eq!(
            Ok(ParserState {
                index: 4,
                result: String::from("joe!")
            }),
            parse_joe.parse("joe!")
        );
    }

    #[test]
    fn hexadecimal() {
        assert_eq!(
            super::hexadecimal().parse("16afx"),
            Ok(ParserState {
                index: 4,
                result: String::from("16af")
            })
        );
        assert_eq!(
            super::hexadecimal().parse("xxx"),
            Err(ParseError::new("Could not match one or more".to_string()))
        )
    }
}

pub fn optional_whitespace<'a>() -> Parser<'a, str, String> {
    character(' ').zero_or_more().map(|s| s.join(""))
}

pub fn whitespace<'a>() -> Parser<'a, str, String> {
    character(' ').one_or_more().map(|s| s.join(""))
}
