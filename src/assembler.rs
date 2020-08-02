use crate::parser_combinator::core;
use crate::parser_combinator::string;

fn move_lit_to_reg<'a>() -> impl core::Parser<str, Typed<'a, Instruction2<'a>>> {
    core::map(
        core::sequence_of(vec![
            Box::new(to_typed("", string::literal("mov".to_string()))),
            Box::new(to_typed("", whitespace())),
            Box::new(parse_hex_literal()),
            Box::new(to_typed("", whitespace())),
            Box::new(parse_register()),
            Box::new(to_typed("", optional_whitespace())),
        ]),
        |res| {
            as_typed(
                "Instruction",
                Instruction2 {
                    instruction: "MOVE_LIT_REG",
                    args: (res[2].value.to_owned(), res[4].value.to_owned()),
                },
            )
        },
    )
}

fn optional_whitespace<'a>() -> impl core::Parser<str, String> {
    core::map(core::zero_or_more(string::character(' ')), |s| s.join(""))
}

fn whitespace<'a>() -> impl core::Parser<str, String> {
    core::map(core::one_or_more(string::character(' ')), |s| s.join(""))
}

fn parse_register<'a>() -> impl core::Parser<str, Typed<'a, String>> {
    core::map(
        core::one_of(vec![
            string::literal(String::from("IP")),
            string::literal(String::from("ACC")),
            string::literal(String::from("R1")),
            string::literal(String::from("R2")),
            string::literal(String::from("R3")),
            string::literal(String::from("R4")),
            string::literal(String::from("R5")),
            string::literal(String::from("R6")),
            string::literal(String::from("R7")),
            string::literal(String::from("R8")),
            string::literal(String::from("SP")),
            string::literal(String::from("FP")),
        ]),
        |reg| as_typed("Register", reg),
    )
}

fn parse_hex_literal<'a>() -> impl core::Parser<str, Typed<'a, String>> {
    core::map(
        core::sequence_of(vec![
            Box::new(string::character('$')),
            Box::new(string::hexadecimal()),
        ]),
        |s| as_typed("Hex Literal", s.join("")),
    )
}

#[derive(Eq, PartialEq, Debug)]
struct Instruction2<'a> {
    instruction: &'a str,
    args: (String, String),
}

#[derive(Eq, PartialEq, Debug)]
struct Typed<'a, T> {
    assembly_type: &'a str,
    value: T,
}

fn as_typed<T>(assembly_type: &str, value: T) -> Typed<T> {
    Typed {
        assembly_type,
        value,
    }
}

fn to_typed<P, Input, Output>(
    assembly_type: &str,
    parser: P,
) -> impl core::Parser<Input, Typed<Output>>
where
    P: core::Parser<Input, Output>,
    Input: core::ParseInput + ?Sized,
{
    core::map(parser, move |res| as_typed(assembly_type, res))
}

#[cfg(test)]
mod tests {
    use crate::parser_combinator::core::{Parser, ParserState};

    #[test]
    fn parse_register() {
        assert_eq!(
            super::parse_register().parse("R1"),
            Ok(ParserState {
                index: 2,
                result: super::Typed {
                    assembly_type: "Register",
                    value: String::from("R1")
                }
            })
        )
    }

    #[test]
    fn parse_hex_literal() {
        assert_eq!(
            super::parse_hex_literal().parse("$aa12"),
            Ok(ParserState {
                index: 5,
                result: super::Typed {
                    assembly_type: "Hex Literal",
                    value: String::from("$aa12")
                }
            })
        )
    }

    #[test]
    fn move_lit_reg() {
        assert_eq!(
            super::move_lit_to_reg().parse("mov $aa12 R1"),
            Ok(ParserState {
                index: 12,
                result: super::Typed {
                    assembly_type: "Instruction",
                    value: super::Instruction2 {
                        instruction: "MOVE_LIT_REG",
                        args: ("$aa12".to_string(), "R1".to_string())
                    }
                }
            })
        )
    }
}