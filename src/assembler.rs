use crate::parser_combinator::core::{ParseError, Parser, ParserState};
use crate::parser_combinator::string;
use crate::parser_combinator::string::{character, optional_whitespace};

fn move_lit_to_reg<'a>() -> Parser<'a, str, Type> {
    Parser::interspersed(
        string::whitespace(),
        vec![
            string::literal("mov".to_string()).map(|_| Type::Ignored),
            Parser::one_of(vec![hex_literal(), square_bracket_expression()]),
            register().left(string::optional_whitespace()),
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

fn square_bracket_expression<'a>() -> Parser<'a, str, Type> {
    Parser::new(|input| {
        let mut index = character('[').parse(input)?.index;
        index = optional_whitespace().parse_at(input, index)?.index;

        let mut result = vec![];
        let mut expect_operator = false;

        loop {
            if expect_operator {
                match input.chars().nth(index) {
                    Some(']') => {
                        index = character(']').parse_at(input, index)?.index;
                        break;
                    }
                    None => {
                        return Err(ParseError {
                            message: "EOL".to_string(),
                            index,
                        })
                    }
                    _ => {
                        let state = operator().parse_at(input, index)?;
                        index = optional_whitespace().parse_at(input, state.index)?.index;
                        expect_operator = false;
                        result.push(state.result);
                    }
                }
            } else {
                let state =
                    Parser::one_of(vec![square_bracket_expression(), hex_literal(), variable()])
                        .parse_at(input, index)?;
                result.push(state.result);
                index = optional_whitespace().parse_at(input, state.index)?.index;
                expect_operator = true;
            }
        }

        Ok(ParserState {
            index,
            result: Type::Expression(result),
        })
    })
}

fn register<'a>() -> Parser<'a, str, Type> {
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

fn hex_literal<'a>() -> Parser<'a, str, Type> {
    string::character('$')
        .right(string::hexadecimal())
        .map(Type::HexLiteral)
}

fn variable<'a>() -> Parser<'a, str, Type> {
    string::character('!')
        .right(string::alphabetic())
        .map(Type::Variable)
}

fn operator<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        string::character('+'),
        string::character('-'),
        string::character('*'),
    ])
    .map(|op| match op.chars().next().unwrap() {
        '+' => Type::Operator(Operator::Plus),
        '-' => Type::Operator(Operator::Minus),
        '*' => Type::Operator(Operator::Star),
        _ => panic!(),
    })
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Type {
    Instruction2 {
        instruction: Instruction,
        arg0: Box<Type>,
        arg1: Box<Type>,
    },
    Expression(Vec<Type>),
    Ignored,
    HexLiteral(String),
    Variable(String),
    Register(String),
    Operator(Operator),
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum Instruction {
    MoveLitReg,
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum Operator {
    Plus,
    Minus,
    Star,
}

#[cfg(test)]
mod tests {
    use super::{Instruction, Operator, Type};
    use crate::parser_combinator::core::ParserState;

    #[test]
    fn register() {
        assert_eq!(
            super::register().parse("R1"),
            Ok(ParserState {
                index: 2,
                result: Type::Register(String::from("R1")),
            })
        )
    }

    #[test]
    fn hex_literal() {
        assert_eq!(
            super::hex_literal().parse("$aa12"),
            Ok(ParserState {
                index: 5,
                result: Type::HexLiteral(String::from("aa12")),
            })
        )
    }

    #[test]
    fn variable() {
        assert_eq!(
            super::variable().parse("!aaj"),
            Ok(ParserState {
                index: 4,
                result: Type::Variable(String::from("aaj")),
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
                    arg1: Box::new(Type::Register("R1".to_string())),
                },
            })
        );
        assert_eq!(
            super::move_lit_to_reg().parse("mov [$aa12 + !a] R1"),
            Ok(ParserState {
                index: 19,
                result: Type::Instruction2 {
                    instruction: Instruction::MoveLitReg,
                    arg0: Box::new(Type::Expression(vec![
                        Type::HexLiteral("aa12".to_string()),
                        Type::Operator(Operator::Plus),
                        Type::Variable("a".to_string())
                    ])),
                    arg1: Box::new(Type::Register("R1".to_string())),
                },
            })
        )
    }

    #[test]
    fn square_bracket_expression() {
        assert_eq!(
            super::square_bracket_expression().parse("[$aa12 + [!uu * !aa] - $1]"),
            Ok(ParserState {
                index: 26,
                result: Type::Expression(vec![
                    Type::HexLiteral("aa12".to_string()),
                    Type::Operator(Operator::Plus),
                    Type::Expression(vec![
                        Type::Variable("uu".to_string()),
                        Type::Operator(Operator::Star),
                        Type::Variable("aa".to_string())
                    ]),
                    Type::Operator(Operator::Minus),
                    Type::HexLiteral("1".to_string()),
                ])
            })
        )
    }
}
