use std::{io::{Write, self, Read}, num::ParseIntError, path::Path, fs::File};
use nom::types::CompleteStr;

use crate::{vm::VM, assembler::{program::program, symbols::SymbolTable}};

pub enum REPLMode {
    Hexadecimal,
    Assembly
}

pub struct REPL {
    command_buffer: Vec<String>,
    vm: VM,
    mode: REPLMode
}

impl REPL {
    pub fn new(mode: REPLMode) -> REPL {
        REPL {
            vm: VM::new(),
            command_buffer: vec![],
            mode,
        }
    }

    pub fn run(&mut self) {
        println!("Welcome to Rocky! Let's be nerds!");
        loop {
            let mut buffer = String::new();
    
            let stdin = io::stdin();
    
            print!(">>> ");
            io::stdout().flush().expect("Unable to flush stdout");
    
            stdin.read_line(&mut buffer).expect("Unable to read line from user");
            let buffer = buffer.trim();

            self.command_buffer.push(buffer.to_string());

            match buffer {
                ".quit" => {
                    println!("Bye! Have a nice day!");
                    std::process::exit(0);
                },
                ".history" => {
                    for command in &self.command_buffer {
                        println!("{command}");
                    }
                },
                ".program" => {
                    println!("Listing instructions currently in VM's program vector:");
                    for instruction in &self.vm.program {
                        println!("{}", instruction);
                    }
                    println!("End of Program Listing");
                },
                ".registers" => {
                    println!("Listing registers and all contents:");
                    println!("{:#?}", self.vm.registers);
                    println!("End of Register Listing")
                },
                ".clear_program" => {
                    self.vm.program.clear();
                },
                ".load_file" => {
                    print!("Please enter the path to the file you wish to load: ");
                    io::stdout().flush().expect("Unable to flush stdout");
                    let mut tmp = String::new();
                    stdin.read_line(&mut tmp).expect("Unable to read line from user");
                    let tmp = tmp.trim();
                    let filename = Path::new(&tmp);
                    let mut f = match File::open(&filename) {
                        Ok(f) => { f }
                        Err(e) => {
                            println!("There was an error opening that file: {e}");
                            continue;
                        }
                    };
                    let mut contents = String::new();
                    f.read_to_string(&mut contents).expect("There was an error reading from the file");
                    let program = match program(CompleteStr(&contents)) {
                        // Rusts pattern matching is pretty powerful an can even be nested
                        Ok((_remainder, program)) => {
                            program
                        },
                        Err(e) => {
                            println!("Unable to parse input: {:?}", e);
                            continue;
                        }
                    };
                    self.vm.program.append(&mut program.to_bytes(&SymbolTable::new()));
                    self.vm.run();
                },
                _ => {
                    match self.mode {
                        REPLMode::Hexadecimal => {
                            let results = Self::parse_hex(buffer);
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
                    
                            let parsed_program = program(CompleteStr(buffer));
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
                    }

                    self.vm.run_once();
                },
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
}
