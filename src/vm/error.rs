use thiserror::Error;

#[derive(Debug, Error)]
pub enum MvmError {
    #[error("invalid opcode provided")]
    InvalidOpcode,

    #[error("io module returned error")]
    IOError(#[from] std::io::Error),

    #[error("unknown mvm error")]
    Unknown
}
