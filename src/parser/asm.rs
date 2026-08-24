use crate::parser::utils::skip_whitespace_and_comments;

// 内部已处理空格，外部无须处理
pub(crate) fn read_identifier<R: std::io::BufRead>(
    input: &mut R,
    cursor_pos: &mut CursorPos,
) -> Result<Option<Vec<u8>>, AsmError> {
    // 跳过空白
    skip_whitespace_and_comments(input, cursor_pos)?;

    let mut id = Vec::new();
    loop {
        let buf = input.fill_buf()
            .map_err(|e| AsmError {
                error: crate::exceptions::Error::IOError(e.to_string()),
                pos: cursor_pos.clone(),
            })?;
        if buf.is_empty() {
            break;
        }
        let ch = buf[0];
        if ch.is_ascii_whitespace() {
            break;
        }
        id.push(ch);
        input.consume(1);
        cursor_pos.push_u8(ch);
    }

    if id.is_empty() {
        Ok(None)
    } else {
        Ok(Some(id))
    }
}

/// 对应UTF-8的位置
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CursorPos {
    pub line: u32,
    pub column: u32,
    buffer: [u8; 4],     // UTF-8 最大 4 字节
    buf_len: u32,      // 当前缓冲区有效长度
    pub func: Option<crate::env::FuncPtr>,
    is_static: bool,
}

impl std::fmt::Display for CursorPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

impl CursorPos {
    pub fn new() -> Self {
        Self {
            line: 1,
            column: 1,
            buffer: [0u8; 4],
            buf_len: 0,
            func: None,
            is_static: false,
        }
    }
    pub fn with_pos(line: u32, column: u32, func: Option<crate::env::FuncPtr>) -> Self {
        Self {
            line,
            column,
            buffer: [0u8; 4],
            buf_len: 0,
            func,
            is_static: false,
        }
    }

    pub fn read_exact<R: std::io::BufRead>(&mut self, input: &mut R, buf: &mut [u8]) -> Result<(), AsmError> {
        ok_or_err(input.read_exact(buf), self)?;
        if self.is_static { return Ok(()) }
        self.push_bytes(buf);
        Ok(())
    }
    pub fn read_until<R: std::io::BufRead>(&mut self, input: &mut R, byte: u8, buf: &mut Vec<u8>) -> Result<usize, AsmError> {
        let len = ok_or_err(input.read_until(byte, buf), self)?;
        if self.is_static { return Ok(len) }
        self.push_bytes(buf);
        Ok(len)
    }

    pub fn push_bytes(&mut self, bytes: &[u8]) -> () {
        if self.is_static { return }
        for &b in bytes {
            self.push_u8(b);
        }
    }
    pub fn push_u8(&mut self, byte: u8) -> () {
        if self.is_static { return }
        // 快速路径：缓冲区为空且字节是 ASCII（单字节）
        if self.buf_len == 0 && byte < 0x80 {
            match byte {
                b'\n' => {
                    self.line += 1;
                    self.column = 1;   // 列号从 1 开始
                }
                b'\r' => {
                    // 忽略 CR，不增加列（适应 CRLF 或孤立 CR）
                }
                _ => {
                    self.column += 1;
                }
            }
            return;
        }

        let buf_len_usize = self.buf_len as usize;
        // 慢速路径：需要拼接 UTF‑8 序列
        // if buf_len_usize >= self.buffer.len() {
        //     // 防御性清空（理论上不会触发）
        //     self.buf_len = 0;
        //     return;
        // }
        self.buffer[buf_len_usize] = byte;
        self.buf_len += 1;

        match std::str::from_utf8(&self.buffer[..buf_len_usize]) {
            Ok(s) => {
                // 解码成功，一定是一个字符（因为每次只加一个字节）
                let ch = s.chars().next().unwrap();
                match ch {
                    '\n' => {
                        self.line += 1;
                        self.column = 1;
                    }
                    '\r' => {
                        // 忽略 CR
                    }
                    _ => {
                        self.column += 1;
                    }
                }
                // 清空缓冲区，准备下一个字符
                self.buf_len = 0;
            }
            Err(e) => {
                if e.error_len().is_some() {
                    // 非法 UTF‑8 序列：丢弃整个缓存的字节，不增加列
                    self.buf_len = 0;
                } else {
                    // 不完整序列：保留缓冲区，等待更多字节
                }
            }
        }
    }

    /// 重置行列
    pub fn reset(&mut self, line: u32, column: u32) -> () {
        self.line = line;
        self.column = column;
        self.buf_len = 0;
    }

    pub fn set_static(&mut self, is_static: bool) -> Result<(), AsmError> {
        if self.is_static == is_static {
            return Err(
                AsmError::new(
                    crate::exceptions::Error::Duplicated(
                        "".to_string()
                    ),
                    self
                )
            )
        }
        self.is_static = is_static;
        Ok(())
    }
}

pub struct StaticCursorPos<'a> {
    pub cursor_pos: &'a mut CursorPos,
}
impl<'a> StaticCursorPos<'a> {
    pub fn new(cursor_pos: &'a mut CursorPos) -> Self {
        assert!(!cursor_pos.is_static, "Already in static mode");
        cursor_pos.is_static = true;
        Self { cursor_pos }
    }
}
impl Drop for StaticCursorPos<'_> {
    fn drop(&mut self) {
        assert!(self.cursor_pos.is_static, "Already disabled static mode");
        self.cursor_pos.is_static = false;
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub struct AsmError {
    pub error: crate::exceptions::Error,
    pub pos: CursorPos,
}
impl AsmError {
    pub fn new(error: crate::exceptions::Error, pos: &CursorPos) -> Self {
        Self {
            error,
            pos: *pos
        }
    }
}
impl std::fmt::Display for AsmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (at {})", self.error, self.pos)
    }
}

#[inline]
pub fn ok_or_err<ResOk, ResErr: std::error::Error>(result: Result<ResOk, ResErr>, pos: &CursorPos) -> Result<ResOk, AsmError>
where crate::exceptions::Error: From<ResErr> {
    match result {
        Ok(res) => Ok(res),
        Err(error) => Err(AsmError{ error: crate::exceptions::Error::from(error), pos: *pos })
    }
}

// TODO: Implement this
// pub(crate) fn get_type_from_asm<R: std::io::BufRead>(input: &mut R) -> Result<Option<VarTypeCodeType>, crate::exceptions::Error> {
//
// }

pub fn compile_assembly_cmd<R: std::io::BufRead, W: std::io::Write>(
    id: &[u8], input: &mut R, output: &mut W, cursor_pos: &mut CursorPos
) -> Result<(), AsmError> {
    use crate::parser::cmd::{Cmd, cmd_u8};
    use crate::parser::hex::{hex_to_u8, hex_to_u32};
    match id {
        b"func" | b"endfunc" => Err(AsmError::new(crate::exceptions::Error::Duplicated("Nested function definition".to_string()), cursor_pos)),
        b"add" => ok_or_err(output.write_all(&[cmd_u8::ADD]), cursor_pos),
        b"sub" => ok_or_err(output.write_all(&[cmd_u8::SUB]), cursor_pos),
        b"mul" => ok_or_err(output.write_all(&[cmd_u8::MUL]), cursor_pos),
        b"div" => ok_or_err(output.write_all(&[cmd_u8::DIV]), cursor_pos),
        b"mod" => ok_or_err(output.write_all(&[cmd_u8::MOD]), cursor_pos),
        b"movc" => {
            let src_val = hex_to_u32(input, cursor_pos)?.to_le_bytes().to_vec();
            let dest_var = hex_to_u32(input, cursor_pos)?.to_le_bytes().to_vec();
            ok_or_err(output.write_all(&[cmd_u8::MOVC]), cursor_pos)?;
            ok_or_err(output.write_all(src_val.as_slice()), cursor_pos)?;
            ok_or_err(output.write_all(dest_var.as_slice()), cursor_pos)?;
            Ok(())
        }
        b"mov" => {
            let dest = hex_to_u32(input, cursor_pos)?.to_le_bytes();
            let src_var = hex_to_u32(input, cursor_pos)?.to_le_bytes();
            ok_or_err(output.write_all(&[cmd_u8::MOV]), cursor_pos)?;
            ok_or_err(output.write_all(dest.as_slice()), cursor_pos)?;
            ok_or_err(output.write_all(src_var.as_slice()), cursor_pos)?;
            Ok(())
        }
        b"retc" => {
            let ret_type = hex_to_u32(input, cursor_pos)?.to_le_bytes().to_vec();
            let ret_val = hex_to_u32(input, cursor_pos)?.to_le_bytes();
            ok_or_err(output.write_all(&[cmd_u8::RETC]), cursor_pos)?;
            ok_or_err(output.write_all(ret_type.as_slice()), cursor_pos)?;
            ok_or_err(output.write_all(ret_val.as_slice()), cursor_pos)?;
            Ok(())
        }
        b"retv" => {
            let ret_var = hex_to_u32(input, cursor_pos)?.to_le_bytes();
            ok_or_err(output.write_all(&[cmd_u8::RETV]), cursor_pos)?;
            ok_or_err(output.write_all(ret_var.as_slice()), cursor_pos)?;
            Ok(())
        }
        b"popret" => ok_or_err(output.write_all(&[cmd_u8::POPRET]), cursor_pos),
        b"newv" => {
            // TODO: Implement custom var types for `newv`
            let type_ = read_identifier(input, cursor_pos)?;
            let type_ = match type_ {
                Some(value) => value,
                None => return Err(
                    AsmError::new(
                        crate::exceptions::Error::EOFError("Unexpected EOF when reading type identifier".into()),
                        cursor_pos
                    )
                ),
            };
            let id = hex_to_u32(input, cursor_pos)?.to_le_bytes();
            ok_or_err(output.write_all(&[cmd_u8::NEWV]), cursor_pos)?;
            ok_or_err(output.write_all(&ok_or_err(crate::value::var_type_asm_to_code(type_.as_slice()), cursor_pos)?.to_le_bytes()), cursor_pos)?;
            ok_or_err(output.write_all(id.as_slice()), cursor_pos)?;
            Ok(())
        }
        b"newp" => {
            use crate::value::ValueType;
            todo!("newp is not implemented yet");
            ok_or_err(output.write_all(&[Cmd::Newp as u8]), cursor_pos)?;
            let dest = hex_to_u32(input, cursor_pos)?.to_le_bytes();
            // TODO: Implement this
            // let type_ = get_type_from_asm(input)?;
            // if crate::value::VarTypeCodeType::MIN as u32 > type_ && type_ > crate::value::VarTypeCodeType::MAX as u32 {
            //     return Err(crate::exceptions::Error::InvalidPointer(format!("Pointer type {} for newp is invalid", type_)))
            // }
            // match type_ {
            //     ValueType::U8 => output
            //     ValueType::U32 => output.write(&crate::value::var_type_code::U32.to_le_bytes())?;
            // }
            ok_or_err(output.write_all(dest.as_slice()), cursor_pos)?;
            Ok(())
        }
        b"delv" => {
            let var_id = hex_to_u32(input, cursor_pos)?.to_le_bytes();
            ok_or_err(output.write_all(&[cmd_u8::DELV]), cursor_pos)?;
            ok_or_err(output.write_all(var_id.as_slice()), cursor_pos)?;
            Ok(())
        }
        b"delp" => {
            let ptr_var_id = hex_to_u32(input, cursor_pos)?.to_le_bytes();
            ok_or_err(output.write_all(&[cmd_u8::DELP]), cursor_pos)?;
            ok_or_err(output.write_all(ptr_var_id.as_slice()), cursor_pos)?;
            Ok(())
        }
        b"nop" => {
            ok_or_err(output.write_all(&[cmd_u8::NOP]), cursor_pos)?;
            Ok(())
        }
        b"call" => {
            let func = hex_to_u32(input, cursor_pos)?.to_le_bytes();
            ok_or_err(output.write_all(&[cmd_u8::CALL]), cursor_pos)?;
            ok_or_err(output.write_all(func.as_slice()), cursor_pos)?;
            Ok(())
        }
        b"pushvar" => {
            let var_id = hex_to_u32(input, cursor_pos)?.to_le_bytes();
            ok_or_err(output.write_all(&[cmd_u8::PUSHVAR]), cursor_pos)?;
            ok_or_err(output.write_all(var_id.as_slice()), cursor_pos)?;
            Ok(())
        }
        b"popvar" => {
            let var_id = hex_to_u32(input, cursor_pos)?.to_le_bytes();
            ok_or_err(output.write_all(&[cmd_u8::POPVAR]), cursor_pos)?;
            ok_or_err(output.write_all(var_id.as_slice()), cursor_pos)?;
            Ok(())
        }
        b"dup" => ok_or_err(output.write_all(&[cmd_u8::DUP]), cursor_pos),
        b"ldc" => {
            let val_id = hex_to_u8(input, cursor_pos)?.to_le_bytes();
            ok_or_err(output.write_all(&[cmd_u8::LDC]), cursor_pos)?;
            ok_or_err(output.write_all(val_id.as_slice()), cursor_pos)?;
            Ok(())
        }
        _ => {
            let mut reader = std::io::Cursor::new(id);
            let mut temp_pos = CursorPos::new();

            let token = String::from_utf8_lossy(id);

            if id.iter().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F')) {
                match crate::parser::hex::hex_to_u64(&mut reader, &mut temp_pos) {
                    Ok(num) => return Err(AsmError::new(
                        crate::exceptions::Error::InvalidIdentifier(
                            format!("Integer {num} is not a valid identifier, did you add redundant arguments?")
                        ),
                        cursor_pos,  // 使用原始位置
                    )),
                    Err(err) if matches!(err.error, crate::exceptions::Error::Overflow(_)) => return Err(AsmError::new(
                        crate::exceptions::Error::InvalidIdentifier(
                            format!("'{}' looks like a valid integer but it overflows. Did you add redundant integer arguments?",
                                    String::from_utf8_lossy(id))
                        ),
                        cursor_pos,
                    )),
                    Err(err) => return Err(err),
                };
            }

            Err(
                AsmError::new(
                    crate::exceptions::Error::InvalidIdentifier(
                        String::from_utf8_lossy(id).to_string()
                    ),
                cursor_pos
                )
            )
        }
    }
}

pub fn compile_assembly<R: std::io::BufRead>(input: &mut R, env: &crate::env::Env, cursor_pos: &mut CursorPos) -> Result<(), AsmError> {
    use crate::parser::utils::skip_whitespace_and_comments;
    use crate::parser::hex::hex_to_u32;
    let mut cur_func_opt: Option<(crate::env::FuncPtr, std::io::BufWriter<Vec<u8>>)> = None;
    let mut constants: Vec<crate::value::Var> = Vec::new();
    loop {
        match skip_whitespace_and_comments(input, cursor_pos) {
            Err(e) => {
                return match e.error {
                    crate::exceptions::Error::EOFError(_) => {
                        match cur_func_opt {
                            Some((_func_ptr, ref mut _writer)) => {
                                Err(AsmError::new(crate::exceptions::Error::SyntaxError("Expected 'endfunc', got <EOF>".to_string()), cursor_pos))
                            }
                            None => Ok(()),
                        }
                    }
                    _ => Err(e)
                }
            }
            Ok(()) => {}
        }
        match read_identifier(input, cursor_pos)? {
            None => {
                return if cur_func_opt.is_none() {
                    Ok(())
                } else {
                    Err(AsmError::new(crate::exceptions::Error::EOFError("Unexpected EOF when reading cmd identifier".to_string()), cursor_pos))
                }
            },
            Some(id) => {
                if id == b"func" {
                    if let Some((func_ptr, _)) = cur_func_opt {
                        return Err(
                            AsmError::new(
                                crate::exceptions::Error::Duplicated(
                                    format!("Nested functions are not allowed. Current defining function is {}", func_ptr)
                                ), cursor_pos
                            )
                        )
                    } else {
                        cur_func_opt = Some((crate::env::FuncPtr(hex_to_u32(input, cursor_pos)?), std::io::BufWriter::new(Vec::new())));
                    }
                    continue;
                } else if id == b"endfunc" {
                    if let Some((cur_func_id, cur_func_bin)) = cur_func_opt.take() {
                        let bytes = cur_func_bin.into_inner().map_err(
                            |e| AsmError::new(crate::exceptions::Error::UnrecognizedError(e.to_string(), 1), cursor_pos)
                        )?;
                        ok_or_err(env.register_func(cur_func_id, bytes, constants.into()), cursor_pos)?;
                        constants = Vec::with_capacity(10);
                    } else {
                        return Err(AsmError::new(crate::exceptions::Error::SyntaxError("Unexpected endfunc".to_string()), cursor_pos))
                    }
                    continue;
                } else if let Some((_, ref mut output)) = cur_func_opt {
                    if id == b"cp" {
                        loop {
                            skip_whitespace_and_comments(input, cursor_pos)?;
                            match read_identifier(input, cursor_pos)? {
                                None => return Err(AsmError::new(crate::exceptions::Error::EOFError("Unexpected end of constant pool".to_string()), cursor_pos)),
                                Some(id) => {
                                    if id == b"endcp" {
                                        break;
                                    }
                                    skip_whitespace_and_comments(input, cursor_pos)?;
                                    let type_ = ok_or_err(crate::value::var_type_asm_to_type(id.as_slice()), cursor_pos)?;
                                    skip_whitespace_and_comments(input, cursor_pos)?;
                                    constants.push(crate::value::construct_var_from_asm(input, &type_, cursor_pos)?);
                                    skip_whitespace_and_comments(input, cursor_pos)?;
                                }
                            }
                        }
                    } else if id == b"endcp" {
                        return Err(AsmError::new(crate::exceptions::Error::SyntaxError("endcp is invalid outside constant pools".to_string()), cursor_pos));
                    } else { compile_assembly_cmd(id.as_slice(), input, output, cursor_pos)?; }
                } else {
                    return Err(AsmError::new(crate::exceptions::Error::SyntaxError(
                        format!("Statements {} must be in a function", String::from_utf8_lossy(id.as_slice()))
                    ), cursor_pos))
                }
            }
        }
    }
}