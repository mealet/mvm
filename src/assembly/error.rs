use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

use super::Source;

pub fn position_to_span(from: usize, to: usize) -> SourceSpan {
    (from, to.wrapping_sub(from)).into()
}

#[derive(Debug, Error, Diagnostic)]
pub enum AssemblyError {
    // Lexer Errors

    #[error("Unknown character found: '{character}'")]
    #[diagnostic(
        severity(Error),
        code(mvm::asm::unknown_character),
    )]
    UnknownCharacter {
        character: char,

        #[source_code]
        src: Source,
        #[label("unknown character")]
        span: SourceSpan
    },

    #[error("Unknown character escape found: '\\{escape}'")]
    #[diagnostic(
        severity(Error),
        code(mvm::asm::unknown_character_escape),
    )]
    UnknownCharacterEscape {
        escape: char,

        #[source_code]
        src: Source,
        #[label("unknown character escape")]
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
    },

    // Parser Errors
    
    #[error("expected `{expected}` token, but found `{found}`")]
    #[diagnostic(
        severity(Error),
        code(mvm::asm::unexpected_token),
    )]
    UnexpectedToken {
        expected: String,
        found: String,

        #[source_code]
        src: Source,
        #[label("this token expected to be `{expected}`")]
        span: SourceSpan
    },

    #[error("{error}")]
    #[diagnostic(
        severity(Error),
        code(mvm::asm::unexpected_token),
    )]
    UnknownExpression {
        error: String,

        #[source_code]
        src: Source,
        #[label("verify this token")]
        span: SourceSpan
    },


    #[error("{error}")]
    #[diagnostic(
        severity(Error),
        code(mvm::asm::unexpected_token),
    )]
    UnsupportedExpression {
        error: String,
        label: String,

        #[source_code]
        src: Source,
        #[label("{label}")]
        span: SourceSpan
    },

    // Semantic Errors

    #[error("Unknown section definition found")]
    #[diagnostic(
        severity(Error),
        code(mvm::asm::unexpected_token),
    )]
    UnknownSection {
        name: String,

        #[source_code]
        src: Source,
        #[label("section `{name}` is invalid")]
        span: SourceSpan
    },

    #[error("Section placement is invalid")]
    #[diagnostic(
        severity(Error),
        code(mvm::asm::unexpected_token),
    )]
    InvalidSectionPlacement {
        label: String,

        #[source_code]
        src: Source,
        #[label("{label}")]
        span: SourceSpan
    },

    #[error("Label `{name}` is already defined in code")]
    #[diagnostic(
        severity(Error),
        code(mvm::asm::unexpected_token),
    )]
    LabelRedefinition {
        name: String,

        #[source_code]
        src: Source,
        #[label(primary, "this label is already defined")]
        redefinition: SourceSpan,

        #[label("`{name}` first declared here")]
        original: SourceSpan,
    },
}
