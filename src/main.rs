pub mod exceptions;
pub mod memory;
pub mod value;
pub mod env;
pub mod parser;

pub fn repl() -> () { todo!() }  // TODO

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let filename: String;
    let filename = match args.get(1) {
        Some(file) => file.clone(),
        None => {
            println!("No files specified, using 'example.tas'");
            "example.tas".to_string()
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