use crate::{
    assembler::{program_parser::program, symbols::SymbolTable, Assembler},
    scheduler::Scheduler,
    vm::VM,
};
use rustyline::{error::ReadlineError, Editor};
use std::{fs::File, io::Read, num::ParseIntError, path::Path};

use self::{
    command_parser::CommandParser,
    hinter::{rk_hints, RkHinter},
};

pub mod command_parser;
pub mod hinter;

pub enum REPLMode {
    Hexadecimal,
    Assembly,
}

pub struct REPL {
    mode: REPLMode,
    vm: VM,
    asm: Assembler,
    scheduler: Scheduler,
    rl: Editor<RkHinter>,
    helper: RkHinter,
}

impl REPL {
    pub fn new(mode: REPLMode) -> Result<Self, ReadlineError> {
        Ok(Self {
            mode,
            vm: VM::new(),
            asm: Assembler::new(),
            scheduler: Scheduler::new(),
            rl: Editor::new()?,
            helper: RkHinter { hints: rk_hints() },
        })
    }

    pub fn run(&mut self) {
        println!("Welcome to Rocky! Let's be nerds!");

        self.rl.set_helper(Some(self.helper.clone()));
        self.rl.load_history("history.txt").ok();

        loop {
            match self.rl.readline(">>> ") {
                Ok(line) => {
                    self.rl.add_history_entry(&line);
                    if line.starts_with("!") {
                        self.execute_command(&line);
                    } else {
                        match self.mode {
                            REPLMode::Hexadecimal => self.execute_hexadecimal(&line),
                            REPLMode::Assembly => self.execute_assembly(&line),
                        }
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("(To quit, press Ctrl+D or type !quit)");
                }
                Err(ReadlineError::Eof) => {
                    self.execute_command("!quit");
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
    }

    fn execute_assembly(&mut self, assembly: &str) {
        let parsed_program = program(assembly);
        if parsed_program.is_err() {
            println!("Unable to parse input");
            return;
        }
        let (_, result) = parsed_program.unwrap();
        let bytecode = result.to_bytes(&SymbolTable::new());
        for byte in bytecode {
            self.vm.add_byte(byte);
        }
        self.vm.run_once();
    }

    fn execute_hexadecimal(&mut self, hexadecimal: &str) {
        let results = Self::parse_hex(hexadecimal);
        match results {
            Ok(bytes) => {
                for byte in bytes {
                    self.vm.add_byte(byte)
                }
                self.vm.run_once();
            }
            Err(_e) => {
                println!("Unable to decode hex string. Please enter 4 groups of 2 hex characters.")
            }
        };
    }

    fn execute_command(&mut self, input: &str) {
        let args = CommandParser::tokenize(input);
        match args[0] {
            "!quit" => self.quit(&args[1..]),
            "!program" => self.program(&args[1..]),
            "!clear_program" => self.clear_program(&args[1..]),
            "!clear_registers" => self.clear_registers(&args[1..]),
            "!registers" => self.registers(&args[1..]),
            "!symbols" => self.symbols(&args[1..]),
            "!load_file" => self.load_file(&args[1..]),
            "!spawn" => self.spawn(&args[1..]),
            _ => {
                println!("Invalid command")
            }
        };
    }

    fn quit(&mut self, _args: &[&str]) {
        println!("Bye! Have a good rest!");
        self.rl.save_history("history.txt").ok();
        std::process::exit(0);
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
                    self.vm.program.append(&mut assembled_program);
                    self.scheduler.get_thread(self.vm.clone());
                }
                Err(errors) => {
                    for error in errors {
                        println!("Unable to parse input: {}", error);
                    }
                }
            }
        }
    }

    fn parse_hex(i: &str) -> Result<Vec<u8>, ParseIntError> {
        let split = i.split(" ").collect::<Vec<&str>>();
        let mut results: Vec<u8> = vec![];
        for hex_string in split {
            let byte = u8::from_str_radix(&hex_string, 16);
            match byte {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(results)
    }

    fn get_data_from_load(&mut self, filename: &Path) -> Option<String> {
        let mut f = match File::open(&filename) {
            Ok(f) => f,
            Err(e) => {
                println!("There was an error opening that file: {:?}", e);
                return None;
            }
        };
        let mut contents = String::new();
        match f.read_to_string(&mut contents) {
            Ok(_bytes_read) => Some(contents),
            Err(e) => {
                println!("there was an error reading that file: {:?}", e);
                None
            }
        }
    }
}
