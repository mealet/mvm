use miette::NamedSource;

use super::{
    Source,
    error::{self, AssemblyError},
    lexer::{Token, TokenType}
};

pub mod expressions;

const BINARY_OPERATORS: [&'static str; 5] = ["+", "-", "*", "/", "%"];
const PRIORITY_BINARY_OPERATORS: [&'static str; 3] = ["*", "/", "%"];

pub struct Parser<'tokens> {
    src: Source,

    tokens: &'tokens [Token],
    eof_token: Token,
    position: usize,

    errors: Vec<AssemblyError>,
    eof: bool
}

impl<'tokens> Parser<'tokens> {
    pub fn new(filename: impl AsRef<str>, source: impl AsRef<str>, tokens: &'tokens [Token]) -> Self {
        Self {
            src: NamedSource::new(filename, source.as_ref().to_owned()),
            tokens,
            position: 0,
            errors: Vec::new(),
            eof_token: Token::new(String::new(), TokenType::EOF, (0, 0).into()),
            eof: false,
        }
    }

    fn error(&mut self, error: AssemblyError) {
        self.errors.push(error);
    }

    fn peek_token(&mut self) -> &Token {
        match self.tokens.get(self.position) {
            Some(token) => token,
            None => {
                self.eof = true;
                &self.eof_token
            }
        }
    }
    
    fn next_token(&mut self) -> &Token {
        self.position += 1;
        return self.peek_token();
    }

    fn skip_token(&mut self) {
        let _ = self.next_token();
    }

    fn expect(&mut self, expected: TokenType) -> bool {
        self.peek_token().token_type == expected 
    }

    fn skip_expected(&mut self, expected: TokenType) -> Result<(), AssemblyError> {
        if self.peek_token().token_type == expected {
            self.skip_token();
            return Ok(());
        }

        Err(AssemblyError::UnexpectedToken {
            expected: expected.to_string().to_lowercase(),
            found: self.peek_token().token_type.to_string().to_lowercase(),
            src: self.src.clone(),
            span: self.peek_token().span
        })
    }
}
