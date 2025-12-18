mod structs;

use super::parser::expressions::Expression;
use crate::vm::Opcode;

use std::collections::HashMap;
use structs::{Label, Constant};

pub struct Codegen {
    pc: u64,

    labels: HashMap<String, Label>,
    labels_refs: HashMap<u64, String>,

    data_section: bool,
    constants: HashMap<String, Constant>,
    constants_refs: HashMap<u64, String>,

    output: Vec<u8>
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            pc: 0,

            labels: HashMap::new(),
            labels_refs: HashMap::new(),

            data_section: false,
            constants: HashMap::new(),
            constants_refs: HashMap::new(),
            
            output: Vec::new()
        }
    }

    pub fn compile(&mut self, ast: &[Expression]) -> &[u8] {
        for expr in ast {
            self.compile_expr(expr);
        }

        &self.output
    }
}

impl Codegen {
    fn push_byte(&mut self, byte: u8) {
        self.pc += 1;
        self.output.push(byte);
    }

    fn add_constant(&mut self, id: String, constant: Constant) {
        if let Some(prev) = self.constants.get(&id) {
            if prev < &constant {
                self.constants.insert(id, constant);
            }

            return;
        }

        self.constants.insert(id, constant);
    }

    fn compile_expr(&mut self, expr: &Expression) {
        match expr {
            Expression::SectionDef { id, span: _ } => {
                self.data_section = id == ".data";
            }

            Expression::EntryDef { label, span: _ } => {
                self.push_byte(0xFF);

                self.labels_refs.insert(self.pc, label.to_owned());

                // 64 bit address number

                self.push_byte(0);
                self.push_byte(0);
                self.push_byte(0);
                self.push_byte(0);

                self.push_byte(0);
                self.push_byte(0);
                self.push_byte(0);
                self.push_byte(0);
            }

            Expression::LabelDef { id, span: _ } => {
                self.labels.insert(id.to_owned(), Label::new(self.pc));
            }

            Expression::LabelRef(label, _) => {
                self.labels_refs.insert(self.pc, label.to_owned());

                // 64 bit address number

                self.push_byte(0);
                self.push_byte(0);
                self.push_byte(0);
                self.push_byte(0);

                self.push_byte(0);
                self.push_byte(0);
                self.push_byte(0);
                self.push_byte(0);
            }

            Expression::UIntConstant(value, _) => {
                if self.data_section {
                    let value_bytes = value.to_be_bytes();

                    self.push_byte(value_bytes[0]);
                    self.push_byte(value_bytes[1]);
                    self.push_byte(value_bytes[2]);
                    self.push_byte(value_bytes[3]);

                    self.push_byte(value_bytes[4]);
                    self.push_byte(value_bytes[5]);
                    self.push_byte(value_bytes[6]);
                    self.push_byte(value_bytes[7]);

                    return;
                }

                let constant = Constant::new(*value);
                self.add_constant(value.to_string(), constant);

                self.constants_refs.insert(self.pc, value.to_string());

                // 64 bit address number

                self.push_byte(0);
                self.push_byte(0);
                self.push_byte(0);
                self.push_byte(0);

                self.push_byte(0);
                self.push_byte(0);
                self.push_byte(0);
                self.push_byte(0);
            }

            _ => unimplemented!()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assembly::{lexer::Lexer, parser::Parser};
    use super::*;

    #[test]
    fn codegen_label_def_test() {
        const FILENAME: &str = "test";
        const CODE: &str = "label_def:";

        let mut lexer = Lexer::new(FILENAME, CODE);
        let tokens = lexer.tokenize().unwrap();

        let mut parser = Parser::new(FILENAME, CODE, &tokens);
        let ast = parser.parse().unwrap();

        let mut codegen = Codegen::new();
        codegen.compile_expr(&ast[0]);

        assert_eq!(codegen.labels.get("label_def"), Some(&Label::new(0)));
        assert_eq!(codegen.pc, 0);
        assert!(codegen.output.is_empty());
    }

    #[test]
    fn codegen_label_ref_test() {
        const FILENAME: &str = "test";
        const CODE: &str = "label: label";

        let mut lexer = Lexer::new(FILENAME, CODE);
        let tokens = lexer.tokenize().unwrap();

        let mut parser = Parser::new(FILENAME, CODE, &tokens);
        let ast = parser.parse().unwrap();

        let mut codegen = Codegen::new();

        codegen.compile_expr(&ast[0]);
        codegen.compile_expr(&ast[1]);

        assert_eq!(codegen.labels.get("label"), Some(&Label::new(0)));
        assert_eq!(codegen.pc, 8);
        assert_eq!(codegen.output, [0,0,0,0, 0,0,0,0]);
    }

    #[test]
    fn codegen_entry_def_test() {
        const FILENAME: &str = "test";
        const CODE: &str = "entry _start";

        let mut lexer = Lexer::new(FILENAME, CODE);
        let tokens = lexer.tokenize().unwrap();

        let mut parser = Parser::new(FILENAME, CODE, &tokens);
        let ast = parser.parse().unwrap();

        let mut codegen = Codegen::new();

        codegen.compile_expr(&ast[0]);

        assert_eq!(codegen.labels_refs.get(&1), Some(&String::from("_start")));
        assert_eq!(codegen.pc, 9);
        assert_eq!(codegen.output, [0xFF, 0,0,0,0, 0,0,0,0]);
    }

    #[test]
    fn codegen_constant_test() {
        const FILENAME: &str = "test";
        const CODE: &str = "$123";

        let mut lexer = Lexer::new(FILENAME, CODE);
        let tokens = lexer.tokenize().unwrap();

        let mut parser = Parser::new(FILENAME, CODE, &tokens);
        let ast = parser.parse().unwrap();

        let mut codegen = Codegen::new();

        codegen.compile_expr(&ast[0]);

        assert_eq!(codegen.constants.get("123"), Some(&Constant::U8(123)));
        assert_eq!(codegen.constants_refs.get(&0), Some(&String::from("123")));
        assert_eq!(codegen.pc, 8);
    }
}
