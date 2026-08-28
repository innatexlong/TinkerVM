/// 每个函数都假设指令标识已被读取

use crate::parser::exec;
use crate::parser::utils;

macro_rules! define_binary_op {
    // 用法：define_binary_op!(add, +, "ADD");
    ($func:ident, $op:tt, $op_name:expr, $($allowed_type:ident)|*) => {
        pub(crate) fn $func(
            operand_stack: &mut Vec<crate::value::Var>,
            cursor: &mut exec::CursorPos,
        ) -> Result<(), exec::ExecError> {
            let src1 = operand_stack.pop()
                .ok_or_else(|| {
                    exec::ExecError::new(
                        crate::exceptions::Error::OutOfIndex(
                            format!("({} src1) operand stack underflow", $op_name),
                        ),
                        *cursor,
                    )
                })?;
            let src2 = operand_stack.pop()
                .ok_or_else(|| {
                    exec::ExecError::new(
                        crate::exceptions::Error::OutOfIndex(
                            format!("({} src2) operand stack underflow", $op_name),
                        ),
                        *cursor,
                    )
                })?;

            #[cfg(debug_assertions)]
            { println!("{} src1: {:?}, src2: {:?}", $op_name, src1, src2); }
            match (&src1, &src2) {
                // 为每种数值类型生成一个分支
                $(
                    (crate::value::Var::$allowed_type(a), crate::value::Var::$allowed_type(b)) => {
                        operand_stack.push(crate::value::Var::$allowed_type(a $op b));
                        Ok(())
                    }
                )*
                _ => {
                    Err(exec::ExecError::new(
                        crate::exceptions::Error::InvalidType(
                            format!(
                                "({}) Type '{:?}' and type '{:?}' doesn't support this operator",
                                $op_name, src1, src2
                            ),
                        ),
                        *cursor,
                    ))
                }
            }
        }
    };
}

macro_rules! define_integer_binary_op {
    ($func:ident, $op:tt, $op_name:expr) => {
        define_binary_op!($func, $op, $op_name, U8 | U16 | U32 | U64 | I8 | I16 | I32 | I64);
    }
}
macro_rules! define_integer_and_float_binary_op {
    ($func:ident, $op:tt, $op_name:expr) => {
        define_binary_op!($func, $op, $op_name, U8 | U16 | U32 | U64 | I8 | I16 | I32 | I64 | F32 | F64);
    }
}

define_integer_and_float_binary_op!(add, +, "ADD");
define_integer_and_float_binary_op!(sub, -, "SUB");
define_integer_and_float_binary_op!(mul, *, "MUL");
define_integer_and_float_binary_op!(div, /, "DIV");
define_integer_binary_op!(mod_, %, "MOD");
define_integer_binary_op!(shl, <<, "SHL");
define_integer_binary_op!(shr, >>, "SHR");
define_integer_binary_op!(bit_or, |, "BITOR");
define_integer_binary_op!(bit_and, &, "BITAND");
define_integer_binary_op!(bit_xor, ^, "BITXOR");

pub(crate) fn jmp<R: std::io::BufRead + std::io::Seek>(input: &mut R, labels: &Vec<u64>, cursor: &mut exec::CursorPos) -> Result<(), exec::ExecError> {
    let pos = utils::read_bin_to_u64(input, "pos of jmp", cursor)?;
    cursor.seek(input, std::io::SeekFrom::Start(pos))?;
    #[cfg(debug_assertions)]
    { println!("JMP {}", pos); }
    Ok(())
}

pub(crate) fn movc<R: std::io::BufRead>(input: &mut R, env: &mut crate::env::Env, constants: &[crate::value::Var], cursor: &mut exec::CursorPos) -> Result<(), exec::ExecError> {
    let val_index = utils::read_bin_to_u16(input, "val_index of movc", cursor)?;
    let dest_var_id = utils::read_bin_to_u16(input, "dest_var_id of movc", cursor)?;
    if let Some(val) = constants.get(val_index as usize) {
        exec::ok_or_err(env.set_var_value(crate::value::VarId(dest_var_id), val.clone()), cursor)
    } else {
        Err(exec::ExecError::new(crate::exceptions::Error::OutOfIndex(format!("(MOVC val_index) {val_index} out of index")), *cursor))
    }
}

pub(crate) fn newv<R: std::io::BufRead>(
    input: &mut R,
    env: &mut crate::env::Env,
    cursor: &mut exec::CursorPos,
) -> Result<(), exec::ExecError> {
    let type_ = crate::value::get_type_from_bin(input, cursor)?;
    let var_id = utils::read_bin_to_u16(input, "var_id of newv", cursor)?;
    let id = crate::value::VarId(var_id);

    // 根据类型分类处理，避免长时间持有锁
    match type_ {
        // ---------- 基本类型：直接内联到变量槽，不占用堆 ----------
        crate::value::ValueType::U8   => env.insert_primitive(id, crate::value::Var::U8(0u8)),
        crate::value::ValueType::U16  => env.insert_primitive(id, crate::value::Var::U16(0u16)),
        crate::value::ValueType::U32  => env.insert_primitive(id, crate::value::Var::U32(0u32)),
        crate::value::ValueType::U64  => env.insert_primitive(id, crate::value::Var::U64(0u64)),
        crate::value::ValueType::I8   => env.insert_primitive(id, crate::value::Var::I8(0i8)),
        crate::value::ValueType::I16  => env.insert_primitive(id, crate::value::Var::I16(0i16)),
        crate::value::ValueType::I32  => env.insert_primitive(id, crate::value::Var::I32(0i32)),
        crate::value::ValueType::I64  => env.insert_primitive(id, crate::value::Var::I64(0i64)),
        crate::value::ValueType::F32  => env.insert_primitive(id, crate::value::Var::F32(0.0f32)),
        crate::value::ValueType::F64  => env.insert_primitive(id, crate::value::Var::F64(0.0f64)),
        crate::value::ValueType::Bool => env.insert_primitive(id, crate::value::Var::Bool(false)),

        // ---------- 引用类型：分配堆内存并存储引用 ----------
        crate::value::ValueType::Str => {
            let pos = {
                let mut pool = env.memory_pool.write().unwrap();
                let pos = exec::ok_or_err(pool.alloc(), cursor)?;
                // 初始化为空字符串
                exec::ok_or_err(pool.set(pos, crate::memory::MemBlock::Str(String::new())), cursor)?;
                pos
            };
            env.insert_var(id, crate::value::TypedPtr { pos, ty: type_ })
        }

        crate::value::ValueType::VoidPtr => env.insert_primitive(id, crate::value::Var::VoidPtr(crate::env::Pos(0usize))),

        crate::value::ValueType::Ptr(_) => {
            let pos = {
                let mut pool = env.memory_pool.write().unwrap();
                let pos = exec::ok_or_err(pool.alloc(), cursor)?;
                // 指针初始化为 Null，或根据指向类型设置默认值（如 Null 变体）
                exec::ok_or_err(pool.set(pos, crate::memory::MemBlock::Null), cursor)?;
                pos
            };
            // 注意 ty 应保留完整的 ValueType::Ptr(inner)
            env.insert_var(id, crate::value::TypedPtr { pos, ty: type_ })
        }

        crate::value::ValueType::Null => {
            // Null 不是一个可声明的变量类型，可返回错误
            return Err(exec::ExecError::new(
                crate::exceptions::Error::InvalidType("Cannot declare variable of type Null".into()),
                *cursor,
            ));
        }
    };

    Ok(())
}

pub(crate) fn delp<R: std::io::BufRead>(input: &mut R, env: &mut crate::env::Env, cursor: &mut exec::CursorPos) -> Result<(), exec::ExecError> {
    let pos = crate::env::Pos(utils::read_bin_to_u32(input, "ptr_var_id of delp", cursor)? as usize);
    exec::ok_or_err(env.memory_pool.write().unwrap().dealloc(pos), cursor)
}

pub(crate) fn delv<R: std::io::BufRead>(input: &mut R, env: &mut crate::env::Env, cursor: &mut exec::CursorPos) -> Result<(), exec::ExecError> {
    let var_id = crate::value::VarId(utils::read_bin_to_u16(input, "var_id of delv", cursor)?);
    exec::ok_or_err(env.drop_var(var_id), cursor)
}

pub(crate) fn pushvar<R: std::io::BufRead>(
    input: &mut R, env: &mut crate::env::Env, operand_stack: &mut Vec<crate::value::Var>, cursor: &mut exec::CursorPos
) -> Result<(), exec::ExecError> {
    let var_id = utils::read_bin_to_u16(input, "var_id of loadvar", cursor)?;
    operand_stack.push(exec::ok_or_err(env.get_var(&crate::value::VarId(var_id)), cursor)?);
    Ok(())
}

pub(crate) fn popvar<R: std::io::BufRead>(
    input: &mut R, env: &mut crate::env::Env, operand_stack: &mut Vec<crate::value::Var>, cursor: &mut exec::CursorPos
) -> Result<(), exec::ExecError> {
    let var_id = utils::read_bin_to_u16(input, "var_id of popvar", cursor)?;
    match operand_stack.pop() {
        Some(var) => exec::ok_or_err(env.set_var_value(crate::value::VarId(var_id), var), cursor),
        None => Err(exec::ExecError::new(crate::exceptions::Error::NotFound("(POPVAR) Operand stack is empty".to_string()), *cursor))
    }
}

pub(crate) fn storevar<R: std::io::BufRead>(
    input: &mut R, env: &mut crate::env::Env, operand_stack: &mut Vec<crate::value::Var>, cursor: &mut exec::CursorPos
) -> Result<(), exec::ExecError> {
    let var_id = utils::read_bin_to_u16(input, "var_id of storevar", cursor)?;
    match operand_stack.last() {
        Some(var) => exec::ok_or_err(env.set_var_value(crate::value::VarId(var_id), var.clone()), cursor),
        None => Err(exec::ExecError::new(crate::exceptions::Error::NotFound("(STOREVAR) Operand stack is empty".to_string()), *cursor))
    }
}

pub(crate) fn dup(operand_stack: &mut Vec<crate::value::Var>, cursor: &mut exec::CursorPos) -> Result<(), exec::ExecError> {
    if let Some(var) = operand_stack.last() {
        operand_stack.push(var.clone());
        Ok(())
    } else {
        Err(exec::ExecError::new(crate::exceptions::Error::NotFound("(DUP) Operand stack is empty".to_string()), *cursor))
    }
}

pub(crate) fn ldc<R: std::io::BufRead>(
    input: &mut R, constants: &[crate::value::Var], operand_stack: &mut Vec<crate::value::Var>, cursor: &mut exec::CursorPos
) -> Result<(), exec::ExecError> {
    let val_id = utils::read_bin_to_u8(input, "val_id of ldc", cursor)? as usize;
    operand_stack.push(constants[val_id].clone());
    Ok(())
}

pub(crate) fn pop(operand_stack: &mut Vec<crate::value::Var>, cursor: &mut exec::CursorPos) -> Result<(), exec::ExecError> {
    match operand_stack.pop() {
        Some(_) => Ok(()),
        None => Err(exec::ExecError::new(crate::exceptions::Error::OutOfIndex("(POP) Operand stack underflow".to_string()), *cursor))
    }
}


/// 将操作数栈顶的 `&Var` 引用转换为指定数值类型变体。
///
/// # 参数
/// - `$cursor`: 可解引用的位置信息（如 `&mut CursorPos`），用于错误报告。
/// - `$ori_var`: 表达式，类型为 `&crate::value::Var`（即栈顶元素的引用）。
/// - `($target_type_variant, $target_type_obj)`: 目标变体名（如 `U16`）和对应的 Rust 类型（如 `u16`）。
/// - `$($allowed_ori_type)|*`: 允许转换的源变体名称（用 `|` 分隔，如 `U8 | U16`）。
///
/// # 行为
/// 若 `$ori_var` 的变体在允许列表中，则提取其内部的引用值（如 `&u8`），**解引用**后通过
/// `< $target_type_obj >::from(…)` 转换为目标值，并包装为 `Var::$target_type_variant`。
/// 若变体不在列表中，则返回 `Err(exec::ExecError)` 并附带类型错误信息。
///
/// # 注意
/// - `$ori_var` **必须**是 `&Var`，因为宏内部用到了解引用。
/// - 确保 `$cursor` 可被 `*` 解引用（通常为 `&mut CursorPos`）。
///
/// # 示例
/// ```rust
/// # use crate::value::Var;
/// # let mut cursor = /* ... */;
/// # let val: &Var = &Var::U8(&8u8);
/// *operand_stack.last_mut().unwrap() = conv_top!(cursor, val, (U16, u16), U8 | U16);
/// // 若 val 为 Var::U8(&8u8)，结果变为 Var::U16(16u16)
/// ```
macro_rules! conv_top {
    ($cursor:ident, $ori_var:ident, ($target_type_variant:ident, $target_type_obj:ty), $($allowed_ori_type:ident)|*) => {
        match $ori_var {
            $(
                crate::value::Var::$allowed_ori_type(val_inside) => {
                    crate::value::Var::$target_type_variant(<$target_type_obj>::from(*val_inside))
                }
            )*
            _ => return Err(
                exec::ExecError::new(
                    crate::exceptions::Error::InvalidType(format!(
                        "(CONV_TOP!) Cannot convert {:?} to {:?}", $ori_var, &[$(crate::value::ValueType::$allowed_ori_type),*]
                    )), *$cursor
                )
            ),
        }
    };
}

pub(crate) fn conv_top<R: std::io::BufRead>(
    input: &mut R, operand_stack: &mut Vec<crate::value::Var>, cursor: &mut exec::CursorPos
) -> Result<(), exec::ExecError> {
    let target_type = crate::value::get_type_from_bin(input, cursor)?;
    match operand_stack.last() {
        Some(val) => {
            match target_type {
                crate::value::ValueType::U16 => {
                    *exec::ok_or_err(
                        operand_stack.last_mut().ok_or_else(|| crate::exceptions::Error::OutOfIndex("(CONV_TOP) Operand stack underflow".to_string())), cursor
                    )? = conv_top!(cursor, val, (U16, u16), U8 | U16);
                    Ok(())
                },
                crate::value::ValueType::U32 => {
                    *exec::ok_or_err(
                        operand_stack.last_mut().ok_or_else(|| crate::exceptions::Error::OutOfIndex("(CONV_TOP) Operand stack underflow".to_string())), cursor
                    )? = conv_top!(cursor, val, (U32, u32), U8 | U16 | U32);
                    Ok(())
                },
                crate::value::ValueType::Null => Err(exec::ExecError::new(crate::exceptions::Error::InvalidType("Cannot convert any types to Null".to_string()), *cursor)),
                _ => Err(exec::ExecError::new(crate::exceptions::Error::InvalidType(format!("(CONV_TOP) Cannot convert {val:?} to {target_type:?}")), *cursor)),
            }
        }
        None => Err(exec::ExecError::new(crate::exceptions::Error::NotFound("(CONV_TOP) Operand stack is empty".to_string()), *cursor))
    }
}