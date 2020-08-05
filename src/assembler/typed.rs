use crate::parser_combinator::core::{ParseInput, Parser};

#[derive(Eq, PartialEq, Debug)]
pub struct Typed<T> {
    pub assembly_type: Type,
    pub value: T,
}
impl<T> Typed<T> {
    pub fn new(assembly_type: Type, value: T) -> Self {
        Typed {
            assembly_type,
            value,
        }
    }
}

pub trait IntoTyped<T> {
    type X;
    fn into_typed(self, assembly_type: Type) -> Self::X;
}
impl<'a, Input: ?Sized + ParseInput, T> IntoTyped<T> for Parser<'a, Input, T> {
    type X = Parser<'a, Input, Typed<T>>;
    fn into_typed(self, assembly_type: Type) -> Parser<'a, Input, Typed<T>> {
        self.map(move |res| Typed::new(assembly_type, res))
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Type {
    Instruction,
    Ignored,
    HexLiteral,
    Variable,
    Register,
}
