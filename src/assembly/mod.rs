use miette::NamedSource;

pub mod error;
pub mod lexer;

pub type Source = NamedSource<String>;
