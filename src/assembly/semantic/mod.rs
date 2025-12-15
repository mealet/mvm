use super::{
    Source,
    parser::expressions::Expression,
    error::{self, AssemblyError},
};

use miette::NamedSource;

#[derive(Debug)]
pub struct Analyzer {
    src: Source,
    errors: Vec<AssemblyError>,
}

impl Analyzer {
    pub fn new(filename: impl AsRef<str>, source: impl AsRef<str>) -> Self {
        Self {
            src: NamedSource::new(filename, source.as_ref().to_owned()),
            errors: Vec::new()
        }
    }

    pub fn analyze(&mut self, ast: &[Expression]) -> Result<(), &[AssemblyError]> {
        ast.into_iter().for_each(|expr| self.visit_expression(expr));

        if !self.errors.is_empty() {
            return Err(&self.errors)
        }

        return Ok(())
    }
}

impl Analyzer {
    fn visit_expression(&mut self, expression: &Expression) {
        todo!()
    }
}
