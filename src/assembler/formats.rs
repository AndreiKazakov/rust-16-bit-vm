use super::parser::{address, hex_literal, register, square_bracket_expression, Type};
use crate::cpu::instruction::Instruction;
use crate::parser_combinator::core::Parser;
use crate::parser_combinator::string;

pub fn lit_reg<'a>(command: &str, instruction: Instruction) -> Parser<'a, str, Type> {
    instruction2(instruction, com(command), hex_or_exp(), register())
}

pub fn reg_lit<'a>(command: &str, instruction: Instruction) -> Parser<'a, str, Type> {
    instruction2(instruction, com(command), register(), hex_or_exp())
}

pub fn lit_off_reg<'a>(command: &str, instruction: Instruction) -> Parser<'a, str, Type> {
    Parser::interspersed(
        string::whitespace(),
        vec![com(command), hex_or_exp(), register(), register()],
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
    instruction2(instruction, com(command), register(), register())
}

pub fn mem_reg<'a>(command: &str, instruction: Instruction) -> Parser<'a, str, Type> {
    instruction2(instruction, com(command), address_or_exp(), register())
}

pub fn reg_mem<'a>(command: &str, instruction: Instruction) -> Parser<'a, str, Type> {
    instruction2(instruction, com(command), register(), address_or_exp())
}

pub fn lit_mem<'a>(command: &str, instruction: Instruction) -> Parser<'a, str, Type> {
    instruction2(instruction, com(command), hex_or_exp(), address_or_exp())
}

pub fn reg_ptr_reg<'a>(command: &str, instruction: Instruction) -> Parser<'a, str, Type> {
    instruction2(
        instruction,
        com(command),
        string::character('&').right(register()),
        register(),
    )
}

pub fn lit<'a>(command: &str, instruction: Instruction) -> Parser<'a, str, Type> {
    Parser::interspersed(string::whitespace(), vec![com(command), hex_or_exp()])
        .map(move |res| to_instruction1(instruction, res))
}

pub fn reg<'a>(command: &str, instruction: Instruction) -> Parser<'a, str, Type> {
    Parser::interspersed(string::whitespace(), vec![com(command), register()])
        .map(move |res| to_instruction1(instruction, res))
}

pub fn no_arg<'a>(command: &str, instruction: Instruction) -> Parser<'a, str, Type> {
    string::literal(command.to_string()).map(move |_| Type::Instruction0 { instruction })
}

fn instruction2<'a>(
    instruction: Instruction,
    command: Parser<'a, str, Type>,
    a: Parser<'a, str, Type>,
    b: Parser<'a, str, Type>,
) -> Parser<'a, str, Type> {
    Parser::interspersed(string::whitespace(), vec![command, a, b])
        .map(move |res| to_instruction2(instruction, res))
}

fn hex_or_exp<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![hex_literal(), square_bracket_expression()])
}

fn address_or_exp<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        address(),
        string::character('&').right(square_bracket_expression()),
    ])
}

fn com<'a>(command: &str) -> Parser<'a, str, Type> {
    string::literal(command.to_string()).map(|_| Type::Ignored)
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
    use crate::cpu::instruction;
    use crate::parser_combinator::core::ParserState;

    #[test]
    fn lit_reg() {
        assert_eq!(
            super::lit_reg("mov", instruction::MOVE_LIT_REG).parse("mov $aa12 R1"),
            Ok(ParserState {
                index: 12,
                result: super::Type::Instruction2 {
                    instruction: instruction::MOVE_LIT_REG,
                    arg0: Box::new(super::Type::HexLiteral(43538)),
                    arg1: Box::new(super::Type::Register("R1".to_string())),
                },
            })
        );
        assert_eq!(
            super::lit_reg("mov", instruction::MOVE_LIT_REG).parse("mov [$aa12 + !a] R1"),
            Ok(ParserState {
                index: 19,
                result: super::Type::Instruction2 {
                    instruction: instruction::MOVE_LIT_REG,
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
            super::lit_off_reg("mov", instruction::MOVE_LIT_OFF_REG).parse("mov $aa12 R3 R1"),
            Ok(ParserState {
                index: 15,
                result: super::Type::Instruction3 {
                    instruction: instruction::MOVE_LIT_OFF_REG,
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
            super::reg_reg("mov", instruction::MOVE_REG_REG).parse("mov R2 R1"),
            Ok(ParserState {
                index: 9,
                result: super::Type::Instruction2 {
                    instruction: instruction::MOVE_REG_REG,
                    arg0: Box::new(super::Type::Register("R2".to_string())),
                    arg1: Box::new(super::Type::Register("R1".to_string())),
                },
            })
        );
    }

    #[test]
    fn mem_reg() {
        assert_eq!(
            super::mem_reg("mov", instruction::MOVE_MEM_REG).parse("mov &123 R1"),
            Ok(ParserState {
                index: 11,
                result: super::Type::Instruction2 {
                    instruction: instruction::MOVE_MEM_REG,
                    arg0: Box::new(super::Type::Address(291)),
                    arg1: Box::new(super::Type::Register("R1".to_string())),
                },
            })
        );
        assert_eq!(
            super::mem_reg("mov", instruction::MOVE_MEM_REG).parse("mov &[$aa12 + !a] R1"),
            Ok(ParserState {
                index: 20,
                result: super::Type::Instruction2 {
                    instruction: instruction::MOVE_MEM_REG,
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
            super::reg_mem("mov", instruction::MOVE_REG_MEM).parse("mov R1 &123"),
            Ok(ParserState {
                index: 11,
                result: super::Type::Instruction2 {
                    instruction: instruction::MOVE_REG_MEM,
                    arg0: Box::new(super::Type::Register("R1".to_string())),
                    arg1: Box::new(super::Type::Address(291)),
                },
            })
        );
        assert_eq!(
            super::reg_mem("mov", instruction::MOVE_REG_MEM).parse("mov R1 &[$aa12 + !a]"),
            Ok(ParserState {
                index: 20,
                result: super::Type::Instruction2 {
                    instruction: instruction::MOVE_REG_MEM,
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
            super::lit_mem("mov", instruction::MOVE_LIT_MEM).parse("mov $aa12 &12"),
            Ok(ParserState {
                index: 13,
                result: super::Type::Instruction2 {
                    instruction: instruction::MOVE_LIT_MEM,
                    arg0: Box::new(super::Type::HexLiteral(43538)),
                    arg1: Box::new(super::Type::Address(18)),
                },
            })
        );
    }

    #[test]
    fn reg_ptr_reg() {
        assert_eq!(
            super::reg_ptr_reg("mov", instruction::MOVE_REG_PTR_REG).parse("mov &R2 R1"),
            Ok(ParserState {
                index: 10,
                result: super::Type::Instruction2 {
                    instruction: instruction::MOVE_REG_PTR_REG,
                    arg0: Box::new(super::Type::Register("R2".to_string())),
                    arg1: Box::new(super::Type::Register("R1".to_string())),
                },
            })
        );
    }

    #[test]
    fn lit() {
        assert_eq!(
            super::lit("mov", instruction::MOVE_LIT_MEM).parse("mov $aa12"),
            Ok(ParserState {
                index: 9,
                result: super::Type::Instruction1 {
                    instruction: instruction::MOVE_LIT_MEM,
                    arg0: Box::new(super::Type::HexLiteral(43538)),
                },
            })
        );
    }

    #[test]
    fn reg() {
        assert_eq!(
            super::reg("mov", instruction::MOVE_LIT_MEM).parse("mov R5"),
            Ok(ParserState {
                index: 6,
                result: super::Type::Instruction1 {
                    instruction: instruction::MOVE_LIT_MEM,
                    arg0: Box::new(super::Type::Register("R5".to_string())),
                },
            })
        );
    }

    #[test]
    fn no_arg() {
        assert_eq!(
            super::no_arg("mov", instruction::MOVE_LIT_MEM).parse("mov"),
            Ok(ParserState {
                index: 3,
                result: super::Type::Instruction0 {
                    instruction: instruction::MOVE_LIT_MEM,
                },
            })
        );
    }
}
