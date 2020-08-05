use crate::parser_combinator::core::Parser;
use crate::parser_combinator::string;
mod typed;
use typed::{IntoTyped, Type, Typed};
mod instruction;
use instruction::{Instruction, Instruction2};

fn move_lit_to_reg<'a>() -> Parser<'a, str, Typed<Instruction2>> {
    Parser::sequence_of(vec![
        string::literal("mov".to_string()).into_typed(Type::Ignored),
        string::whitespace().into_typed(Type::Ignored),
        parse_hex_literal(),
        string::whitespace().into_typed(Type::Ignored),
        parse_register(),
        string::optional_whitespace().into_typed(Type::Ignored),
    ])
    .map(|mut res| {
        let second = res.remove(4);
        let first = res.remove(2);
        Typed::new(
            Type::Instruction,
            Instruction2 {
                instruction: Instruction::MoveLitReg,
                args: (first, second),
            },
        )
    })
}

fn parse_register<'a>() -> Parser<'a, str, Typed<String>> {
    Parser::one_of(vec![
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
    ])
    .map(|reg| Typed::new(Type::Register, reg))
}

fn parse_hex_literal<'a>() -> Parser<'a, str, Typed<String>> {
    Parser::sequence_of(vec![string::character('$'), string::hexadecimal()])
        .map(|s| Typed::new(Type::HexLiteral, s[1].to_owned()))
}

fn parse_variable<'a>() -> Parser<'a, str, Typed<String>> {
    Parser::sequence_of(vec![string::character('!'), string::alphabetic()])
        .map(|s| Typed::new(Type::Variable, s[1].to_owned()))
}

#[cfg(test)]
mod tests {
    use super::Instruction;
    use crate::parser_combinator::core::ParserState;

    #[test]
    fn parse_register() {
        assert_eq!(
            super::parse_register().parse("R1"),
            Ok(ParserState {
                index: 2,
                result: super::Typed {
                    assembly_type: super::Type::Register,
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
                    assembly_type: super::Type::HexLiteral,
                    value: String::from("aa12")
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
                    assembly_type: super::Type::Instruction,
                    value: super::Instruction2 {
                        instruction: Instruction::MoveLitReg,
                        args: (
                            super::Typed {
                                value: "aa12".to_string(),
                                assembly_type: super::Type::HexLiteral
                            },
                            super::Typed {
                                value: "R1".to_string(),
                                assembly_type: super::Type::Register
                            }
                        )
                    }
                }
            })
        )
    }
}
