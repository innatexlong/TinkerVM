use std::fmt::Debug;

#[repr(u32)]  // 底层整数表示
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("Segmentation fault: {0} (code {self:#X})")]
    SegFault(String) = 0xC000_0005u32,
    #[error("Variable not found: {0} (code {self:#X})")]
    VarNotFound(String) = 0xD000_0005u32,
    #[error("Invalid operation: {0} (code {self:#X})")]
    InvalidOperation(String) = 0x0000_000Au32,
    #[error("Size mismatch: {0} (code {self:#X})")]
    SizeMismatch(String),
    #[error("Invalid argument type: {0} (code {self:#X})")]
    InvalidArgType(String),
    #[error("Invalid variable type: {0} (code {self:#X})")]
    InvalidVarType(String),
    #[error("Out of memory: {0} (code {self:#X})")]
    OutOfMemory(String),
    #[error("IOError: {0} (code {self:#X})")]
    IOError(String),
    #[error("IOError: {0} (code {self:#X})")]
    EOFError(String),
    #[error("Invalid identifier: {0} (code {self:#X})")]
    InvalidIdentifier(String),
    #[error("Unrecognized error: {0} (code {1:#X})")]
    UnrecognizedError(String, u32),
}

impl Error {
    pub const fn code(&self) -> u32 {
        match self {
            Self::SegFault(_) => 0xC000_0005,
            Self::VarNotFound(_) => 0xD000_0005,
            Self::InvalidOperation(_) => 0x0000_000A,
            Self::SizeMismatch(_) => 0x0000_000B,
            Self::InvalidVarType(_) => 0x0000_000C,
            Self::InvalidArgType(_) => 0x0000_000D,
            Self::OutOfMemory(_) => 0x0000_000E,
            Self::IOError(_) => 0x0000_000F,
            Self::EOFError(_) => 0x0000_0010,
            Self::InvalidIdentifier(_) => 0x0000_0011,
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
                // 获取原始 OS 错误码（如果存在）
                let code = err.raw_os_error().unwrap_or(0) as u32;
                Error::UnrecognizedError(err.to_string(), code)
            }
        }
    }
}
