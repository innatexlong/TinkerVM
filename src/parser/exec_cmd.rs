/// 每个函数都假设指令标识已被读取

use crate::parser::exec;
use crate::parser::utils;

pub(crate) fn add(operand_stack: &mut Vec<crate::value::Var>, cursor: &mut exec::CursorPos) -> Result<(), exec::ExecError> {
    // let dest = utils::read_bin_to_u32(input, "dest arg of add", cursor)?;

    let src1 = operand_stack.pop()
        .ok_or_else(|| exec::ExecError::new(crate::exceptions::Error::OutOfMemory("operand stack for add src1 underflow".to_string()), *cursor))?;
    let src2 = operand_stack.pop()
        .ok_or_else(|| exec::ExecError::new(crate::exceptions::Error::OutOfMemory("operand stack for add src2 underflow".to_string()), *cursor))?;

    match src1 {
        crate::value::Var::U32(src1_val) => {
            if let crate::value::Var::U32(src2_val) = src2 {
                // TODO: check the type of dest_var
                operand_stack.push(crate::value::Var::U32(src1_val + src2_val));
                Ok(())
            }
            else { Err(exec::ExecError::new(crate::exceptions::Error::InvalidType("src2 must be u32".to_string()), *cursor)) }
        }
        _ => Err(exec::ExecError::new(crate::exceptions::Error::InvalidType(format!("{src1:?}")), *cursor))
    }
}

pub(crate) fn movc<R: std::io::BufRead>(input: &mut R, env: &mut crate::env::Env, cursor: &mut exec::CursorPos) -> Result<(), exec::ExecError> {
    let src_val = utils::read_bin_to_u32(input, "src_val of movc", cursor)?;
    let dest_var_id = utils::read_bin_to_u32(input, "dest_var_id of movc", cursor)?;
    exec::ok_or_err(env.set_var_value(crate::value::VarId(dest_var_id), crate::value::Var::U32(src_val)), *cursor)
}

pub(crate) fn newv<R: std::io::BufRead>(input: &mut R, env: &mut crate::env::Env, cursor: &mut exec::CursorPos) -> Result<(), exec::ExecError> {
    let type_ = crate::value::get_type_from_bin(input, cursor)?;
    let var_id = utils::read_bin_to_u32(input, "var_id of newv", cursor)?;
    let mut pool_guard = env.memory_pool.write().unwrap();
    let pos = exec::ok_or_err(pool_guard.alloc(), *cursor)?;
    match type_ {
        crate::value::ValueType::U32 => {
            exec::ok_or_err(pool_guard.set(pos, crate::memory::MemBlock::U32(0u32)), *cursor)?;
        }
        _ => { return Err(exec::ExecError::new(crate::exceptions::Error::InvalidType(format!("Unsupported type {type_:?}")), *cursor)) }
    }
    exec::ok_or_err(env.set_var_pos(crate::value::VarId(var_id), crate::value::TypedPtr { pos, ty: type_ }), *cursor)
}

pub(crate) fn delp<R: std::io::BufRead>(input: &mut R, env: &mut crate::env::Env, cursor: &mut exec::CursorPos) -> Result<(), exec::ExecError> {
    let pos = crate::env::Pos(utils::read_bin_to_u32(input, "ptr_var_id of delp", cursor)? as usize);
    exec::ok_or_err(env.memory_pool.write().unwrap().dealloc(pos), *cursor)
}

pub(crate) fn delv<R: std::io::BufRead>(input: &mut R, env: &mut crate::env::Env, cursor: &mut exec::CursorPos) -> Result<(), exec::ExecError> {
    let var_id = crate::value::VarId(utils::read_bin_to_u32(input, "var_id of delv", cursor)?);
    exec::ok_or_err(env.drop_var(var_id), *cursor)
}

pub(crate) fn popvar<R: std::io::BufRead>(
    input: &mut R, env: &mut crate::env::Env, operand_stack: &mut Vec<crate::value::Var>, cursor: &mut exec::CursorPos
) -> Result<(), exec::ExecError> {
    let var_id = utils::read_bin_to_u32(input, "var_id of storevar", cursor)?;
    let var = operand_stack.pop();
    match var {
        Some(var) => exec::ok_or_err(env.set_var_value(crate::value::VarId(var_id), var), *cursor),
        None => Err(exec::ExecError::new(crate::exceptions::Error::NotFound("Operand stack is empty".to_string()), *cursor))
    }
}

pub(crate) fn pushvar<R: std::io::BufRead>(
    input: &mut R, env: &mut crate::env::Env, operand_stack: &mut Vec<crate::value::Var>, cursor: &mut exec::CursorPos
) -> Result<(), exec::ExecError> {
    let var_id = utils::read_bin_to_u32(input, "var_id of loadvar", cursor)?;
    operand_stack.push(exec::ok_or_err(env.get_var(&crate::value::VarId(var_id)), *cursor)?);
    Ok(())
}

pub(crate) fn ldc<R: std::io::BufRead>(
    input: &mut R, constants: &[crate::value::Var], operand_stack: &mut Vec<crate::value::Var>, cursor: &mut exec::CursorPos
) -> Result<(), exec::ExecError> {
    let val_id = utils::read_bin_to_u8(input, "val_id of ldc", cursor)? as usize;
    operand_stack.push(constants[val_id].clone());
    Ok(())
}