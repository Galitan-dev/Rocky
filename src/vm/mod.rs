use crate::{
    assembler::{PIE_HEADER_LENGTH, PIE_HEADER_PREFIX},
    instruction::Opcode,
};
use byteorder::{LittleEndian, ReadBytesExt};
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

        self.memory_heap = MemoryHeap::from_bytes(
            &mut Cursor::new(&self.program[PIE_HEADER_LENGTH..]),
            &mut Cursor::new(&self.program[PIE_HEADER_PREFIX.len()..PIE_HEADER_PREFIX.len() + 12]),
        );
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
        // println!(
        //     "{}: {} => {opcode:?}",
        //     self.program_counter, self.program[self.program_counter]
        // );
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
        let index = self.next_16_bits() as usize;
        std::str::from_utf8(&self.memory_heap.get_slice(index))
    }

    pub fn add_byte(&mut self, byte: u8) {
        self.program.push(byte);
    }

    pub fn add_bytes(&mut self, mut b: Vec<u8>) {
        self.program.append(&mut b);
    }

    pub fn set_program(&mut self, prog: Vec<u8>, mem: MemoryHeap) {
        self.memory_heap = mem.clone();
        self.program = Self::prepend_header(prog, mem);
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
            Cursor::new(&self.program[PIE_HEADER_PREFIX.len()..PIE_HEADER_PREFIX.len() + 16]);
        rdr.read_u32::<LittleEndian>().unwrap() as usize
            + rdr.read_u32::<LittleEndian>().unwrap() as usize
    }

    pub fn prepend_header(mut b: Vec<u8>, mem: MemoryHeap) -> Vec<u8> {
        let mut prepension = vec![];
        for byte in PIE_HEADER_PREFIX.into_iter() {
            prepension.push(byte.clone());
        }

        prepension.append(&mut mem.header());

        while prepension.len() < PIE_HEADER_LENGTH {
            prepension.push(0);
        }

        prepension.append(&mut mem.to_bytes());
        prepension.append(&mut b);
        prepension
    }
}
