// parser/utils.rs

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
pub(crate) fn read_bin_to_u32<R: std::io::BufRead>(input: &mut R) -> Result<u32, crate::exceptions::Error> {
    let mut buf = [0u8; 4];
    match input.read(&mut buf) {
        Ok(size) => {
            if size < 4 { Err(crate::exceptions::Error::EOFError(format!("dest arg of ADD requires 4 bytes, got {size}"))) }
            else { Ok(u32::from_le_bytes(buf)) }
        }
        Err(err) => Err(crate::exceptions::Error::from(err))
    }
}

pub(crate) fn skip_whitespace_and_comments<R: std::io::BufRead>(input: &mut R) -> Result<(), crate::exceptions::Error> {
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
            let n = input.read_until(b'\n', &mut tmp).map_err(|e| crate::exceptions::Error::IOError(e.to_string()))?;
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
