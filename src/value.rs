#[derive(Debug, Clone, PartialEq)]
pub enum Var {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Bool(bool),
    String(String),
    Pointer(TypedPtr),  // 这样指针值本身也是有类型的（即指向的类型）
    Null
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueSlot {
    /// 基本类型直接内联在变量槽中（如 int、float、bool）
    Primitive(Var),
    /// 引用类型存 TypedPtr（类似 Java 引用）
    Reference(TypedPtr),
}

impl ValueSlot {
    pub fn is_primitive(&self) -> bool {
        matches!(self, ValueSlot::Primitive(_))
    }

    pub fn is_reference(&self) -> bool {
        matches!(self, ValueSlot::Reference(_))
    }

    pub fn as_ref(&self) -> Option<&TypedPtr> {
        match self {
            ValueSlot::Reference(r) => Some(r),
            _ => None,
        }
    }
}

macro_rules! define_value_type_tags {
    // 入口：匹配 `Variant` 或 `Variant(FieldType)`
    ($($variant:ident $( ( $($field:ty),* ) )?),* $(,)?) => {
        #[allow(non_upper_case_globals)]
        pub mod var_type_code {
            use crate::value::VarTypeCodeType;
            define_value_type_tags!(@step 0, $($variant),*);
        }
    };

    // 递归：为当前变体生成常量，编号从 0 递增
    (@step $n:expr, $head:ident $(, $tail:ident)* $(,)?) => {
        pub const $head: VarTypeCodeType = $n;
        define_value_type_tags!(@step $n + 1, $($tail),*);
    };

    // 递归终止
    (@step $n:expr,) => {};
}

// 调用时直接复制枚举变体即可，字段类型会被忽略
define_value_type_tags!(
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Bool,
    String,
    Ptr,
    Null,
);

/// 值类型（可递归描述指针）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValueType {
    /// 基础类型
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
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

    #[must_use]
    pub fn is_ptr(&self) -> bool {
        matches!(self, ValueType::Ptr(_))
    }

    /// 是否为引用类型（在变量槽中应存 TypedPtr）
    pub fn is_reference_type(&self) -> bool {
        matches!(self, ValueType::String | ValueType::Ptr(_))
    }

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
// pub mod var_type_code {
//     use crate::value::VarTypeCodeType;
//     #[derive(Debug)]
//     #[repr(u8)]
//     enum VarTypeEnum {
//         U8,
//         U16,
//         U32,
//         U64,
//         I8,
//         I16,
//         I32,
//         I64,
//         F32,
//         F64,
//         Bool,
//         String,
//         Pointer,
//     }
//     pub const U8: VarTypeCodeType = VarTypeCodeType as u8;
//     pub const U16: VarTypeCodeType = 1;
//     pub const U32: VarTypeCodeType = 2;
//     pub const U64: VarTypeCodeType = 3;
//     pub const F32: VarTypeCodeType = 4;
//     pub const F64: VarTypeCodeType = 5;
//     pub const BOOL: VarTypeCodeType = 6;
//     pub const STRING: VarTypeCodeType = 7;
//     pub const POINTER: VarTypeCodeType = 8;
// }

pub fn var_type_asm_to_code(var_type: &[u8]) -> Result<VarTypeCodeType, crate::exceptions::Error> {
    match var_type {
        b"u8" => Ok(var_type_code::U8),
        b"u16" => Ok(var_type_code::U16),
        b"u32" => Ok(var_type_code::U32),
        b"u64" => Ok(var_type_code::U64),
        b"i8" => Ok(var_type_code::I8),
        b"i16" => Ok(var_type_code::I16),
        b"i32" => Ok(var_type_code::I32),
        b"i64" => Ok(var_type_code::I64),
        b"f32" => Ok(var_type_code::F32),
        b"f64" => Ok(var_type_code::F64),
        b"bool" => Ok(var_type_code::Bool),
        b"str" => Ok(var_type_code::String),
        b"ptr" => Err(crate::exceptions::Error::InvalidType("ptr is not supported yet".to_string())),
        _ => Err(crate::exceptions::Error::InvalidType(String::from_utf8_lossy(var_type).into_owned()))
    }
}

pub fn var_type_asm_to_type(var_type: &[u8]) -> Result<ValueType, crate::exceptions::Error> {
    match var_type {
        b"u8" => Ok(ValueType::U8),
        b"u16" => Ok(ValueType::U16),
        b"u32" => Ok(ValueType::U32),
        b"u64" => Ok(ValueType::U64),
        b"i8" => Ok(ValueType::I8),
        b"i16" => Ok(ValueType::I16),
        b"i32" => Ok(ValueType::I32),
        b"i64" => Ok(ValueType::I64),
        b"f32" => Ok(ValueType::F32),
        b"f64" => Ok(ValueType::F64),
        b"bool" => Ok(ValueType::Bool),
        b"str" => Ok(ValueType::String),
        b"ptr" => Err(crate::exceptions::Error::InvalidType("ptr is not supported yet".to_string())),
        _ => Err(crate::exceptions::Error::InvalidType(String::from_utf8_lossy(var_type).into_owned()))
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

pub(crate) fn get_type_from_bin<R: std::io::BufRead>(input: &mut R, cursor: &mut crate::parser::exec::CursorPos)
    -> Result<ValueType, crate::parser::exec::ExecError>
{
    let mut buf = [0u8; 1];
    cursor.read_exact(input, &mut buf)?;
    match buf[0] {
        var_type_code::U8 => Ok(ValueType::U8),
        var_type_code::U16 => Ok(ValueType::U16),
        var_type_code::U32 => Ok(ValueType::U32),
        var_type_code::U64 => Ok(ValueType::U64),
        var_type_code::I8 => Ok(ValueType::I8),
        var_type_code::I16 => Ok(ValueType::I16),
        var_type_code::I32 => Ok(ValueType::I32),
        var_type_code::I64 => Ok(ValueType::I64),
        var_type_code::F32 => Ok(ValueType::F32),
        var_type_code::F64 => Ok(ValueType::F64),
        var_type_code::Bool => Ok(ValueType::Bool),
        var_type_code::String => Ok(ValueType::String),
        var_type_code::Ptr => {
            Ok(ValueType::Ptr(std::sync::Arc::from(get_type_from_bin(input, cursor)?)))
        }
        var_type_code::Ptr..=u8::MAX => todo!(),
    }
}

pub(crate) fn construct_var_from_asm<R: std::io::BufRead>(
    input: &mut R, type_: &ValueType, cursor_pos: &mut crate::parser::asm::CursorPos
) -> Result<Var, crate::parser::asm::AsmError> {
    use crate::parser::hex;
    match type_ {
        ValueType::U8 => { Ok(Var::U8(hex::hex_to_u8(input, cursor_pos)?)) }
        ValueType::U16 => { Ok(Var::U16(hex::hex_to_u16(input, cursor_pos)?)) }
        ValueType::U32 => { Ok(Var::U32(hex::hex_to_u32(input, cursor_pos)?)) }
        ValueType::U64 => { Ok(Var::U64(hex::hex_to_u64(input, cursor_pos)?)) }
        ValueType::I8 | ValueType::I16 | ValueType::I32 | ValueType::I64 => {
            todo!()
        }
        ValueType::F32 | ValueType::F64 => {
            todo!()
        }
        ValueType::Bool => {
            todo!()
        }
        ValueType::String => {
            // TODO: 从输入读取字符串，在内存池分配，返回 Var::String(ptr)
            todo!()
        }
        ValueType::Ptr(inner) => {
            // TODO: 读取指针位置，构造 TypedPtr
            todo!()
        }
        ValueType::Null => Ok(Var::Null),
    }
}