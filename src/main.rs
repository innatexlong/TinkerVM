pub mod exceptions;
pub mod memory;
pub mod value;
pub mod env;
pub mod parser;

pub fn repl() -> () {
    use std::io::Read;
    let memory_pool = memory::MemoryPool::new(0, 1024);
    let env = env::Env::new(std::sync::Arc::new(std::sync::RwLock::new(memory_pool)), None);
    let env_arc = std::sync::Arc::new(std::sync::RwLock::new(env));
    println!("TinkerVM REPL");
    println!("After entering the complete codes, please press Ctrl+Z (Ctrl+D on Unix/Linux)");
    println!("-----------------------------------------------------------------------------");
    loop {
        use std::io::Write;
        print!(">>> ");
        std::io::stdout().flush().unwrap();
        let mut stdin = std::io::stdin();
        let mut env_arc_guard = env_arc.write().unwrap();
        let mut cursor = parser::asm::CursorPos::new();
        let mut input_vec = Vec::new();
        if stdin.read_to_end(&mut input_vec).unwrap() == 0 {
            break;
        }
        let mut input_reader = std::io::BufReader::new(std::io::Cursor::new(input_vec));

        if let Err(err) = parser::asm::compile_assembly(&mut input_reader, &mut env_arc_guard, &mut cursor) {
            println!("{:?}", err);
            println!("---------------");
        } else {
            drop(env_arc_guard);

            println!("------RUN------");

            match parser::exec::run(env_arc.clone()) {
                Err(err) => println!("{:?}", err),
                Ok(res) => println!("{res}"),
            }
            println!("------END------");
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let filename: String;
    let filename = match args.get(1) {
        Some(file) => file.clone(),
        None => {
            repl();
            return Ok(())
            // println!("No files specified, using 'example.tas'");
            // "example.tas".to_string()
        }
    };
    let file = match std::fs::File::open(&filename) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::InvalidFilename => {
            println!("Filename `{}` is invalid", &filename);
            std::process::exit(1);
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("Cannot find `{}`", &filename);
            std::process::exit(1);
        },
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            println!("Permission denied when opening `{}`", &filename);
            std::process::exit(1);
        },
        Err(e) => {
            println!("Cannot open the file `{}`: {}", &filename, e.to_string());
            std::process::exit(1);
        }
    };
    let mut reader = std::io::BufReader::new(file);
    let memory_pool = memory::MemoryPool::new(0, 1024);
    let env = env::Env::new(
        std::sync::Arc::new(std::sync::RwLock::new(memory_pool)),
        None
    );
    let mut cursor_pos = parser::asm::CursorPos::new();
    parser::asm::compile_assembly(&mut reader, &env, &mut cursor_pos)?;
    for entry in &env.funcs {
        let (key, value) = entry.pair();
        println!("Func {key}: {:?}", value);
    }

    let env_arc = std::sync::Arc::new(std::sync::RwLock::new(env));
    let result = parser::exec::run(env_arc);

    match result {
        Ok(return_val) => println!("Return code: {}", return_val),
        Err(e) => eprintln!("[debug.error] {}", e)
    }
    Ok(())
}