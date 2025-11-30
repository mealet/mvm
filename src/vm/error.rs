use thiserror::Error;

#[derive(Debug, Error)]
pub enum MvmError {
    #[error("invalid opcode provided")]
    InvalidOpcode,

    #[error("segmentation fault (address: {0})")]
    SegmentationFault(u64),

    #[error("io module returned error")]
    IOError(#[from] std::io::Error),

    #[error("unknown mvm error")]
    Unknown
}
