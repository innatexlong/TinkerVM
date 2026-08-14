use crate::parser::cmd::{cmd_u8};
use crate::parser::exec_cmd;

pub fn get_var(env: &crate::env::Env, id: crate::value::VarId) -> Result<crate::value::Var, crate::exceptions::Error> {
    // 查找指针
    let ptr = env.lookup_var(&id)
        .ok_or_else(|| crate::exceptions::Error::VarNotFound(format!("{id}")))?;
    // 在锁内克隆值
    env.with_var(ptr, |var| var.clone())
}

pub fn func<R: std::io::BufRead>(input: &mut R, parent_env: &mut crate::env::Env) -> Result<crate::value::Var, crate::exceptions::Error> {
    let mut child_env = crate::env::Env::new(parent_env.memory_pool.clone(), parent_env.parent.clone());
    let mut buffer = [0u8; 1];
    loop {
        match input.read_exact(&mut buffer) {
            Ok(()) => {
                match buffer {
                    [cmd_u8::ADD] => {
                        exec_cmd::add(input, &mut child_env)?;
                    },
                    [cmd_u8::MOVC] => {
                        let mut src_val = [0u8; 4];
                        let mut dest = [0u8; 4];
                        match input.read(&mut src_val) {
                            Ok(size) => {
                                if size < 4 { return Err(crate::exceptions::Error::EOFError(format!("src_val of MOVC requires 4 bytes, got {size}"))); }
                            }
                            Err(err) => return Err(crate::exceptions::Error::from(err))
                        }
                        match input.read(&mut dest) {
                            Ok(size) => {
                                if size < 4 { return Err(crate::exceptions::Error::EOFError(format!("dest of MOVC requires 4 bytes, got {size}"))); }
                            }
                            Err(err) => return Err(crate::exceptions::Error::from(err))
                        }
                    }
                    _ => { return Err(crate::exceptions::Error::InvalidOperation(format!("bin '{:#X}'", buffer[0]))) }
                }
            },
            Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof
            => return Err(crate::exceptions::Error::EOFError("Unexpected EOF when running VM".to_string())),
            Err(err) => return Err(err.into())
        };
    }
}

pub fn run<R: std::io::BufRead>(input: &mut R, root_env: &mut crate::env::Env) -> Result<u32, crate::exceptions::Error> {
    // TODO: for the true main function
    match func(input, root_env) {
        Ok(crate::value::Var::U32(value)) => Ok(value),
        Ok(crate::value::Var::U64(value)) => { println!("[vm warn] main() should return u32, not u64"); Ok(value as u32) },
        Err(e) => Err(e),
        _ => todo!()
    }

    // match func(input, parent_env) {
    //     Ok(crate::value::Var::) => {
    //         match var_t {
    //             crate::env::VarType::U32(value) => {
    //                 Ok(value)
    //             }
    //             crate::env::VarType::U32Ptr(pos) => {
    //                 Err(Box::from(crate::exceptions::Error::InvalidVarType(format!("main() must return u32, not *u32({pos})"))))
    //             },
    //             crate::env::VarType::VoidPtr(_) => {
    //                 Err(Box::from(crate::exceptions::Error::InvalidVarType("main() must return u32, not *void".to_string())))
    //             },
    //             crate::env::VarType::None => todo!()
    //         }
    //     }
    //     Err(err) => Err(err)
    // }
}