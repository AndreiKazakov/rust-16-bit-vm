use super::parser::{address, hex_literal, register, square_bracket_expression, Instruction, Type};
use crate::parser_combinator::core::Parser;
use crate::parser_combinator::string;

pub fn lit_reg<'a>(command: &str, instruction: Instruction) -> Parser<'a, str, Type> {
    Parser::interspersed(
        string::whitespace(),
        vec![
            string::literal(command.to_string()).map(|_| Type::Ignored),
            Parser::one_of(vec![hex_literal(), square_bracket_expression()]),
            register().left(string::optional_whitespace()),
        ],
    )
    .map(move |res| to_instruction2(instruction, res))
}

pub fn lit_off_reg<'a>(command: &str, instruction: Instruction) -> Parser<'a, str, Type> {
    Parser::interspersed(
        string::whitespace(),
        vec![
            string::literal(command.to_string()).map(|_| Type::Ignored),
            Parser::one_of(vec![hex_literal(), square_bracket_expression()]),
            register(),
            register().left(string::optional_whitespace()),
        ],
    )
    .map(move |mut res| {
        let third = res.remove(3);
        let second = res.remove(2);
        let first = res.remove(1);
        Type::Instruction3 {
            instruction,
            arg0: Box::new(first),
            arg1: Box::new(second),
            arg2: Box::new(third),
        }
    })
}

pub fn reg_reg<'a>(command: &str, instruction: Instruction) -> Parser<'a, str, Type> {
    Parser::interspersed(
        string::whitespace(),
        vec![
            string::literal(command.to_string()).map(|_| Type::Ignored),
            register(),
            register().left(string::optional_whitespace()),
        ],
    )
    .map(move |res| to_instruction2(instruction, res))
}

pub fn mem_reg<'a>(command: &str, instruction: Instruction) -> Parser<'a, str, Type> {
    Parser::interspersed(
        string::whitespace(),
        vec![
            string::literal(command.to_string()).map(|_| Type::Ignored),
            Parser::one_of(vec![
                address(),
                string::character('&').right(square_bracket_expression()),
            ]),
            register().left(string::optional_whitespace()),
        ],
    )
    .map(move |res| to_instruction2(instruction, res))
}

pub fn reg_mem<'a>(command: &str, instruction: Instruction) -> Parser<'a, str, Type> {
    Parser::interspersed(
        string::whitespace(),
        vec![
            string::literal(command.to_string()).map(|_| Type::Ignored),
            register(),
            Parser::one_of(vec![
                address(),
                string::character('&').right(square_bracket_expression()),
            ])
            .left(string::optional_whitespace()),
        ],
    )
    .map(move |res| to_instruction2(instruction, res))
}

pub fn lit_mem<'a>(command: &str, instruction: Instruction) -> Parser<'a, str, Type> {
    Parser::interspersed(
        string::whitespace(),
        vec![
            string::literal(command.to_string()).map(|_| Type::Ignored),
            Parser::one_of(vec![hex_literal(), square_bracket_expression()]),
            Parser::one_of(vec![
                address(),
                string::character('&').right(square_bracket_expression()),
            ])
            .left(string::optional_whitespace()),
        ],
    )
    .map(move |res| to_instruction2(instruction, res))
}

pub fn reg_ptr_reg<'a>(command: &str, instruction: Instruction) -> Parser<'a, str, Type> {
    Parser::interspersed(
        string::whitespace(),
        vec![
            string::literal(command.to_string()).map(|_| Type::Ignored),
            string::character('&').right(register()),
            register().left(string::optional_whitespace()),
        ],
    )
    .map(move |res| to_instruction2(instruction, res))
}

pub fn lit<'a>(command: &str, instruction: Instruction) -> Parser<'a, str, Type> {
    Parser::interspersed(
        string::whitespace(),
        vec![
            string::literal(command.to_string()).map(|_| Type::Ignored),
            Parser::one_of(vec![hex_literal(), square_bracket_expression()])
                .left(string::optional_whitespace()),
        ],
    )
    .map(move |res| to_instruction1(instruction, res))
}

pub fn reg<'a>(command: &str, instruction: Instruction) -> Parser<'a, str, Type> {
    Parser::interspersed(
        string::whitespace(),
        vec![
            string::literal(command.to_string()).map(|_| Type::Ignored),
            register().left(string::optional_whitespace()),
        ],
    )
    .map(move |res| to_instruction1(instruction, res))
}

pub fn no_arg<'a>(command: &str, instruction: Instruction) -> Parser<'a, str, Type> {
    string::literal(command.to_string())
        .map(move |_| Type::Instruction0 { instruction })
        .left(string::optional_whitespace())
}

fn to_instruction1(instruction: Instruction, mut parsed_instruction: Vec<Type>) -> Type {
    let first = parsed_instruction.remove(1);
    Type::Instruction1 {
        instruction,
        arg0: Box::new(first),
    }
}

fn to_instruction2(instruction: Instruction, mut parsed_instruction: Vec<Type>) -> Type {
    let second = parsed_instruction.remove(2);
    let first = parsed_instruction.remove(1);
    Type::Instruction2 {
        instruction,
        arg0: Box::new(first),
        arg1: Box::new(second),
    }
}

#[cfg(test)]
mod tests {
    use crate::assembler::parser::Instruction;
    use crate::parser_combinator::core::ParserState;

    #[test]
    fn lit_reg() {
        assert_eq!(
            super::lit_reg("mov", Instruction::MoveLitReg).parse("mov $aa12 R1"),
            Ok(ParserState {
                index: 12,
                result: super::Type::Instruction2 {
                    instruction: super::Instruction::MoveLitReg,
                    arg0: Box::new(super::Type::HexLiteral(43538)),
                    arg1: Box::new(super::Type::Register("R1".to_string())),
                },
            })
        );
        assert_eq!(
            super::lit_reg("mov", Instruction::MoveLitReg).parse("mov [$aa12 + !a] R1"),
            Ok(ParserState {
                index: 19,
                result: super::Type::Instruction2 {
                    instruction: super::Instruction::MoveLitReg,
                    arg0: Box::new(super::Type::BinaryOperation {
                        a: Box::new(super::Type::HexLiteral(43538)),
                        op: Box::new(super::Type::Operator(super::super::parser::Operator::Plus)),
                        b: Box::new(super::Type::Variable("a".to_string()))
                    }),
                    arg1: Box::new(super::Type::Register("R1".to_string())),
                },
            })
        )
    }

    #[test]
    fn lit_off_reg() {
        assert_eq!(
            super::lit_off_reg("mov", Instruction::MoveLitOffReg).parse("mov $aa12 R3 R1"),
            Ok(ParserState {
                index: 15,
                result: super::Type::Instruction3 {
                    instruction: super::Instruction::MoveLitOffReg,
                    arg0: Box::new(super::Type::HexLiteral(43538)),
                    arg1: Box::new(super::Type::Register("R3".to_string())),
                    arg2: Box::new(super::Type::Register("R1".to_string())),
                },
            })
        );
    }

    #[test]
    fn reg_reg() {
        assert_eq!(
            super::reg_reg("mov", Instruction::MoveRegReg).parse("mov R2 R1"),
            Ok(ParserState {
                index: 9,
                result: super::Type::Instruction2 {
                    instruction: super::Instruction::MoveRegReg,
                    arg0: Box::new(super::Type::Register("R2".to_string())),
                    arg1: Box::new(super::Type::Register("R1".to_string())),
                },
            })
        );
    }

    #[test]
    fn mem_reg() {
        assert_eq!(
            super::mem_reg("mov", Instruction::MoveMemReg).parse("mov &123 R1"),
            Ok(ParserState {
                index: 11,
                result: super::Type::Instruction2 {
                    instruction: super::Instruction::MoveMemReg,
                    arg0: Box::new(super::Type::Address(291)),
                    arg1: Box::new(super::Type::Register("R1".to_string())),
                },
            })
        );
        assert_eq!(
            super::mem_reg("mov", Instruction::MoveMemReg).parse("mov &[$aa12 + !a] R1"),
            Ok(ParserState {
                index: 20,
                result: super::Type::Instruction2 {
                    instruction: super::Instruction::MoveMemReg,
                    arg0: Box::new(super::Type::BinaryOperation {
                        a: Box::new(super::Type::HexLiteral(43538)),
                        op: Box::new(super::Type::Operator(super::super::parser::Operator::Plus)),
                        b: Box::new(super::Type::Variable("a".to_string()))
                    }),
                    arg1: Box::new(super::Type::Register("R1".to_string())),
                },
            })
        )
    }

    #[test]
    fn reg_mem() {
        assert_eq!(
            super::reg_mem("mov", Instruction::MoveRegMem).parse("mov R1 &123"),
            Ok(ParserState {
                index: 11,
                result: super::Type::Instruction2 {
                    instruction: super::Instruction::MoveRegMem,
                    arg0: Box::new(super::Type::Register("R1".to_string())),
                    arg1: Box::new(super::Type::Address(291)),
                },
            })
        );
        assert_eq!(
            super::reg_mem("mov", Instruction::MoveRegMem).parse("mov R1 &[$aa12 + !a]"),
            Ok(ParserState {
                index: 20,
                result: super::Type::Instruction2 {
                    instruction: super::Instruction::MoveRegMem,
                    arg0: Box::new(super::Type::Register("R1".to_string())),
                    arg1: Box::new(super::Type::BinaryOperation {
                        a: Box::new(super::Type::HexLiteral(43538)),
                        op: Box::new(super::Type::Operator(super::super::parser::Operator::Plus)),
                        b: Box::new(super::Type::Variable("a".to_string()))
                    }),
                },
            })
        )
    }

    #[test]
    fn lit_mem() {
        assert_eq!(
            super::lit_mem("mov", Instruction::MoveLitMem).parse("mov $aa12 &12"),
            Ok(ParserState {
                index: 13,
                result: super::Type::Instruction2 {
                    instruction: super::Instruction::MoveLitMem,
                    arg0: Box::new(super::Type::HexLiteral(43538)),
                    arg1: Box::new(super::Type::Address(18)),
                },
            })
        );
    }

    #[test]
    fn reg_ptr_reg() {
        assert_eq!(
            super::reg_ptr_reg("mov", Instruction::MoveRegPtrReg).parse("mov &R2 R1"),
            Ok(ParserState {
                index: 10,
                result: super::Type::Instruction2 {
                    instruction: super::Instruction::MoveRegPtrReg,
                    arg0: Box::new(super::Type::Register("R2".to_string())),
                    arg1: Box::new(super::Type::Register("R1".to_string())),
                },
            })
        );
    }

    #[test]
    fn lit() {
        assert_eq!(
            super::lit("mov", Instruction::MoveLitMem).parse("mov $aa12"),
            Ok(ParserState {
                index: 9,
                result: super::Type::Instruction1 {
                    instruction: super::Instruction::MoveLitMem,
                    arg0: Box::new(super::Type::HexLiteral(43538)),
                },
            })
        );
    }

    #[test]
    fn reg() {
        assert_eq!(
            super::reg("mov", Instruction::MoveLitMem).parse("mov R5"),
            Ok(ParserState {
                index: 6,
                result: super::Type::Instruction1 {
                    instruction: super::Instruction::MoveLitMem,
                    arg0: Box::new(super::Type::Register("R5".to_string())),
                },
            })
        );
    }

    #[test]
    fn no_arg() {
        assert_eq!(
            super::no_arg("mov", Instruction::MoveLitMem).parse("mov"),
            Ok(ParserState {
                index: 3,
                result: super::Type::Instruction0 {
                    instruction: super::Instruction::MoveLitMem,
                },
            })
        );
    }
}
