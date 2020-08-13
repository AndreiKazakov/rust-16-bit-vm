// Numeric enums do not work very well here, cause there is no good/fast way of converting u8 to enum
// Maybe would be possible to implement this as a macro
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub struct Instruction {
    pub opcode: u8,
    size: u8,
}

const LIT_REG: u8 = 4;
const REG_LIT: u8 = 4;
const REG_LIT8: u8 = 3;
const REG_REG: u8 = 3;
const REG_MEM: u8 = 4;
const MEM_REG: u8 = 4;
const LIT_MEM: u8 = 5;
const REG_PTR_REG: u8 = 3;
const LIT_OFF_REG: u8 = 5;
const NONE: u8 = 1;
const REG: u8 = 2;
const LIT: u8 = 3;

pub const MOVE_LIT_MEM: Instruction = Instruction {
    opcode: 0x09,
    size: LIT_MEM,
};
pub const MOVE_LIT_REG: Instruction = Instruction {
    opcode: 0x10,
    size: LIT_REG,
};
pub const MOVE_REG_REG: Instruction = Instruction {
    opcode: 0x11,
    size: REG_REG,
};
pub const MOVE_REG_MEM: Instruction = Instruction {
    opcode: 0x12,
    size: REG_MEM,
};
pub const MOVE_MEM_REG: Instruction = Instruction {
    opcode: 0x13,
    size: MEM_REG,
};
pub const PSH_LIT: Instruction = Instruction {
    opcode: 0x16,
    size: LIT,
};
pub const PSH_REG: Instruction = Instruction {
    opcode: 0x17,
    size: REG,
};
pub const POP_REG: Instruction = Instruction {
    opcode: 0x18,
    size: REG,
};
pub const CAL_LIT: Instruction = Instruction {
    opcode: 0x19,
    size: LIT,
};
pub const CAL_REG: Instruction = Instruction {
    opcode: 0x1a,
    size: REG,
};
pub const RET: Instruction = Instruction {
    opcode: 0x1b,
    size: NONE,
};
pub const MOVE_REG_PTR_REG: Instruction = Instruction {
    opcode: 0x1c,
    size: REG_PTR_REG,
};
pub const MOVE_LIT_OFF_REG: Instruction = Instruction {
    opcode: 0x1d,
    size: LIT_OFF_REG,
};

pub const ADD_REG_REG: Instruction = Instruction {
    opcode: 0x14,
    size: REG_REG,
};
pub const ADD_LIT_REG: Instruction = Instruction {
    opcode: 0x30,
    size: LIT_REG,
};
pub const SUB_LIT_REG: Instruction = Instruction {
    opcode: 0x31,
    size: LIT_REG,
};
pub const SUB_REG_LIT: Instruction = Instruction {
    opcode: 0x32,
    size: REG_LIT,
};
pub const SUB_REG_REG: Instruction = Instruction {
    opcode: 0x33,
    size: REG_REG,
};
pub const MUL_LIT_REG: Instruction = Instruction {
    opcode: 0x34,
    size: LIT_REG,
};
pub const MUL_REG_REG: Instruction = Instruction {
    opcode: 0x35,
    size: REG_REG,
};
pub const INC_REG: Instruction = Instruction {
    opcode: 0x36,
    size: REG,
};
pub const DEC_REG: Instruction = Instruction {
    opcode: 0x37,
    size: REG,
};

pub const LSF_REG_LIT8: Instruction = Instruction {
    opcode: 0x40,
    size: REG_LIT8,
};
pub const LSF_REG_REG: Instruction = Instruction {
    opcode: 0x41,
    size: REG_REG,
};
pub const RSF_REG_LIT8: Instruction = Instruction {
    opcode: 0x42,
    size: REG_LIT8,
};
pub const RSF_REG_REG: Instruction = Instruction {
    opcode: 0x43,
    size: REG_REG,
};
pub const AND_REG_LIT: Instruction = Instruction {
    opcode: 0x44,
    size: REG_LIT,
};
pub const AND_REG_REG: Instruction = Instruction {
    opcode: 0x45,
    size: REG_REG,
};
pub const OR_REG_LIT: Instruction = Instruction {
    opcode: 0x46,
    size: REG_LIT,
};
pub const OR_REG_REG: Instruction = Instruction {
    opcode: 0x47,
    size: REG_REG,
};
pub const XOR_REG_LIT: Instruction = Instruction {
    opcode: 0x48,
    size: REG_LIT,
};
pub const XOR_REG_REG: Instruction = Instruction {
    opcode: 0x49,
    size: REG_REG,
};
pub const NOT_REG: Instruction = Instruction {
    opcode: 0x4a,
    size: REG,
};

pub const JNE_LIT_MEM: Instruction = Instruction {
    opcode: 0x50,
    size: LIT_MEM,
};
pub const JNE_REG_MEM: Instruction = Instruction {
    opcode: 0x51,
    size: REG_MEM,
};
pub const JEQ_LIT_MEM: Instruction = Instruction {
    opcode: 0x52,
    size: LIT_MEM,
};
pub const JEQ_REG_MEM: Instruction = Instruction {
    opcode: 0x53,
    size: REG_MEM,
};
pub const JGT_LIT_MEM: Instruction = Instruction {
    opcode: 0x54,
    size: LIT_MEM,
};
pub const JGT_REG_MEM: Instruction = Instruction {
    opcode: 0x55,
    size: REG_MEM,
};
pub const JLT_LIT_MEM: Instruction = Instruction {
    opcode: 0x56,
    size: LIT_MEM,
};
pub const JLT_REG_MEM: Instruction = Instruction {
    opcode: 0x57,
    size: REG_MEM,
};
pub const JGE_LIT_MEM: Instruction = Instruction {
    opcode: 0x58,
    size: LIT_MEM,
};
pub const JGE_REG_MEM: Instruction = Instruction {
    opcode: 0x59,
    size: REG_MEM,
};
pub const JLE_LIT_MEM: Instruction = Instruction {
    opcode: 0x5a,
    size: LIT_MEM,
};
pub const JLE_REG_MEM: Instruction = Instruction {
    opcode: 0x5b,
    size: REG_MEM,
};

pub const HLT: Instruction = Instruction {
    opcode: 0xff,
    size: NONE,
};
