use super::{
    Source,
    error::{self, AssemblyError},
    lexer::{Token, TokenType}
};

pub struct Parser<'tokens> {
    src: Source,

    tokens: &'tokens [Token],
    position: usize,

    errors: Vec<AssemblyError>,
    eof: bool
}

impl<'tokens> Parser<'tokens> {
    pub fn new(filename: impl AsRef<str>, source: impl AsRef<str>, tokens: &'tokens [Token]) -> Self {
        todo!()
    }
}
