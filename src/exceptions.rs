use std::fmt::Debug;
use crate::exceptions::codes::{EOF_ERROR, INVALID_ARG_TYPE, INVALID_IDENTIFIER, INVALID_OP, INVALID_VAR_TYPE, IO_ERROR, OUT_OF_MEMORY, SEGFAULT, VAR_NOT_FOUND};

mod codes {
    pub const SEGFAULT: u32 = 0xC000_0005;
    pub const VAR_NOT_FOUND: u32 = 0xD000_0005;
    pub const INVALID_OP: u32 = 0x0000_000A;
    pub const SIZE_MISMATCH: u32 = 0x0000_000B;
    pub const INVALID_VAR_TYPE: u32 = 0x0000_000C;
    pub const INVALID_ARG_TYPE: u32 = 0x0000_000D;
    pub const OUT_OF_MEMORY: u32 = 0x0000_000E;
    pub const IO_ERROR: u32 = 0x0000_000F;
    pub const EOF_ERROR: u32 = 0x0000_0010;
    pub const INVALID_IDENTIFIER: u32 = 0x0000_0011;
}

#[repr(u32)]  // 底层整数表示
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("Segmentation fault: {0} (code {self:#X})")]
    SegFault(String) = SEGFAULT,
    #[error("Variable not found: {0} (code {self:#X})")]
    VarNotFound(String) = VAR_NOT_FOUND,
    #[error("Invalid operation: {0} (code {self:#X})")]
    InvalidOperation(String) = INVALID_OP,
    #[error("Size mismatch: {0} (code {self:#X})")]
    SizeMismatch(String) = codes::SIZE_MISMATCH,
    #[error("Invalid argument type: {0} (code {self:#X})")]
    InvalidArgType(String) = INVALID_ARG_TYPE,
    #[error("Invalid variable type: {0} (code {self:#X})")]
    InvalidVarType(String) = INVALID_VAR_TYPE,
    #[error("Out of memory: {0} (code {self:#X})")]
    OutOfMemory(String) = OUT_OF_MEMORY,
    #[error("IOError: {0} (code {self:#X})")]
    IOError(String) = IO_ERROR,
    #[error("IOError: {0} (code {self:#X})")]
    EOFError(String) = EOF_ERROR,
    #[error("Invalid identifier: {0} (code {self:#X})")]
    InvalidIdentifier(String) = INVALID_IDENTIFIER,
    #[error("Unrecognized error: {0} (code {1:#X})")]
    UnrecognizedError(String, u32),
}

impl Error {
    pub const fn code(&self) -> u32 {
        match self {
            Self::SegFault(_) => SEGFAULT,
            Self::VarNotFound(_) => VAR_NOT_FOUND,
            Self::InvalidOperation(_) => INVALID_OP,
            Self::SizeMismatch(_) => codes::SIZE_MISMATCH,
            Self::InvalidArgType(_) => INVALID_ARG_TYPE,
            Self::InvalidVarType(_) => INVALID_VAR_TYPE,
            Self::OutOfMemory(_) => OUT_OF_MEMORY,
            Self::IOError(_) => IO_ERROR,
            Self::EOFError(_) => EOF_ERROR,
            Self::InvalidIdentifier(_) => INVALID_IDENTIFIER,
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
            std::io::ErrorKind::NotFound => Error::VarNotFound(err.to_string()),
            std::io::ErrorKind::InvalidData => Error::InvalidVarType(err.to_string()),
            _ => {
                // 获取原始 OS 错误码
                let code = err.raw_os_error().unwrap_or(0) as u32;
                Error::UnrecognizedError(err.to_string(), code)
            }
        }
    }
}
