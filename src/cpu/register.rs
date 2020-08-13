pub type Register = usize;

pub const IP: usize = 0;
pub const ACC: usize = 2;
pub const R1: usize = 4;
pub const R2: usize = 6;
pub const R3: usize = 8;
pub const R4: usize = 10;
pub const R5: usize = 12;
pub const R6: usize = 14;
pub const R7: usize = 16;
pub const R8: usize = 18;
pub const SP: usize = 20;
pub const FP: usize = 22;
pub const LIST: [usize; 12] = [IP, ACC, R1, R2, R3, R4, R5, R6, R7, R8, SP, FP];
pub const GENERAL_PURPOSE_LIST: [usize; 8] = [R1, R2, R3, R4, R5, R6, R7, R8];
pub const SIZE: usize = LIST.len() * 2;

pub fn get_from_string(s: &str) -> usize {
    match s {
        "IP" => IP,
        "ACC" => ACC,
        "R1" => R1,
        "R2" => R2,
        "R3" => R3,
        "R4" => R4,
        "R5" => R5,
        "R6" => R6,
        "R7" => R7,
        "R8" => R8,
        "SP" => SP,
        "FP" => FP,
        x => panic!("Unrecognized register {}", x)
    }
}