pub mod utils;
pub mod hex;
pub mod asm;
pub mod cmd;
pub mod exec;
pub mod exec_cmd;

// fn read_non_whitespace_byte<R: std::io::BufRead>(input: &mut R) -> std::io::Result<Option<u8>> {
//     loop {
//         let buffer = input.fill_buf()?;          // 获取当前缓冲区内容
//         if buffer.is_empty() {
//             return Ok(None);                     // EOF
//         }
//         if let Some(&b) = buffer.iter().find(|&&c| !c.is_ascii_whitespace()) {
//             // 找到第一个非空白字节，消费掉它之前的所有字节（含该字节）
//             let pos = buffer.iter().position(|&c| c == b).unwrap(); // 安全，因为已知存在
//             input.consume(pos + 1);
//             return Ok(Some(b));
//         } else {
//             // 整个缓冲区都是空白，消耗全部
//             let len = buffer.len();
//             input.consume(len);
//         }
//     }
// }

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
//     pub fn skip_whitespace_and_comments(&mut self) -> Result<(), crate::exceptions::Error> {
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
//             .map_or(Ok(()), |r| r.as_ref().map_err(|_| crate::exceptions::Error::InvalidOperation))?;
//         Ok(())
//     }
//
//     /// 读取一个标识符（连续字母，如 "add", "movc"），**调用前应确保已跳过空白和注释**
//     pub fn read_identifier(&mut self) -> Result<String, crate::exceptions::Error> {
//         // 先跳过空白（以防未调 skip）
//         self.skip_whitespace_and_comments()?;
//
//         // 检查是否有字母开头
//         let first = match self.iter.peek() {
//             Some(Ok(b)) if b.is_ascii_alphabetic() => *b,
//             Some(Ok(_)) => return Err(crate::exceptions::Error::InvalidOperation), // 预期字母但遇到其他字符
//             Some(Err(e)) => return Err(crate::exceptions::Error::InvalidOperation), // IO错误
//             None => return Err(crate::exceptions::Error::InvalidOperation),         // EOF
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
//     pub fn read_hex_number(&mut self) -> Result<u32, crate::exceptions::Error> {
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
//             return Err(crate::exceptions::Error::InvalidOperation); // 没有读取到数字
//         }
//
//         u32::from_str_radix(&hex_str, 16).map_err(|_| crate::exceptions::Error::InvalidOperation)
//     }
//
//     /// 读取一个字符，并检查是否为空白（用于强制空格），若不符合则返回错误
//     pub fn expect_space(&mut self) -> Result<(), crate::exceptions::Error> {
//         match self.iter.next() {
//             Some(Ok(b)) if b.is_ascii_whitespace() => Ok(()),
//             Some(Ok(_)) => Err(crate::exceptions::Error::InvalidOperation), // 非空白
//             Some(Err(e)) => Err(crate::exceptions::Error::InvalidOperation), // IO 错误
//             None => Err(crate::exceptions::Error::InvalidOperation),         // EOF
//         }
//     }
// }
//
// // ---------- 可选：检查是否到达 EOF ----------
// impl<R: Read> Lexer<R> {
//     pub fn is_eof(&mut self) -> Result<bool, crate::exceptions::Error> {
//         // 先跳过空白和注释，因为它们不影响 EOF 判断
//         self.skip_whitespace_and_comments()?;
//         Ok(self.iter.peek().is_none())
//     }
// }

