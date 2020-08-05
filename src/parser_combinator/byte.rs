use super::core::{ParseError, Parser, ParserState};

fn match_literal(expected: &[u8]) -> Parser<[u8], ()> {
    Parser::new(move |input: &[u8]| match input.get(0..expected.len()) {
        Some(next) if next == expected => Ok(ParserState {
            index: expected.len(),
            result: (),
        }),
        _ => Err(ParseError::new(format!(
            "Could not match literal: {:?}",
            expected
        ))),
    })
}

#[test]
fn literal_parser() {
    // vec![1,23].iter().map()
    let parse_joe = match_literal(b"Hello Joe!");
    assert_eq!(
        parse_joe.parse(b"Hello Joe!"),
        Ok(ParserState {
            index: 10,
            result: ()
        }),
    );
    assert_eq!(
        parse_joe.parse(b"Hello Joe! Hello Robert!"),
        Ok(ParserState {
            index: 10,
            result: ()
        }),
    );
    assert_eq!(
        parse_joe.parse(b"Hello Mike!"),
        Err(ParseError::new(String::from(
            "Could not match literal: [72, 101, 108, 108, 111, 32, 74, 111, 101, 33]"
        ),)),
    );
}
