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
    fn term(&mut self) -> Expression {
        let expr_offset = self.peek_token().span.offset();
        let current = self.peek_token().clone();

        match current.token_type {
            TokenType::Identifier => {
                self.skip_token();
                return Expression::LabelRef(current.value, current.span);
            },

            TokenType::Constant => {
                self.skip_token();

                let value = current.value.parse::<u64>().unwrap_or_else(|err| {
                    self.error(AssemblyError::ConstantParseError {
                        const_type: format!("u64"),
                        parser_error: err.to_string(),
                        src: self.src.clone(),
                        span: current.span
                    });

                    0
                });

                return Expression::UIntConstant(value, current.span)
            },

            TokenType::StringConstant => {
                self.skip_token();

                return Expression::StringConstant(current.value, current.span)
            },

            TokenType::AsmConstant => {
                self.skip_token();

                let value = current.value.strip_prefix("$").unwrap_or(current.value.as_str());
                return Expression::AsmConstant(value.to_string(), current.span);
            },

            TokenType::AsmReg => {
                self.skip_token();

                let value = current.value.strip_prefix("%").unwrap_or(current.value.as_str());
                return Expression::AsmReg(value.to_string(), current.span);
            },

            TokenType::CurrentPtr => {
                self.skip_token();
                return Expression::CurrentPtr(current.span);
            }

            _ => Expression::None
        }
    }

    fn expression(&mut self) -> Expression {
        let expr_offset = self.peek_token().span.offset();
        let node = self.term();

        let current = self.peek_token().clone();

        if let Expression::None = node {
            return match current.token_type {
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

                TokenType::Keyword => {
                    match current.value.as_str() {
                        "section" => {
                            let identifier = self.next_token().clone();

                            if !self.expect(TokenType::Identifier) {
                                self.error(AssemblyError::UnexpectedToken {
                                    expected: TokenType::Identifier.to_string().to_lowercase(),
                                    found: identifier.token_type.to_string().to_lowercase(),
                                    src: self.src.clone(),
                                    span: identifier.span
                                });
                                self.skip_token();
                                return Expression::None;
                            }
                            
                            self.skip_token();
                            let span_end = identifier.span.offset() + identifier.span.len();

                            return Expression::SectionDef {
                                id: identifier.value,
                                span: error::position_to_span(expr_offset, span_end)
                            }
                        },
                        "entry" => {
                            let identifier = self.next_token().clone();

                            if !self.expect(TokenType::Identifier) {
                                self.error(AssemblyError::UnexpectedToken {
                                    expected: TokenType::Identifier.to_string().to_lowercase(),
                                    found: identifier.token_type.to_string().to_lowercase(),
                                    src: self.src.clone(),
                                    span: identifier.span
                                });
                                self.skip_token();
                                return Expression::None;
                            }
                            
                            self.skip_token();
                            let span_end = identifier.span.offset() + identifier.span.len();

                            return Expression::EntryDef {
                                label: identifier.value,
                                span: error::position_to_span(expr_offset, span_end)
                            }
                        },
                        "ascii" => {
                            let str_constant = self.next_token().clone();

                            if !self.expect(TokenType::StringConstant) {
                                self.error(AssemblyError::UnexpectedToken {
                                    expected: TokenType::StringConstant.to_string().to_lowercase(),
                                    found: str_constant.token_type.to_string().to_lowercase(),
                                    src: self.src.clone(),
                                    span: str_constant.span
                                });
                                self.skip_token();
                                return Expression::None;
                            }

                            self.skip_token();
                            let span_end = str_constant.span.offset() + str_constant.span.len();

                            return Expression::Directive {
                                directive: String::from("ascii"),
                                args: vec![
                                    Expression::StringConstant(str_constant.value, str_constant.span)
                                ],
                                span: error::position_to_span(expr_offset, span_end)
                            }
                        },

                        _ => unimplemented!()
                    }
                }

                TokenType::Label => {
                    self.skip_token();

                    let value = current.value.strip_suffix(":").unwrap_or(&current.value);

                    return Expression::LabelDef {
                        id: value.to_string(),
                        span: current.span
                    }
                }

                TokenType::LBrack => {
                    let expr_start = self.position;
                    self.skip_token();

                    let expr = self.expression();
                    let expr_length = expr.get_span().len() + 2;

                    self.skip_expected(TokenType::RBrack);

                    return Expression::ComptimeExpr {
                        expr: Box::new(expr),
                        span: (expr_start, expr_length).into()
                    }
                },

                TokenType::EOF => {
                    self.eof = true;
                    Expression::None
                },

                _ => {
                    self.skip_token();

                    self.error(AssemblyError::UnknownExpression {
                        error: format!("Unknown expression found"),
                        src: self.src.clone(),
                        span: current.span
                    });

                    Expression::None
                }
            }
        }

        // expressions with non-none nodes

        match current.token_type {
            TokenType::Operator => {
                if !BINARY_OPERATORS.contains(&current.value.as_str()) {
                    self.error(AssemblyError::UnsupportedExpression {
                        error: format!("Unsupported expression with operator found"),
                        label: format!("operator {} is not supported", current.value),
                        src: self.src.clone(),
                        span: current.span
                    });
                }

                let op = current.value.clone();
                self.skip_token();

                let lhs = Box::new(node);
                let rhs = Box::new(self.expression());

                let rhs_span = rhs.get_span();
                let span_end = rhs_span.offset() + rhs_span.len();

                if PRIORITY_BINARY_OPERATORS.contains(&op.as_str()) {
                    let new_node = rhs.clone();
                    let old_lhs = lhs.clone();

                    if let Expression::BinaryExpr {
                        op,
                        lhs,
                        rhs,
                        span,
                    } = *new_node
                    {
                        let lhs_new = old_lhs;
                        let rhs_new = lhs;

                        let priority_node = Expression::BinaryExpr {
                            op: current.value,
                            lhs: lhs_new,
                            rhs: rhs_new,
                            span,
                        };

                        return Expression::BinaryExpr {
                            op,
                            lhs: Box::new(priority_node),
                            rhs,
                            span: error::position_to_span(expr_offset, span_end),
                        };
                    }
                }

                return Expression::BinaryExpr {
                    op,
                    lhs,
                    rhs,
                    span: error::position_to_span(expr_offset, span_end)
                };
            }

            TokenType::EOF => {
                self.eof = true;
                node
            },

            _ => node
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
                    span: (
                        0,
                        "section .data".len()
                    ).into()
                }
            ]
        );
    }

    #[test]
    fn parser_entry_def_test() {
        const FILENAME: &str = "test";
        const CODE: &str = "entry _start";

        let mut lexer = Lexer::new(FILENAME, CODE);
        let tokens = lexer.tokenize().unwrap();

        let mut parser = Parser::new(FILENAME, CODE, &tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast,
            [
                Expression::EntryDef {
                    label: String::from("_start"),
                    span: (
                        0,
                        "entry _start".len()
                    ).into()
                }
            ]
        );
    }

    #[test]
    fn parser_label_def_test() {
        const FILENAME: &str = "test";
        const CODE: &str = "label_def:";

        let mut lexer = Lexer::new(FILENAME, CODE);
        let tokens = lexer.tokenize().unwrap();

        let mut parser = Parser::new(FILENAME, CODE, &tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast,
            [
                Expression::LabelDef {
                    id: String::from("label_def"),
                    span: (
                        0,
                        "label_def:".len()
                    ).into()
                }
            ]
        );
    }

    #[test]
    fn parser_ascii_directive_test() {
        const FILENAME: &str = "test";
        const CODE: &str = "ascii \"hello\"";

        let mut lexer = Lexer::new(FILENAME, CODE);
        let tokens = lexer.tokenize().unwrap();

        let mut parser = Parser::new(FILENAME, CODE, &tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast,
            [
                Expression::Directive {
                    directive: String::from("ascii"),
                    args: vec![
                        Expression::StringConstant(
                            String::from("hello"),
                            (6, "\"hello\"".len()).into()
                        )
                    ],
                    span: (
                        0,
                        CODE.len()
                    ).into()
                }
            ]
        );
    }

    #[test]
    fn parser_comptime_expr_test() {
        const FILENAME: &str = "test";
        const CODE: &str = "[. + $1]";

        let mut lexer = Lexer::new(FILENAME, CODE);
        let tokens = lexer.tokenize().unwrap();

        let mut parser = Parser::new(FILENAME, CODE, &tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast,
            [
                Expression::ComptimeExpr {
                    expr: Box::new(Expression::BinaryExpr {
                        op: String::from("+"),
                        lhs: Box::new(Expression::CurrentPtr((1, 1).into())),
                        rhs: Box::new(Expression::UIntConstant(1, (5, 2).into())),
                        span: (1, ". + $1".len()).into()
                    }),
                    span: (
                        0,
                        CODE.len()
                    ).into()
                }
            ]
        );
    }

    #[test]
    fn parser_instr_no_args_test() {
        const FILENAME: &str = "test";
        const CODE: &str = "halt";

        let mut lexer = Lexer::new(FILENAME, CODE);
        let tokens = lexer.tokenize().unwrap();

        let mut parser = Parser::new(FILENAME, CODE, &tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast,
            [
                Expression::Instruction {
                    name: String::from("halt"),
                    args: Vec::new(),
                    span: (0, 4).into()
                }
            ]
        );
    }

    #[test]
    fn parser_instr_1_arg_test() {
        const FILENAME: &str = "test";
        const CODE: &str = "int $80";

        let mut lexer = Lexer::new(FILENAME, CODE);
        let tokens = lexer.tokenize().unwrap();

        let mut parser = Parser::new(FILENAME, CODE, &tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast,
            [
                Expression::Instruction {
                    name: String::from("int"),
                    args: vec![
                        Expression::UIntConstant(80, (4, "$80".len()).into())
                    ],
                    span: (
                        0,
                        CODE.len()
                    ).into()
                }
            ]
        );
    }

    #[test]
    fn parser_instr_2_arg_test() {
        const FILENAME: &str = "test";
        const CODE: &str = "mov %r0, $123";

        let mut lexer = Lexer::new(FILENAME, CODE);
        let tokens = lexer.tokenize().unwrap();

        let mut parser = Parser::new(FILENAME, CODE, &tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast,
            [
                Expression::Instruction {
                    name: String::from("mov"),
                    args: vec![
                        Expression::AsmReg(String::from("r0"), (4, "%r0".len()).into()),
                        Expression::UIntConstant(123, (9, "$123".len()).into())
                    ],
                    span: (
                        0,
                        CODE.len()
                    ).into()
                }
            ]
        );
    }

    #[test]
    fn parser_uint_constant_test() {
        const FILENAME: &str = "test";
        const CODE: &str = "$123";

        let mut lexer = Lexer::new(FILENAME, CODE);
        let tokens = lexer.tokenize().unwrap();

        let mut parser = Parser::new(FILENAME, CODE, &tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast,
            [
                Expression::UIntConstant(123, (0, 4).into())
            ]
        );
    }

    #[test]
    fn parser_string_constant_test() {
        const FILENAME: &str = "test";
        const CODE: &str = "\"hello, world!\"";

        let mut lexer = Lexer::new(FILENAME, CODE);
        let tokens = lexer.tokenize().unwrap();

        let mut parser = Parser::new(FILENAME, CODE, &tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast,
            [
                Expression::StringConstant(String::from("hello, world!"), (0, CODE.len()).into())
            ]
        );
    }

    #[test]
    fn parser_asm_constant_test() {
        const FILENAME: &str = "test";
        const CODE: &str = "$syscall";

        let mut lexer = Lexer::new(FILENAME, CODE);
        let tokens = lexer.tokenize().unwrap();

        let mut parser = Parser::new(FILENAME, CODE, &tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast,
            [
                Expression::AsmConstant(String::from("syscall"), (0, CODE.len()).into())
            ]
        );
    }

    #[test]
    fn parser_asm_reg_test() {
        const FILENAME: &str = "test";
        const CODE: &str = "%r0 %accumulator %call";

        let mut lexer = Lexer::new(FILENAME, CODE);
        let tokens = lexer.tokenize().unwrap();

        let mut parser = Parser::new(FILENAME, CODE, &tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast,
            [
                Expression::AsmReg(String::from("r0"), (0, 3).into()),
                Expression::AsmReg(String::from("accumulator"), (4, 12).into()),
                Expression::AsmReg(String::from("call"), (17, 5).into()),
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
                Expression::LabelRef(String::from("label_ref"), (0, "label_ref".len()).into())
            ]
        );
    }

    #[test]
    fn parser_current_ptr_test() {
        const FILENAME: &str = "test";
        const CODE: &str = ".";

        let mut lexer = Lexer::new(FILENAME, CODE);
        let tokens = lexer.tokenize().unwrap();

        let mut parser = Parser::new(FILENAME, CODE, &tokens);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast,
            [
                Expression::CurrentPtr((0, 1).into())
            ]
        );
    }
}
