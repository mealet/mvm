use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

use super::Source;

#[derive(Debug, Error, Diagnostic)]
pub enum AssemblyError {
    #[error("Unknown character found: '{character}'")]
    #[diagnostic(
        severity(Error),
        code(mvm::asm::unknown_character),
    )]
    UnknownCharacter {
        character: String,

        #[source_code]
        src: Source,
        #[label("here")]
        span: SourceSpan
    }
}
