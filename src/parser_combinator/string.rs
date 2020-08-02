use super::core::{map, one_of, one_or_more, Parser, ParserState};

pub fn literal(expected: String) -> impl Parser<str, String> {
    move |input: &str| match input.get(0..expected.len()) {
        Some(next) if next == expected => Ok(ParserState {
            index: expected.len(),
            result: expected.clone(),
        }),
        _ => Err(format!("Could not match literal: \"{}\"", expected)),
    }
}

// pub fn literal_<'a>(expected: &'a str) -> impl Parser<str, String> {
//     move |input: &'a str| match input.get(0..expected.len()) {
//         Some(next) if next == expected => Ok(ParserState {
//             index: expected.len(),
//             result: expected.to_string(),
//         }),
//         _ => Err(format!("Could not match literal: \"{}\"", expected)),
//     }
// }

pub fn character(c: char) -> impl Parser<str, String> {
    move |input: &str| match input.chars().next() {
        Some(ch) if ch == c => Ok(ParserState {
            index: 1,
            result: c.to_string(),
        }),
        Some(ch) => Err(format!("Expected {} found {}", c, ch)),
        None => Err("Unexpected end of line".to_string()),
    }
}

pub fn hexadecimal() -> impl Parser<str, String> {
    map(
        one_or_more(move |input: &str| match input.chars().next() {
            Some(c) if c.is_ascii_hexdigit() => Ok(ParserState {
                index: 1,
                result: c,
            }),
            _ => Err("Not a hex digit".to_string()),
        }),
        |v| v.iter().collect(),
    )
}

pub fn upper_or_lower(s: String) -> impl Parser<str, String> {
    map(
        one_of(vec![literal(s.to_lowercase()), literal(s.to_uppercase())]),
        move |_| s.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::{literal, upper_or_lower, Parser, ParserState};

    #[test]
    fn literal_parser() {
        let parse_joe = literal(String::from("Hello Joe!"));
        assert_eq!(
            Ok(ParserState {
                index: 10,
                result: String::from("Hello Joe!")
            }),
            parse_joe.parse(&String::from("Hello Joe!"))
        );
        let string = String::from("Hello Joe! Hello Robert!");
        assert_eq!(
            Ok(ParserState {
                index: 10,
                result: String::from("Hello Joe!")
            }),
            parse_joe.parse(&string)
        );
        assert_eq!(
            Err(String::from("Could not match literal: \"Hello Joe!\"")),
            parse_joe.parse(&String::from("Hello Mike!"))
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
            parse_joe.parse(&String::from("JOE!"))
        );
        assert_eq!(
            Ok(ParserState {
                index: 4,
                result: String::from("joe!")
            }),
            parse_joe.parse(&String::from("joe!"))
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
            Err("Could not match one or more at 0".to_string())
        )
    }
}
