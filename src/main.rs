#[macro_use]
extern crate nom;
extern crate clap;
extern crate uuid;
extern crate chrono;
extern crate rustyline;
extern crate rustyline_derive;
extern crate colored;

use std::{path::Path, fs::File, io::Read};
use clap::{App, load_yaml};
use repl::{REPL, REPLMode};
use rustyline::error::ReadlineError;

pub mod vm;
pub mod instruction;
pub mod repl;
pub mod assembler;
pub mod scheduler;

fn main() -> Result<(), ReadlineError> {
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
					let events = vm.run();
					println!("VM Events");
					println!("--------------------------");
					for event in &events {
						println!("{:#?}", event);
					};
					std::process::exit(0);
				},
				Err(errors) => {
					println!("Encountered {} assembler error(s):", errors.len());
					for error in errors {
						println!("{error}");
					}
				}
			}
		},
		None => {
			start_repl()?;
		}
	};

	Ok(())
}

fn start_repl() -> Result<(), ReadlineError> {
	let mut repl = REPL::new(REPLMode::Assembly)?;
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