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
                self.labels.insert(id.to_owned(), Label::new(self.pc, self.data_section));
            }

            Expression::Directive { directive, args, span: _ } => {
                match directive.as_str() {
                    "ascii" => {
                        assert_eq!(args.len(), 1);

                        if let Some(Expression::StringConstant(string, _)) = args.get(0) {
                            let str_bytes = string.bytes();
                            let addr_bytes = self.pc.to_be_bytes();

                            self.push_byte(addr_bytes[0]);
                            self.push_byte(addr_bytes[1]);
                            self.push_byte(addr_bytes[2]);
                            self.push_byte(addr_bytes[3]);

                            self.push_byte(addr_bytes[4]);
                            self.push_byte(addr_bytes[5]);
                            self.push_byte(addr_bytes[6]);
                            self.push_byte(addr_bytes[7]);

                            str_bytes.for_each(|byte| self.push_byte(byte));
                        }
                    },

                    _ => unimplemented!()
                }
            }

            Expression::ComptimeExpr { expr, span: _ } => {
                let value = self.calculate_comptime_expr(expr);
                let bytes = value.to_be_bytes();

                self.push_byte(bytes[0]);
                self.push_byte(bytes[1]);
                self.push_byte(bytes[2]);
                self.push_byte(bytes[3]);

                self.push_byte(bytes[4]);
                self.push_byte(bytes[5]);
                self.push_byte(bytes[6]);
                self.push_byte(bytes[7]);
            },

            Expression::Instruction { name, args, span: _ } => {
                match name.as_str() {
                    "halt" => {
                        self.push_byte(Opcode::Halt as u8);
                    }

                    "ret" => {
                        self.push_byte(Opcode::Return as u8);
                    }

                    "call" => {
                        self.push_byte(Opcode::Call as u8);
                        self.compile_expr(args.get(0).unwrap());
                    }

                    "int" => {
                        self.push_byte(Opcode::Interrupt as u8);
                        self.compile_expr(args.get(0).unwrap());
                    }

                    "mov" => {
                        // mov %dest, ...
                        if let Some(Expression::AsmReg(_, _)) = args.get(0) {

                            match args.get(1) {
                                Some(Expression::UIntConstant(value, _)) => {
                                    let constant = Constant::new(*value);

                                    match constant {
                                        Constant::U8(_) => self.push_byte(Opcode::Mov8 as u8),
                                        Constant::U16(_) => self.push_byte(Opcode::Mov16 as u8),
                                        Constant::U32(_) => self.push_byte(Opcode::Mov32 as u8),
                                        Constant::U64(_) => self.push_byte(Opcode::Mov64 as u8),
                                    }

                                    self.compile_expr(args.get(0).unwrap());
                                    self.compile_expr(args.get(1).unwrap());
                                }

                                Some(Expression::AsmReg(_, _)) => {
                                    self.push_byte(Opcode::MovR2R as u8);
                                    self.compile_expr(args.get(0).unwrap());
                                    self.compile_expr(args.get(1).unwrap());
                                }

                                Some(Expression::LabelRef(_, _)) => {
                                    self.push_byte(Opcode::Mov64 as u8);
                                    self.compile_expr(args.get(0).unwrap());
                                    self.compile_expr(args.get(1).unwrap());
                                }

                                _ => unreachable!()
                            }

                            return;
                        }
                    }

                    "add" => {
                        match args.get(1) {
                            Some(Expression::UIntConstant(value, _)) => {
                                let constant = Constant::new(*value);

                                match constant {
                                    Constant::U8(_) => self.push_byte(Opcode::Add8 as u8),
                                    Constant::U16(_) => self.push_byte(Opcode::Add16 as u8),
                                    Constant::U32(_) => self.push_byte(Opcode::Add32 as u8),
                                    Constant::U64(_) => self.push_byte(Opcode::Add64 as u8),
                                }

                                self.compile_expr(args.get(0).unwrap());
                                self.compile_expr(args.get(1).unwrap());
                            }

                            Some(Expression::AsmReg(_, _)) => {
                                self.push_byte(Opcode::AddR2R as u8);
                                self.compile_expr(args.get(0).unwrap());
                                self.compile_expr(args.get(1).unwrap());
                            }

                            Some(Expression::LabelRef(_, _)) => {
                                self.push_byte(Opcode::Add64 as u8);
                                self.compile_expr(args.get(0).unwrap());
                                self.compile_expr(args.get(1).unwrap());
                            }

                            _ => unreachable!()
                        }
                    }

                    "sub" => {
                        match args.get(1) {
                            Some(Expression::UIntConstant(value, _)) => {
                                let constant = Constant::new(*value);

                                match constant {
                                    Constant::U8(_) => self.push_byte(Opcode::Sub8 as u8),
                                    Constant::U16(_) => self.push_byte(Opcode::Sub16 as u8),
                                    Constant::U32(_) => self.push_byte(Opcode::Sub32 as u8),
                                    Constant::U64(_) => self.push_byte(Opcode::Sub64 as u8),
                                }

                                self.compile_expr(args.get(0).unwrap());
                                self.compile_expr(args.get(1).unwrap());
                            }

                            Some(Expression::AsmReg(_, _)) => {
                                self.push_byte(Opcode::SubR2R as u8);
                                self.compile_expr(args.get(0).unwrap());
                                self.compile_expr(args.get(1).unwrap());
                            }

                            Some(Expression::LabelRef(_, _)) => {
                                self.push_byte(Opcode::Sub64 as u8);
                                self.compile_expr(args.get(0).unwrap());
                                self.compile_expr(args.get(1).unwrap());
                            }

                            _ => unreachable!()
                        }
                    }

                    "mul" => {
                        match args.get(1) {
                            Some(Expression::UIntConstant(value, _)) => {
                                let constant = Constant::new(*value);

                                match constant {
                                    Constant::U8(_) => self.push_byte(Opcode::Mul8 as u8),
                                    Constant::U16(_) => self.push_byte(Opcode::Mul16 as u8),
                                    Constant::U32(_) => self.push_byte(Opcode::Mul32 as u8),
                                    Constant::U64(_) => self.push_byte(Opcode::Mul64 as u8),
                                }

                                self.compile_expr(args.get(0).unwrap());
                                self.compile_expr(args.get(1).unwrap());
                            }

                            Some(Expression::AsmReg(_, _)) => {
                                self.push_byte(Opcode::MulR2R as u8);
                                self.compile_expr(args.get(0).unwrap());
                                self.compile_expr(args.get(1).unwrap());
                            }

                            Some(Expression::LabelRef(_, _)) => {
                                self.push_byte(Opcode::Mul64 as u8);
                                self.compile_expr(args.get(0).unwrap());
                                self.compile_expr(args.get(1).unwrap());
                            }

                            _ => unreachable!()
                        }
                    }

                    "div" => {
                        match args.get(1) {
                            Some(Expression::UIntConstant(value, _)) => {
                                let constant = Constant::new(*value);

                                match constant {
                                    Constant::U8(_) => self.push_byte(Opcode::Div8 as u8),
                                    Constant::U16(_) => self.push_byte(Opcode::Div16 as u8),
                                    Constant::U32(_) => self.push_byte(Opcode::Div32 as u8),
                                    Constant::U64(_) => self.push_byte(Opcode::Div64 as u8),
                                }

                                self.compile_expr(args.get(0).unwrap());
                                self.compile_expr(args.get(1).unwrap());
                            }

                            Some(Expression::AsmReg(_, _)) => {
                                self.push_byte(Opcode::DivR2R as u8);
                                self.compile_expr(args.get(0).unwrap());
                                self.compile_expr(args.get(1).unwrap());
                            }

                            Some(Expression::LabelRef(_, _)) => {
                                self.push_byte(Opcode::Div64 as u8);
                                self.compile_expr(args.get(0).unwrap());
                                self.compile_expr(args.get(1).unwrap());
                            }

                            _ => unreachable!()
                        }
                    }

                    "cmp" => {
                        match args.get(1) {
                            Some(Expression::UIntConstant(value, _)) => {
                                let constant = Constant::new(*value);

                                match constant {
                                    Constant::U8(_) => self.push_byte(Opcode::Cmp8 as u8),
                                    Constant::U16(_) => self.push_byte(Opcode::Cmp16 as u8),
                                    Constant::U32(_) => self.push_byte(Opcode::Cmp32 as u8),
                                    Constant::U64(_) => self.push_byte(Opcode::Cmp64 as u8),
                                }

                                self.compile_expr(args.get(0).unwrap());
                                self.compile_expr(args.get(1).unwrap());
                            }

                            Some(Expression::AsmReg(_, _)) => {
                                self.push_byte(Opcode::CmpR2R as u8);
                                self.compile_expr(args.get(0).unwrap());
                                self.compile_expr(args.get(1).unwrap());
                            }

                            Some(Expression::LabelRef(_, _)) => {
                                self.push_byte(Opcode::Cmp64 as u8);
                                self.compile_expr(args.get(0).unwrap());
                                self.compile_expr(args.get(1).unwrap());
                            }

                            _ => unreachable!()
                        }
                    }

                    "xadd" => {
                        self.push_byte(Opcode::XAdd as u8);
                        self.compile_expr(args.get(0).unwrap());
                        self.compile_expr(args.get(1).unwrap());
                    }

                    "push8" => {
                        self.push_byte(Opcode::Push8 as u8);
                        self.compile_expr(args.get(0).unwrap());
                    },

                    "push16" => {
                        self.push_byte(Opcode::Push16 as u8);
                        self.compile_expr(args.get(0).unwrap());
                    },

                    "push32" => {
                        self.push_byte(Opcode::Push32 as u8);
                        self.compile_expr(args.get(0).unwrap());
                    },

                    "push64" => {
                        self.push_byte(Opcode::Push64 as u8);
                        self.compile_expr(args.get(0).unwrap());
                    },

                    "pop8" => {
                        self.push_byte(Opcode::Pop8 as u8);
                        self.compile_expr(args.get(0).unwrap());
                    },

                    "pop16" => {
                        self.push_byte(Opcode::Pop16 as u8);
                        self.compile_expr(args.get(0).unwrap());
                    },

                    "pop32" => {
                        self.push_byte(Opcode::Pop32 as u8);
                        self.compile_expr(args.get(0).unwrap());
                    },

                    "pop64" => {
                        self.push_byte(Opcode::Pop64 as u8);
                        self.compile_expr(args.get(0).unwrap());
                    },

                    "frame8" => {
                        self.push_byte(Opcode::Frame8 as u8);
                        self.compile_expr(args.get(0).unwrap());
                        self.compile_expr(args.get(1).unwrap());
                    },

                    "frame16" => {
                        self.push_byte(Opcode::Frame16 as u8);
                        self.compile_expr(args.get(0).unwrap());
                        self.compile_expr(args.get(1).unwrap());
                    },

                    "frame32" => {
                        self.push_byte(Opcode::Frame32 as u8);
                        self.compile_expr(args.get(0).unwrap());
                        self.compile_expr(args.get(1).unwrap());
                    },

                    "frame64" => {
                        self.push_byte(Opcode::Frame64 as u8);
                        self.compile_expr(args.get(0).unwrap());
                        self.compile_expr(args.get(1).unwrap());
                    },

                    "peek8" => {
                        self.push_byte(Opcode::Peek8 as u8);
                        self.compile_expr(args.get(0).unwrap());
                        self.compile_expr(args.get(1).unwrap());
                    },

                    "peek16" => {
                        self.push_byte(Opcode::Peek16 as u8);
                        self.compile_expr(args.get(0).unwrap());
                        self.compile_expr(args.get(1).unwrap());
                    },

                    "peek32" => {
                        self.push_byte(Opcode::Peek32 as u8);
                        self.compile_expr(args.get(0).unwrap());
                        self.compile_expr(args.get(1).unwrap());
                    },

                    "peek64" => {
                        self.push_byte(Opcode::Peek64 as u8);
                        self.compile_expr(args.get(0).unwrap());
                        self.compile_expr(args.get(1).unwrap());
                    },

                    "jmp" => {
                        self.push_byte(Opcode::Jmp as u8);
                        self.compile_expr(args.get(0).unwrap());
                    }

                    "jz" => {
                        self.push_byte(Opcode::Jz as u8);
                        self.compile_expr(args.get(0).unwrap());
                    }

                    "jnz" => {
                        self.push_byte(Opcode::Jnz as u8);
                        self.compile_expr(args.get(0).unwrap());
                    }

                    "je" => {
                        self.push_byte(Opcode::Je as u8);
                        self.compile_expr(args.get(0).unwrap());
                        self.compile_expr(args.get(1).unwrap());
                    }

                    "jne" => {
                        self.push_byte(Opcode::Jne as u8);
                        self.compile_expr(args.get(0).unwrap());
                        self.compile_expr(args.get(1).unwrap());
                    }

                    _ => unimplemented!()
                }
            },
            Expression::BinaryExpr { op, lhs, rhs, span } => unreachable!(),

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

            Expression::StringConstant(_, _) => unreachable!(),

            Expression::AsmConstant(name, _) => {
                match name.as_str() {
                    "syscall" => {
                        self.push_byte(80);
                    }
                    _ => unreachable!()
                }
            },
            Expression::AsmReg(name, _) => {
                const REGISTERS_INDEXES: [&str; 15] = [
                    "r0", "r1", "r2", "r3", "r4", "r5", "r6",
                    "r7", "r8", "call", "accumulator", "instruction_ptr", "stack_ptr",
                    "frame_ptr", "mem_ptr"
                ];
                
                self.push_byte(REGISTERS_INDEXES.iter().position(|el| el == name).unwrap_or_default() as u8);
            },

            Expression::CurrentPtr(_) => unreachable!(),

            _ => unimplemented!()
        }
    }

    fn calculate_comptime_expr(&self, expr: &Expression) -> u64 {
        match expr {
            Expression::ComptimeExpr { expr, span: _ } => {
                self.calculate_comptime_expr(expr)
            }

            Expression::BinaryExpr { op, lhs, rhs, span: _ } => {
                let lhs = self.calculate_comptime_expr(lhs);
                let rhs = self.calculate_comptime_expr(rhs);

                match op.as_str() {
                    "+" => lhs.wrapping_add(rhs),
                    "-" => lhs.wrapping_sub(rhs),
                    "*" => lhs.wrapping_mul(rhs),
                    "/" => lhs.wrapping_div(rhs),
                    "%" => if rhs == 0 { 0 } else { lhs % rhs },
                    _ => unreachable!()
                }
            }

            Expression::LabelRef(label, _) => {
                self.labels.get(label).unwrap().ptr
            }

            Expression::UIntConstant(value, _) => {
                *value
            }

            Expression::CurrentPtr(_) => {
                self.pc
            }

            _ => unreachable!()
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

        assert_eq!(codegen.labels.get("label_def"), Some(&Label::new(0, false)));
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

        assert_eq!(codegen.labels.get("label"), Some(&Label::new(0, false)));
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

    #[test]
    fn codegen_asm_constant_test() {
        const FILENAME: &str = "test";
        const CODE: &str = "$syscall";

        let mut lexer = Lexer::new(FILENAME, CODE);
        let tokens = lexer.tokenize().unwrap();

        let mut parser = Parser::new(FILENAME, CODE, &tokens);
        let ast = parser.parse().unwrap();

        let mut codegen = Codegen::new();

        codegen.compile_expr(&ast[0]);

        assert_eq!(codegen.pc, 1);
        assert_eq!(codegen.output.get(0), Some(&80));
    }

    #[test]
    fn codegen_asm_regs_test() {
        const FILENAME: &str = "test";
        const CODE: &str = "%r0 %r1 %r2 %r3 %r4 %r5 %r6 %r7 %r8 %call %accumulator %instruction_ptr %stack_ptr %frame_ptr %mem_ptr";

        let mut lexer = Lexer::new(FILENAME, CODE);
        let tokens = lexer.tokenize().unwrap();

        let mut parser = Parser::new(FILENAME, CODE, &tokens);
        let ast = parser.parse().unwrap();

        let mut codegen = Codegen::new();

        for ref expr in ast {
            codegen.compile_expr(expr);
        }

        assert_eq!(codegen.pc, 15);
        assert_eq!(codegen.output, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]);
    }

    #[test]
    fn codegen_comtpime_expr_test() {
        const FILENAME: &str = "test";
        const CODE: &str = "[$1 + $1 + $1]";

        let mut lexer = Lexer::new(FILENAME, CODE);
        let tokens = lexer.tokenize().unwrap();

        let mut parser = Parser::new(FILENAME, CODE, &tokens);
        let ast = parser.parse().unwrap();

        let mut codegen = Codegen::new();

        for ref expr in ast {
            codegen.compile_expr(expr);
        }

        assert_eq!(codegen.pc, 8);
        assert_eq!(codegen.output, [0, 0, 0, 0, 0, 0, 0, 3]);
    }

    #[test]
    fn codegen_comtpime_with_currentptr_expr_test() {
        const FILENAME: &str = "test";
        const CODE: &str = "$1 [. + $1]";

        let mut lexer = Lexer::new(FILENAME, CODE);
        let tokens = lexer.tokenize().unwrap();

        let mut parser = Parser::new(FILENAME, CODE, &tokens);
        let ast = parser.parse().unwrap();

        let mut codegen = Codegen::new();

        for ref expr in ast {
            codegen.compile_expr(expr);
        }

        assert_eq!(codegen.pc, 16);
        assert_eq!(codegen.output, [0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,9]);
    }

    #[test]
    fn codegen_comtpime_with_label_expr_test() {
        const FILENAME: &str = "test";
        const CODE: &str = "label: [label + $1]";

        let mut lexer = Lexer::new(FILENAME, CODE);
        let tokens = lexer.tokenize().unwrap();

        let mut parser = Parser::new(FILENAME, CODE, &tokens);
        let ast = parser.parse().unwrap();

        let mut codegen = Codegen::new();

        for ref expr in ast {
            codegen.compile_expr(expr);
        }

        assert_eq!(codegen.pc, 8);
        assert_eq!(codegen.output, [0, 0, 0, 0, 0, 0, 0, 1]);
    }

    #[test]
    fn codegen_mov_expr_test() {
        const FILENAME: &str = "test";
        const CODE: &str = "mov %r0, $123";

        let mut lexer = Lexer::new(FILENAME, CODE);
        let tokens = lexer.tokenize().unwrap();

        let mut parser = Parser::new(FILENAME, CODE, &tokens);
        let ast = parser.parse().unwrap();

        let mut codegen = Codegen::new();

        for ref expr in ast {
            codegen.compile_expr(expr);
        }

        assert_eq!(codegen.pc, 1 + 1 + 8);
        assert_eq!(
            codegen.constants_refs,
            HashMap::from([
                (2, String::from("123"))
            ])
        );
        assert_eq!(codegen.output, [Opcode::Mov8 as u8, 0, /* address */ 0,0,0,0, 0,0,0,0 ]);
    }
}
