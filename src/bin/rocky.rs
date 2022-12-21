extern crate rocky;

use clap::{parser::RawValues, ArgMatches};
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
    let args = get_arguments(&matches);

    match args {
        Args::RunFile((num_threads, filename)) => run_file(num_threads, filename),
        Args::Repl(mode) => start_repl(mode)?,
        Args::AddSshKey(pub_key_file) => println!("User tried to add SSH key at {pub_key_file}!"),
    }

    Ok(())
}

fn start_repl(mode: REPLMode) -> Result<(), ReadlineError> {
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

fn run_file(num_threads: usize, filename: &str) {
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

fn get_arguments<'a>(matches: &'a ArgMatches) -> Args<'a> {
    let (command, args) = matches.subcommand().map_or(("rocky", matches), |s| s);
    match command {
        "rocky" => match unwrap(args.get_raw("input_file")) {
            Some(input_file) => Args::RunFile((
                match unwrap(args.get_raw("threads")) {
                    Some(number) => match number.parse::<usize>() {
                        Ok(v) => v,
                        Err(_e) => {
                            println!(
                                "Invalid argument for number of threads: {}. Using default.",
                                number
                            );
                            num_cpus::get()
                        }
                    },
                    None => num_cpus::get(),
                },
                input_file,
            )),
            None => Args::Repl({
                if args.get_flag("hexadecimal") {
                    REPLMode::Hexadecimal
                } else {
                    REPLMode::Assembly
                }
            }),
        },
        "add-ssh-key" => Args::AddSshKey(unwrap(args.get_raw("pub_key_file")).unwrap()),
        _ => panic!("Invalid Command \"{command}\""),
    }
}

fn unwrap(raw: Option<RawValues>) -> Option<&str> {
    raw.map(|mut v| v.next().unwrap().to_str().unwrap())
}

#[derive(Debug, Clone)]
enum Args<'a> {
    Repl(REPLMode),
    RunFile((usize, &'a str)),
    AddSshKey(&'a str),
}
