use super::{
    Source,
    parser::expressions::Expression,
    error::{self, AssemblyError},
};

use miette::{NamedSource, SourceSpan};
use std::collections::HashMap;

mod macros;

#[derive(Debug)]
pub struct Analyzer {
    src: Source,
    errors: Vec<AssemblyError>,

    section: Section,
    labels: HashMap<String, SourceSpan>,

    labels_analyzed: bool,
    comptime_mode: bool,
}

#[derive(Debug, PartialEq)]
pub enum Section {
    Data,
    Text,
    None
}

impl TryFrom<&str> for Section {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "data" | ".data" => Ok(Section::Data),
            "text" | ".text" => Ok(Section::Text),
            _ => Err(())
        }
    }
}

impl Analyzer {
    pub fn new(filename: impl AsRef<str>, source: impl AsRef<str>) -> Self {
        Self {
            src: NamedSource::new(filename, source.as_ref().to_owned()),
            errors: Vec::new(),
            section: Section::None,
            labels: HashMap::new(),
            labels_analyzed: false,
            comptime_mode: false,
        }
    }

    pub fn analyze(&mut self, ast: &[Expression]) -> Result<(), &[AssemblyError]> {
        // analyzing all labels definitions
        ast
            .into_iter()
            .filter(|expr| matches!(expr, Expression::LabelDef { .. }))
            .for_each(|expr| self.visit_expression(expr));

        self.labels_analyzed = true;

        ast.into_iter().for_each(|expr| self.visit_expression(expr));

        if !self.errors.is_empty() {
            return Err(&self.errors)
        }

        return Ok(())
    }

    fn error(&mut self, error: AssemblyError) {
        self.errors.push(error);
    }
}

impl Analyzer {
    fn visit_expression(&mut self, expression: &Expression) {
        match expression {
            Expression::SectionDef { id, span } => {
                match Section::try_from(id.as_str()) {
                    Ok(section) => {
                        if self.section == Section::Text {
                            self.error(AssemblyError::InvalidSectionPlacement {
                                label: format!("section `{}` must be placed before `.text`", id),
                                src: self.src.clone(),
                                span: *span
                            });
                            return;
                        }

                        if self.section == Section::None
                        && section == Section::Text {
                            self.error(AssemblyError::InvalidSectionPlacement {
                                label: format!("compiler requires `.data` section before `.text`"),
                                src: self.src.clone(),
                                span: *span
                            });
                            return;
                        }

                        self.section = section;
                    },
                    Err(_) => {
                        self.error(AssemblyError::UnknownSection {
                            name: id.clone(),
                            src: self.src.clone(),
                            span: *span
                        });
                        return;
                    }
                }
            }

            Expression::EntryDef { label, span } => {
                if !self.labels.contains_key(label) {
                    self.error(AssemblyError::UnknownLabel {
                        name: label.clone(),
                        src: self.src.clone(),
                        span: *span
                    });
                }
            }

            Expression::LabelDef { id, span } => {
                if self.labels_analyzed { return };

                if let Some(original_span) = self.labels.get(id) {
                    self.error(AssemblyError::LabelRedefinition {
                        name: id.clone(),
                        src: self.src.clone(),
                        redefinition: *span,
                        original: *original_span
                    });
                    return;
                }

                self.labels.insert(id.clone(), *span);
            }

            Expression::Directive { directive, args, span } => {
                match directive.as_str() {
                    "ascii" => {
                        // this arguments must be verified in parser
                        assert!(args.len() == 1);
                        assert!(matches!(args.get(0), Some(Expression::StringConstant(_, _))));

                        if self.section != Section::Data {
                            self.error(AssemblyError::InvalidDirective {
                                name: directive.to_owned(),
                                label: format!("must be placed in `.data` section"),
                                src: self.src.clone(),
                                span: *span
                            })
                        }
                    },

                    _ => unreachable!()
                }
            }

            Expression::ComptimeExpr { expr, span } => {
                let prev_mode = self.comptime_mode;
                self.comptime_mode = true;

                self.visit_expression(expr);

                self.comptime_mode = prev_mode;
            }

            Expression::Instruction { name, args, span } => {
                // arguments lengths are verified in parser

                match name.as_str() {
                    "call" => macros::assert_arg!(self, "label", args.get(0).unwrap(), Expression::LabelRef(_, _)),
                    "int" => {
                        let arg = args.get(0).unwrap();

                        macros::assert_arg!(self, "u8", arg, Expression::UIntConstant(_, _) | Expression::AsmConstant(_, _));

                        if let Expression::UIntConstant(value, span) = arg {
                            macros::verify_boundary!(self, *value, *span, u8);
                        }
                    },

                    "mov" => {
                        assert!(args.len() == 2);

                        let dest = args.get(0).unwrap();
                        let src = args.get(1).unwrap();

                        // mov %reg, ...
                        if matches!(dest, Expression::AsmReg(_, _)) {
                            if !matches!(src, Expression::UIntConstant(_, _) | Expression::AsmReg(_, _) | Expression::LabelRef(_, _)) {
                                self.error(AssemblyError::InvalidArgument {
                                    label: format!("this expected to be number/register/label"),
                                    src: self.src.clone(),
                                    span: src.get_span()
                                });
                            }

                            return;
                        }
                        
                        // mov address, ...
                        if matches!(dest, Expression::UIntConstant(_, _) | Expression::LabelRef(_, _)) {
                            macros::assert_arg!(self, "register", src, Expression::AsmReg(_, _));

                            return;
                        }

                        self.error(AssemblyError::InvalidArgument {
                            label: format!("destination expected to be register/address"),
                            src: self.src.clone(),
                            span: dest.get_span()
                        });
                    }

                    "push8" | "push16" | "push32" | "push64" => macros::assert_arg!(self, "register", args.get(0).unwrap(), Expression::AsmReg(_, _)),
                    "pop8" | "pop16" | "pop32" | "pop64" => macros::assert_arg!(self, "register", args.get(0).unwrap(), Expression::AsmReg(_, _)),

                    "frame8" | "frame16" | "frame32" | "frame64" => {
                        let dest = args.get(0).unwrap();
                        let address = args.get(1).unwrap();

                        macros::assert_arg!(self, "register", dest, Expression::AsmReg(_, _));
                        macros::assert_arg!(self, "u16", address, Expression::UIntConstant(_, _));

                        if let Expression::UIntConstant(value, span) = address {
                            macros::verify_boundary!(self, *value, *span, u16);
                        }
                    },

                    "peek8" | "peek16" | "peek32" | "peek64" => {
                        let dest = args.get(0).unwrap();
                        let address = args.get(1).unwrap();

                        macros::assert_arg!(self, "register", dest, Expression::AsmReg(_, _));
                        macros::assert_arg!(self, "u16", address, Expression::UIntConstant(_, _));

                        if let Expression::UIntConstant(value, span) = address {
                            macros::verify_boundary!(self, *value, *span, u16);
                        }
                    },

                    "add" | "sub" | "mul" | "div" | "cmp" => {
                        let dest = args.get(0).unwrap();
                        let src = args.get(1).unwrap();

                        macros::assert_arg!(self, "register", dest, Expression::AsmReg(_, _));

                        if !matches!(src, Expression::UIntConstant(_, _) | Expression::AsmReg(_, _) | Expression::LabelRef(_, _)) {
                            self.error(AssemblyError::InvalidArgument {
                                label: format!("this expected to be number/register/label"),
                                src: self.src.clone(),
                                span: src.get_span()
                            });
                        }
                    }

                    "xadd" => {
                        let dest = args.get(0).unwrap();
                        let src = args.get(1).unwrap();

                        macros::assert_arg!(self, "register", dest, Expression::AsmReg(_, _));
                        macros::assert_arg!(self, "register", src, Expression::AsmReg(_, _));
                    },

                    "jmp" | "jz" | "jnz" => {
                        let label = args.get(0).unwrap();
                        macros::assert_arg!(self, "label", label, Expression::LabelRef(_, _));
                    },

                    "je" | "jne" => {
                        let value = args.get(0).unwrap();
                        let label = args.get(1).unwrap();

                        macros::assert_arg!(self, "u64", value, Expression::UIntConstant(_, _));
                        macros::assert_arg!(self, "label", label, Expression::LabelRef(_, _));
                    }

                    _ => unimplemented!(),
                }
            }

            Expression::BinaryExpr { op, lhs, rhs, span } => {
                if !self.comptime_mode {
                    self.error(AssemblyError::ComptimeException {
                        error: String::from("Usage of comptime expression without compile time mode"),
                        label: format!("binary expressions are allowed only in compile time mode: \"[EXPR]\""),
                        src: self.src.clone(),
                        span: *span
                    });
                    return;
                }

                self.visit_expression(lhs);
                self.visit_expression(rhs);
            }

            Expression::UIntConstant(_, _) => {},
            Expression::StringConstant(_, span) => {
                self.error(AssemblyError::NotAllowed {
                    label: String::from("string constants are not allowed without directives"),
                    src: self.src.clone(),
                    span: *span
                })
            },

            Expression::AsmConstant(_, _) => {},
            Expression::AsmReg(_, span) => {
                if self.comptime_mode {
                    self.error(AssemblyError::ComptimeException {
                        error: String::from("Runtime element found in compile time mode"),
                        label: String::from("registers values are unknown at compile time"),
                        src: self.src.clone(),
                        span: *span
                    });
                    return;
                }
            }

            Expression::LabelRef(label_name, span) => {
                if !self.labels.contains_key(label_name) {
                    self.error(AssemblyError::UnknownLabel {
                        name: label_name.clone(),
                        src: self.src.clone(),
                        span: *span
                    });
                    return;
                }
            }

            Expression::CurrentPtr(span) => {
                if !self.comptime_mode {
                    self.error(AssemblyError::ComptimeException {
                        error: String::from("Usage of comptime expression without compile time mode"),
                        label: format!("current pointer is allowed only in comptime expression: \"[EXPR]\""),
                        src: self.src.clone(),
                        span: *span
                    });
                    return;
                }
            }

            _ => {}
        }
    }
}
