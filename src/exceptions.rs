#[repr(u32)]
pub enum ErrorCode {
    SegFault = 0xC000_0005,
    NotFound = 0xD000_0005,
    WildPointer = 0xE000_0005,
    Duplicated = 0xF000_0005,
    InvalidOperation = 0x0000_000A,
    SizeMismatch,
    InvalidType,
    OutOfIndex,
    OutOfMemory,
    IOError,
    EOFError,
    InvalidIdentifier,
    SyntaxError,
    UnrecognizedError,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("Segmentation fault: {0}")]
    SegFault(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Invalid pointer: {0}")]
    WildPointer(String),
    #[error("Duplicated: {0}")]
    Duplicated(String),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    #[error("Size mismatch: {0}")]
    SizeMismatch(String),
    #[error("Invalid type: {0}")]
    InvalidType(String),
    #[error("Out of index: {0}")]
    OutOfIndex(String),
    #[error("Out of memory: {0}")]
    OutOfMemory(String),
    #[error("IOError: {0}")]
    IOError(String),
    #[error("EOFError: {0}")]
    EOFError(String),
    #[error("Invalid identifier: {0}")]
    InvalidIdentifier(String),
    #[error("Syntax Error: {0}")]
    SyntaxError(String),
    #[error("{0} (code {1:#X})")]
    UnrecognizedError(String, u32),
}

impl Error {
    pub const fn code(&self) -> u32 {
        match self {
            Self::SegFault(_) => ErrorCode::SegFault as u32,
            Self::NotFound(_) => ErrorCode::NotFound as u32,
            Self::WildPointer(_) => ErrorCode::WildPointer as u32,
            Self::Duplicated(_) => ErrorCode::Duplicated as u32,
            Self::InvalidOperation(_) => ErrorCode::InvalidOperation as u32,
            Self::SizeMismatch(_) => ErrorCode::SizeMismatch as u32,
            Self::InvalidType(_) => ErrorCode::InvalidType as u32,
            Self::OutOfIndex(_) => ErrorCode::OutOfIndex as u32,
            Self::OutOfMemory(_) => ErrorCode::OutOfMemory as u32,
            Self::IOError(_) => ErrorCode::IOError as u32,
            Self::EOFError(_) => ErrorCode::EOFError as u32,
            Self::InvalidIdentifier(_) => ErrorCode::InvalidIdentifier as u32,
            Self::SyntaxError(_) => ErrorCode::SyntaxError as u32,
            Self::UnrecognizedError(_, code) => *code,
        }
    }
}

impl std::fmt::UpperHex for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 将枚举转换为其底层的 u32 值，然后委托给整数的 UpperHex 实现
        std::fmt::UpperHex::fmt(&(self.code()), f)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::OutOfMemory => Error::OutOfMemory(err.to_string()),
            std::io::ErrorKind::InvalidData => Error::InvalidType(err.to_string()),
            std::io::ErrorKind::UnexpectedEof => Error::EOFError(err.to_string()),
            std::io::ErrorKind::Unsupported => Error::InvalidOperation(err.to_string()),
            _ => {
                // 获取原始 OS 错误码
                let code = err.raw_os_error().unwrap_or(0) as u32;
                Error::UnrecognizedError(err.to_string(), code)
            }
        }
    }
}
