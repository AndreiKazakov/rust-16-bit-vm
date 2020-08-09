use crate::parser_combinator::core::Parser;
mod parser;
use parser::{Instruction, Type};
mod formats;
use formats::{
    lit, lit_mem, lit_off_reg, lit_reg, mem_reg, no_arg, reg, reg_lit, reg_mem, reg_ptr_reg,
    reg_reg,
};

pub fn assembly_parser<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        mov(),
        add(),
        sub(),
        mul(),
        lsf(),
        rsf(),
        and(),
        or(),
        xor(),
        jeq(),
        jne(),
        jgt(),
        jlt(),
        jle(),
        jge(),
        psh(),
        pop(),
        inc(),
        dec(),
        not(),
        cal(),
        ret(),
        hlt(),
    ])
}

fn mov<'a>() -> Parser<'a, str, Type> {
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

fn add<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit_reg("add", Instruction::AddLitReg),
        reg_reg("add", Instruction::AddRegReg),
    ])
}

fn sub<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit_reg("sub", Instruction::SubLitReg),
        reg_reg("sub", Instruction::SubRegReg),
        reg_lit("sub", Instruction::SubRegLit),
    ])
}

fn mul<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit_reg("mul", Instruction::MulLitReg),
        reg_reg("mul", Instruction::MulRegReg),
    ])
}

fn lsf<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        reg_lit("lsf", Instruction::LsfRegLit),
        reg_reg("lsf", Instruction::LsfRegReg),
    ])
}

fn rsf<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        reg_lit("rsf", Instruction::RsfRegLit),
        reg_reg("rsf", Instruction::RsfRegReg),
    ])
}

fn and<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit_reg("and", Instruction::AndLitReg),
        reg_reg("and", Instruction::AndRegReg),
    ])
}

fn or<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit_reg("or", Instruction::OrLitReg),
        reg_reg("or", Instruction::OrRegReg),
    ])
}

fn xor<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit_reg("xor", Instruction::XorLitReg),
        reg_reg("xor", Instruction::XorRegReg),
    ])
}

fn jeq<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit_mem("jeq", Instruction::JeqLitMem),
        reg_mem("jeq", Instruction::JeqRegMem),
    ])
}

fn jne<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit_mem("jne", Instruction::JneLitMem),
        reg_mem("jne", Instruction::JneRegMem),
    ])
}

fn jgt<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit_mem("jgt", Instruction::JgtLitMem),
        reg_mem("jgt", Instruction::JgtRegMem),
    ])
}

fn jlt<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit_mem("jlt", Instruction::JltLitMem),
        reg_mem("jlt", Instruction::JltRegMem),
    ])
}

fn jle<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit_mem("jle", Instruction::JleLitMem),
        reg_mem("jle", Instruction::JleRegMem),
    ])
}

fn jge<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit_mem("jge", Instruction::JgeLitMem),
        reg_mem("jge", Instruction::JgeRegMem),
    ])
}

fn psh<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit("psh", Instruction::PshLit),
        reg("psh", Instruction::PshReg),
    ])
}

fn pop<'a>() -> Parser<'a, str, Type> {
    reg("pop", Instruction::PopReg)
}

fn inc<'a>() -> Parser<'a, str, Type> {
    reg("inc", Instruction::IncReg)
}

fn dec<'a>() -> Parser<'a, str, Type> {
    reg("dec", Instruction::DecReg)
}

fn not<'a>() -> Parser<'a, str, Type> {
    reg("not", Instruction::NotReg)
}

fn cal<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit("cal", Instruction::CalLit),
        reg("cal", Instruction::CalReg),
    ])
}

fn ret<'a>() -> Parser<'a, str, Type> {
    no_arg("ret", Instruction::Ret)
}

fn hlt<'a>() -> Parser<'a, str, Type> {
    no_arg("hlt", Instruction::Hlt)
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
