use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

use super::Source;

pub fn position_to_span(from: usize, to: usize) -> SourceSpan {
    (from, to.wrapping_sub(from)).into()
}

#[derive(Debug, Error, Diagnostic)]
pub enum AssemblyError {
    #[error("Unknown character found: '{character}'")]
    #[diagnostic(
        severity(Error),
        code(mvm::asm::unknown_character),
    )]
    UnknownCharacter {
        character: char,

        #[source_code]
        src: Source,
        #[label("here")]
        span: SourceSpan
    },

    #[error("{error}")]
    #[diagnostic(
        severity(Error),
        code(mvm::asm::constant_error),
    )]
    InvalidConstant {
        error: String,
        label: String,

        #[source_code]
        src: Source,
        #[label("{label}")]
        span: SourceSpan
    },

    #[error("Number parser returned an error: {parser_error}")]
    #[diagnostic(
        severity(Error),
        code(mvm::asm::number_parse_error),
    )]
    ConstantParseError {
        const_type: String,
        parser_error: String,

        #[source_code]
        src: Source,
        #[label("constant type: {const_type}")]
        span: SourceSpan
    }
}
