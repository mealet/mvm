mod structs;

use super::parser::expressions::Expression;

use std::collections::HashMap;
use structs::Label;

pub struct Codegen {
    pc: u64,

    labels: HashMap<String, Label>,
    labels_refs: HashMap<u64, String>,

    data_section: Vec<u8>,
    data_constants: HashMap<String, u64>,

    output: Vec<u8>
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            pc: 0,

            labels: HashMap::new(),
            labels_refs: HashMap::new(),

            data_section: Vec::new(),
            data_constants: HashMap::new(),
            
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
    fn compile_expr(&mut self, expr: &Expression) {
        match expr {
            Expression::LabelDef { id, span: _ } => {
                self.labels.insert(id.to_owned(), Label::new(self.pc));
            }

            Expression::LabelRef(label, _) => {
                self.labels_refs.insert(self.pc, label.to_owned());

                // 64 bit address number

                self.output.push(0);
                self.output.push(0);
                self.output.push(0);
                self.output.push(0);

                self.output.push(0);
                self.output.push(0);
                self.output.push(0);
                self.output.push(0);

                self.pc += 8;
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
        let code = codegen.compile_expr(&ast[0]);

        assert_eq!(codegen.labels.get("label_def"), Some(&Label::new(0)));
    }
}
