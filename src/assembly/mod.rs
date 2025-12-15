use miette::NamedSource;

pub mod error;
pub mod lexer;
pub mod parser;
pub mod semantic;

pub type Source = NamedSource<String>;
