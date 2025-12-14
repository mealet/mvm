macro_rules! std_symbol {
    ($ch: literal, $typ: expr) => {
        ($ch, Token::new(String::from($ch), $typ, (0, 1).into()), )
    };
}

macro_rules! std_keyword {
    ($name: literal) => {
        (
            $name.to_string(),
            Token::new($name.to_string(), TokenType::Keyword, (0, $name.len()).into()),
        )
    };
}

macro_rules! std_instruction {
    ($name: literal) => {
        (
            $name.to_string(),
            Token::new($name.to_string(), TokenType::Instruction, (0, $name.len()).into()),
        )
    };
}

macro_rules! std_constant {
    ($name: literal) => {
        (
            $name.to_string(),
            Token::new($name.to_string(), TokenType::AsmConstant, (0, $name.len()).into()),
        )
    };
}

macro_rules! std_reg {
    ($name: literal) => {
        (
            $name.to_string(),
            Token::new($name.to_string(), TokenType::AsmReg, (0, $name.len()).into()),
        )
    };
}

pub(crate) use std_symbol;
pub(crate) use std_keyword;
pub(crate) use std_constant;
pub(crate) use std_instruction;
pub(crate) use std_reg;
