use std::{io::{Write, self, Read}, num::ParseIntError, path::Path, fs::File};
use nom::types::CompleteStr;
use crate::{vm::VM, assembler::{program_parser::program, symbols::SymbolTable, Assembler}, scheduler::Scheduler};

use self::command_parser::CommandParser;

pub mod command_parser;

pub enum REPLMode {
    Hexadecimal,
    Assembly
}

pub struct REPL {
    command_buffer: Vec<String>,
    mode: REPLMode,
    vm: VM,
    asm: Assembler,
    scheduler: Scheduler,
}

impl REPL {
    pub fn new(mode: REPLMode) -> REPL {
        REPL {
            command_buffer: Vec::new(),
            mode,
            vm: VM::new(),
            asm: Assembler::new(),
            scheduler: Scheduler::new(),
        }
    }

    pub fn run(&mut self) {
        println!("Welcome to Rocky! Let's be nerds!");
        loop {
            let mut buffer = String::new();
    
            let stdin = io::stdin();
    
            print!(">>> ");
            io::stdout().flush().expect("Unable to flush stdout");
    
            stdin
            .read_line(&mut buffer)
            .expect("Unable to read line from user");

            let historical_copy = buffer.clone();
            self.command_buffer.push(historical_copy);

            if buffer.starts_with("!") {
                self.execute_command(&buffer);
            } else {
                match self.mode {
                    REPLMode::Hexadecimal => {
                        let results = Self::parse_hex(&buffer);
                        match results {
                            Ok(bytes) => {
                                for byte in bytes {
                                    self.vm.add_byte(byte)
                                }
                            },
                            Err(_e) => {
                                println!("Unable to decode hex string. Please enter 4 groups of 2 hex characters.")
                            }
                        };
                    },
                    REPLMode::Assembly => {
                        let parsed_program = program(CompleteStr(&buffer));
                        if !parsed_program.is_ok() {
                            println!("Unable to parse input");
                            continue;
                        }
                        let (_, result) = parsed_program.unwrap();
                        let bytecode = result.to_bytes(&SymbolTable::new());
                        for byte in bytecode {
                            self.vm.add_byte(byte);
                        }
                    }
                };
                self.vm.run_once();
            }
        }
    }

    fn execute_command(&mut self, input: &str) {
        let args = CommandParser::tokenize(input);
        match args[0] {
            "!quit" => self.quit(&args[1..]),
            "!history" => self.history(&args[1..]),
            "!program" => self.program(&args[1..]),
            "!clear_program" => self.clear_program(&args[1..]),
            "!clear_registers" => self.clear_registers(&args[1..]),
            "!registers" => self.registers(&args[1..]),
            "!symbols" => self.symbols(&args[1..]),
            "!load_file" => self.load_file(&args[1..]),
            "!spawn" => self.spawn(&args[1..]),
            _ => { println!("Invalid command") }
        };
    }

    fn quit(&self, _args: &[&str]) {
        println!("Bye! Have a good rest!");
        std::process::exit(0);
    }

    fn history(&self, _args: &[&str]) {
        for command in &self.command_buffer {
            print!("{command}");
        }
    }

    fn program(&self, _args: &[&str]) {
        println!("Listing instructions currently in VM's program vector:");
        for instruction in &self.vm.program {
            println!("{}", instruction);
        }
        println!("End of Program Listing");
    }

    fn registers(&self, _args: &[&str]) {
        println!("Listing registers and all contents:");
        println!("{:#?}", self.vm.registers);
        println!("End of Register Listing")
    }

    fn symbols(&self, _args: &[&str]) {
        println!("Listing symbols and all contents:");
        for symbol in &self.asm.symbols.symbols {
            println!("{symbol:#?}");
        }
        println!("End of Symbols Listing")
    }

    fn clear_program(&mut self, _args: &[&str]) {
        self.vm.program.clear();
        self.vm.pc = 0;
    }

    fn clear_registers(&mut self, _args: &[&str]) {
        self.vm.registers = [0; 32];
    }

    fn load_file(&mut self, args: &[&str]) {
        if args.len() == 0 {
            println!("Usage: !load_file path/to/file.rk");
            return;
        }

        let filename = Path::new(args[0]);
        let contents = self.get_data_from_load(filename);
        if let Some(contents) = contents {
            match self.asm.assemble(&contents) {
                Ok(mut assembled_program) => {
                    println!("Sending assembled program to VM");
                    self.vm.program.append(&mut assembled_program);
                    self.vm.ro_data.append(&mut self.asm.ro);
                    self.vm.run();
                }
                Err(errors) => {
                    for error in errors {
                        println!("Unable to parse input: {error}");
                    }
                }
            }
        }
    }

    fn spawn(&mut self, args: &[&str]) {
        if args.len() == 0 {
            println!("Usage: !spawn path/to/file.rk");
            return;
        }

        let filename = Path::new(args[0]);
        let contents = self.get_data_from_load(filename);
        if let Some(contents) = contents {
            match self.asm.assemble(&contents) {
                Ok(mut assembled_program) => {
                    println!("Sending assembled program to VM");
                    self.vm.ro_data.append(&mut self.asm.ro);
                    self.vm.program.append(&mut assembled_program);
                    // println!("{:#?}", self.vm.program);
                    self.scheduler.get_thread(self.vm.clone());
                },
                Err(errors) => {
                    for error in errors {
                        println!("Unable to parse input: {}", error);
                    }
                }
            }
        }
    }

    fn parse_hex(i: &str) -> Result<Vec<u8>, ParseIntError>{
        let split = i.split(" ").collect::<Vec<&str>>();
        let mut results: Vec<u8> = vec![];
        for hex_string in split {
            let byte = u8::from_str_radix(&hex_string, 16);
            match byte {
                Ok(result) => {
                    results.push(result);
                },
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(results)
    }

    fn get_data_from_load(&mut self, filename: &Path) -> Option<String> {
        let mut f = match File::open(&filename) {
            Ok(f) => { f }
            Err(e) => {
                println!("There was an error opening that file: {:?}", e);
                return None;
            }
        };
        let mut contents = String::new();
        match f.read_to_string(&mut contents) {
            Ok(_bytes_read) => {
                Some(contents)
            },
            Err(e) => {
                println!("there was an error reading that file: {:?}", e);
                None
            }
        }
    }
}
