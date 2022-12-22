use std::{fs::File, io::Read, path::Path};

use assembler::Assembler;
use cli::{REPLArgs, RunFileArgs};
use repl::REPL;
use rustyline::error::ReadlineError;
use vm::VM;

extern crate anyhow;
extern crate chrono;
extern crate clap;
extern crate colored;
extern crate futures;
extern crate nom;
extern crate num_cpus;
extern crate rustyline;
extern crate rustyline_derive;
extern crate thrussh;
extern crate thrussh_keys;
extern crate tokio;
extern crate uuid;

pub mod assembler;
pub mod cli;
pub mod instruction;
pub mod repl;
pub mod scheduler;
pub mod ssh;
pub mod vm;

pub fn start_repl(args: REPLArgs) -> Result<(), ReadlineError> {
    let mut repl = REPL::new(args.mode)?;
    repl.run();
    Ok(())
}

fn read_file(tmp: &str) -> String {
    let filename = Path::new(tmp);
    match File::open(Path::new(&filename)) {
        Ok(mut fh) => {
            let mut contents = String::new();
            match fh.read_to_string(&mut contents) {
                Ok(_) => {
                    return contents;
                }
                Err(e) => {
                    println!("There was an error reading file: {:?}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            println!("File not found: {:?}", e);
            std::process::exit(1)
        }
    }
}

pub fn run_file(args: RunFileArgs) {
    let program = read_file(args.filename);
    let mut asm = Assembler::new();
    let mut vm = VM::new();
    vm.logical_cores = args.num_threads;
    let program = asm.assemble(&program);
    match program {
        Ok(p) => {
            vm.add_bytes(p);
            let events = vm.run();
            println!("--------------------------");
            println!("VM Events");
            println!("--------------------------");
            for event in &events {
                println!("{:#?}", event);
            }
            println!("--------------------------");
            println!("Non-null Registers");
            println!("--------------------------");
            for (register, value) in vm.registers.iter().enumerate() {
                if *value != 0 {
                    println!("${register} = {value}");
                }
            }
            println!("--------------------------");
            println!("Memory Heap as UTF-8 Strings");
            println!("--------------------------");
            for bytes in vm.memory_heap.into_iter() {
                println!("{}", std::str::from_utf8(&bytes).unwrap());
            }

            std::process::exit(0);
        }
        Err(errors) => {
            println!("Encountered {} assembler error(s):", errors.len());
            for error in errors {
                println!("{error}");
            }
        }
    }
}
