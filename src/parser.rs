use std::error;
use crate::exceptions;

pub fn func() -> Result<crate::env::TempVar, Box<dyn error::Error>> {

}

pub fn run<R: std::io::Read>(input: &mut R) -> Result<(), Box<dyn error::Error>> {

}

fn read_non_whitespace_byte<R: std::io::BufRead>(input: &mut R) -> Result<Option<u8>> {
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

fn hex_char_to_byte(c: u8) -> Result<u8, Box<dyn error::Error>> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(format!("Invalid hex character: '{}'", c as char).into()),
    }
}

pub fn compile_hex<R: std::io::BufRead, W: std::io::Write>(
    input: &mut R,
    output: &mut W,
) -> Result<(), Box<dyn error::Error>> {
    let mut buf = [0u8; 1];  // 重用缓冲区
    loop {
        // 1. 跳过空白，读取第一个十六进制数字
        let first = match read_non_whitespace_byte(input)? {
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

// 内部已做字节序转换和空白字符跳过，外部调用者无须处理
pub fn hex_to_u32<R: std::io::Read>(input: &mut R) -> std::io::Result<u32> {
    let mut res = 0u32;
    let mut count = 0;
    let mut buf = [0u8; 1];

    loop {
        match input.read_exact(&mut buf) {
            Ok(()) => {
                let cur = buf[0] as char;
                if cur.is_ascii_whitespace() {
                    break;
                }
                if cur == '\'' {
                    continue;
                }
                // 将十六进制字符转换为数值
                let digit = cur.to_digit(16).ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid hex digit")
                })?;
                if count >= 8 {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "overflow"));
                }
                res = (res << 4) | digit;
                count += 1;
            }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break, // 遇到 EOF 也停止
            Err(e) => return Err(e),
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

fn skip_whitespace_and_comments<R: std::io::BufRead>(input: &mut R) -> Result<(), exceptions::Error> {
    loop {
        let buf = input.fill_buf()?;
        if buf.is_empty() {
            return Ok(());
        }

        let ch = buf[0];
        if ch.is_ascii_whitespace() {
            input.consume(1);
            continue;
        }
        if ch == b';' {
            input.consume(1);
            // 读取直到换行（或 EOF），并丢弃内容
            let mut tmp = Vec::new();
            let n = input.read_until(b'\n', &mut tmp).map_err(|e| exceptions::Error::IOError(e.to_string()))?;
            if n == 0 { // EOF
                return Ok(());
            }
            // 如果最后读到的是 \n，它已被消费；若有 \r，留在缓冲中，会在下次空白循环中跳过
            continue;
        }
        // 遇到有效字符，停止跳过
        break;
    }
    Ok(())
}

// 内部已处理空格，外部无须处理
fn read_identifier<R: std::io::BufRead>(input: &mut R) -> Result<Option<Vec<u8>>, exceptions::Error> {
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
    Ok(Some(id))
}

fn compile_assembly_cmd<R: std::io::BufRead, W: std::io::Write>(input: &mut R, output: &mut W)
    -> Result<(), exceptions::Error> {
    match read_identifier(input)? {
        None => Err(exceptions::Error::EOFError("Unexpected EOF when reading identifier".into())),
        Some(id) => {
            if id == b"add " {
                let dest = hex_to_u32(input)?.to_le_bytes().to_vec();
                let src1 = hex_to_u32(input)?.to_le_bytes().to_vec();
                let src2 = hex_to_u32(input)?.to_le_bytes().to_vec();
                output.write(dest.as_slice())?;
                output.write(src1.as_slice())?;
                output.write(src2.as_slice())?;
                Ok(())
            } else if id == b"movc " {
                let dest = hex_to_u32(input)?.to_le_bytes().to_vec();
                let src_val = hex_to_u32(input)?.to_le_bytes().to_vec();
                output.write(dest.as_slice())?;
                output.write(src_val.as_slice())?;
                Ok(())
            } else if id == b"mov " {
                let dest = hex_to_u32(input)?.to_le_bytes().to_vec();
                let src_var = hex_to_u32(input)?.to_le_bytes().to_vec();
                output.write(dest.as_slice())?;
                output.write(src_var.as_slice())?;
                Ok(())
            } else if id == b"retc " {
                let ret_val = hex_to_u32(input)?.to_le_bytes().to_vec();
                output.write(ret_val.as_slice())?;
                Ok(())
            } else if id == b"retv " {
                let ret_var = hex_to_u32(input)?.to_le_bytes().to_vec();
                output.write(ret_var.as_slice())?;
                Ok(())
            }/* else if id == b"new " {

            }*/ else {
                Err(exceptions::Error::InvalidOperation(
                    id.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ")
                ))
            }
        }
    }
}

pub fn compile_assembly<R: std::io::BufRead, W: std::io::Write>(input: &mut R, output: &mut W) -> Result<(), exceptions::Error> {
    loop {
        match skip_whitespace_and_comments(input) {
            Err(e) => return Err(e),
            Ok(()) => {}
        }
        match compile_assembly_cmd(input, output) {
            Err(exceptions::Error::InvalidOperation(str))
                => return Err(exceptions::Error::InvalidOperation(str)),
            Err(exceptions::Error::EOFError(e))
                => return Err(exceptions::Error::IOError(e).into()),
            Err(exceptions::Error::IOError(str))
                => return Err(exceptions::Error::IOError(format!("{}", str))),
            Err(exceptions::Error::UnrecognizedError(str, code)) => return Err(exceptions::Error::UnrecognizedError(str, code).into()),
            Err(e) => return Err(
                exceptions::Error::UnrecognizedError(
                    e.to_string(), e.code()
                )
            ),
            Ok(()) => {}
        };
    }
}

// use std::io::{Read, Bytes};
// use std::iter::Peekable;
// use thiserror::Error;

// // ---------- Lexer 结构体 ----------
// pub struct Lexer<R: Read> {
//     iter: Peekable<Bytes<R>>,   // 包装字节迭代器，支持预览
// }
//
// impl<R: Read> Lexer<R> {
//     /// 从任意实现了 Read 的类型创建 Lexer（自动取得所有权，如需借用可传入 &mut R）
//     pub fn new(input: R) -> Self {
//         Self {
//             iter: input.bytes().peekable(),
//         }
//     }
//
//     /// 跳过所有空白字符（空格、换行、制表符等）以及注释（以 ';' 开头到行末）
//     pub fn skip_whitespace_and_comments(&mut self) -> Result<(), exceptions::Error> {
//         while let Some(Ok(b)) = self.iter.peek() {
//             if b.is_ascii_whitespace() {
//                 self.iter.next(); // 消费空白
//                 continue;
//             }
//             if *b == b';' {
//                 self.iter.next(); // 消费 ';'
//                 // 跳过直到换行或 EOF
//                 while let Some(Ok(b)) = self.iter.peek() {
//                     if *b == b'\n' {
//                         self.iter.next(); // 消费换行，跳出内层循环
//                         break;
//                     }
//                     self.iter.next();
//                 }
//                 continue;
//             }
//             break; // 遇到非空白、非注释字符，停止跳过
//         }
//         // 遇到 I/O 错误时，将其转换为 InvalidOperation（或您可以选择其他变体）
//         self.iter
//             .peek()
//             .map_or(Ok(()), |r| r.as_ref().map_err(|_| exceptions::Error::InvalidOperation))?;
//         Ok(())
//     }
//
//     /// 读取一个标识符（连续字母，如 "add", "movc"），**调用前应确保已跳过空白和注释**
//     pub fn read_identifier(&mut self) -> Result<String, exceptions::Error> {
//         // 先跳过空白（以防未调 skip）
//         self.skip_whitespace_and_comments()?;
//
//         // 检查是否有字母开头
//         let first = match self.iter.peek() {
//             Some(Ok(b)) if b.is_ascii_alphabetic() => *b,
//             Some(Ok(_)) => return Err(exceptions::Error::InvalidOperation), // 预期字母但遇到其他字符
//             Some(Err(e)) => return Err(exceptions::Error::InvalidOperation), // IO错误
//             None => return Err(exceptions::Error::InvalidOperation),         // EOF
//         };
//         self.iter.next(); // 消费第一个字母
//
//         let mut ident = String::from(first as char);
//         while let Some(Ok(b)) = self.iter.peek() {
//             if b.is_ascii_alphabetic() {
//                 ident.push(*b as char);
//                 self.iter.next();
//             } else {
//                 break;
//             }
//         }
//         Ok(ident)
//     }
//
//     /// 读取一个十六进制数字（格式：一个或多个 0-9A-Fa-f，以空白或分隔），返回 u32
//     pub fn read_hex_number(&mut self) -> Result<u32, exceptions::Error> {
//         self.skip_whitespace_and_comments()?;
//
//         let mut hex_str = String::new();
//         // 收集十六进制字符
//         while let Some(Ok(b)) = self.iter.peek() {
//             if b.is_ascii_hexdigit() {
//                 hex_str.push(*b as char);
//                 self.iter.next();
//             } else {
//                 break;
//             }
//         }
//
//         if hex_str.is_empty() {
//             return Err(exceptions::Error::InvalidOperation); // 没有读取到数字
//         }
//
//         u32::from_str_radix(&hex_str, 16).map_err(|_| exceptions::Error::InvalidOperation)
//     }
//
//     /// 读取一个字符，并检查是否为空白（用于强制空格），若不符合则返回错误
//     pub fn expect_space(&mut self) -> Result<(), exceptions::Error> {
//         match self.iter.next() {
//             Some(Ok(b)) if b.is_ascii_whitespace() => Ok(()),
//             Some(Ok(_)) => Err(exceptions::Error::InvalidOperation), // 非空白
//             Some(Err(e)) => Err(exceptions::Error::InvalidOperation), // IO 错误
//             None => Err(exceptions::Error::InvalidOperation),         // EOF
//         }
//     }
// }
//
// // ---------- 可选：检查是否到达 EOF ----------
// impl<R: Read> Lexer<R> {
//     pub fn is_eof(&mut self) -> Result<bool, exceptions::Error> {
//         // 先跳过空白和注释，因为它们不影响 EOF 判断
//         self.skip_whitespace_and_comments()?;
//         Ok(self.iter.peek().is_none())
//     }
// }

