use super::{
    Source,
    parser::expressions::Expression,
    error::{self, AssemblyError},
};

use miette::{NamedSource, SourceSpan};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Analyzer {
    src: Source,
    errors: Vec<AssemblyError>,

    section: Section,
    labels: HashMap<String, SourceSpan>,
    labels_analyzed: bool,
}

#[derive(Debug, PartialEq)]
enum Section {
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
            labels_analyzed: false
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

            Expression::LabelRef(label_name, span) => {
                if !self.labels.contains_key(label_name) {
                    self.error(AssemblyError::UnknownLabel {
                        name: label_name.clone(),
                        src: self.src.clone(),
                        span: *span
                    });
                }
            }

            _ => {}
        }
    }
}
