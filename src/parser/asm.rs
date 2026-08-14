// 内部已处理空格，外部无须处理
pub(crate) fn read_identifier<R: std::io::BufRead>(input: &mut R) -> Result<Option<Vec<u8>>, crate::exceptions::Error> {
    let mut id = Vec::new();
    let n = input
        .read_until(b' ', &mut id)?;
    if n == 0 {
        return Ok(None); // 没有更多数据
    }
    // // 可选：去掉分隔符
    // if id.last() == Some(&b' ') {
    //     id.pop();
    // }
    while id.last() == Some(&b' ') || id.last() == Some(&b'\n') || id.last() == Some(&b'\r') {
        id.pop();
    }
    Ok(Some(id))
}

pub fn compile_assembly_cmd<R: std::io::BufRead, W: std::io::Write>(input: &mut R, output: &mut W)
                                                                    -> Result<(), crate::exceptions::Error> {
    use crate::parser::cmd::Cmd;
    use crate::parser::hex::hex_to_u32;
    match read_identifier(input)? {
        None => Err(crate::exceptions::Error::EOFError("Unexpected EOF when reading cmd identifier".into())),
        Some(id) => {
            if id == b"add" {
                output.write_all(&[Cmd::Add as u8])?;
                let dest = hex_to_u32(input)?.to_le_bytes().to_vec();
                let src1 = hex_to_u32(input)?.to_le_bytes().to_vec();
                let src2 = hex_to_u32(input)?.to_le_bytes().to_vec();
                output.write_all(dest.as_slice())?;
                output.write_all(src1.as_slice())?;
                output.write_all(src2.as_slice())?;
                Ok(())
            } else if id == b"movc" {
                output.write_all(&[Cmd::Movc as u8])?;
                let dest_var = hex_to_u32(input)?.to_le_bytes().to_vec();
                let src_val = hex_to_u32(input)?.to_le_bytes().to_vec();
                output.write_all(dest_var.as_slice())?;
                output.write_all(src_val.as_slice())?;
                Ok(())
            } else if id == b"mov" {
                output.write_all(&[Cmd::Mov as u8])?;
                let dest = hex_to_u32(input)?.to_le_bytes();
                let src_var = hex_to_u32(input)?.to_le_bytes();
                output.write_all(dest.as_slice())?;
                output.write_all(src_var.as_slice())?;
                Ok(())
            } else if id == b"retc" {
                output.write_all(&[Cmd::Retc as u8])?;
                let ret_type = hex_to_u32(input)?.to_le_bytes().to_vec();
                let ret_val = hex_to_u32(input)?.to_le_bytes();
                output.write_all(ret_type.as_slice())?;
                output.write_all(ret_val.as_slice())?;
                Ok(())
            } else if id == b"retv" {
                output.write_all(&[Cmd::Retv as u8])?;
                let ret_var = hex_to_u32(input)?.to_le_bytes();
                output.write_all(ret_var.as_slice())?;
                Ok(())
            } else if id == b"newv" {
                // TODO: Decide how to use newv
                output.write_all(&[Cmd::Newv as u8])?;
                let type_ = read_identifier(input)?;
                let type_ = match type_ {
                    Some(value) => value,
                    None => return Err(crate::exceptions::Error::EOFError("Unexpected EOF when reading type identifier".into())),
                };
                let id = hex_to_u32(input)?.to_le_bytes();
                crate::value::var_type_bytes_to_code(type_.as_slice())?;
                output.write_all(id.as_slice())?;
                Ok(())
            } else if id == b"newp" {
                output.write_all(&[Cmd::Newp as u8])?;
                let dest = hex_to_u32(input)?.to_le_bytes();
                let type_ = hex_to_u32(input)?;
                if crate::value::VarTypeCodeType::MIN as u32 > type_ && type_ > crate::value::VarTypeCodeType::MAX as u32 {
                    return Err(crate::exceptions::Error::InvalidPointer(format!("Pointer type {} for newp is invalid", type_)))
                }
                match type_ as crate::value::VarTypeCodeType {
                    crate::value::var_type_code::U32 => {},
                    crate::value::var_type_code::U64 |
                    crate::value::var_type_code::POINTER => return Err(
                        crate::exceptions::Error::InvalidPointer("Pointer for newp cannot be a **Var, considering using newvp".to_string())
                    ),
                    _ => return Err(crate::exceptions::Error::InvalidPointer(format!("Pointer type {} for newp is invalid", type_)))
                };
                output.write_all(dest.as_slice())?;
                output.write_all(type_.to_le_bytes().as_slice())?;
                Ok(())
            }
            // TODO: newvp
            else if id == b"nop" {
                output.write_all(&[Cmd::Nop as u8])?;
                Ok(())
            } else {
                Err(crate::exceptions::Error::InvalidOperation(
                    id.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ")
                ))
            }
        }
    }
}

pub fn compile_assembly<R: std::io::BufRead, W: std::io::Write>(input: &mut R, output: &mut W) -> Result<(), crate::exceptions::Error> {
    use crate::parser::utils::skip_whitespace_and_comments;
    loop {
        match skip_whitespace_and_comments(input) {
            Err(crate::exceptions::Error::EOFError(_)) => {
                return match output.flush() {
                    Ok(()) => Ok(()),
                    Err(e) => Err(crate::exceptions::Error::IOError(e.to_string())),
                };
            }
            Err(e) => return Err(e),
            Ok(()) => {}
        }
        // 检查是否到达 EOF
        let buf = input.fill_buf().map_err(|e| crate::exceptions::Error::IOError(e.to_string()))?;
        if buf.is_empty() {
            output.flush().map_err(|e| crate::exceptions::Error::IOError(e.to_string()))?;
            return Ok(());
        }
        match compile_assembly_cmd(input, output) {
            Err(crate::exceptions::Error::InvalidOperation(str))
            => return Err(crate::exceptions::Error::InvalidOperation(str)),
            Err(crate::exceptions::Error::EOFError(e))
            => return Err(crate::exceptions::Error::IOError(e).into()),
            Err(crate::exceptions::Error::IOError(str))
            => return Err(crate::exceptions::Error::IOError(format!("{}", str))),
            Err(crate::exceptions::Error::UnrecognizedError(str, code)) => return Err(crate::exceptions::Error::UnrecognizedError(str, code).into()),
            Err(e) => return Err(
                crate::exceptions::Error::UnrecognizedError(
                    e.to_string(), e.code()
                )
            ),
            Ok(()) => {}
        };
    }
}