use std::collections::HashMap;

use formats::{
    lit, lit_mem, lit_off_reg, lit_reg, mem_reg, no_arg, reg, reg_lit, reg_lit8, reg_mem,
    reg_ptr_reg, reg_reg,
};
use parser::{label, Type};

use crate::cpu::instruction;
use crate::cpu::register::get_from_string;
use crate::parser_combinator::core::{Parser, ParserState};
use crate::parser_combinator::string::{character, optional_whitespace};

mod formats;
mod parser;

pub fn compile(code: &str) -> Vec<u8> {
    match assembly_parser().parse(code) {
        Ok(ParserState { result, index }) => {
            if code.len() != index {
                panic!("Could not parse from index {}", index);
            }
            let mut res = vec![];
            let mut labels = HashMap::new();
            let mut current_address = 0;

            for t in &result {
                match t {
                    Type::Label(label) => {
                        labels.insert(label, current_address);
                    }
                    Type::Instruction0 { instruction, .. } => current_address += instruction.size,
                    Type::Instruction1 { instruction, .. } => current_address += instruction.size,
                    Type::Instruction2 { instruction, .. } => current_address += instruction.size,
                    Type::Instruction3 { instruction, .. } => current_address += instruction.size,
                    _ => panic!("Unexpected instruction on top level: {:?}", t),
                }
            }

            for t in &result {
                res.extend(encode(t, &labels))
            }

            res
        }
        Err(err) => panic!("Could not compile: {}", err.message),
    }
}

fn encode(t: &Type, labels: &HashMap<&String, u16>) -> Vec<u8> {
    match t {
        Type::Instruction0 { instruction } => vec![instruction.opcode],
        Type::Instruction1 { instruction, arg0 } => {
            let mut res = vec![instruction.opcode];
            res.extend(encode(arg0, labels));
            res
        }
        Type::Instruction2 {
            instruction,
            arg0,
            arg1,
        } => {
            let mut res = vec![instruction.opcode];
            res.extend(encode(arg0, labels));
            res.extend(encode(arg1, labels));
            res
        }
        Type::Instruction3 {
            instruction,
            arg0,
            arg1,
            arg2,
        } => {
            let mut res = vec![instruction.opcode];
            res.extend(encode(arg0, labels));
            res.extend(encode(arg1, labels));
            res.extend(encode(arg2, labels));
            res
        }
        Type::BinaryOperation { .. } => panic!("Not supported yet"),
        Type::Ignored => panic!("ignored node was left after processing"),
        Type::HexLiteral(val) => val.to_be_bytes().to_vec(),
        Type::HexLiteral8(val) => vec![*val],
        Type::Address(val) => val.to_be_bytes().to_vec(),
        Type::Variable(name) => labels[name].to_be_bytes().to_vec(),
        Type::Register(val) => vec![get_from_string(val) as u8],
        Type::Operator(_) => panic!("Not supported yet"),
        Type::Label(_) => Vec::with_capacity(0),
    }
}

fn assembly_parser<'a>() -> Parser<'a, str, Vec<Type>> {
    assembly_instruction()
        .left(optional_whitespace())
        .left(character('\n'))
        .one_or_more()
}

fn assembly_instruction<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        label(),
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
        lit_reg("mov", instruction::MOVE_LIT_REG),
        lit_off_reg("mov", instruction::MOVE_LIT_OFF_REG),
        reg_reg("mov", instruction::MOVE_REG_REG),
        lit_mem("mov", instruction::MOVE_LIT_MEM),
        mem_reg("mov", instruction::MOVE_MEM_REG),
        reg_ptr_reg("mov", instruction::MOVE_REG_PTR_REG),
        reg_mem("mov", instruction::MOVE_REG_MEM),
    ])
}

fn add<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit_reg("add", instruction::ADD_LIT_REG),
        reg_reg("add", instruction::ADD_REG_REG),
    ])
}

fn sub<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit_reg("sub", instruction::SUB_LIT_REG),
        reg_reg("sub", instruction::SUB_REG_REG),
        reg_lit("sub", instruction::SUB_REG_LIT),
    ])
}

fn mul<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit_reg("mul", instruction::MUL_LIT_REG),
        reg_reg("mul", instruction::MUL_REG_REG),
    ])
}

fn lsf<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        reg_lit8("lsf", instruction::LSF_REG_LIT8),
        reg_reg("lsf", instruction::LSF_REG_REG),
    ])
}

fn rsf<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        reg_lit8("rsf", instruction::RSF_REG_LIT8),
        reg_reg("rsf", instruction::RSF_REG_REG),
    ])
}

fn and<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit_reg("and", instruction::AND_REG_LIT),
        reg_reg("and", instruction::AND_REG_REG),
    ])
}

fn or<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit_reg("or", instruction::OR_REG_LIT),
        reg_reg("or", instruction::OR_REG_REG),
    ])
}

fn xor<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit_reg("xor", instruction::XOR_REG_LIT),
        reg_reg("xor", instruction::XOR_REG_REG),
    ])
}

fn jeq<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit_mem("jeq", instruction::JEQ_LIT_MEM),
        reg_mem("jeq", instruction::JEQ_REG_MEM),
    ])
}

fn jne<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit_mem("jne", instruction::JNE_LIT_MEM),
        reg_mem("jne", instruction::JNE_REG_MEM),
    ])
}

fn jgt<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit_mem("jgt", instruction::JGT_LIT_MEM),
        reg_mem("jgt", instruction::JGT_REG_MEM),
    ])
}

fn jlt<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit_mem("jlt", instruction::JLT_LIT_MEM),
        reg_mem("jlt", instruction::JLT_REG_MEM),
    ])
}

fn jle<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit_mem("jle", instruction::JLE_LIT_MEM),
        reg_mem("jle", instruction::JLE_REG_MEM),
    ])
}

fn jge<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit_mem("jge", instruction::JGE_LIT_MEM),
        reg_mem("jge", instruction::JGE_REG_MEM),
    ])
}

fn psh<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit("psh", instruction::PSH_LIT),
        reg("psh", instruction::PSH_REG),
    ])
}

fn pop<'a>() -> Parser<'a, str, Type> {
    reg("pop", instruction::POP_REG)
}

fn inc<'a>() -> Parser<'a, str, Type> {
    reg("inc", instruction::INC_REG)
}

fn dec<'a>() -> Parser<'a, str, Type> {
    reg("dec", instruction::DEC_REG)
}

fn not<'a>() -> Parser<'a, str, Type> {
    reg("not", instruction::NOT_REG)
}

fn cal<'a>() -> Parser<'a, str, Type> {
    Parser::one_of(vec![
        lit("cal", instruction::CAL_LIT),
        reg("cal", instruction::CAL_REG),
    ])
}

fn ret<'a>() -> Parser<'a, str, Type> {
    no_arg("ret", instruction::RET)
}

fn hlt<'a>() -> Parser<'a, str, Type> {
    no_arg("hlt", instruction::HLT)
}

#[cfg(test)]
mod tests {
    #[test]
    fn compile() {
        let input = "mov $4200 R1\nmov R1 &AAAA\nmov $1000 R1\nmov &AAAA R2\nadd R1 R2\n";
        assert_eq!(
            super::compile(input),
            vec![
                0x10, 0x42, 0, 4, 0x12, 4, 0xaa, 0xaa, 0x10, 0x10, 0, 4, 0x13, 0xAA, 0xAA, 6, 0x14,
                4, 6
            ]
        )
    }

    #[test]
    fn compile_with_labels() {
        let input = "mov $2345 ACC\nstart:\njeq $4200 &[!start]\n";
        assert_eq!(
            super::compile(input),
            vec![0x10, 0x23, 0x45, 0x02, 0x52, 0x42, 0x00, 0x00, 0x04]
        )
    }

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
