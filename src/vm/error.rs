use thiserror::Error;

#[derive(Debug, Error)]
pub enum MvmError {
    #[error("invalid opcode provided (value: {0})")]
    InvalidOpcode(u8),

    #[error("segmentation fault (address: {0})")]
    SegmentationFault(u64),

    #[error("memory is out of bounds")]
    OutOfBounds,

    #[error("stack data overflow")]
    StackOverflow,

    #[error("stack pointer goes out of frame")]
    StackOutOfFrame,

    #[error("pop on empty (or small) stack")]
    EmptyStackPop,

    #[error("call stack is overflowed")]
    CallStackOverflow,

    #[error("empty call stack is being popped")]
    EmptyCallStackPop,

    #[error("unknown interrupt is being called")]
    UnknownInterrupt,

    #[error("catched division by zero")]
    DivisionByZero,

    #[error("write entry rejected by system")]
    WriteEntryRejected,

    #[error("no program `text` section found")]
    NoTextSection,

    #[error("io module returned error")]
    IOError(#[from] std::io::Error),

    #[error("unknown mvm error")]
    Unknown
}
