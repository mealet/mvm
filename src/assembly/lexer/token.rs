use miette::SourceSpan;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Token {
    pub value: String,
    pub token_type: TokenType,
    pub span: SourceSpan,
}

impl Token {
    pub fn new(value: String, token_type: TokenType, span: SourceSpan) -> Self {
        Self {
            value,
            token_type,
            span,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TokenType {
    Identifier,  // abc
    Instruction, // mov, jmp, ...
    Keyword,     // section, entry, ...
    Label,       // label:
    CurrentPtr,  // .

    Constant,       // $123, $0xFF, $0b101
    StringConstant, // "hello"
    AsmConstant,    // $syscall, ...
    AsmReg,         // %r0, %r1, %call, ...
    Operator,       // +, -, *, /, %, !, ...

    Comma,  // ,
    LBrack, // [
    RBrack, // ]

    Eof,       // end of file
    Undefined, // undefined token type
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
