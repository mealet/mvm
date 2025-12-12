use std::collections::HashMap;
use miette::NamedSource;

use super::{
    Source,
    error::{self, AssemblyError}
};
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

impl Lexer {
    pub fn tokenize(&mut self) -> Result<Token, AssemblyError> {
        match self.peek_char() {
            chr if chr.is_ascii_whitespace() || chr == '\r' => {
                while self.peek_char().is_ascii_whitespace() || self.peek_char() == '\r' {
                    let _ = self.next_char();
                }

                self.tokenize()
            }

            // comment
            ';' => {
                while self.peek_char() != '\n' && self.peek_char() != '\0' {
                    let _ = self.next_char();
                }

                self.tokenize()
            }

            unknown_character => {
                Err(AssemblyError::UnknownCharacter {
                    character: unknown_character,
                    src: self.src.clone(),
                    span: (self.position, 1).into()
                })
            }
        }
    }
}

impl Lexer {
    fn character_escape(escape: char) -> Option<char> {
        match escape {
            '0' => Some('\0'),
            'n' => Some('\n'),
            't' => Some('\t'),
            'r' => Some('\r'),
            '\\' => Some('\\'),
            _ => None
        }
    }

    fn get_number(&mut self) -> Result<Token, AssemblyError> {
        #[derive(PartialEq, Debug)]
        enum ParseMode {
            Decimal,
            Hexadecimal,
            Binary,
            Float
        }

        let mut value = String::new();
        let mut mode = ParseMode::Decimal;
        let span_start = self.position;

        while self.peek_char().is_ascii_digit()
            || self.peek_char().is_ascii_hexdigit()
            || ['_', '.', 'x', 'b'].contains(&self.peek_char())
        {
            if self.peek_char() == '0' {
                let after_zero = self.next_char();

                match after_zero {
                    'b' => {
                        if mode != ParseMode::Decimal || !value.is_empty() {
                            return Err(AssemblyError::InvalidNumberConstant {
                                const_type: format!("{mode:?}").to_lowercase(),
                                src: self.src.clone(),
                                span: error::position_to_span(span_start, self.position)
                            })
                        }

                        mode = ParseMode::Binary;
                        let _ = self.next_char();
                        continue;
                    },

                    'x' => {
                        if mode != ParseMode::Decimal || !value.is_empty() {
                            return Err(AssemblyError::InvalidNumberConstant {
                                const_type: format!("{mode:?}").to_lowercase(),
                                src: self.src.clone(),
                                span: error::position_to_span(span_start, self.position)
                            })
                        }

                        mode = ParseMode::Hexadecimal;
                        let _ = self.next_char();
                        continue;
                    },

                    _ => {
                        value.push('0');
                        continue;
                    }
                }
            }

            match self.peek_char() {
                '_' => {},
                '.' => {
                    if mode != ParseMode::Decimal {
                        return Err(AssemblyError::InvalidNumberConstant {
                            const_type: format!("{mode:?}").to_lowercase(),
                            src: self.src.clone(),
                            span: error::position_to_span(span_start, self.position)
                        });
                    }

                    mode = ParseMode::Float;
                    value.push('.');
                }

                chr => value.push(chr)
            }

            let _ =self.next_char();
        }

        if value.is_empty() {
            return Ok(Token::new(0.to_string(), TokenType::Constant, error::position_to_span(span_start, self.position)));
        }

        match mode {
            ParseMode::Decimal => {
                let result = value.trim().parse::<i64>();

                if let Err(error) = result {
                    return Err(AssemblyError::ConstantParseError {
                        const_type: format!("{mode:?}").to_lowercase(),
                        parser_error: error.to_string(),
                        src: self.src.clone(),
                        span: error::position_to_span(span_start, self.position)
                    });
                }

                return Ok(Token::new(
                    result.unwrap().to_string(),
                    TokenType::Constant,
                    error::position_to_span(span_start, self.position)
                ));
            }

            ParseMode::Binary => {
                let result = i64::from_str_radix(value.trim(), 2);

                if let Err(error) = result {
                    return Err(AssemblyError::ConstantParseError {
                        const_type: format!("{mode:?}").to_lowercase(),
                        parser_error: error.to_string(),
                        src: self.src.clone(),
                        span: error::position_to_span(span_start, self.position)
                    });
                }

                return Ok(Token::new(
                    result.unwrap().to_string(),
                    TokenType::Constant,
                    error::position_to_span(span_start, self.position)
                ));
            }

            ParseMode::Hexadecimal => {
                let result = i64::from_str_radix(value.trim(), 16);

                if let Err(error) = result {
                    return Err(AssemblyError::ConstantParseError {
                        const_type: format!("{mode:?}").to_lowercase(),
                        parser_error: error.to_string(),
                        src: self.src.clone(),
                        span: error::position_to_span(span_start, self.position)
                    });
                }

                return Ok(Token::new(
                    result.unwrap().to_string(),
                    TokenType::Constant,
                    error::position_to_span(span_start, self.position)
                ));
            }

            ParseMode::Float => {
                let result = value.trim().parse::<f64>();

                if let Err(error) = result {
                    return Err(AssemblyError::ConstantParseError {
                        const_type: format!("{mode:?}").to_lowercase(),
                        parser_error: error.to_string(),
                        src: self.src.clone(),
                        span: error::position_to_span(span_start, self.position)
                    });
                }

                return Ok(Token::new(
                    result.unwrap().to_string(),
                    TokenType::Constant,
                    error::position_to_span(span_start, self.position)
                ));
            }
        }
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

    #[test]
    fn lexer_get_number_basic_test() {
        let mut lexer = Lexer::new("test", "123");
        let number_result = lexer.get_number();

        assert!(number_result.is_ok());
        
        let number = number_result.unwrap();

        assert_eq!(number.value, "123");
        assert_eq!(number.token_type, TokenType::Constant);
        assert_eq!(number.span.offset(), 0);
        assert_eq!(number.span.len(), 3);
    }

    #[test]
    fn lexer_get_number_binary_test() {
        let mut lexer = Lexer::new("test", "0b1111");
        let number_result = lexer.get_number();

        assert!(number_result.is_ok());
        
        let number = number_result.unwrap();

        assert_eq!(number.value, "15");
        assert_eq!(number.token_type, TokenType::Constant);
        assert_eq!(number.span.offset(), 0);
        assert_eq!(number.span.len(), 6);
    }

    #[test]
    fn lexer_get_number_hexadecimal_test() {
        let mut lexer = Lexer::new("test", "0xFF");
        let number_result = lexer.get_number();

        assert!(number_result.is_ok());
        
        let number = number_result.unwrap();

        assert_eq!(number.value, "255");
        assert_eq!(number.token_type, TokenType::Constant);
        assert_eq!(number.span.offset(), 0);
        assert_eq!(number.span.len(), 4);
    }

    #[test]
    fn lexer_get_number_float_test() {
        let mut lexer = Lexer::new("test", "0.314000");
        let number_result = lexer.get_number();

        assert!(number_result.is_ok());
        
        let number = number_result.unwrap();

        assert_eq!(number.value, "0.314");
        assert_eq!(number.token_type, TokenType::Constant);
        assert_eq!(number.span.offset(), 0);
        assert_eq!(number.span.len(), 8);
    }


    #[test]
    fn lexer_get_number_error_1_test() {
        let mut lexer = Lexer::new("test", "1.1.");
        let number_result = lexer.get_number();

        assert!(number_result.is_err());
    }

    #[test]
    fn lexer_get_number_error_2_test() {
        let mut lexer = Lexer::new("test", "0bb");
        let number_result = lexer.get_number();

        assert!(number_result.is_err());
    }

    #[test]
    fn lexer_get_number_error_3_test() {
        let mut lexer = Lexer::new("test", "0xx");
        let number_result = lexer.get_number();

        assert!(number_result.is_err());
    }
}
