use std::ops::{Index, RangeFrom};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParserState<T> {
    pub index: usize,
    pub result: T,
}

pub type ParseError = String;
pub type ParseResult<Output> = Result<ParserState<Output>, ParseError>;

pub trait ParseInput: Index<RangeFrom<usize>, Output = Self> {
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

impl<'a, Input: ?Sized + ParseInput, Output> Parser<'a, Input, Output> {
    pub fn new<F>(fun: F) -> Parser<'a, Input, Output>
    where
        F: Fn(&'a Input) -> ParseResult<Output> + 'a,
    {
        Parser { fun: Box::new(fun) }
    }

    pub fn parse(&self, slice: &'a Input) -> ParseResult<Output> {
        (self.fun)(slice)
    }

    fn parse_at(&self, input: &'a Input, index: usize) -> ParseResult<Output> {
        input
            .get_from(index)
            .ok_or_else(|| String::from("End of line"))
            .and_then(|x: &Input| self.parse(x))
            .map(|state| ParserState {
                index: state.index + index,
                ..state
            })
    }

    pub fn map<F, B>(self, map_fn: F) -> Parser<'a, Input, B>
    where
        F: Fn(Output) -> B + 'a,
    {
        Parser::new(move |input: &'a Input| {
            self.parse(input).map(|state| ParserState {
                result: map_fn(state.result),
                index: state.index,
            })
        })
    }

    pub fn map_err<F>(self, err_map_fn: F) -> Parser<'a, Input, Output>
    where
        F: Fn(ParseError) -> ParseError + 'a,
    {
        Parser::new(move |input| self.parse(input).map_err(|err| err_map_fn(err)))
    }

    pub fn and_then<F, B>(self, chain_fn: F) -> Parser<'a, Input, B>
    where
        F: Fn(ParserState<Output>) -> ParseResult<B> + 'a,
    {
        Parser::new(move |input| self.parse(input).and_then(|state| chain_fn(state)))
    }

    pub fn zero_or_more(self) -> Parser<'a, Input, Vec<Output>> {
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

    pub fn one_or_more(self) -> Parser<'a, Input, Vec<Output>> {
        self.zero_or_more().and_then(|state| {
            if state.result.is_empty() {
                Err(format!("Could not match one or more at {}", state.index))
            } else {
                Ok(state)
            }
        })
    }

    pub fn sequence_of(parsers: Vec<Parser<Input, Output>>) -> Parser<Input, Vec<Output>> {
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

    pub fn one_of(parsers: Vec<Parser<Input, Output>>) -> Parser<Input, Output> {
        Parser::new(move |input| {
            for p in parsers.iter() {
                match p.parse(&input) {
                    Err(_) => continue,
                    result @ Ok(_) => return result,
                }
            }
            Err(String::from("Could not match any parsers"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{ParseResult, Parser, ParserState};

    fn parse_char<'a>(ch: char) -> Parser<'a, str, ()> {
        Parser::new(move |input: &str| match input.chars().next() {
            Some(c) if c == ch => Ok(ParserState {
                index: 1,
                result: (),
            }),
            _ => Err(String::from("nope")),
        })
    }

    #[test]
    fn test_parse() {
        assert_eq!(
            parse_char('a').parse("aaa"),
            Ok(ParserState {
                index: 1,
                result: ()
            })
        )
    }

    #[test]
    fn test_parse_at() {
        assert_eq!(
            parse_char('c').parse_at("aaac", 3),
            Ok(ParserState {
                index: 4,
                result: ()
            })
        )
    }

    #[test]
    fn test_map() {
        assert_eq!(
            parse_char('a').map(|_| "bbb").parse("azzz"),
            Ok(ParserState {
                index: 1,
                result: "bbb"
            })
        )
    }

    #[test]
    fn test_map_err() {
        assert_eq!(
            parse_char('a')
                .map_err(|string: String| string.replace("no", "yu"))
                .parse("zzz"),
            Err("yupe".to_string())
        )
    }

    #[test]
    fn test_chain() {
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
            Err("nope".to_string())
        );
        assert_eq!(
            parse_char('a')
                .and_then(|_| -> ParseResult<i8> { Err(String::from("aaaaa")) })
                .parse("azzz"),
            Err(String::from("aaaaa"))
        );
    }

    #[test]
    fn test_zero_or_more() {
        assert_eq!(
            parse_char('a').zero_or_more().parse("aaabbb"),
            Ok(ParserState {
                index: 3,
                result: vec![(), (), ()]
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
    fn test_one_or_more() {
        assert_eq!(
            parse_char('a').one_or_more().parse("aaabbb"),
            Ok(ParserState {
                index: 3,
                result: vec![(), (), ()]
            })
        );
        assert_eq!(
            parse_char('a').one_or_more().parse("bbb"),
            Err(String::from("Could not match one or more at 0"))
        );
    }

    #[test]
    fn test_sequence_of() {
        assert_eq!(
            Parser::sequence_of(vec![parse_char('a'), parse_char('b'), parse_char('c'),])
                .parse("abcd"),
            Ok(ParserState {
                index: 3,
                result: vec![(), (), ()]
            })
        )
    }

    #[test]
    fn test_one_of() {
        let vec1 = vec![parse_char('a'), parse_char('b'), parse_char('c')];
        assert_eq!(
            Parser::one_of(vec1).parse("bzz"),
            Ok(ParserState {
                index: 1,
                result: ()
            })
        )
    }
}
