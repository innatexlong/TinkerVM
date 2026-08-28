use crate::parser::utils::skip_whitespace_and_comments;

// 内部已处理空格，外部无须处理
pub(crate) fn read_identifier<R: std::io::BufRead>(
    input: &mut R, cursor_pos: &mut CursorPos,
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

fn read_type_identifier<R: std::io::BufRead>(
    input: &mut R,
    cursor_pos: &mut CursorPos,
) -> Result<Vec<u8>, AsmError> {
    let mut id = Vec::new();
    loop {
        let buf = input.fill_buf().map_err(|e| AsmError {
            error: crate::exceptions::Error::IOError(e.to_string()),
            pos: cursor_pos.clone(),
        })?;
        if buf.is_empty() {
            break;
        }
        let ch = buf[0];
        if ch.is_ascii_whitespace() || ch == b'<' || ch == b'>' {
            break;
        }
        id.push(ch);
        input.consume(1);
        cursor_pos.push_u8(ch);
    }
    Ok(id)
}

// 消费一个期望的字符，失败时返回错误
fn expect_char<R: std::io::BufRead>(
    input: &mut R,
    cursor_pos: &mut CursorPos,
    expected: u8,
) -> Result<(), AsmError> {
    let buf = input.fill_buf().map_err(|e| AsmError {
        error: crate::exceptions::Error::IOError(e.to_string()),
        pos: cursor_pos.clone(),
    })?;
    if buf.is_empty() {
        return Err(AsmError {
            error: crate::exceptions::Error::InvalidType(format!(
                "unexpected end of input, expected '{}'",
                expected as char
            )),
            pos: cursor_pos.clone(),
        });
    }
    if buf[0] != expected {
        return Err(AsmError {
            error: crate::exceptions::Error::InvalidType(format!(
                "expected '{}', found '{}'",
                expected as char, buf[0] as char
            )),
            pos: cursor_pos.clone(),
        });
    }
    let buf_first = buf[0];
    input.consume(1);
    cursor_pos.push_u8(buf_first);
    Ok(())
}

pub(crate) fn get_type_from_asm<R: std::io::BufRead>(
    input: &mut R,
    cursor_pos: &mut CursorPos,
) -> Result<crate::value::ValueType, AsmError> {
    // 跳过前导空白
    skip_whitespace_and_comments(input, cursor_pos)?;

    // 读取类型标识符
    let ident = read_type_identifier(input, cursor_pos)?;
    if ident.is_empty() {
        return Err(AsmError {
            error: crate::exceptions::Error::InvalidType(
                "expected type identifier, found '<', '>', or end of input".to_string(),
            ),
            pos: cursor_pos.clone(),
        });
    }

    if ident == b"ptr" {
        // 期望 '<'
        skip_whitespace_and_comments(input, cursor_pos)?;
        expect_char(input, cursor_pos, b'<')?;

        // 递归解析内部类型
        let inner = get_type_from_asm(input, cursor_pos)?;

        // 期望 '>'
        skip_whitespace_and_comments(input, cursor_pos)?;
        expect_char(input, cursor_pos, b'>')?;

        // 构造指针类型
        Ok(crate::value::ValueType::Ptr(std::sync::Arc::new(inner)))
    } else {
        // 尝试解析为基础类型
        crate::value::var_type_asm_to_type(&ident).map_err(|e| AsmError {
            error: e,
            pos: cursor_pos.clone(),
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct OutputCursorPos {
    pub pos: u64
}
impl OutputCursorPos {
    /// 以1为第一个字符
    #[inline]
    pub const fn new() -> Self {
        Self { pos: 0 }
    }
    #[inline]
    pub const fn with_pos(pos: u64) -> Self {
        Self { pos }
    }
    #[inline]
    pub const fn push_bytes(&mut self, bytes: &[u8]) -> () {
        self.pos += size_of_val(bytes) as u64;
    }
    #[inline]
    pub const fn push_u8(&mut self, byte: u8) -> () {
        self.pos += size_of_val(&byte) as u64;
    }
    #[inline]
    pub fn write_all<W: std::io::Write>(&mut self, writer: &mut W, buf: &[u8], cursor_pos: &CursorPos) -> Result<(), AsmError> {
        ok_or_err(writer.write_all(buf), cursor_pos)?;
        self.pos += buf.len() as u64;
        Ok(())
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

        match std::str::from_utf8(&self.buffer[..self.buf_len as usize]) {
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
    id: &[u8], input: &mut R, output: &mut W, cursor_pos: &mut CursorPos, output_pos: &mut OutputCursorPos
) -> Result<(), AsmError> {
    use crate::parser::cmd::cmd_u8;
    use crate::parser::hex::{hex_to_u8, hex_to_u16, hex_to_u32};
    match id {
        b"func" | b"endfunc" => Err(AsmError::new(crate::exceptions::Error::Duplicated("Nested function definition".to_string()), cursor_pos)),
        b"add" => output_pos.write_all(output, &[cmd_u8::ADD], cursor_pos),
        b"sub" => output_pos.write_all(output, &[cmd_u8::SUB], cursor_pos),
        b"mul" => output_pos.write_all(output, &[cmd_u8::MUL], cursor_pos),
        b"div" => output_pos.write_all(output, &[cmd_u8::DIV], cursor_pos),
        b"mod" => output_pos.write_all(output, &[cmd_u8::MOD], cursor_pos),
        b"bitand" => output_pos.write_all(output, &[cmd_u8::BIT_AND], cursor_pos),
        b"bitor" => output_pos.write_all(output, &[cmd_u8::BIT_OR], cursor_pos),
        b"xor" => output_pos.write_all(output, &[cmd_u8::XOR], cursor_pos),
        b"movc" => {
            let dest_var = hex_to_u16(input, cursor_pos)?.to_le_bytes();
            let val_index = hex_to_u16(input, cursor_pos)?.to_le_bytes();
            output_pos.write_all(output, &[cmd_u8::MOVC], cursor_pos)?;
            output_pos.write_all(output, dest_var.as_slice(), cursor_pos)?;
            output_pos.write_all(output, val_index.as_slice(), cursor_pos)?;
            Ok(())
        }
        b"mov" => {
            let dest = hex_to_u16(input, cursor_pos)?.to_le_bytes();
            let src_var = hex_to_u16(input, cursor_pos)?.to_le_bytes();
            output_pos.write_all(output, &[cmd_u8::MOV], cursor_pos)?;
            output_pos.write_all(output, dest.as_slice(), cursor_pos)?;
            output_pos.write_all(output, src_var.as_slice(), cursor_pos)?;
            Ok(())
        }
        b"retc" => {
            let ret_val = hex_to_u16(input, cursor_pos)?.to_le_bytes();
            output_pos.write_all(output, &[cmd_u8::RETC], cursor_pos)?;
            output_pos.write_all(output, ret_val.as_slice(), cursor_pos)?;
            Ok(())
        }
        b"retv" => {
            let ret_var = hex_to_u16(input, cursor_pos)?.to_le_bytes();
            output_pos.write_all(output, &[cmd_u8::RETV], cursor_pos)?;
            output_pos.write_all(output, ret_var.as_slice(), cursor_pos)?;
            Ok(())
        }
        b"popret" => output_pos.write_all(output, &[cmd_u8::POPRET], cursor_pos),
        b"newv" => {
            // TODO: Implement custom var types for `newv`
            let type_ = match read_identifier(input, cursor_pos)? {
                Some(value) => value,
                None => return Err(
                    AsmError::new(
                        crate::exceptions::Error::EOFError("(NEWV) Unexpected EOF when reading type identifier".into()),
                        cursor_pos
                    )
                ),
            };
            let id = hex_to_u16(input, cursor_pos)?.to_le_bytes();
            output_pos.write_all(output, &[cmd_u8::NEWV], cursor_pos)?;
            output_pos.write_all(output, &ok_or_err(crate::value::var_type_asm_to_code(type_.as_slice()), cursor_pos)?.to_le_bytes(), cursor_pos)?;
            output_pos.write_all(output, id.as_slice(), cursor_pos)?;
            Ok(())
        }
        // b"newp" => {
        //     use crate::value::ValueType;
        //     // todo!("newp is not implemented yet");
        //     ok_or_err(output.write_all(&[Cmd::Newp as u8]), cursor_pos)?;
        //     let dest = hex_to_u16(input, cursor_pos)?.to_le_bytes();
        //     // TODO: Implement this
        //     let type_ = get_type_from_asm(input, cursor_pos)?;
        //     // if crate::value::VarTypeCodeType::MIN as u32 > type_ && type_ > crate::value::VarTypeCodeType::MAX as u32 {
        //     //     return Err(crate::exceptions::Error::InvalidType(format!("Pointer type {} for newp is invalid", type_)))
        //     // }
        //     crate::value::construct_var_from_asm(input, &type_, cursor_pos)?;
        //     // match type_ {
        //     //     ValueType::U8 => output.write
        //     //     ValueType::U32 => output.write(&crate::value::var_type_code::U32.to_le_bytes())?;
        //     // }
        //     ok_or_err(output.write_all(dest.as_slice()), cursor_pos)?;
        //     Ok(())
        // }
        b"delv" => {
            let var_id = hex_to_u16(input, cursor_pos)?.to_le_bytes();
            output_pos.write_all(output, &[cmd_u8::DELV], cursor_pos)?;
            output_pos.write_all(output, var_id.as_slice(), cursor_pos)?;
            Ok(())
        }
        b"delp" => {
            let ptr_var_id = hex_to_u16(input, cursor_pos)?.to_le_bytes();
            output_pos.write_all(output, &[cmd_u8::DELP], cursor_pos)?;
            output_pos.write_all(output, ptr_var_id.as_slice(), cursor_pos)?;
            Ok(())
        }
        b"nop" => {
            output_pos.write_all(output, &[cmd_u8::NOP], cursor_pos)?;
            Ok(())
        }
        b"call" => {
            let func = hex_to_u32(input, cursor_pos)?.to_le_bytes();
            output_pos.write_all(output, &[cmd_u8::CALL], cursor_pos)?;
            output_pos.write_all(output, func.as_slice(), cursor_pos)?;
            Ok(())
        }
        b"pushvar" => {
            let var_id = hex_to_u16(input, cursor_pos)?.to_le_bytes();
            output_pos.write_all(output, &[cmd_u8::PUSHVAR], cursor_pos)?;
            output_pos.write_all(output, var_id.as_slice(), cursor_pos)?;
            Ok(())
        }
        b"popvar" => {
            let var_id = hex_to_u16(input, cursor_pos)?.to_le_bytes();
            output_pos.write_all(output, &[cmd_u8::POPVAR], cursor_pos)?;
            output_pos.write_all(output, var_id.as_slice(), cursor_pos)?;
            Ok(())
        }
        b"storevar" => {
            let var_id = hex_to_u16(input, cursor_pos)?.to_le_bytes();
            output_pos.write_all(output, &[cmd_u8::STOREVAR], cursor_pos)?;
            output_pos.write_all(output, var_id.as_slice(), cursor_pos)?;
            Ok(())
        }
        b"dup" => output_pos.write_all(output, &[cmd_u8::DUP], cursor_pos),
        b"ldc" => {
            let val_id = hex_to_u8(input, cursor_pos)?.to_le_bytes();
            output_pos.write_all(output, &[cmd_u8::LDC], cursor_pos)?;
            output_pos.write_all(output, val_id.as_slice(), cursor_pos)?;
            Ok(())
        }
        b"pop" => output_pos.write_all(output, &[cmd_u8::POP], cursor_pos),
        b"convtop" => {
            let type_ = match read_identifier(input, cursor_pos)? {
                Some(value) => value,
                None => return Err(
                    AsmError::new(
                        crate::exceptions::Error::EOFError("(CONVTOP) Unexpected EOF when reading type identifier".into()),
                        cursor_pos
                    )
                ),
            };
            output_pos.write_all(output, &[cmd_u8::CONV_TOP], cursor_pos)?;
            output_pos.write_all(output, &ok_or_err(crate::value::var_type_asm_to_code(type_.as_slice()), cursor_pos)?.to_le_bytes(), cursor_pos)?;
            Ok(())
        }
        _ => {
            let mut reader = std::io::Cursor::new(id);
            let mut temp_pos = CursorPos::new();

            if id.iter().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F')) {
                return match crate::parser::hex::hex_to_u64(&mut reader, &mut temp_pos) {
                    Ok(num) => Err(AsmError::new(
                        crate::exceptions::Error::InvalidIdentifier(
                            format!("Integer {num} is not a valid identifier, did you add redundant arguments?")
                        ),
                        cursor_pos,  // 使用原始位置
                    )),
                    Err(err) if matches!(err.error, crate::exceptions::Error::Overflow(_)) => Err(AsmError::new(
                        crate::exceptions::Error::InvalidIdentifier(
                            format!("'{}' looks like a valid integer but it overflows. Did you add redundant integer arguments?",
                                    String::from_utf8_lossy(id))
                        ),
                        cursor_pos,
                    )),
                    Err(err) => Err(err),
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



struct AssemblerState {
    func_opt: Option<(crate::env::FuncPtr, std::io::BufWriter<Vec<u8>>)>,
    constants: Vec<crate::value::Var>,
    labels: dashmap::DashMap<Vec<u8>, u64>,
}

impl AssemblerState {
    fn new() -> Self {
        Self {
            func_opt: None,
            constants: Vec::with_capacity(10),
            labels: dashmap::DashMap::with_capacity(16),
        }
    }

    fn start_func(&mut self, ptr: crate::env::FuncPtr, pos: &mut CursorPos) -> Result<(), AsmError> {
        if self.func_opt.is_some() {
            return Err(AsmError::new(
                crate::exceptions::Error::Duplicated(
                    "Nested functions are not allowed".to_string()
                ),
                pos
            ));
        }
        self.func_opt = Some((ptr, std::io::BufWriter::new(Vec::new())));
        self.constants.clear();
        self.labels.clear();
        Ok(())
    }

    fn end_func(&mut self, env: &crate::env::Env, pos: &mut CursorPos) -> Result<(), AsmError> {
        let (func_ptr, writer) = self.func_opt.take()
            .ok_or_else(|| AsmError::new(
                crate::exceptions::Error::SyntaxError("Unexpected endfunc".to_string()),
                pos
            ))?;

        // 这里需要额外错误码，故保留 map_err（也可以改为 ok_or_err 但需要实现 From）
        let bytes = writer.into_inner().map_err(|e| {
            AsmError::new(
                crate::exceptions::Error::UnrecognizedError(e.to_string(), 1),
                pos
            )
        })?;

        let label_ids: Vec<u64> = self.labels.iter().map(|pair| *pair.value()).collect();
        ok_or_err(env.register_func(func_ptr, bytes, label_ids, self.constants.clone().into()), pos)?;

        self.constants.clear();
        self.labels.clear();
        Ok(())
    }

    fn define_label(&mut self, name: Vec<u8>, pos: &OutputCursorPos) -> Result<(), AsmError> {
        self.labels.insert(name, pos.pos);
        Ok(())
    }

    fn get_label_pos(&self, name: &[u8], pos: &mut CursorPos) -> Result<u64, AsmError> {
        self.labels.get(name)
            .map(|entry| *entry.value())
            .ok_or_else(|| AsmError::new(
                crate::exceptions::Error::OutOfIndex(
                    format!("Unknown label {}", String::from_utf8_lossy(name))
                ),
                pos
            ))
    }
}

pub fn compile_assembly<R: std::io::BufRead>(
    input: &mut R,
    env: &crate::env::Env,
    cursor_pos: &mut CursorPos,
) -> Result<(), AsmError> {
    use crate::parser::utils::skip_whitespace_and_comments;
    use crate::parser::hex::hex_to_u32;

    let mut state = AssemblerState::new();
    let mut output_pos = OutputCursorPos::new();

    loop {
        // 跳过空白和注释，处理 EOF
        if let Err(e) = skip_whitespace_and_comments(input, cursor_pos) {
            return match e.error {
                crate::exceptions::Error::EOFError(_) => {
                    if state.func_opt.is_some() {
                        Err(AsmError::new(
                            crate::exceptions::Error::SyntaxError("Expected 'endfunc', got <EOF>".to_string()),
                            cursor_pos
                        ))
                    } else {
                        Ok(())
                    }
                }
                _ => Err(e),
            };
        }

        let id = match read_identifier(input, cursor_pos)? {
            Some(id) => id,
            None => {
                return if state.func_opt.is_none() {
                    Ok(())
                } else {
                    Err(AsmError::new(
                        crate::exceptions::Error::EOFError("Unexpected EOF when reading cmd identifier".to_string()),
                        cursor_pos
                    ))
                };
            }
        };

        match id.as_slice() {
            b"func" => {
                let ptr = crate::env::FuncPtr(hex_to_u32(input, cursor_pos)?);
                state.start_func(ptr, cursor_pos)?;
            }
            b"endfunc" => {
                state.end_func(env, cursor_pos)?;
            }
            b"label" => {
                let name = read_identifier(input, cursor_pos)?
                    .ok_or_else(|| AsmError::new(
                        crate::exceptions::Error::EOFError("Unexpected EOF when reading label id".to_string()),
                        cursor_pos
                    ))?;
                state.define_label(name, &output_pos)?;
            }
            b"jmp" => {
                let name = read_identifier(input, cursor_pos)?
                    .ok_or_else(|| AsmError::new(crate::exceptions::Error::EOFError("Missing label after jmp".into()), cursor_pos))?;

                // 提前检查函数上下文
                let label_pos = state.get_label_pos(&name, cursor_pos)?; // 未定义时内部应返回错误
                let mut output = &mut state.func_opt.as_mut()
                    .ok_or_else(|| AsmError::new(crate::exceptions::Error::SyntaxError("jmp outside function".into()), cursor_pos))?
                    .1; // 假设 func_opt 是 (Something, &mut Vec<u8>)

                // 写入操作码和地址（建议根据目标位数选择大小）
                output_pos.write_all(&mut output, &[crate::parser::cmd::cmd_u8::JMP], cursor_pos)?;
                output_pos.write_all(&mut output, &label_pos.to_le_bytes(), cursor_pos)?;
            }
            b"cp" => {
                loop {
                    skip_whitespace_and_comments(input, cursor_pos)?;
                    match read_identifier(input, cursor_pos)? {
                        None => return Err(AsmError::new(
                            crate::exceptions::Error::EOFError("Unexpected end of constant pool".to_string()),
                            cursor_pos
                        )),
                        Some(token) => {
                            if token == b"endcp" {
                                break;
                            }
                            skip_whitespace_and_comments(input, cursor_pos)?;
                            let type_ = ok_or_err(crate::value::var_type_asm_to_type(&token), cursor_pos)?;
                            skip_whitespace_and_comments(input, cursor_pos)?;
                            let var = crate::value::construct_var_from_asm(input, &type_, cursor_pos)?;
                            state.constants.push(var);
                            skip_whitespace_and_comments(input, cursor_pos)?;
                        }
                    }
                }
            }
            b"endcp" => {
                return Err(AsmError::new(
                    crate::exceptions::Error::SyntaxError("endcp is invalid outside constant pools".to_string()),
                    cursor_pos
                ));
            }
            _ => {
                let output = &mut state.func_opt.as_mut()
                    .ok_or_else(|| AsmError::new(
                        crate::exceptions::Error::SyntaxError(
                            format!("Statement {} must be in a function", String::from_utf8_lossy(&id))
                        ),
                        cursor_pos
                    ))?
                    .1;
                compile_assembly_cmd(&id, input, output, cursor_pos, &mut output_pos)?;
            }
        }
    }
}

// pub fn compile_assembly<R: std::io::BufRead>(input: &mut R, env: &crate::env::Env, cursor_pos: &mut CursorPos) -> Result<(), AsmError> {
//     use crate::parser::utils::skip_whitespace_and_comments;
//     use crate::parser::hex::hex_to_u32;
//     let mut cur_func_opt: Option<(crate::env::FuncPtr, std::io::BufWriter<Vec<u8>>)> = None;
//     let mut constants: Vec<crate::value::Var> = Vec::new();
//     let labels = dashmap::DashMap::<Vec<u8>, usize>::with_capacity(16);
//     let mut cur_label_id = 0;
//     loop {
//         match skip_whitespace_and_comments(input, cursor_pos) {
//             Err(e) => {
//                 return match e.error {
//                     crate::exceptions::Error::EOFError(_) => {
//                         match cur_func_opt {
//                             Some((_func_ptr, ref mut _writer)) => {
//                                 Err(AsmError::new(crate::exceptions::Error::SyntaxError("Expected 'endfunc', got <EOF>".to_string()), cursor_pos))
//                             }
//                             None => Ok(()),
//                         }
//                     }
//                     _ => Err(e)
//                 }
//             }
//             Ok(()) => {}
//         }
//         match read_identifier(input, cursor_pos)? {
//             None => {
//                 return if cur_func_opt.is_none() {
//                     Ok(())
//                 } else {
//                     Err(AsmError::new(crate::exceptions::Error::EOFError("Unexpected EOF when reading cmd identifier".to_string()), cursor_pos))
//                 }
//             },
//             Some(id) => {
//                 if id == b"func" {
//                     if let Some((func_ptr, _)) = cur_func_opt {
//                         return Err(
//                             AsmError::new(
//                                 crate::exceptions::Error::Duplicated(
//                                     format!("Nested functions are not allowed. Current defining function is {}", func_ptr)
//                                 ), cursor_pos
//                             )
//                         )
//                     } else {
//                         cur_func_opt = Some((crate::env::FuncPtr(hex_to_u32(input, cursor_pos)?), std::io::BufWriter::new(Vec::new())));
//                     }
//                     continue;
//                 } else if id == b"endfunc" {
//                     if let Some((cur_func_id, cur_func_bin)) = cur_func_opt.take() {
//                         let bytes = cur_func_bin.into_inner().map_err(
//                             |e| AsmError::new(crate::exceptions::Error::UnrecognizedError(e.to_string(), 1), cursor_pos)
//                         )?;
//                         let mut labels_vec = Vec::with_capacity(labels.len());
//                         for pair in &labels {
//                             labels_vec.push(*pair.pair().1);
//                         }
//                         ok_or_err(env.register_func(cur_func_id, bytes, labels_vec, constants.into()), cursor_pos)?;
//                         constants = Vec::with_capacity(10);
//                         labels.clear();
//                         cur_label_id = 0;
//                     } else {
//                         return Err(AsmError::new(crate::exceptions::Error::SyntaxError("Unexpected endfunc".to_string()), cursor_pos))
//                     }
//                     continue;
//                 } else if let Some((_, ref mut output)) = cur_func_opt {
//                     if id == b"label" {
//                         match read_identifier(input, cursor_pos)? {
//                             None => return Err(AsmError::new(crate::exceptions::Error::EOFError("Unexpected EOF when reading label id".to_string()), cursor_pos)),
//                             Some(id) => {
//                                 labels.insert(id, cur_label_id);
//                                 cur_label_id += 1;
//                             }
//                         }
//                     } else if id == b"jmp" {
//                         match read_identifier(input, cursor_pos)? {
//                             None => return Err(AsmError::new(crate::exceptions::Error::EOFError("Unexpected EOF when reading label id".to_string()), cursor_pos)),
//                             Some(id) => {
//                                 ok_or_err(output.write_all(&[crate::parser::cmd::cmd_u8::JMP]), cursor_pos)?;
//                                 ok_or_err(
//                                     output.write_all(
//                                         &ok_or_err(
//                                             labels.get(&id).ok_or_else(
//                                                 || crate::exceptions::Error::OutOfIndex(
//                                                     format!("Unknown label {}", String::from_utf8_lossy(id.as_slice()).to_string())
//                                                 )
//                                             ),
//                                             cursor_pos
//                                         )?.value().to_le_bytes()
//                                     ), cursor_pos
//                                 )?;
//                             }
//                         }
//                     } else if id == b"cp" {
//                         loop {
//                             skip_whitespace_and_comments(input, cursor_pos)?;
//                             match read_identifier(input, cursor_pos)? {
//                                 None => return Err(AsmError::new(crate::exceptions::Error::EOFError("Unexpected end of constant pool".to_string()), cursor_pos)),
//                                 Some(id) => {
//                                     if id == b"endcp" {
//                                         break;
//                                     }
//                                     skip_whitespace_and_comments(input, cursor_pos)?;
//                                     let type_ = ok_or_err(crate::value::var_type_asm_to_type(id.as_slice()), cursor_pos)?;
//                                     skip_whitespace_and_comments(input, cursor_pos)?;
//                                     constants.push(crate::value::construct_var_from_asm(input, &type_, cursor_pos)?);
//                                     skip_whitespace_and_comments(input, cursor_pos)?;
//                                 }
//                             }
//                         }
//                     } else if id == b"endcp" {
//                         return Err(AsmError::new(crate::exceptions::Error::SyntaxError("endcp is invalid outside constant pools".to_string()), cursor_pos));
//                     } else { compile_assembly_cmd(id.as_slice(), input, output, cursor_pos)?; }
//                 } else {
//                     return Err(AsmError::new(crate::exceptions::Error::SyntaxError(
//                         format!("Statements {} must be in a function", String::from_utf8_lossy(id.as_slice()))
//                     ), cursor_pos))
//                 }
//             }
//         }
//     }
// }