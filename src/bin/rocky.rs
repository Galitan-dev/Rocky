use rocky::{
    assembler::Assembler,
    cli::cli,
    repl::{REPLMode, REPL},
    vm::VM,
};
use rustyline::error::ReadlineError;
use std::{fs::File, io::Read, path::Path};

fn main() -> Result<(), ReadlineError> {
    let matches = cli().get_matches();

    let target_file = matches.get_raw("input_file");

    match target_file {
        Some(mut filenames) => {
            let filename = filenames.next().unwrap().to_str().unwrap();
            let program = read_file(filename);
            let mut asm = Assembler::new();
            let mut vm = VM::new();
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
