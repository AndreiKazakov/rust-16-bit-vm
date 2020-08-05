use crate::parser_combinator::core::Parser;
use crate::parser_combinator::string;

fn move_lit_to_reg<'a>() -> Parser<'a, str, Type> {
    Parser::interspersed(
        string::whitespace(),
        vec![
            string::literal("mov".to_string()).map(|_| Type::Ignored),
            parse_hex_literal(),
            parse_register().left(string::optional_whitespace()),
        ],
    )
    .map(|mut res| {
        let second = res.remove(2);
        let first = res.remove(1);
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
    .map(Type::Register)
}

fn parse_hex_literal<'a>() -> Parser<'a, str, Type> {
    string::character('$')
        .right(string::hexadecimal())
        .map(Type::HexLiteral)
}

fn parse_variable<'a>() -> Parser<'a, str, Type> {
    string::character('!')
        .right(string::alphabetic())
        .map(Type::Variable)
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
    fn parse_variable() {
        assert_eq!(
            super::parse_variable().parse("!aaj"),
            Ok(ParserState {
                index: 4,
                result: Type::Variable(String::from("aaj"))
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
