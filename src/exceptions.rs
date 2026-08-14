pub mod codes {
    pub const SEGFAULT: u32 = 0xC000_0005;
    pub const VAR_NOT_FOUND: u32 = 0xD000_0005;
    pub const INVALID_POINTER: u32 = 0xE000_0005;
    pub const INVALID_OP: u32 = 0x0000_000A;
    pub const SIZE_MISMATCH: u32 = 0x0000_000B;
    pub const INVALID_VAR_TYPE: u32 = 0x0000_000C;
    pub const INVALID_ARG_TYPE: u32 = 0x0000_000D;
    pub const OUT_OF_MEMORY: u32 = 0x0000_000E;
    pub const INVALID_FREE: u32 = 0x0000_000F;
    pub const IO_ERROR: u32 = 0x0000_0010;
    pub const EOF_ERROR: u32 = 0x0000_0011;
    pub const INVALID_IDENTIFIER: u32 = 0x0000_0012;
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("Segmentation fault: {0}")]
    SegFault(String),
    #[error("Variable not found: {0}")]
    VarNotFound(String),
    #[error("Invalid pointer: {0}")]
    InvalidPointer(String),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    #[error("Size mismatch: {0}")]
    SizeMismatch(String),
    #[error("Invalid argument type: {0}")]
    InvalidArgType(String),
    #[error("Invalid variable type: {0}")]
    InvalidVarType(String),
    #[error("Out of memory: {0}")]
    OutOfMemory(String),
    #[error("Invalid free: {0}")]
    InvalidFree(String),
    #[error("IOError: {0}")]
    IOError(String),
    #[error("EOFError: {0}")]
    EOFError(String),
    #[error("Invalid identifier: {0}")]
    InvalidIdentifier(String),
    #[error("{0} (code {1:#X})")]
    UnrecognizedError(String, u32),
}

impl Error {
    pub const fn code(&self) -> u32 {
        match self {
            Self::SegFault(_) => codes::SEGFAULT,
            Self::VarNotFound(_) => codes::VAR_NOT_FOUND,
            Self::InvalidPointer(_) => codes::INVALID_POINTER,
            Self::InvalidOperation(_) => codes::INVALID_OP,
            Self::SizeMismatch(_) => codes::SIZE_MISMATCH,
            Self::InvalidArgType(_) => codes::INVALID_ARG_TYPE,
            Self::InvalidVarType(_) => codes::INVALID_VAR_TYPE,
            Self::OutOfMemory(_) => codes::OUT_OF_MEMORY,
            Self::InvalidFree(_) => codes::INVALID_FREE,
            Self::IOError(_) => codes::IO_ERROR,
            Self::EOFError(_) => codes::EOF_ERROR,
            Self::InvalidIdentifier(_) => codes::INVALID_IDENTIFIER,
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
            std::io::ErrorKind::InvalidData => Error::InvalidVarType(err.to_string()),
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
