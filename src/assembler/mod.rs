use byteorder::{LittleEndian, WriteBytesExt};

use crate::instruction::Opcode;

use self::{
    error::AssemblerError,
    instruction_parser::AssemblerInstruction,
    program_parser::{program, Program},
    symbols::{Symbol, SymbolTable, SymbolType},
};

pub mod directive_parser;
pub mod error;
pub mod instruction_parser;
pub mod label_parser;
pub mod opcode_parser;
pub mod operand_parser;
pub mod program_parser;
pub mod register_parser;
pub mod symbols;
pub mod utils;

pub const PIE_HEADER_PREFIX: [u8; 5] = [114, 111, 99, 107, 121];
pub const PIE_HEADER_LENGTH: usize = 64;

#[derive(Debug, PartialEq)]
pub enum Token {
    Opcode { code: Opcode },
    Register { reg_num: u8 },
    IntegerOperand { value: i32 },
    LabelDeclaration { name: String },
    LabelUsage { name: String },
    Directive { name: String },
    RkString { name: String },
}

#[derive(Debug, PartialEq)]
pub enum AssemblerPhase {
    First,
    Second,
}

#[derive(Debug)]
pub struct Assembler {
    phase: AssemblerPhase,
    pub symbols: SymbolTable,
    pub ro: Vec<u8>,
    pub bytecode: Vec<u8>,
    ro_offset: u32,
    sections: Vec<AssemblerSection>,
    current_section: Option<AssemblerSection>,
    current_instruction: u32,
    errors: Vec<AssemblerError>,
}

impl Assembler {
    pub fn new() -> Assembler {
        Assembler {
            phase: AssemblerPhase::First,
            symbols: SymbolTable::new(),
            ro: Vec::new(),
            bytecode: Vec::new(),
            ro_offset: 0,
            sections: Vec::new(),
            current_section: None,
            current_instruction: 0,
            errors: Vec::new(),
        }
    }

    pub fn assemble(&mut self, raw: &str) -> Result<Vec<u8>, Vec<AssemblerError>> {
        match program(raw) {
            Ok((remainder, program)) => {
                if remainder != "" {
                    println!("Remaining {remainder:?}");
                    return Err(vec![AssemblerError::UnterminatedProgram]);
                }

                self.process_first_phase(&program);

                if !self.errors.is_empty() {
                    return Err(self.errors.clone());
                };

                if self.sections.len() != 2 {
                    println!("Did not find at least two sections.");
                    self.errors.push(AssemblerError::InsufficientSections);
                    return Err(self.errors.clone());
                }

                let mut body = self.process_second_phase(&program);

                let mut assembled_program = self.write_pie_header();

                // this will empty both body and ro
                assembled_program.append(&mut self.ro);
                assembled_program.append(&mut body);
                Ok(assembled_program)
            }
            Err(e) => {
                println!("There was an error parsing the code: {:?}", e);
                Err(vec![AssemblerError::ParseError {
                    error: e.to_string(),
                }])
            }
        }
    }

    fn process_first_phase(&mut self, p: &Program) {
        for i in &p.instructions {
            if i.is_label() {
                if self.current_section.is_some() {
                    self.process_label_declaration(&i);
                } else {
                    self.errors.push(AssemblerError::NoSegmentDeclarationFound {
                        instruction: self.current_instruction,
                    });
                }
            }

            if i.is_directive() {
                self.process_directive(i);
            }

            self.current_instruction += 1;
        }

        self.phase = AssemblerPhase::Second;
    }

    fn process_label_declaration(&mut self, i: &AssemblerInstruction) {
        let name = match i.label_name() {
            Some(name) => name,
            None => {
                self.errors
                    .push(AssemblerError::StringConstantDeclaredWithoutLabel {
                        instruction: self.current_instruction,
                    });
                return;
            }
        };

        if self.symbols.has_symbol(&name) {
            self.errors.push(AssemblerError::SymbolAlreadyDeclared);
            return;
        }

        let symbol = Symbol::new(name, SymbolType::Label, 0);
        self.symbols.add_symbol(symbol);
    }

    fn process_directive(&mut self, i: &AssemblerInstruction) {
        let directive_name = match i.directive_name() {
            Some(name) => name,
            None => {
                println!("Directive has an invalid name: {:?}", i);
                return;
            }
        };

        if i.has_operands() {
            match directive_name.as_ref() {
                "asciiz" => {
                    self.handle_asciiz(i);
                }
                _ => {
                    self.errors.push(AssemblerError::UnknownDirectiveFound {
                        directive: directive_name.clone(),
                    });
                    return;
                }
            }
        } else {
            self.process_section_header(&directive_name);
        }
    }

    fn handle_asciiz(&mut self, i: &AssemblerInstruction) {
        if self.phase != AssemblerPhase::First {
            return;
        }

        match i.string_constant() {
            Some(s) => {
                match i.label_name() {
                    Some(name) => self.symbols.set_symbol_offset(&name, self.ro_offset),
                    None => {
                        println!("Found a string constant with no associated label!");
                        return;
                    }
                };

                for byte in s.as_bytes() {
                    self.ro.push(*byte);
                    self.ro_offset += 1;
                }

                self.ro.push(0);
                self.ro_offset += 1;
            }
            None => {
                println!("String constant following an .asciiz was empty");
            }
        }
    }

    fn process_section_header(&mut self, header_name: &str) {
        let new_section: AssemblerSection = header_name.into();
        if new_section == AssemblerSection::Unknown {
            println!(
                "Found an section header that is unknown: {:#?}",
                header_name
            );
            return;
        }
        self.sections.push(new_section.clone());
        self.current_section = Some(new_section);
    }

    fn process_second_phase(&mut self, p: &Program) -> Vec<u8> {
        self.current_instruction = 0;
        let mut program = Vec::new();
        for i in &p.instructions {
            if i.is_opcode() {
                let mut bytes = i.to_bytes(&self.symbols);
                program.append(&mut bytes);
            }
            if i.is_directive() {
                self.process_directive(i);
            }
            self.current_instruction += 1
        }
        program
    }

    fn write_pie_header(&self) -> Vec<u8> {
        let mut header = vec![];
        for byte in &PIE_HEADER_PREFIX {
            header.push(byte.clone());
        }

        let mut wtr: Vec<u8> = Vec::new();

        wtr.write_u32::<LittleEndian>(self.ro.len() as u32).unwrap();

        header.append(&mut wtr);

        while header.len() < PIE_HEADER_LENGTH {
            header.push(0 as u8);
        }

        header
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssemblerSection {
    Data { starting_instruction: Option<u32> },
    Code { starting_instruction: Option<u32> },
    Unknown,
}

impl Default for AssemblerSection {
    fn default() -> Self {
        AssemblerSection::Unknown
    }
}

impl<'a> From<&'a str> for AssemblerSection {
    fn from(name: &str) -> AssemblerSection {
        match name {
            "data" => AssemblerSection::Data {
                starting_instruction: None,
            },
            "code" => AssemblerSection::Code {
                starting_instruction: None,
            },
            _ => AssemblerSection::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::VM;

    #[test]
    fn test_assemble_program() {
        let mut asm = Assembler::new();
        let test_string = ".data\n.code\nload $0 #100\nload $1 #1\nload $2 #0\ntest: inc $0\nneq $0 $2\njmpe @test\nhlt";
        let program = asm.assemble(test_string).unwrap();
        let mut vm = VM::new();
        assert_eq!(program.len(), 85);
        vm.add_bytes(program);
        assert_eq!(vm.program.len(), 85);
    }

    #[test]
    fn test_code_start_offset_written() {
        let mut asm = Assembler::new();
        let test_string = ".data\ntest1: .asciiz 'Hello'\n.code\nload $0 #100\nload $1 #1\nload $2 #0\ntest: inc $0\nneq $0 $2\njmpe @test\nhlt";
        let program = asm.assemble(test_string);
        assert_eq!(program.is_ok(), true);
        assert_eq!(program.unwrap()[PIE_HEADER_PREFIX.len()], 6);
    }
}
