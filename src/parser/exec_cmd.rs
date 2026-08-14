/// 每个函数都假设指令标识已被读取

use crate::parser::exec;

pub(crate) fn add<R: std::io::BufRead>(input: &mut R, env: &mut crate::env::Env) -> Result<(), crate::exceptions::Error> {
    let dest = crate::parser::utils::read_bin_to_u32(input)?;
    let src1 = crate::parser::utils::read_bin_to_u32(input)?;
    let src2 = crate::parser::utils::read_bin_to_u32(input)?;

    let src1_var = exec::get_var(&env, crate::value::VarId(src1))?;
    let src2_var = exec::get_var(&env, crate::value::VarId(src2))?;

    match src1_var {
        crate::value::Var::U32(src1_val) => {
            if let crate::value::Var::U32(src2_val) = src2_var {
                // TODO: check the type of dest_var
                env.set_var_current(crate::value::VarId(dest), crate::value::Var::U32(src1_val + src2_val))?;
                Ok(())
            }
            else { Err(crate::exceptions::Error::InvalidVarType("src2 must be u32".to_string())) }
        }
        _ => Err(crate::exceptions::Error::InvalidVarType(format!("{src1}")))
    }
}