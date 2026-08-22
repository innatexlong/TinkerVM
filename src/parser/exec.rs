use crate::parser::cmd::cmd_u8;
use crate::parser::exec_cmd;
use crate::parser::utils;

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub operand_stack: Vec<crate::value::Var>,
}
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct CursorPos {
    pub pos: usize,
}
impl CursorPos {
    #[inline]
    pub const fn new() -> Self {
        Self { pos: 1usize }
    }
    #[inline]
    pub const fn with_pos(pos: usize) -> Self {
        Self { pos }
    }
    #[inline]
    pub const fn push_u8(&mut self, _byte: u8) -> () {
        self.pos += 1;
    }
    #[inline]
    pub const fn push_bytes(&mut self, bytes: &[u8]) -> () {
        self.pos += bytes.len();
    }
    #[inline]
    pub fn read<R: std::io::BufRead>(&mut self, input: &mut R, buf: &mut [u8]) -> Result<usize, ExecError> {
        ok_or_err(input.read(buf), *self)
    }
    #[inline]
    pub fn read_exact<R: std::io::BufRead>(&mut self, input: &mut R, buf: &mut [u8]) -> Result<(), ExecError> {
        ok_or_err(input.read_exact(buf), *self)?;
        self.pos += buf.len();
        Ok(())
    }
}
#[derive(Debug, Clone, Eq, PartialEq, thiserror::Error)]
pub struct ExecError {
    pub error: crate::exceptions::Error,
    pub pos: CursorPos,
}
impl ExecError {
    pub fn new(error: crate::exceptions::Error, pos: CursorPos) -> Self {
        Self { error, pos }
    }
}
impl std::fmt::Display for ExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({:?} at 1)", self.error, self.pos)
    }
}

pub(crate) fn ok_or_err<ResOk, ResErr: std::error::Error>(result: Result<ResOk, ResErr>, pos: CursorPos) -> Result<ResOk, ExecError>
where crate::exceptions::Error: From<ResErr> {
    match result {
        Ok(value) => Ok(value),
        Err(error) => Err(ExecError::new(crate::exceptions::Error::from(error), pos)),
    }
}

pub fn func(
    func_id: &crate::env::FuncPtr, parent_env: std::sync::Arc<std::sync::RwLock<crate::env::Env>>, args: Vec<crate::value::Var>
) -> Result<crate::value::Var, ExecError> {
    // use std::io::Read;
    let mut cursor = CursorPos::new();
    let func_info = ok_or_err(parent_env.read().unwrap().get_func(func_id), cursor)?;
    let input = &mut std::io::BufReader::new(func_info.code.as_slice());
    let child_env_arc = std::sync::Arc::new(
        std::sync::RwLock::new(crate::env::Env::new(parent_env.read().unwrap().memory_pool.clone(), Some(parent_env.clone())))
    );
    let mut frame = CallFrame { operand_stack: args };

    loop {
        let mut buffer = [0u8; 1];
        match cursor.read_exact(input, &mut buffer) {
            Ok(()) => {
                match buffer {
                    // TODO: Automatically push bytes to cursor
                    [cmd_u8::ADD] => {
                        exec_cmd::add(&mut frame.operand_stack, &mut cursor)?;
                    }
                    [cmd_u8::SUB] => {
                    }
                    [cmd_u8::RETV] => {
                        let var_id = crate::value::VarId(utils::read_bin_to_u32(input, "var_id of retv", &mut cursor)?);
                        return ok_or_err(child_env_arc.read().unwrap().get_var(&var_id), cursor);
                    }
                    [cmd_u8::MOVC] => {
                        let mut child_env = child_env_arc.write().unwrap();
                        exec_cmd::movc(input, &mut child_env, &mut cursor)?;
                    }
                    [cmd_u8::NEWV] => {
                        let mut child_env = child_env_arc.write().unwrap();
                        exec_cmd::newv(input, &mut child_env, &mut cursor)?;
                    }
                    [cmd_u8::DELP] => {
                        let mut child_env = child_env_arc.write().unwrap();
                        exec_cmd::delp(input, &mut child_env, &mut cursor)?;
                    }
                    [cmd_u8::DELV] => {
                        let mut child_env = child_env_arc.write().unwrap();
                        exec_cmd::delv(input, child_env.as_mut_ref(), &mut cursor)?;
                    }
                    [cmd_u8::NOP] => { cursor.pos += size_of_val(&cmd_u8::NOP); continue },
                    [cmd_u8::CALL] => {
                        let func_id = crate::env::FuncPtr(utils::read_bin_to_u32(input, "func_id of call", &mut cursor)?);
                        frame.operand_stack.push(func(&func_id, child_env_arc.clone(), frame.operand_stack.clone())?);
                    }
                    [cmd_u8::PUSHVAR] => {
                        exec_cmd::pushvar(input, &mut child_env_arc.write().unwrap(), &mut frame.operand_stack, &mut cursor)?;
                    }
                    [cmd_u8::POPVAR] => {
                        exec_cmd::popvar(input, &mut child_env_arc.write().unwrap(), &mut frame.operand_stack, &mut cursor)?;
                    }
                    [cmd_u8::LDC] => {
                        exec_cmd::ldc(input, &func_info.constants, &mut frame.operand_stack, &mut cursor)?;
                    }
                    _ => { return Err(ExecError::new(crate::exceptions::Error::InvalidOperation(format!("bin '{:#X}'", buffer[0])), cursor)) }
                }
            }
            Err(error) => return match error.error {
                crate::exceptions::Error::EOFError(_)
                    => Err(ExecError::new(crate::exceptions::Error::EOFError(format!("Unexpected EOF when running VM, at {}", cursor.pos)), cursor)),
                _ => Err(error)
            }
        };
    }
}

pub fn run(root_env: std::sync::Arc<std::sync::RwLock<crate::env::Env>>) -> Result<u32, crate::exceptions::Error> {
    // TODO: for the true main function
    match func(&crate::env::FuncPtr(0), root_env, Vec::new()) {
        Ok(crate::value::Var::U8(_)) => Err(crate::exceptions::Error::InvalidType("main() should return u32, not u8".to_string())),
        Ok(crate::value::Var::U16(_)) => Err(crate::exceptions::Error::InvalidType("main() should return u32, not u16".to_string())),
        Ok(crate::value::Var::U32(value)) => Ok(value),
        Ok(crate::value::Var::U64(_)) => Err(crate::exceptions::Error::InvalidType("main() should return u32, not u64".to_string())),
        Ok(crate::value::Var::I8(_)) => Err(crate::exceptions::Error::InvalidType("main() should return u32, not u8".to_string())),
        Ok(crate::value::Var::I16(_)) => Err(crate::exceptions::Error::InvalidType("main() should return u32, not u16".to_string())),
        Ok(crate::value::Var::I32(_)) => Err(crate::exceptions::Error::InvalidType("main() should return u32, not u16".to_string())),
        Ok(crate::value::Var::I64(_)) => Err(crate::exceptions::Error::InvalidType("main() should return u32, not u64".to_string())),
        Err(e) => { eprintln!("at {:?}", e.pos); Err(e.error) },
        _ => todo!()
    }
}