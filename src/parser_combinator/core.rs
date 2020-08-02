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

pub trait Parser<Input, Output>
where
    Input: ?Sized + ParseInput,
{
    fn parse_slice(&self, slice: &Input) -> ParseResult<Output>;
    fn parse(&self, input: &Input) -> ParseResult<Output> {
        self.parse_at(input, 0)
    }
    fn parse_at(&self, input: &Input, index: usize) -> ParseResult<Output> {
        input
            .get_from(index)
            .ok_or_else(|| String::from("End of line"))
            .and_then(|x: &Input| self.parse_slice(x))
            .map(|state| ParserState {
                index: state.index + index,
                ..state
            })
    }
}
impl<F, Input, Output> Parser<Input, Output> for F
where
    F: Fn(&Input) -> ParseResult<Output>,
    Input: ?Sized + ParseInput,
{
    fn parse_slice(&self, slice: &Input) -> ParseResult<Output> {
        self(slice)
    }
}

pub fn map<P, F, Input, A, B>(parser: P, map_fn: F) -> impl Parser<Input, B>
where
    P: Parser<Input, A>,
    F: Fn(A) -> B,
    Input: ?Sized + ParseInput,
{
    move |input: &Input| {
        parser.parse_slice(input).map(|state| ParserState {
            result: map_fn(state.result),
            index: state.index,
        })
    }
}

fn map_err<P, F, Input, Output>(parser: P, err_map_fn: F) -> impl Parser<Input, Output>
where
    P: Parser<Input, Output>,
    F: Fn(ParseError) -> ParseError,
    Input: ?Sized + ParseInput,
{
    move |input: &Input| parser.parse_slice(input).map_err(|err| err_map_fn(err))
}

fn and_then<P, F, Input, A, B>(parser: P, chain_fn: F) -> impl Parser<Input, B>
where
    P: Parser<Input, A>,
    F: Fn(ParserState<A>) -> ParseResult<B>,
    Input: ?Sized + ParseInput,
{
    move |input: &Input| parser.parse_slice(input).and_then(|state| chain_fn(state))
}

pub fn zero_or_more<P, Input, Output>(parser: P) -> impl Parser<Input, Vec<Output>>
where
    P: Parser<Input, Output>,
    Input: ?Sized + ParseInput,
{
    move |input: &Input| {
        let mut result = Vec::new();
        let mut index = 0;

        while let Ok(state) = parser.parse_at(input, index) {
            result.push(state.result);
            index = state.index;
        }
        Ok(ParserState { result, index })
    }
}

pub fn one_or_more<P, Input, Output>(parser: P) -> impl Parser<Input, Vec<Output>>
where
    P: Parser<Input, Output>,
    Input: ?Sized + ParseInput,
{
    and_then(zero_or_more(parser), |state| {
        if state.result.is_empty() {
            Err(format!("Could not match one or more at {}", state.index))
        } else {
            Ok(state)
        }
    })
}

pub fn sequence_of<Input, Output>(
    parsers: Vec<Box<dyn Parser<Input, Output>>>,
) -> impl Parser<Input, Vec<Output>>
where
    Input: ?Sized + ParseInput,
{
    move |input: &Input| {
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
    }
}

pub fn one_of<P, Input, Output>(parsers: Vec<P>) -> impl Parser<Input, Output>
where
    P: Parser<Input, Output>,
    Input: ?Sized + ParseInput,
{
    move |input: &Input| {
        for p in parsers.iter() {
            match p.parse(&input) {
                Err(_) => continue,
                result @ Ok(_) => return result,
            }
        }
        Err(String::from("Could not match any parsers"))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        and_then, map, map_err, one_of, one_or_more, sequence_of, zero_or_more, ParseResult,
        Parser, ParserState,
    };

    fn parse_char(ch: char) -> impl Parser<str, ()> {
        move |input: &str| match input.chars().next() {
            Some(c) if c == ch => Ok(ParserState {
                index: 1,
                result: (),
            }),
            _ => Err(String::from("nope")),
        }
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
            map(parse_char('a'), |_| "bbb").parse("azzz"),
            Ok(ParserState {
                index: 1,
                result: "bbb"
            })
        )
    }

    #[test]
    fn test_map_err() {
        assert_eq!(
            map_err(parse_char('a'), |string: String| string.replace("no", "yu")).parse("zzz"),
            Err("yupe".to_string())
        )
    }

    #[test]
    fn test_chain() {
        assert_eq!(
            and_then(parse_char('a'), |state| Ok(ParserState {
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
            and_then(parse_char('a'), |state| Ok(ParserState {
                index: state.index,
                result: "bbb"
            }))
            .parse("zzz"),
            Err("nope".to_string())
        );
        assert_eq!(
            and_then(parse_char('a'), |_| -> ParseResult<i8> {
                Err(String::from("aaaaa"))
            })
            .parse("azzz"),
            Err(String::from("aaaaa"))
        );
    }

    #[test]
    fn test_zero_or_more() {
        assert_eq!(
            zero_or_more(parse_char('a')).parse("aaabbb"),
            Ok(ParserState {
                index: 3,
                result: vec![(), (), ()]
            })
        );
        assert_eq!(
            zero_or_more(parse_char('a')).parse("bbb"),
            Ok(ParserState {
                index: 0,
                result: vec![]
            })
        );
    }

    #[test]
    fn test_one_or_more() {
        assert_eq!(
            one_or_more(parse_char('a')).parse("aaabbb"),
            Ok(ParserState {
                index: 3,
                result: vec![(), (), ()]
            })
        );
        assert_eq!(
            one_or_more(parse_char('a')).parse("bbb"),
            Err(String::from("Could not match one or more at 0"))
        );
    }

    #[test]
    fn test_sequence_of() {
        assert_eq!(
            sequence_of(vec![
                Box::new(parse_char('a')),
                Box::new(parse_char('b')),
                Box::new(parse_char('c')),
            ])
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
            one_of(vec1).parse("bzz"),
            Ok(ParserState {
                index: 1,
                result: ()
            })
        )
    }
}
