use crate::{
    assembler::{PIE_HEADER_LENGTH, PIE_HEADER_PREFIX},
    instruction::Opcode,
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;
use uuid::Uuid;

use self::{
    events::{VMEvent, VMEventType},
    memory::MemoryHeap,
    operator::Operator,
};

pub mod events;
pub mod memory;
pub mod operator;
#[cfg(test)]
pub mod tests;

#[derive(Debug, Clone)]
pub struct VM {
    pub registers: [i32; 32],
    pub program_counter: usize,
    pub program: Vec<u8>,
    pub read_only_data: Vec<u8>,
    pub logical_cores: usize,
    pub memory_heap: MemoryHeap,
    remainder: u32,
    equal_flag: bool,
    id: Uuid,
    events: Vec<VMEvent>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            registers: [0; 32],
            remainder: 0,
            equal_flag: false,
            program_counter: 0,
            program: Vec::new(),
            read_only_data: Vec::new(),
            memory_heap: MemoryHeap::new(0),
            events: Vec::new(),
            id: Uuid::new_v4(),
            logical_cores: num_cpus::get(),
        }
    }

    // Loops as long as instructions can be executed.
    pub fn run(&mut self) -> Vec<VMEvent> {
        self.events
            .push(VMEvent::now(VMEventType::Start, self.id.clone()));

        if !self.verify_header() {
            self.events
                .push(VMEvent::now(VMEventType::Crash, self.id.clone()));
            println!("Header was incorrect");
            return self.events.clone();
        }

        self.read_only_data = self.program
            [PIE_HEADER_LENGTH..PIE_HEADER_LENGTH + self.get_starting_offset()]
            .to_vec();
        self.program_counter = PIE_HEADER_LENGTH + self.get_starting_offset();

        let mut is_done = None;
        while is_done.is_none() {
            is_done = self.execute_instruction();
        }

        self.events.push(VMEvent::now(
            VMEventType::GracefulStop {
                code: is_done.unwrap(),
            },
            self.id.clone(),
        ));

        self.events.clone()
    }

    // Executes one instruction. Meant to allow for more controlled execution of the VM
    pub fn run_once(&mut self) {
        self.execute_instruction();
    }

    fn decode_opcode(&mut self) -> Opcode {
        let opcode = Opcode::from(self.program[self.program_counter]);
        // println!("{}: {} => {opcode:?}", self.pc, self.program[self.pc]);
        self.program_counter += 1;
        return opcode;
    }

    fn next_8_bits(&mut self) -> u8 {
        let result = self.program[self.program_counter];
        self.program_counter += 1;
        result
    }

    fn next_16_bits(&mut self) -> u16 {
        let result = ((self.program[self.program_counter] as u16) << 8)
            | self.program[self.program_counter + 1] as u16;
        self.program_counter += 2;
        result
    }

    fn read_data(&mut self) -> Result<&str, std::str::Utf8Error> {
        let starting_offset = self.next_16_bits() as usize;
        let mut ending_offset = starting_offset;
        let slice = self.read_only_data.as_slice();
        while slice[ending_offset] != 0 {
            ending_offset += 1;
        }
        std::str::from_utf8(&slice[starting_offset..ending_offset])
    }

    pub fn add_byte(&mut self, byte: u8) {
        self.program.push(byte);
    }

    pub fn add_bytes(&mut self, mut b: Vec<u8>) {
        self.program.append(&mut b);
    }

    pub fn set_program(&mut self, prog: Vec<u8>, ro: Vec<u8>) {
        self.read_only_data = ro.clone();
        self.program = Self::prepend_header(prog, ro);
        self.program_counter = PIE_HEADER_LENGTH + self.get_starting_offset();
    }

    fn verify_header(&self) -> bool {
        if self.program[0..PIE_HEADER_PREFIX.len()] != PIE_HEADER_PREFIX {
            return false;
        }
        true
    }

    fn get_starting_offset(&self) -> usize {
        let mut rdr =
            Cursor::new(&self.program[PIE_HEADER_PREFIX.len()..PIE_HEADER_PREFIX.len() + 4]);
        rdr.read_u32::<LittleEndian>().unwrap() as usize
    }

    pub fn prepend_header(mut b: Vec<u8>, mut ro: Vec<u8>) -> Vec<u8> {
        let mut prepension = vec![];
        for byte in PIE_HEADER_PREFIX.into_iter() {
            prepension.push(byte.clone());
        }

        let mut wtr: Vec<u8> = Vec::new();

        wtr.write_u32::<LittleEndian>(ro.len() as u32).unwrap();

        prepension.append(&mut wtr);

        while prepension.len() < PIE_HEADER_LENGTH {
            prepension.push(0);
        }
        prepension.append(&mut ro);
        prepension.append(&mut b);
        prepension
    }
}
