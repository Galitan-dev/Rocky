use crate::{
    assembler::{PIE_HEADER_LENGTH, PIE_HEADER_PREFIX},
    vm::cursor::ProgramCursor,
};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;
use uuid::Uuid;

use self::{
    events::{VMEvent, VMEventType},
    memory::MemoryHeap,
    operator::Operator,
};

pub mod cursor;
pub mod events;
pub mod memory;
pub mod operator;
#[cfg(test)]
pub mod tests;

#[derive(Debug, Clone)]
pub struct VM {
    pub registers: [i32; 32],
    pub program: Vec<u8>,
    pub program_cursor: Cursor<Vec<u8>>,
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
            program: Vec::new(),
            program_cursor: Cursor::new(Vec::new()),
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
        self.program_cursor
            .set_position((PIE_HEADER_LENGTH + self.get_starting_offset()) as u64);

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

    fn read_data(&mut self) -> Option<&str> {
        if let Some(index) = self.program_cursor.next_16_bits() {
            std::str::from_utf8(&self.memory_heap.get_slice(index as usize)).ok()
        } else {
            None
        }
    }

    pub fn add_byte(&mut self, byte: u8) {
        self.program.push(byte);
        self.update_program_cursor();
    }

    pub fn update_program_cursor(&mut self) {
        let pos = self.program_cursor.position();
        self.program_cursor = Cursor::new(self.program.clone());
        self.program_cursor.set_position(pos);
    }

    pub fn add_bytes(&mut self, mut b: Vec<u8>) {
        self.program.append(&mut b);
        self.update_program_cursor();
    }

    pub fn set_program(&mut self, prog: Vec<u8>, mem: MemoryHeap) {
        self.memory_heap = mem.clone();
        self.program = Self::prepend_header(prog, mem);
        self.update_program_cursor();
        self.program_cursor
            .set_position((PIE_HEADER_LENGTH + self.get_starting_offset()) as u64);
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
