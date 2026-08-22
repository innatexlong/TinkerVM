// parser/utils.rs
use crate::parser::{asm, exec};

pub(crate) fn read_non_whitespace_byte<R: std::io::BufRead>(input: &mut R) -> std::io::Result<Option<u8>> {
    loop {
        let buffer = input.fill_buf()?;          // 获取当前缓冲区内容
        if buffer.is_empty() {
            return Ok(None);                     // EOF
        }
        if let Some(&b) = buffer.iter().find(|&&c| !c.is_ascii_whitespace()) {
            // 找到第一个非空白字节，消费掉它之前的所有字节（含该字节）
            let pos = buffer.iter().position(|&c| c == b).unwrap(); // 安全，因为已知存在
            input.consume(pos + 1);
            return Ok(Some(b));
        } else {
            // 整个缓冲区都是空白，消耗全部
            let len = buffer.len();
            input.consume(len);
        }
    }
}

#[inline]
pub(crate) fn read_bin_to_obj<R: std::io::BufRead, T, const BYTE_COUNT: usize, F: Fn([u8; BYTE_COUNT]) -> T>(
    input: &mut R, what_requires: &str, cursor: &mut exec::CursorPos,
    from_le_bytes_fn: F
) -> Result<T, exec::ExecError> {
    let mut buf = [0u8; BYTE_COUNT];
    match cursor.read(input, &mut buf) {
        Ok(size) => {
            if size < BYTE_COUNT { Err(exec::ExecError::new(crate::exceptions::Error::EOFError(format!("{what_requires} requires 4 bytes, got {size}")), *cursor)) }
            else { cursor.pos += buf.len(); Ok(from_le_bytes_fn(buf)) }
        }
        Err(err) => Err(err)
    }
}
#[inline]
pub(crate) fn read_bin_to_u8<R: std::io::BufRead>(
    input: &mut R, what_requires: &str, cursor: &mut exec::CursorPos
) -> Result<u8, exec::ExecError> {
    read_bin_to_obj(input, what_requires, cursor, u8::from_le_bytes)
}
#[inline]
pub(crate) fn read_bin_to_u16<R: std::io::BufRead>(
    input: &mut R, what_requires: &str, cursor: &mut exec::CursorPos
) -> Result<u16, exec::ExecError> {
    read_bin_to_obj(input, what_requires, cursor, u16::from_le_bytes)
}
#[inline]
pub(crate) fn read_bin_to_u32<R: std::io::BufRead>(
    input: &mut R, what_requires: &str, cursor: &mut exec::CursorPos
) -> Result<u32, exec::ExecError> {
    read_bin_to_obj(input, what_requires, cursor, u32::from_le_bytes)
}
#[inline]
pub(crate) fn read_bin_to_u64<R: std::io::BufRead>(
    input: &mut R, what_requires: &str, cursor: &mut exec::CursorPos
) -> Result<u64, exec::ExecError> {
    read_bin_to_obj(input, what_requires, cursor, u64::from_le_bytes)
}

pub(crate) fn skip_whitespace_and_comments<R: std::io::BufRead>(input: &mut R, cursor: &mut asm::CursorPos) -> Result<(), asm::AsmError> {
    loop {
        let buf = asm::ok_or_err(input.fill_buf(), cursor)?;
        if buf.is_empty() {
            return Ok(());
        }

        let ch = buf[0];
        if ch.is_ascii_whitespace() {
            input.consume(1);
            cursor.push_u8(ch);
            continue;
        } else if ch == b';' {
            input.consume(1);
            cursor.push_u8(ch);
            // 读取直到换行（或 EOF），并丢弃内容
            let mut tmp = Vec::new();
            let n = cursor.read_until(input, b'\n', &mut tmp)?;
            if n == 0 { // EOF
                return Ok(());
            }
            continue;
        }
        // 遇到有效字符，停止跳过
        break;
    }
    Ok(())
}
