use crate::parser_combinator::core::Parser;
mod parser;
use parser::{Instruction, Type};
mod formats;
use formats::{lit_mem, lit_off_reg, lit_reg, mem_reg, reg_mem, reg_ptr_reg, reg_reg};

pub fn mov<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit_reg("mov", Instruction::MoveLitReg),
        lit_off_reg("mov", Instruction::MoveLitOffReg),
        reg_reg("mov", Instruction::MoveRegReg),
        lit_mem("mov", Instruction::MoveLitMem),
        mem_reg("mov", Instruction::MoveMemReg),
        reg_ptr_reg("mov", Instruction::MoveRegPtrReg),
        reg_mem("mov", Instruction::MoveRegMem),
    ])
}

#[cfg(test)]
mod tests {
    #[test]
    fn mov() {
        let input = vec![
            "mov $aaa R1",
            "mov [!aaa] R1",
            "mov R2 R1",
            "mov &R2 R1",
            "mov R2 &333",
            "mov $122 &333",
            "mov [!kk] &333",
            "mov [[$22 - $22] + !kk] &[$333 - $33 * !xxx]",
            "mov &333 R2",
            "mov $aa R3 R1",
        ];
        for line in input {
            assert!(super::mov().parse(line).is_ok(), line)
        }
    }
}
