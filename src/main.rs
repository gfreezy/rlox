use std;
use std::io;
use std::io::{stdout, Read, Write};

use crate::vm::VM;
use std::fs::File;
use std::process::exit;

mod chunk;
mod compiler;
mod debug;
mod error;
mod scanner;
mod token_type;
mod value;
mod vm;

use error::{Error, Result};

fn main() -> Result<()> {
    let args = std::env::args().into_iter().collect::<Vec<String>>();
    if args.len() == 1 {
        repl();
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        eprintln!("Usage: rlox [path]");
        std::process::exit(64);
    }
    Ok(())
}

fn repl() {
    let mut vm = VM::new();
    let mut line = String::new();
    loop {
        print!("> ");
        stdout().flush().expect("flush");

        line.clear();
        let ret = match io::stdin().read_line(&mut line).expect("read line") {
            0 => break,
            _ => vm.interpret_source(&line),
        };

        match ret {
            Ok(_) => (),
            Err(e) => eprintln!("{}", e),
        }
    }
}

fn run_file(path: &str) {
    let mut file = File::open(path).expect("open file");
    let mut source_bytes = Vec::new();
    let _size = file.read_to_end(&mut source_bytes).expect("read file");
    let source = String::from_utf8(source_bytes).expect("no valid utf-8");
    let mut vm = VM::new();
    let ret = vm.interpret_source(&source);

    match ret {
        Err(Error::CompileError { .. }) => exit(65),
        Err(Error::RuntimeError { .. }) => exit(70),
        Ok(_) => (),
        Err(_) => unreachable!(),
    }
}
