// Numeric enums do not work very well here, cause there is no good/fast way of converting u8 to enum
// Maybe would be possible to implement this as a macro
pub const MOVE_LIT_MEM: u8 = 0x09;
pub const MOVE_LIT_REG: u8 = 0x10;
pub const MOVE_REG_REG: u8 = 0x11;
pub const MOVE_REG_MEM: u8 = 0x12;
pub const MOVE_MEM_REG: u8 = 0x13;
pub const REG_ADD: u8 = 0x14;
pub const PUSH_LIT: u8 = 0x16;
pub const PUSH_REG: u8 = 0x17;
pub const POP: u8 = 0x18;
pub const CAL_LIT: u8 = 0x19;
pub const CAL_REG: u8 = 0x1a;
pub const RET: u8 = 0x1b;
pub const MOVE_REG_PTR_REG: u8 = 0x1c;
pub const MOVE_LIT_OFF_REG: u8 = 0x1d;

pub const ADD_LIT_REG: u8 = 0x30;
pub const SUB_LIT_REG: u8 = 0x31;
pub const SUB_REG_LIT: u8 = 0x32;
pub const SUB_REG_REG: u8 = 0x33;
pub const MUL_LIT_REG: u8 = 0x34;
pub const MUL_REG_REG: u8 = 0x35;
pub const INC_REG: u8 = 0x36;
pub const DEC_REG: u8 = 0x37;

pub const LST_REG_LIT: u8 = 0x40;
pub const LST_REG_REG: u8 = 0x41;
pub const RST_REG_LIT: u8 = 0x42;
pub const RST_REG_REG: u8 = 0x43;
pub const AND_REG_LIT: u8 = 0x44;
pub const AND_REG_REG: u8 = 0x45;
pub const OR_REG_LIT: u8 = 0x46;
pub const OR_REG_REG: u8 = 0x47;
pub const XOR_REG_LIT: u8 = 0x48;
pub const XOR_REG_REG: u8 = 0x49;
pub const NOT: u8 = 0x4a;

pub const JNE_LIT: u8 = 0x50;
pub const JNE_REG: u8 = 0x51;
pub const JEQ_LIT: u8 = 0x52;
pub const JEQ_REG: u8 = 0x53;
pub const JGT_LIT: u8 = 0x54;
pub const JGT_REG: u8 = 0x55;
pub const JLT_LIT: u8 = 0x56;
pub const JLT_REG: u8 = 0x57;
pub const JGE_LIT: u8 = 0x58;
pub const JGE_REG: u8 = 0x59;
pub const JLE_LIT: u8 = 0x5a;
pub const JLE_REG: u8 = 0x5b;

pub const HLT: u8 = 0xff;
