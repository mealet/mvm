use std::collections::HashMap;
use miette::NamedSource;

use super::{Source, error::AssemblyError};
pub use token::{Token, TokenType};

mod token;
mod macros;

pub struct Lexer {
    src: Source,

    std_symbols: HashMap<char, Token>,
    std_keywords: HashMap<String, Token>,
    std_constants: HashMap<String, Token>,
    std_instructions: HashMap<String, Token>,

    input: Vec<char>,
    prev: char,
    position: usize,
}

impl Lexer {
    pub fn new(filename: impl AsRef<str>, source: impl AsRef<str>) -> Self {
        let mut lexer = Self {
            src: NamedSource::new(filename.as_ref(), source.as_ref().to_owned()),
            
            std_symbols: HashMap::from([
                macros::std_symbol!('.', TokenType::CurrentPtr),
                macros::std_symbol!(',', TokenType::Comma),
                macros::std_symbol!('[', TokenType::LBrack),
                macros::std_symbol!(']', TokenType::RBrack),

                macros::std_symbol!('+', TokenType::Operator),
                macros::std_symbol!('-', TokenType::Operator),
                macros::std_symbol!('*', TokenType::Operator),
                macros::std_symbol!('/', TokenType::Operator),
                macros::std_symbol!('%', TokenType::Operator),
                macros::std_symbol!('!', TokenType::Operator),
            ]),
            std_keywords: HashMap::from([
                macros::std_keyword!("section"),
                macros::std_keyword!("entry"),
                macros::std_keyword!("ascii"),
            ]),
            std_constants: HashMap::from([
                macros::std_constant!("r0"),
                macros::std_constant!("r1"),
                macros::std_constant!("r2"),
                macros::std_constant!("r3"),
                macros::std_constant!("r4"),
                macros::std_constant!("r5"),
                macros::std_constant!("r6"),
                macros::std_constant!("r7"),
                macros::std_constant!("r8"),
                macros::std_constant!("syscall"),
                macros::std_constant!("accumulator"),
                macros::std_constant!("instruction_ptr"),
                macros::std_constant!("stack_ptr"),
                macros::std_constant!("frame_ptr"),
                macros::std_constant!("mem_ptr"),
            ]),
            std_instructions: HashMap::from([
                macros::std_instruction!("halt"),
                macros::std_instruction!("ret"),
                macros::std_instruction!("call"),
                macros::std_instruction!("int"),
                macros::std_instruction!("mov"),

                macros::std_instruction!("push8"),
                macros::std_instruction!("push16"),
                macros::std_instruction!("push32"),
                macros::std_instruction!("push64"),

                macros::std_instruction!("pop8"),
                macros::std_instruction!("pop16"),
                macros::std_instruction!("pop32"),
                macros::std_instruction!("pop64"),

                macros::std_instruction!("frame8"),
                macros::std_instruction!("frame16"),
                macros::std_instruction!("frame32"),
                macros::std_instruction!("frame64"),

                macros::std_instruction!("peek8"),
                macros::std_instruction!("peek16"),
                macros::std_instruction!("peek32"),
                macros::std_instruction!("peek64"),

                macros::std_instruction!("add"),
                macros::std_instruction!("sub"),
                macros::std_instruction!("mul"),
                macros::std_instruction!("div"),
                macros::std_instruction!("cmp"),

                macros::std_instruction!("jmp"),
                macros::std_instruction!("jz"),
                macros::std_instruction!("jnz"),
                macros::std_instruction!("je"),
                macros::std_instruction!("jne"),
            ]),

            input: source.as_ref().chars().collect::<Vec<char>>(),
            prev: '\0',
            position: 0,
        };

        lexer
    }

    fn peek_prev(&self) -> char {
        return self.prev;
    }

    fn peek_char(&self) -> char {
        match self.input.get(self.position) {
            Some(chr) => *chr,
            None => '\0'
        }
    }

    fn next_char(&mut self) -> char {
        self.prev = self.peek_char();

        self.position += 1;
        return self.peek_char();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexer_movement_test() {
        let mut lexer = Lexer::new("test", "123");
        
        assert_eq!(lexer.peek_char(), '1');
        assert_eq!(lexer.peek_prev(), '\0');

        assert_eq!(lexer.next_char(), '2');
        assert_eq!(lexer.peek_prev(), '1');

        assert_eq!(lexer.next_char(), '3');
        assert_eq!(lexer.peek_prev(), '2');

        assert_eq!(lexer.next_char(), '\0');
        assert_eq!(lexer.peek_prev(), '3');
    }
}
