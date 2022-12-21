use std::{fs::File, io::Read, path::Path};

use assembler::Assembler;
use repl::{REPLMode, REPL};
use rustyline::error::ReadlineError;
use vm::VM;

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
pub mod vm;

pub fn start_repl(mode: REPLMode) -> Result<(), ReadlineError> {
    let mut repl = REPL::new(mode)?;
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

pub fn run_file(num_threads: usize, filename: &str) {
    let program = read_file(filename);
    let mut asm = Assembler::new();
    let mut vm = VM::new();
    vm.logical_cores = num_threads;
    let program = asm.assemble(&program);
    match program {
        Ok(p) => {
            vm.add_bytes(p);
            let events = vm.run();
            println!("VM Events");
            println!("--------------------------");
            for event in &events {
                println!("{:#?}", event);
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
