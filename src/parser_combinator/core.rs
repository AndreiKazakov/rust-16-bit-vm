use std::ops::{Index, Range};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParserState<T> {
    pub index: usize,
    pub result: T,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub index: usize,
}
impl ParseError {
    pub fn new(message: String) -> ParseError {
        ParseError { message, index: 0 }
    }
}
pub type ParseResult<Output> = Result<ParserState<Output>, ParseError>;

pub trait ParseInput: Index<Range<usize>, Output = Self> {
    fn get_from(&self, i: usize) -> Option<&Self>;
}
impl ParseInput for str {
    fn get_from(&self, i: usize) -> Option<&Self> {
        self.get(i..)
    }
}
impl<T> ParseInput for [T] {
    fn get_from(&self, i: usize) -> Option<&Self> {
        self.get(i..)
    }
}

pub struct Parser<'a, Input: ?Sized + ParseInput, Output: 'a> {
    fun: Box<dyn Fn(&'a Input) -> ParseResult<Output> + 'a>,
}

impl<'a, I: ?Sized + ParseInput, O> Parser<'a, I, O> {
    pub fn new<F>(fun: F) -> Parser<'a, I, O>
    where
        F: Fn(&'a I) -> ParseResult<O> + 'a,
    {
        Parser { fun: Box::new(fun) }
    }

    pub fn parse(&self, slice: &'a I) -> ParseResult<O> {
        (self.fun)(slice)
    }

    pub fn parse_at(&self, input: &'a I, index: usize) -> ParseResult<O> {
        input
            .get_from(index)
            .ok_or_else(|| ParseError::new(String::from("End of line")))
            .and_then(|x: &I| self.parse(x))
            .map(|state| ParserState {
                index: state.index + index,
                ..state
            })
            .map_err(|err| ParseError {
                message: err.message,
                index: index + err.index,
            })
    }

    pub fn map<F, B>(self, map_fn: F) -> Parser<'a, I, B>
    where
        F: Fn(O) -> B + 'a,
    {
        Parser::new(move |input: &'a I| {
            self.parse(input).map(|state| ParserState {
                result: map_fn(state.result),
                index: state.index,
            })
        })
    }

    pub fn map_err<F>(self, err_map_fn: F) -> Parser<'a, I, O>
    where
        F: Fn(ParseError) -> ParseError + 'a,
    {
        Parser::new(move |input| self.parse(input).map_err(|err| err_map_fn(err)))
    }

    pub fn and_then<F, B>(self, chain_fn: F) -> Parser<'a, I, B>
    where
        F: Fn(ParserState<O>) -> ParseResult<B> + 'a,
    {
        Parser::new(move |input| self.parse(input).and_then(|state| chain_fn(state)))
    }

    pub fn zero_or_more(self) -> Parser<'a, I, Vec<O>> {
        Parser::new(move |input| {
            let mut result = Vec::new();
            let mut index = 0;

            while let Ok(state) = self.parse_at(input, index) {
                result.push(state.result);
                index = state.index;
            }
            Ok(ParserState { result, index })
        })
    }

    pub fn one_or_more(self) -> Parser<'a, I, Vec<O>> {
        self.zero_or_more().and_then(|state| {
            if state.result.is_empty() {
                Err(ParseError::new(String::from("Could not match one or more")))
            } else {
                Ok(state)
            }
        })
    }

    pub fn left<B>(self, b: Parser<'a, I, B>) -> Parser<'a, I, O> {
        Parser::new(move |input| {
            let a_res = self.parse(input)?;
            let b_res = b.parse_at(input, a_res.index)?;
            Ok(ParserState {
                index: b_res.index,
                ..a_res
            })
        })
    }

    pub fn right<B>(self, b: Parser<'a, I, B>) -> Parser<'a, I, B> {
        Parser::new(move |input| {
            let a_res = self.parse(input)?;
            b.parse_at(input, a_res.index)
        })
    }

    pub fn sequence_of(parsers: Vec<Parser<I, O>>) -> Parser<I, Vec<O>> {
        Parser::new(move |input| {
            let mut i = 0;
            let mut results = Vec::with_capacity(parsers.len());

            for p in parsers.iter() {
                match p.parse_at(&input, i) {
                    Err(err) => return Err(err),
                    Ok(ParserState { index, result }) => {
                        results.push(result);
                        i = index;
                    }
                }
            }

            Ok(ParserState {
                result: results,
                index: i,
            })
        })
    }

    pub fn interspersed<S>(
        separator: Parser<'a, I, S>,
        parsers: Vec<Parser<'a, I, O>>,
    ) -> Parser<'a, I, Vec<O>> {
        Parser::new(move |input| {
            let mut i = 0;
            let mut results = Vec::with_capacity(parsers.len());

            for (parser_index, p) in parsers.iter().enumerate() {
                match p.parse_at(&input, i) {
                    Err(err) => return Err(err),
                    Ok(ParserState { index, result }) => {
                        results.push(result);
                        i = index;
                    }
                }
                if parser_index != parsers.len() - 1 {
                    match separator.parse_at(&input, i) {
                        Err(err) => return Err(err),
                        Ok(ParserState { index, result: _ }) => i = index,
                    }
                }
            }

            Ok(ParserState {
                result: results,
                index: i,
            })
        })
    }

    pub fn between<B, A>(
        before: Parser<'a, I, B>,
        parser: Parser<'a, I, O>,
        after: Parser<'a, I, A>,
    ) -> Parser<'a, I, O> {
        Parser::new(move |input| {
            let s1 = before.parse(input)?;
            let result = parser.parse_at(input, s1.index)?;
            let s2 = after.parse_at(input, result.index)?;
            Ok(ParserState {
                index: s2.index,
                ..result
            })
        })
    }

    pub fn one_of(parsers: Vec<Parser<I, O>>) -> Parser<I, O> {
        Parser::new(move |input| {
            let mut errors = Vec::with_capacity(parsers.len());
            for p in parsers.iter() {
                match p.parse(&input) {
                    Err(err) => errors.push(err),
                    result @ Ok(_) => return result,
                }
            }
            Err(ParseError::new(
                format!(
                    "Could not match any parsers:\n{}",
                    errors
                        .iter()
                        .map(|err| format!("\t{}\n", err.message))
                        .collect::<String>(),
                )
                .to_string(),
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{ParseError, ParseResult, Parser, ParserState};

    fn parse_char<'a>(ch: char) -> Parser<'a, str, char> {
        Parser::new(move |input: &str| match input.chars().next() {
            Some(c) if c == ch => Ok(ParserState {
                index: 1,
                result: ch,
            }),
            _ => Err(ParseError::new(String::from("nope"))),
        })
    }

    #[test]
    fn parse() {
        assert_eq!(
            parse_char('a').parse("aaa"),
            Ok(ParserState {
                index: 1,
                result: 'a'
            })
        )
    }

    #[test]
    fn parse_at() {
        assert_eq!(
            parse_char('c').parse_at("aaac", 3),
            Ok(ParserState {
                index: 4,
                result: 'c'
            })
        )
    }

    #[test]
    fn map() {
        assert_eq!(
            parse_char('a').map(|_| "bbb").parse("azzz"),
            Ok(ParserState {
                index: 1,
                result: "bbb"
            })
        )
    }

    #[test]
    fn map_err() {
        assert_eq!(
            parse_char('a')
                .map_err(|err| ParseError::new(err.message.replace("no", "yu")))
                .parse("zzz"),
            Err(ParseError::new("yupe".to_string()))
        )
    }

    #[test]
    fn chain() {
        assert_eq!(
            parse_char('a')
                .and_then(|state| Ok(ParserState {
                    index: state.index,
                    result: "bbb"
                }))
                .parse("azzz"),
            Ok(ParserState {
                index: 1,
                result: "bbb"
            })
        );
        assert_eq!(
            parse_char('a')
                .and_then(|state| Ok(ParserState {
                    index: state.index,
                    result: "bbb"
                }))
                .parse("zzz"),
            Err(ParseError::new("nope".to_string()))
        );
        assert_eq!(
            parse_char('a')
                .and_then(|_| -> ParseResult<i8> { Err(ParseError::new(String::from("aaaaa"))) })
                .parse("azzz"),
            Err(ParseError::new(String::from("aaaaa")))
        );
    }

    #[test]
    fn zero_or_more() {
        assert_eq!(
            parse_char('a').zero_or_more().parse("aaabbb"),
            Ok(ParserState {
                index: 3,
                result: vec!['a', 'a', 'a']
            })
        );
        assert_eq!(
            parse_char('a').zero_or_more().parse("bbb"),
            Ok(ParserState {
                index: 0,
                result: vec![]
            })
        );
    }

    #[test]
    fn one_or_more() {
        assert_eq!(
            parse_char('a').one_or_more().parse("aaabbb"),
            Ok(ParserState {
                index: 3,
                result: vec!['a', 'a', 'a']
            })
        );
        assert_eq!(
            parse_char('a').one_or_more().parse("bbb"),
            Err(ParseError::new(String::from("Could not match one or more"),))
        );
    }

    #[test]
    fn sequence_of() {
        assert_eq!(
            Parser::sequence_of(vec![parse_char('a'), parse_char('b'), parse_char('c'),])
                .parse("abcd"),
            Ok(ParserState {
                index: 3,
                result: vec!['a', 'b', 'c']
            })
        )
    }

    #[test]
    fn interspersed() {
        assert_eq!(
            Parser::interspersed(
                parse_char(' '),
                vec![parse_char('a'), parse_char('b'), parse_char('c'),]
            )
            .parse("a b cd"),
            Ok(ParserState {
                index: 5,
                result: vec!['a', 'b', 'c']
            })
        )
    }

    #[test]
    fn between() {
        assert_eq!(
            Parser::between(parse_char(' '), parse_char('a'), parse_char(' '),).parse(" a "),
            Ok(ParserState {
                index: 3,
                result: 'a'
            })
        )
    }

    #[test]
    fn left() {
        assert_eq!(
            parse_char('a').left(parse_char('b')).parse("ab"),
            Ok(ParserState {
                index: 2,
                result: 'a'
            })
        )
    }

    #[test]
    fn right() {
        assert_eq!(
            parse_char('a').right(parse_char('b')).parse("ab"),
            Ok(ParserState {
                index: 2,
                result: 'b'
            })
        )
    }

    #[test]
    fn one_of() {
        let vec1 = vec![parse_char('a'), parse_char('b'), parse_char('c')];
        assert_eq!(
            Parser::one_of(vec1).parse("bzz"),
            Ok(ParserState {
                index: 1,
                result: 'b'
            })
        )
    }
}
