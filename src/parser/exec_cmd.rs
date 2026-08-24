/// 每个函数都假设指令标识已被读取

use crate::parser::exec;
use crate::parser::utils;

pub(crate) fn add(operand_stack: &mut Vec<crate::value::Var>, cursor: &mut exec::CursorPos) -> Result<(), exec::ExecError> {
    // let dest = utils::read_bin_to_u32(input, "dest arg of add", cursor)?;

    let src1 = operand_stack.pop()
        .ok_or_else(|| exec::ExecError::new(crate::exceptions::Error::OutOfMemory("(ADD src1) operand stack underflow".to_string()), *cursor))?;
    let src2 = operand_stack.pop()
        .ok_or_else(|| exec::ExecError::new(crate::exceptions::Error::OutOfMemory("(ADD src2) operand stack underflow".to_string()), *cursor))?;

    // 一次性匹配两个值的类型
    match (&src1, &src2) {
        (crate::value::Var::U32(a), crate::value::Var::U32(b)) => {
            operand_stack.push(crate::value::Var::U32(a + b));
            Ok(())
        }
        _ => Err(exec::ExecError::new(
            crate::exceptions::Error::InvalidType(
                format!("(ADD) Type '{src1:?}' and type '{src2:?}' not equal"),
            ),
            *cursor,
        )),
    }
}

pub(crate) fn mul(operand_stack: &mut Vec<crate::value::Var>, cursor: &mut exec::CursorPos) -> Result<(), exec::ExecError> {
    // let dest = utils::read_bin_to_u32(input, "dest arg of add", cursor)?;

    let src1 = operand_stack.pop()
        .ok_or_else(|| exec::ExecError::new(crate::exceptions::Error::OutOfMemory("(MUL src1) operand stack underflow".to_string()), *cursor))?;
    let src2 = operand_stack.pop()
        .ok_or_else(|| exec::ExecError::new(crate::exceptions::Error::OutOfMemory("(MUL src2) operand stack underflow".to_string()), *cursor))?;

    // 一次性匹配两个值的类型
    match (&src1, &src2) {
        (crate::value::Var::U32(a), crate::value::Var::U32(b)) => {
            operand_stack.push(crate::value::Var::U32(a * b));
            Ok(())
        }
        _ => Err(exec::ExecError::new(
            crate::exceptions::Error::InvalidType(
                format!("(MUL) Type '{src1:?}' and type '{src2:?}' not equal"),
            ),
            *cursor,
        )),
    }
}

pub(crate) fn movc<R: std::io::BufRead>(input: &mut R, env: &mut crate::env::Env, cursor: &mut exec::CursorPos) -> Result<(), exec::ExecError> {
    let src_val = utils::read_bin_to_u32(input, "src_val of movc", cursor)?;
    let dest_var_id = utils::read_bin_to_u32(input, "dest_var_id of movc", cursor)?;
    exec::ok_or_err(env.set_var_value(crate::value::VarId(dest_var_id), crate::value::Var::U32(src_val)), cursor)
}

pub(crate) fn newv<R: std::io::BufRead>(
    input: &mut R,
    env: &mut crate::env::Env,
    cursor: &mut exec::CursorPos,
) -> Result<(), exec::ExecError> {
    let type_ = crate::value::get_type_from_bin(input, cursor)?;
    let var_id = utils::read_bin_to_u32(input, "var_id of newv", cursor)?;
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
        crate::value::ValueType::String => {
            let pos = {
                let mut pool = env.memory_pool.write().unwrap();
                let pos = exec::ok_or_err(pool.alloc(), cursor)?;
                // 初始化为空字符串
                exec::ok_or_err(pool.set(pos, crate::memory::MemBlock::String(String::new())), cursor)?;
                pos
            };
            env.insert_var(id, crate::value::TypedPtr { pos, ty: type_ })
        }

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
    let var_id = crate::value::VarId(utils::read_bin_to_u32(input, "var_id of delv", cursor)?);
    exec::ok_or_err(env.drop_var(var_id), cursor)
}

pub(crate) fn popvar<R: std::io::BufRead>(
    input: &mut R, env: &mut crate::env::Env, operand_stack: &mut Vec<crate::value::Var>, cursor: &mut exec::CursorPos
) -> Result<(), exec::ExecError> {
    let var_id = utils::read_bin_to_u32(input, "var_id of popvar", cursor)?;
    let var = operand_stack.pop();
    match var {
        Some(var) => exec::ok_or_err(env.set_var_value(crate::value::VarId(var_id), var), cursor),
        None => Err(exec::ExecError::new(crate::exceptions::Error::NotFound("(POPVAR) Operand stack is empty".to_string()), *cursor))
    }
}

pub(crate) fn pushvar<R: std::io::BufRead>(
    input: &mut R, env: &mut crate::env::Env, operand_stack: &mut Vec<crate::value::Var>, cursor: &mut exec::CursorPos
) -> Result<(), exec::ExecError> {
    let var_id = utils::read_bin_to_u32(input, "var_id of loadvar", cursor)?;
    operand_stack.push(exec::ok_or_err(env.get_var(&crate::value::VarId(var_id)), cursor)?);
    Ok(())
}

pub(crate) fn dup(
    operand_stack: &mut Vec<crate::value::Var>, cursor: &mut exec::CursorPos
) -> Result<(), exec::ExecError> {
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