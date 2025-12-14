use miette::NamedSource;

use super::{
    Source,
    error::{self, AssemblyError},
    lexer::{Token, TokenType}
};

use expressions::Expression;

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

    pub fn parse(&mut self) -> Result<Vec<Expression>, &[AssemblyError]> {
        let mut output = Vec::new();

        while self.position < self.tokens.len() {
            let expr = self.expression();

            if !matches!(expr, Expression::None) {
                output.push(expr);
            }

            if self.eof { break };
        }

        if !self.errors.is_empty() {
            return Err(&self.errors)
        }

        Ok(output)
    }
}

impl<'tokens> Parser<'tokens> {
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

    fn skip_to_new_expression(&mut self) {
        while ![TokenType::Keyword, TokenType::Label, TokenType::Instruction, TokenType::EOF].contains(&self.peek_token().token_type) {
            self.skip_token();
        }
    }
}

impl<'tokens> Parser<'tokens> {
    fn expression(&mut self) -> Expression {
        let expr_offset = self.peek_token().span.offset();
        let current = self.peek_token().clone();

        dbg!(&current);

        match current.token_type {
            TokenType::Identifier => {
                self.skip_token();
                return Expression::LabelRef(current.value, current.span);
            }

            TokenType::Instruction => {
                let mut args = Vec::new();

                self.skip_token();

                match current.value.as_str() {
                    // no arguments instructions
                    "halt" | "ret" => {
                        return Expression::Instruction {
                            name: current.value,
                            args,
                            span: current.span
                        }
                    }

                    // 1 argument instructions
                    "call" | "int" | "push8" | "push16" | "push32" |
                    "push64" | "pop8" | "pop16" | "pop32" | "pop64" |
                    "jmp" | "jz" | "jnz" => {
                        let last_arg = self.expression();
                        let last_arg_span = last_arg.get_span();

                        args.push(last_arg);

                        return Expression::Instruction {
                            name: current.value,
                            args,
                            span: error::position_to_span(
                                current.span.offset(),
                                (last_arg_span.offset() + last_arg_span.len())
                            )
                        }
                    }

                    // 2 argument instructions
                    "mov" | "frame8" | "frame16" | "frame32" | "frame64" |
                    "peek8" | "peek16" | "peek32" | "peek64" | "add" | "xadd" |
                    "sub" | "mul" | "div" | "cmp" | "je" | "jne" => {
                        args.push(self.expression());

                        if let Err(err) = self.skip_expected(TokenType::Comma) {
                            self.error(err);

                            self.skip_token();
                            self.skip_token();

                            return Expression::None;
                        }

                        let last_arg = self.expression();
                        let last_arg_span = last_arg.get_span();

                        args.push(last_arg);

                        return Expression::Instruction {
                            name: current.value,
                            args,
                            span: error::position_to_span(
                                current.span.offset(),
                                (last_arg_span.offset() + last_arg_span.len())
                            )
                        };
                    }

                    _ => unimplemented!()
                }
            }

            TokenType::EOF => {
                self.eof = true;
                Expression::None
            },

            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assembly::lexer::Lexer;
    use super::*;

    #[test]
    fn parser_section_def_test() {
        const FILENAME: &str = "test";
        const CODE: &str = "section .data";

        let mut lexer = Lexer::new(FILENAME, CODE);
        let tokens = lexer.tokenize().unwrap();

        let mut parser = Parser::new(FILENAME, CODE, &tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast,
            [
                Expression::SectionDef {
                    id: String::from(".data"),
                    span: (0, 13).into()
                }
            ]
        );
    }

    #[test]
    fn parser_label_ref_test() {
        const FILENAME: &str = "test";
        const CODE: &str = "label_ref";

        let mut lexer = Lexer::new(FILENAME, CODE);
        let tokens = lexer.tokenize().unwrap();

        let mut parser = Parser::new(FILENAME, CODE, &tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast,
            [
                Expression::LabelRef(String::from("label_ref"), (0, 9).into())
            ]
        );
    }
}
