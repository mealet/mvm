use miette::NamedSource;
use std::collections::HashMap;

use super::{
    Source,
    error::{self, AssemblyError},
};
pub use token::{Token, TokenType};

mod macros;
mod token;

const ALLOWED_ID_CHARS: [char; 3] = ['_', '.', ':'];

pub struct Lexer {
    src: Source,
    position: usize,

    std_symbols: HashMap<char, Token>,
    std_keywords: HashMap<String, Token>,
    std_constants: HashMap<String, Token>,
    std_registers: HashMap<String, Token>,
    std_instructions: HashMap<String, Token>,

    input: Vec<char>,
    prev: char,

    errors: Vec<AssemblyError>,
}

impl Lexer {
    pub fn new(filename: impl AsRef<str>, source: impl AsRef<str>) -> Self {
        Self {
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
            std_registers: HashMap::from([
                macros::std_reg!("r0"),
                macros::std_reg!("r1"),
                macros::std_reg!("r2"),
                macros::std_reg!("r3"),
                macros::std_reg!("r4"),
                macros::std_reg!("r5"),
                macros::std_reg!("r6"),
                macros::std_reg!("r7"),
                macros::std_reg!("r8"),
                macros::std_reg!("call"),
                macros::std_reg!("accumulator"),
                macros::std_reg!("instruction_ptr"),
                macros::std_reg!("stack_ptr"),
                macros::std_reg!("frame_ptr"),
                macros::std_reg!("mem_ptr"),
            ]),
            std_constants: HashMap::from([
                // interrupts
                macros::std_constant!("syscall"), // kept for old examples
                macros::std_constant!("int_syscall"),
                macros::std_constant!("int_accinc"),
                // syscalls
                macros::std_constant!("sys_exit"),
                macros::std_constant!("sys_read"),
                macros::std_constant!("sys_write"),
                macros::std_constant!("sys_alloc"),
                macros::std_constant!("sys_free"),
            ]),
            std_instructions: HashMap::from([
                macros::std_instruction!("halt"),
                macros::std_instruction!("ret"),
                macros::std_instruction!("call"),
                macros::std_instruction!("int"),
                macros::std_instruction!("dbg"),
                macros::std_instruction!("mov"),
                macros::std_instruction!("load8"),
                macros::std_instruction!("load16"),
                macros::std_instruction!("load32"),
                macros::std_instruction!("load64"),
                macros::std_instruction!("store8"),
                macros::std_instruction!("store16"),
                macros::std_instruction!("store32"),
                macros::std_instruction!("store64"),
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
            errors: Vec::new(),
        }
    }

    #[allow(unused)]
    fn peek_prev(&self) -> char {
        self.prev
    }

    fn peek_char(&self) -> char {
        match self.input.get(self.position) {
            Some(chr) => *chr,
            None => '\0',
        }
    }

    fn next_char(&mut self) -> char {
        self.prev = self.peek_char();

        self.position += 1;
        self.peek_char()
    }

    fn skip_char(&mut self) {
        let _ = self.next_char();
    }

    fn skip_to_whitespace(&mut self) {
        while !self.peek_char().is_whitespace() && self.peek_char() != '\0' {
            self.skip_char();
        }
    }

    fn error(&mut self, error: AssemblyError) {
        self.errors.push(error);
    }
}

impl Lexer {
    pub fn tokenize(&mut self) -> Result<Vec<Token>, &[AssemblyError]> {
        let mut output: Vec<Token> = Vec::new();

        while !self.is_eof() {
            match self.peek_char() {
                chr if chr.is_whitespace() || ['\n', '\r'].contains(&chr) => self.skip_char(),

                '\0' => break,

                // comment
                ';' => {
                    while self.peek_char() != '\n' && self.peek_char() != '\0' {
                        self.skip_char();
                    }
                }

                // constant
                '$' => {
                    let span_start = self.position;
                    let after_prefix = self.next_char();

                    match after_prefix {
                        digit if digit.is_ascii_digit() => match self.get_number() {
                            Ok(mut token) => {
                                token.span = (span_start, token.span.len() + 1).into();
                                output.push(token);
                            }
                            Err(error) => {
                                self.error(*error);
                            }
                        },

                        id if id.is_ascii_alphabetic() => {
                            let mut id = String::new();
                            let id_offset = self.position;

                            while self.peek_char().is_ascii_alphanumeric()
                                || ALLOWED_ID_CHARS.contains(&self.peek_char())
                            {
                                id.push(self.peek_char());
                                self.skip_char();
                            }

                            if let Some(token) = self.std_constants.get(&id) {
                                let mut token = token.clone();
                                token.span = (span_start, token.span.len() + 1).into();

                                output.push(token);
                            } else {
                                self.error(AssemblyError::InvalidConstant {
                                    error: format!("Assembly constant \"{id}\" not found"),
                                    label: "verify this identifier".to_string(),
                                    src: self.src.clone(),
                                    span: error::position_to_span(id_offset, self.position),
                                });
                            }
                        }

                        _ => {
                            let err_offset = self.position;
                            self.skip_to_whitespace();

                            self.error(AssemblyError::InvalidConstant {
                                error: "Undefined constant sequence found after `$` prefix"
                                    .to_string(),
                                label: "ensure that this constant is valid".to_string(),
                                src: self.src.clone(),
                                span: error::position_to_span(err_offset, self.position),
                            });
                        }
                    }
                }

                // register
                '%' => {
                    let span_start = self.position;

                    self.skip_char();

                    let mut id = String::new();
                    let id_offset = self.position;

                    while self.peek_char().is_ascii_alphanumeric()
                        || ALLOWED_ID_CHARS.contains(&self.peek_char())
                    {
                        id.push(self.peek_char());
                        self.skip_char();
                    }

                    if let Some(token) = self.std_registers.get(&id) {
                        let mut token = token.clone();
                        token.span = (span_start, token.span.len() + 1).into();

                        output.push(token);
                    } else {
                        self.error(AssemblyError::InvalidConstant {
                            error: format!("Assembly register \"{id}\" not found"),
                            label: "verify this identifier".to_string(),
                            src: self.src.clone(),
                            span: error::position_to_span(id_offset, self.position),
                        });
                    }
                }

                '"' => {
                    let mut value = String::new();
                    let string_offset = self.position;

                    self.skip_char();

                    while self.peek_char() != '"' {
                        if self.peek_char() == '\\' {
                            self.skip_char();

                            let character_escape = Self::character_escape(self.peek_char());

                            if let Some(character_escape) = character_escape {
                                value.push(character_escape);
                                self.skip_char();
                                continue;
                            }

                            self.error(AssemblyError::UnknownCharacterEscape {
                                escape: self.peek_char(),
                                src: self.src.clone(),
                                span: (self.position - 1, 2).into(),
                            })
                        }

                        value.push(self.peek_char());
                        self.skip_char();
                    }

                    self.skip_char(); // skipping the `"` char

                    output.push(Token::new(
                        value,
                        TokenType::StringConstant,
                        error::position_to_span(string_offset, self.position),
                    ));
                }

                symbol if self.std_symbols.contains_key(&symbol) => {
                    let next = self.next_char();
                    self.position -= 1;

                    if symbol == '.' && next.is_ascii_alphabetic() {
                        let mut id = String::new();
                        let id_offset = self.position;

                        while self.peek_char().is_ascii_alphanumeric()
                            || ALLOWED_ID_CHARS.contains(&self.peek_char())
                        {
                            id.push(self.peek_char());
                            self.skip_char();
                        }

                        if !id.is_empty() {
                            let token_type = if id.ends_with(":") {
                                TokenType::Label
                            } else {
                                TokenType::Identifier
                            };
                            output.push(Token::new(
                                id,
                                token_type,
                                error::position_to_span(id_offset, self.position),
                            ));
                            continue;
                        }
                    }

                    let mut token = self.std_symbols.get(&symbol).unwrap().clone();
                    token.span = (self.position, token.span.len()).into();

                    output.push(token);
                    self.skip_char();
                }

                digit if digit.is_ascii_digit() => {
                    let span_offset = self.position;

                    self.skip_to_whitespace();

                    let span_end = if self.peek_char() == ' ' {
                        self.position
                    } else {
                        self.position - 1
                    };

                    self.error(AssemblyError::InvalidConstant {
                        error: "Numerical constants are not allowed without `$` prefix".to_string(),
                        label: "add the `$` prefix before constant here".to_string(),
                        src: self.src.clone(),
                        span: error::position_to_span(span_offset, span_end),
                    });
                }

                id_character if id_character.is_ascii_alphabetic() || id_character == '_' => {
                    let mut id = String::new();
                    let id_offset = self.position;

                    while self.peek_char().is_ascii_alphanumeric()
                        || ALLOWED_ID_CHARS.contains(&self.peek_char())
                    {
                        id.push(self.peek_char());
                        self.skip_char();
                    }

                    if let Some(keyword) = self.std_keywords.get(&id) {
                        let mut token = keyword.clone();
                        token.span = (id_offset, token.span.len()).into();

                        output.push(token);

                        continue;
                    }

                    if let Some(instruction) = self.std_instructions.get(&id) {
                        let mut token = instruction.clone();
                        token.span = (id_offset, token.span.len()).into();

                        output.push(token);
                        continue;
                    }

                    let token_type = if id.ends_with(":") {
                        TokenType::Label
                    } else {
                        TokenType::Identifier
                    };
                    output.push(Token::new(
                        id,
                        token_type,
                        error::position_to_span(id_offset, self.position),
                    ));
                }

                unknown_character => {
                    self.error(AssemblyError::UnknownCharacter {
                        character: unknown_character,
                        src: self.src.clone(),
                        span: (self.position, 1).into(),
                    });
                    self.skip_char();
                }
            }
        }

        if !self.errors.is_empty() {
            return Err(&self.errors);
        }

        if output
            .last()
            .unwrap_or(&Token::new(
                Default::default(),
                TokenType::Undefined,
                (0, 0).into(),
            ))
            .token_type
            != TokenType::Eof
        {
            output.push(Token::new(
                Default::default(),
                TokenType::Eof,
                (0, 0).into(),
            ));
        }

        Ok(output)
    }
}

impl Lexer {
    fn is_eof(&self) -> bool {
        self.peek_char() == '\0'
    }

    fn character_escape(escape: char) -> Option<char> {
        match escape {
            '0' => Some('\0'),
            'n' => Some('\n'),
            't' => Some('\t'),
            'r' => Some('\r'),
            '\\' => Some('\\'),
            _ => None,
        }
    }

    fn get_number(&mut self) -> Result<Token, Box<AssemblyError>> {
        #[derive(PartialEq, Debug)]
        enum ParseMode {
            Decimal,
            Hexadecimal,
            Binary,
            Float,
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
                            return Err(Box::new(AssemblyError::InvalidConstant {
                                error: "Invalid binary number constant found".to_string(),
                                label: format!("detected constant type is: {mode:?}")
                                    .to_lowercase(),
                                src: self.src.clone(),
                                span: error::position_to_span(span_start, self.position),
                            }));
                        }

                        mode = ParseMode::Binary;
                        self.skip_char();
                        continue;
                    }

                    'x' => {
                        if mode != ParseMode::Decimal || !value.is_empty() {
                            return Err(Box::new(AssemblyError::InvalidConstant {
                                error: "Invalid hexadecimal number constant found".to_string(),
                                label: format!("detected constant type is: {mode:?}")
                                    .to_lowercase(),
                                src: self.src.clone(),
                                span: error::position_to_span(span_start, self.position),
                            }));
                        }

                        mode = ParseMode::Hexadecimal;
                        self.skip_char();
                        continue;
                    }

                    _ => {
                        value.push('0');
                        continue;
                    }
                }
            }

            match self.peek_char() {
                '_' => {}
                '.' => {
                    if mode != ParseMode::Decimal {
                        return Err(Box::new(AssemblyError::InvalidConstant {
                            error: "Invalid floating number constant found".to_string(),
                            label: format!("detected constant type is: {mode:?}").to_lowercase(),
                            src: self.src.clone(),
                            span: error::position_to_span(span_start, self.position),
                        }));
                    }

                    mode = ParseMode::Float;
                    value.push('.');
                }

                chr => value.push(chr),
            }

            self.skip_char();
        }

        if value.is_empty() {
            return Ok(Token::new(
                0.to_string(),
                TokenType::Constant,
                error::position_to_span(span_start, self.position),
            ));
        }

        match mode {
            ParseMode::Decimal => {
                let result = value.trim().parse::<i64>();

                if let Err(error) = result {
                    return Err(Box::new(AssemblyError::ConstantParseError {
                        const_type: format!("{mode:?}").to_lowercase(),
                        parser_error: error.to_string(),
                        src: self.src.clone(),
                        span: error::position_to_span(span_start, self.position),
                    }));
                }

                Ok(Token::new(
                    result.unwrap().to_string(),
                    TokenType::Constant,
                    error::position_to_span(span_start, self.position),
                ))
            }

            ParseMode::Binary => {
                let result = i64::from_str_radix(value.trim(), 2);

                if let Err(error) = result {
                    return Err(Box::new(AssemblyError::ConstantParseError {
                        const_type: format!("{mode:?}").to_lowercase(),
                        parser_error: error.to_string(),
                        src: self.src.clone(),
                        span: error::position_to_span(span_start, self.position),
                    }));
                }

                Ok(Token::new(
                    result.unwrap().to_string(),
                    TokenType::Constant,
                    error::position_to_span(span_start, self.position),
                ))
            }

            ParseMode::Hexadecimal => {
                let result = i64::from_str_radix(value.trim(), 16);

                if let Err(error) = result {
                    return Err(Box::new(AssemblyError::ConstantParseError {
                        const_type: format!("{mode:?}").to_lowercase(),
                        parser_error: error.to_string(),
                        src: self.src.clone(),
                        span: error::position_to_span(span_start, self.position),
                    }));
                }

                Ok(Token::new(
                    result.unwrap().to_string(),
                    TokenType::Constant,
                    error::position_to_span(span_start, self.position),
                ))
            }

            ParseMode::Float => {
                let result = value.trim().parse::<f64>();

                if let Err(error) = result {
                    return Err(Box::new(AssemblyError::ConstantParseError {
                        const_type: format!("{mode:?}").to_lowercase(),
                        parser_error: error.to_string(),
                        src: self.src.clone(),
                        span: error::position_to_span(span_start, self.position),
                    }));
                }

                Ok(Token::new(
                    result.unwrap().to_string(),
                    TokenType::Constant,
                    error::position_to_span(span_start, self.position),
                ))
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

    #[test]
    fn lexer_std_symbols_test() {
        let mut lexer = Lexer::new("test", ".,[]");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(
            tokens,
            [
                Token::new(String::from("."), TokenType::CurrentPtr, (0, 1).into()),
                Token::new(String::from(","), TokenType::Comma, (1, 1).into()),
                Token::new(String::from("["), TokenType::LBrack, (2, 1).into()),
                Token::new(String::from("]"), TokenType::RBrack, (3, 1).into()),
                Token::new(String::from(""), TokenType::Eof, (0, 0).into()),
            ]
        );
    }

    #[test]
    fn lexer_numbers_test() {
        let mut lexer = Lexer::new("test", "$123 $0xFF $0b1111 $1.23");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(
            tokens,
            [
                Token::new(String::from("123"), TokenType::Constant, (0, 4).into()),
                Token::new(String::from("255"), TokenType::Constant, (5, 5).into()),
                Token::new(String::from("15"), TokenType::Constant, (11, 7).into()),
                Token::new(String::from("1.23"), TokenType::Constant, (19, 5).into()),
                Token::new(String::from(""), TokenType::Eof, (0, 0).into()),
            ]
        );
    }

    #[test]
    fn lexer_registers_constants_test() {
        let mut lexer = Lexer::new(
            "test",
            "%r0 %r1 %r2 %r3 %r4 %r5 %r6 %r7 %r8 %call %accumulator %instruction_ptr %stack_ptr %frame_ptr %mem_ptr",
        );
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(
            tokens,
            [
                Token::new(String::from("r0"), TokenType::AsmReg, (0, 3).into()),
                Token::new(String::from("r1"), TokenType::AsmReg, (4, 3).into()),
                Token::new(String::from("r2"), TokenType::AsmReg, (8, 3).into()),
                Token::new(String::from("r3"), TokenType::AsmReg, (12, 3).into()),
                Token::new(String::from("r4"), TokenType::AsmReg, (16, 3).into()),
                Token::new(String::from("r5"), TokenType::AsmReg, (20, 3).into()),
                Token::new(String::from("r6"), TokenType::AsmReg, (24, 3).into()),
                Token::new(String::from("r7"), TokenType::AsmReg, (28, 3).into()),
                Token::new(String::from("r8"), TokenType::AsmReg, (32, 3).into()),
                Token::new(String::from("call"), TokenType::AsmReg, (36, 5).into()),
                Token::new(
                    String::from("accumulator"),
                    TokenType::AsmReg,
                    (42, 12).into()
                ),
                Token::new(
                    String::from("instruction_ptr"),
                    TokenType::AsmReg,
                    (55, 16).into()
                ),
                Token::new(
                    String::from("stack_ptr"),
                    TokenType::AsmReg,
                    (72, 10).into()
                ),
                Token::new(
                    String::from("frame_ptr"),
                    TokenType::AsmReg,
                    (83, 10).into()
                ),
                Token::new(String::from("mem_ptr"), TokenType::AsmReg, (94, 8).into()),
                Token::new(String::from(""), TokenType::Eof, (0, 0).into()),
            ]
        );
    }

    #[test]
    fn lexer_identifiers_test() {
        let mut lexer = Lexer::new("test", "asd .data he1");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(
            tokens,
            [
                Token::new(String::from("asd"), TokenType::Identifier, (0, 3).into()),
                Token::new(String::from(".data"), TokenType::Identifier, (4, 5).into()),
                Token::new(String::from("he1"), TokenType::Identifier, (10, 3).into()),
                Token::new(String::from(""), TokenType::Eof, (0, 0).into()),
            ]
        );
    }

    #[test]
    fn lexer_keywords_test() {
        let mut lexer = Lexer::new("test", "section entry ascii");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(
            tokens,
            [
                Token::new(String::from("section"), TokenType::Keyword, (0, 7).into()),
                Token::new(String::from("entry"), TokenType::Keyword, (8, 5).into()),
                Token::new(String::from("ascii"), TokenType::Keyword, (14, 5).into()),
                Token::new(String::from(""), TokenType::Eof, (0, 0).into()),
            ]
        );
    }

    #[test]
    fn lexer_labels_test() {
        let mut lexer = Lexer::new("test", "label: asd:");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(
            tokens,
            [
                Token::new(String::from("label:"), TokenType::Label, (0, 6).into()),
                Token::new(String::from("asd:"), TokenType::Label, (7, 4).into()),
                Token::new(String::from(""), TokenType::Eof, (0, 0).into()),
            ]
        );
    }

    #[test]
    fn lexer_string_test() {
        let mut lexer = Lexer::new("test", "\"hello\"");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(
            tokens,
            [
                Token::new(
                    String::from("hello"),
                    TokenType::StringConstant,
                    (0, 7).into()
                ),
                Token::new(String::from(""), TokenType::Eof, (0, 0).into()),
            ]
        );
    }
}
