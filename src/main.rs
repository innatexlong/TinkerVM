use crate::exceptions::Error;

pub mod parser;
pub mod exceptions;
pub mod env;

fn repl() -> () {}

// fn main() -> Result<(), Box<dyn std::error::Error>> {
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = match std::fs::File::open("123.txt") {
        Ok(f) => f,
        Err(e) => {
            eprintln!("打开文件失败: {}", e);
            return Ok(());
        }
    };

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        repl()
    }
    let filename = &args[1];
    let file = match std::fs::File::open(filename) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::InvalidFilename => {
            println!("Filename `{}` is invalid", filename);
            std::process::exit(1);
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("Cannot find `{}`", filename);
            std::process::exit(1);
        },
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            println!("Permission denied when opening `{}`", filename);
            std::process::exit(1);
        },
        Err(e) => {
            println!("Cannot open the file `{}`: {}", filename, e.to_string());
            std::process::exit(1);
        }
    };
    let mut reader = std::io::BufReader::new(file);
    let mut compiled_bin: Vec<u8> = Vec::new();
    {
        let mut writer = std::io::BufWriter::new(&mut compiled_bin);
        parser::compile_assembly(&mut reader, &mut writer)?;
    }
    Ok(parser::run(&mut std::io::Cursor::new(&mut compiled_bin))?)
    //
    // let global_env = env::Env::new();
    //
    // // 收集输入，直到遇到含 '>' 的行，截取该行 '>' 之前的部分
    // let mut str_buf = String::with_capacity(1024);
    // let stdin = io::stdin();
    // let mut lines = stdin.lock().lines();
    //
    // while let Some(line) = lines.next() {
    //     let line = line?; // 将 io::Result<String> 转为 String
    //     if let Some(pos) = line.find('>') {
    //         str_buf.push_str(&line[..pos]);
    //         break;
    //     }
    //     str_buf.push_str(&line);
    //     str_buf.push('\n');
    // }
    //
    // // 用 Cursor 将收集的字符串转为可读流
    // let mut input_cursor = io::Cursor::new(str_buf.into_bytes());
    // // 输出缓冲区
    // let mut output_cursor = io::Cursor::new(Vec::<u8>::new());
    //
    // // 调用编译函数
    // parser::compile_assembly(&mut input_cursor, &mut output_cursor)?;
    //
    // // 取编译结果，作为字节流再次交给 main 执行
    // let output_bytes = output_cursor.into_inner();
    // let mut bytes_cursor = io::Cursor::new(output_bytes);
    //
    // // 执行（泛参 true 表示某种编译期开关）
    // let ret_code = parser::main::<true>(&global_env, &mut bytes_cursor)?;
    //
    // println!("Ret code: {}", ret_code);
    // Ok(())
}