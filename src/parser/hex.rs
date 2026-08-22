// parser/hex.rs

use std::fmt::UpperHex;
use crate::parser::utils;
use crate::parser::asm;

/// 内部已做字节序转换和空白字符跳过，外部调用者无须处理
pub(crate) fn hex_to_uint<R: std::io::BufRead, T>(
    input: &mut R, max_digit_count: usize, cursor_pos: &mut asm::CursorPos
) -> Result<T, asm::AsmError>
    where T: Sized + std::ops::BitOr<T, Output=T> + std::ops::Shl<T, Output=T> + From<u8> + UpperHex,
{
    let mut res: T = T::from(0u8);
    let mut count = 0;
    let mut buf = [0u8; 1];

    utils::skip_whitespace_and_comments(input, cursor_pos)?;

    loop {
        match cursor_pos.read_exact(input, &mut buf) {
            Ok(()) => {
                let cur = buf[0] as char;
                if cur.is_ascii_whitespace() {
                    break;
                }
                if cur == '\'' {
                    continue;
                }
                // 将十六进制字符转换为数值
                let digit = u8::try_from(
                    cur.to_digit(16).ok_or_else(
                        || {
                            asm::AsmError::new(
                                crate::exceptions::Error::SyntaxError(format!("Invalid hex digit '{}'", cur)),
                                *cursor_pos,
                            )
                        }
                    )?
                ).map_err(|_| asm::AsmError::new(crate::exceptions::Error::SyntaxError(format!("Invalid hex char {cur}")), *cursor_pos))?;
                if count >= max_digit_count {
                    return Err(asm::AsmError::new(
                        crate::exceptions::Error::from(
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidData, format!(
                                    "{res:0X}'{cur} overflow, maximum digit count {max_digit_count}"
                                )
                            )
                        ), *cursor_pos)
                    )
                }
                res = (res << T::from(4u8)) | (T::from(digit));
                count += 1;
            }
            Err(e) => {
                match e.error {
                    crate::exceptions::Error::EOFError(_) => break,
                    _ => return Err(e)
                }
            }
        }
    }

    // 根据目标平台大小端调整（Rust 中通常按小端序处理）
    // 这里我们返回小端序存储的整数，如果当前平台是大端则交换字节
    #[cfg(target_endian = "big")]
    {
        res = res.swap_bytes();
    }
    Ok(res)
}

#[inline]
pub(crate) fn hex_to_u8<R: std::io::BufRead>(input: &mut R, cursor_pos: &mut asm::CursorPos) -> Result<u8, asm::AsmError> {
    hex_to_uint(input, size_of::<u8>() * 2, cursor_pos)
}
#[inline]
pub(crate) fn hex_to_u16<R: std::io::BufRead>(input: &mut R, cursor_pos: &mut asm::CursorPos) -> Result<u16, asm::AsmError> {
    hex_to_uint(input, size_of::<u16>() * 2, cursor_pos)
}
#[inline]
pub(crate) fn hex_to_u32<R: std::io::BufRead>(input: &mut R, cursor_pos: &mut asm::CursorPos) -> Result<u32, asm::AsmError> {
    hex_to_uint(input, size_of::<u32>() * 2, cursor_pos)
}
#[inline]
pub(crate) fn hex_to_u64<R: std::io::BufRead>(input: &mut R, cursor_pos: &mut asm::CursorPos) -> Result<u64, asm::AsmError> {
    hex_to_uint(input, size_of::<u64>() * 2, cursor_pos)
}

#[inline]
pub(crate) fn hex_char_to_byte(c: u8) -> Result<u8, Box<dyn std::error::Error>> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(format!("Invalid hex character: '{}'", c as char).into()),
    }
}

#[deprecated]
pub fn compile_hex<R: std::io::BufRead, W: std::io::Write>(
    input: &mut R,
    output: &mut W,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = [0u8; 1];  // 重用缓冲区
    loop {
        // 1. 跳过空白，读取第一个十六进制数字
        let first = match utils::read_non_whitespace_byte(input)? {
            Some(b) => b,
            None => return Ok(()),
        };

        // 2. 读取第二个字节（不允许空白）
        match input.read(&mut buf)? {
            0 => return Err("Unexpected EOF: missing second hex digit".into()),
            1 => {
                let second = buf[0];
                if second.is_ascii_whitespace() {
                    return Err("Unexpected whitespace in hex pair".into());
                }
                let high = hex_char_to_byte(first)?;
                let low = hex_char_to_byte(second)?;
                output.write_all(&[(high << 4) | low])?;
            }
            _ => unreachable!(),
        }
    }
}


