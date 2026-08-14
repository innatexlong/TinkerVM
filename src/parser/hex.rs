use crate::parser::utils;

// 内部已做字节序转换和空白字符跳过，外部调用者无须处理
pub(crate) fn hex_to_u32<R: std::io::Read>(input: &mut R) -> std::io::Result<u32> {
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

pub(crate) fn hex_char_to_byte(c: u8) -> Result<u8, Box<dyn std::error::Error>> {
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


