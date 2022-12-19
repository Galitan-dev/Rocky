#[macro_use]
extern crate nom;
#[macro_use]
extern crate clap;

use std::{path::Path, fs::File, io::Read};
use clap::App;
use repl::{REPL, REPLMode};

pub mod vm;
pub mod instruction;
pub mod repl;
pub mod assembler;


fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let target_file = matches.value_of("INPUT_FILE");
    match target_file {
        Some(filename) => {
            let program = read_file(filename);
            let mut asm = assembler::Assembler::new();
            let mut vm = vm::VM::new();
            let program = asm.assemble(&program);
            vm.ro_data = asm.ro;
            match program {
                Ok(p) => {
                    vm.add_bytes(p);
                    vm.run();
                    std::process::exit(0);
                },
                Err(errors) => {
                    println!("{errors:?}");
                }
            }
        },
        None => {
            start_repl();
        }
    }
}

fn start_repl() {
    let mut repl = REPL::new(REPLMode::Assembly);
    repl.run();
}

fn read_file(tmp: &str) -> String {
    let filename = Path::new(tmp);
    match File::open(Path::new(&filename)) {
      Ok(mut fh) => {
        let mut contents = String::new();
        match fh.read_to_string(&mut contents) {
          Ok(_) => {
            return contents;
          },
          Err(e) => {
            println!("There was an error reading file: {:?}", e);
            std::process::exit(1);
          }
        }
      },
      Err(e) => {
        println!("File not found: {:?}", e);
        std::process::exit(1)
      }
    }
}