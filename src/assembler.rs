use crate::parser_combinator::core::Parser;
use crate::parser_combinator::string;

fn move_lit_to_reg<'a>() -> Parser<'a, str, Type> {
    Parser::sequence_of(vec![
        string::literal("mov".to_string()).map(|_| Type::Ignored),
        string::whitespace().map(|_| Type::Ignored),
        parse_hex_literal(),
        string::whitespace().map(|_| Type::Ignored),
        parse_register(),
        string::optional_whitespace().map(|_| Type::Ignored),
    ])
    .map(|mut res| {
        let second = res.remove(4);
        let first = res.remove(2);
        Type::Instruction2 {
            instruction: Instruction::MoveLitReg,
            arg0: Box::new(first),
            arg1: Box::new(second),
        }
    })
}

fn parse_register<'a>() -> Parser<'a, str, Type> {
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
    .map(|reg| Type::Register(reg))
}

fn parse_hex_literal<'a>() -> Parser<'a, str, Type> {
    Parser::sequence_of(vec![string::character('$'), string::hexadecimal()])
        .map(|s| Type::HexLiteral(s[1].to_owned()))
}

fn parse_variable<'a>() -> Parser<'a, str, Type> {
    Parser::sequence_of(vec![string::character('!'), string::alphabetic()])
        .map(|s| Type::Variable(s[1].to_owned()))
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Type {
    Instruction2 {
        instruction: Instruction,
        arg0: Box<Type>,
        arg1: Box<Type>,
    },
    Ignored,
    HexLiteral(String),
    Variable(String),
    Register(String),
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum Instruction {
    MoveLitReg,
}

#[cfg(test)]
mod tests {
    use super::{Instruction, Type};
    use crate::parser_combinator::core::ParserState;

    #[test]
    fn parse_register() {
        assert_eq!(
            super::parse_register().parse("R1"),
            Ok(ParserState {
                index: 2,
                result: Type::Register(String::from("R1"))
            })
        )
    }

    #[test]
    fn parse_hex_literal() {
        assert_eq!(
            super::parse_hex_literal().parse("$aa12"),
            Ok(ParserState {
                index: 5,
                result: Type::HexLiteral(String::from("aa12"))
            })
        )
    }

    #[test]
    fn move_lit_reg() {
        assert_eq!(
            super::move_lit_to_reg().parse("mov $aa12 R1"),
            Ok(ParserState {
                index: 12,
                result: Type::Instruction2 {
                    instruction: Instruction::MoveLitReg,
                    arg0: Box::new(Type::HexLiteral("aa12".to_string())),
                    arg1: Box::new(Type::Register("R1".to_string()))
                }
            })
        )
    }
}
