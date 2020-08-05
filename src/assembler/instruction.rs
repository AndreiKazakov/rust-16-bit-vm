use super::typed::Typed;

#[derive(Eq, PartialEq, Debug)]
pub struct Instruction2 {
    pub instruction: Instruction,
    pub args: (Typed<String>, Typed<String>),
}

#[derive(Eq, PartialEq, Debug)]
pub enum Instruction {
    MoveLitReg,
}
