#[derive(Debug, Clone, PartialEq)]
pub enum Var {
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Bool(bool),
    String(String),
    Pointer(TypedPtr),  // 这样指针值本身也是有类型的（即指向的类型）
    Null
}

/// 值类型（可递归描述指针）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValueType {
    /// 基础类型
    U32,
    U64,
    F32,
    F64,
    Bool,
    String,

    /// 指针类型，指向某种类型（可嵌套）
    Ptr(std::sync::Arc<ValueType>),
    // 可以再加 Null 类型
    Null,
}

impl ValueType {
    /// 获取指针指向的目标类型（如果是 Ptr 的话）
    pub fn target_type(&self) -> Option<&ValueType> {
        match self {
            ValueType::Ptr(inner) => Some(inner.as_ref()),
            _ => None,
        }
    }

    /// 判断是否为某种指针（多级）
    #[must_use]
    pub fn is_ptr(&self) -> bool {
        matches!(self, ValueType::Ptr(_))
    }

    /// 返回指针层级数（0 表示非指针）
    pub fn ptr_depth(&self) -> usize {
        let mut depth = 0;
        let mut current = self;
        while let ValueType::Ptr(inner) = current {
            depth += 1;
            current = inner.as_ref();
        }
        depth
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedPtr {
    pub pos: crate::env::Pos,  // 内存池位置
    pub ty: ValueType,  // 指向的目标类型，例如 U32、String、Ptr(...) 等
}

pub type VarTypeCodeType = u8;
pub mod var_type_code {
    use crate::value::VarTypeCodeType;
    pub const U8: VarTypeCodeType = 0;
    pub const U16: VarTypeCodeType = 1;
    pub const U32: VarTypeCodeType = 2;
    pub const U64: VarTypeCodeType = 3;
    pub const F32: VarTypeCodeType = 4;
    pub const F64: VarTypeCodeType = 5;
    pub const BOOL: VarTypeCodeType = 6;
    pub const STRING: VarTypeCodeType = 7;
    pub const POINTER: VarTypeCodeType = 8;
}

pub fn var_type_bytes_to_code(var_type: &[u8]) -> Result<VarTypeCodeType, crate::exceptions::Error> {
    match var_type {
        b"u8" => Ok(var_type_code::U8),
        b"u16" => Ok(var_type_code::U16),
        b"u32" => Ok(var_type_code::U32),
        b"u64" => Ok(var_type_code::U64),
        b"f32" => Ok(var_type_code::F32),
        b"f64" => Ok(var_type_code::F64),
        b"bool" => Ok(var_type_code::BOOL),
        b"str" => Ok(var_type_code::STRING),
        b"ptr" => Err(crate::exceptions::Error::InvalidVarType("ptr is not supported yet".to_string())),
        _ => Err(crate::exceptions::Error::InvalidVarType(String::from_utf8_lossy(var_type).into_owned()))
    }
}

// impl VarType {
//     pub const fn code(&self) -> VarTypeCodeType {
//         match self {
//             VarType::U32(_) => var_type_code::U32,
//             VarType::U32Ptr(_) => var_type_code::U32_PTR,
//             VarType::VoidPtr(_) => var_type_code::VOID_PTR,
//             VarType::None => var_type_code::NONE,
//         }
//     }
// }
// impl std::fmt::Display for VarType {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         match self {
//             VarType::U32(u) => write!(f, "u32(val={})", u),
//             VarType::U32Ptr(p) => write!(f, "u32ptr(pos={})", p),
//             VarType::VoidPtr(p) => write!(f, "void_ptr(pos={})", p),
//             &VarType::None => write!(f, "none"),
//         }
//     }
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VarId(pub u32);

impl std::fmt::Display for VarId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}