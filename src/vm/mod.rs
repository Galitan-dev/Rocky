use crate::{
    assembler::{PIE_HEADER_LENGTH, PIE_HEADER_PREFIX},
    instruction::Opcode,
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Cursor;
use uuid::Uuid;

use self::{
    events::{VMEvent, VMEventType},
    operator::Operator,
};

pub mod events;
pub mod operator;

#[derive(Debug, Clone)]
pub struct VM {
    pub registers: [i32; 32],
    pub program_counter: usize,
    pub program: Vec<u8>,
    pub read_only_data: Vec<u8>,
    pub logical_cores: usize,
    memory_heap: Vec<u8>,
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
            memory_heap: Vec::new(),
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_vm() {
        let test_vm = VM::new();
        assert_eq!(test_vm.registers[0], 0)
    }

    mod opcode {
        use super::*;

        #[test]
        fn test_hlt() {
            let mut test_vm = VM::new();
            test_vm.program = vec![0];
            test_vm.program = VM::prepend_header(test_vm.program, Vec::new());
            test_vm.run();
            assert_eq!(test_vm.program_counter, PIE_HEADER_LENGTH + 1);
        }

        #[test]
        fn test_igl() {
            let mut test_vm = VM::new();
            test_vm.set_program(vec![200], Vec::new());
            test_vm.run();
            assert_eq!(test_vm.program_counter, PIE_HEADER_LENGTH + 1);
        }

        #[test]
        fn test_load() {
            let mut test_vm = VM::new();
            test_vm.set_program(vec![1, 0, 1, 244], Vec::new());
            test_vm.run();
            assert_eq!(test_vm.registers[0], 500);
        }

        mod math {
            use super::*;

            #[test]
            fn test_add() {
                let mut test_vm = VM::new();
                test_vm.registers[0] = 50;
                test_vm.registers[1] = 25;
                test_vm.set_program(vec![2, 0, 1, 0], Vec::new());
                test_vm.run_once();
                assert_eq!(test_vm.registers[0], 75);
            }

            #[test]
            fn test_sub() {
                let mut test_vm = VM::new();
                test_vm.registers[0] = 50;
                test_vm.registers[1] = 25;
                test_vm.set_program(vec![3, 0, 1, 0], Vec::new());
                test_vm.run_once();
                assert_eq!(test_vm.registers[0], 25);
            }

            #[test]
            fn test_mul() {
                let mut test_vm = VM::new();
                test_vm.registers[0] = 50;
                test_vm.registers[1] = 5;
                test_vm.set_program(vec![4, 0, 1, 0], Vec::new());
                test_vm.run_once();
                assert_eq!(test_vm.registers[0], 250);
            }

            #[test]
            fn test_div() {
                let mut test_vm = VM::new();
                test_vm.registers[0] = 50;
                test_vm.registers[1] = 6;
                test_vm.set_program(vec![5, 0, 1, 0], Vec::new());
                test_vm.run_once();
                assert_eq!(test_vm.registers[0], 8);
                assert_eq!(test_vm.remainder, 2);
            }
        }

        mod jump {
            use super::*;

            #[test]
            fn test_jmp() {
                let mut test_vm = VM::new();
                test_vm.registers[0] = 5;
                test_vm.set_program(vec![6, 0], Vec::new());
                test_vm.run_once();
                assert_eq!(test_vm.program_counter, PIE_HEADER_LENGTH + 5);
            }

            #[test]
            fn test_jmpf() {
                let mut test_vm = VM::new();
                test_vm.registers[0] = 2;
                test_vm.set_program(vec![7, 0], Vec::new());
                test_vm.run_once();
                assert_eq!(test_vm.program_counter, PIE_HEADER_LENGTH + 4);
            }

            #[test]
            fn test_jmpb() {
                let mut test_vm = VM::new();
                test_vm.registers[0] = 2;
                test_vm.set_program(vec![8, 0], Vec::new());
                test_vm.run_once();
                assert_eq!(test_vm.program_counter, PIE_HEADER_LENGTH);
            }
        }

        mod logic {
            use super::*;

            #[test]
            fn test_eq() {
                let mut test_vm = VM::new();
                test_vm.registers[0] = 10;
                test_vm.registers[1] = 10;
                test_vm.set_program(vec![9, 0, 1, 0, 9, 0, 1, 0], Vec::new());
                test_vm.run_once();
                assert_eq!(test_vm.equal_flag, true);
                test_vm.registers[1] = 20;
                test_vm.run_once();
                assert_eq!(test_vm.equal_flag, false);
            }

            #[test]
            fn test_neq() {
                let mut test_vm = VM::new();
                test_vm.registers[0] = 10;
                test_vm.registers[1] = 20;
                test_vm.set_program(vec![10, 0, 1, 0, 10, 0, 1, 0], Vec::new());
                test_vm.run_once();
                assert_eq!(test_vm.equal_flag, true);
                test_vm.registers[1] = 10;
                test_vm.run_once();
                assert_eq!(test_vm.equal_flag, false);
            }

            #[test]
            fn test_gt() {
                let mut test_vm = VM::new();
                test_vm.registers[0] = 20;
                test_vm.registers[1] = 10;
                test_vm.set_program(vec![11, 0, 1, 0, 11, 0, 1, 0, 11, 0, 1, 0], Vec::new());
                test_vm.run_once();
                assert_eq!(test_vm.equal_flag, true);
                test_vm.registers[1] = 30;
                test_vm.run_once();
                assert_eq!(test_vm.equal_flag, false);
                test_vm.registers[1] = 20;
                test_vm.run_once();
                assert_eq!(test_vm.equal_flag, false);
            }

            #[test]
            fn test_lt() {
                let mut test_vm = VM::new();
                test_vm.registers[0] = 20;
                test_vm.registers[1] = 30;
                test_vm.set_program(vec![12, 0, 1, 0, 12, 0, 1, 0, 12, 0, 1, 0], Vec::new());
                test_vm.run_once();
                assert_eq!(test_vm.equal_flag, true);
                test_vm.registers[1] = 10;
                test_vm.run_once();
                assert_eq!(test_vm.equal_flag, false);
                test_vm.registers[1] = 20;
                test_vm.run_once();
                assert_eq!(test_vm.equal_flag, false);
            }

            #[test]
            fn test_gtq() {
                let mut test_vm = VM::new();
                test_vm.registers[0] = 20;
                test_vm.registers[1] = 10;
                test_vm.set_program(vec![13, 0, 1, 0, 13, 0, 1, 0, 13, 0, 1, 0], Vec::new());
                test_vm.run_once();
                assert_eq!(test_vm.equal_flag, true);
                test_vm.registers[1] = 30;
                test_vm.run_once();
                assert_eq!(test_vm.equal_flag, false);
                test_vm.registers[1] = 20;
                test_vm.run_once();
                assert_eq!(test_vm.equal_flag, true);
            }

            #[test]
            fn test_ltq() {
                let mut test_vm = VM::new();
                test_vm.registers[0] = 20;
                test_vm.registers[1] = 30;
                test_vm.set_program(vec![14, 0, 1, 0, 14, 0, 1, 0, 14, 0, 1, 0], Vec::new());
                test_vm.run_once();
                assert_eq!(test_vm.equal_flag, true);
                test_vm.registers[1] = 10;
                test_vm.run_once();
                assert_eq!(test_vm.equal_flag, false);
                test_vm.registers[1] = 20;
                test_vm.run_once();
                assert_eq!(test_vm.equal_flag, true);
            }

            #[test]
            fn test_jeq() {
                let mut test_vm = VM::new();
                test_vm.registers[0] = 4;
                test_vm.equal_flag = true;
                test_vm.set_program(vec![15, 0, 0, 0, 15, 0], Vec::new());
                test_vm.run_once();
                assert_eq!(test_vm.program_counter, PIE_HEADER_LENGTH + 4);
                test_vm.equal_flag = false;
                test_vm.run_once();
                assert_eq!(test_vm.program_counter, PIE_HEADER_LENGTH + 6);
            }
        }

        #[test]
        fn test_aloc() {
            let mut test_vm = VM::new();
            test_vm.registers[0] = 1024;
            test_vm.set_program(vec![16, 0, 0, 0], Vec::new());
            test_vm.run_once();
            assert_eq!(test_vm.memory_heap.len(), 1024);
        }

        #[test]
        fn test_prts_opcode() {
            let mut test_vm = VM::new();
            test_vm.set_program(vec![17, 0, 0, 0], vec![72, 101, 108, 108, 111, 0]);
            test_vm.run_once();
        }

        mod time {
            use chrono::Utc;

            use super::*;

            #[test]
            fn test_slp_opcode() {
                let mut test_vm = VM::new();
                test_vm.registers[0] = 100;
                test_vm.set_program(vec![18, 0], Vec::new());
                let start = Utc::now().timestamp_millis();
                test_vm.run_once();
                assert!(Utc::now().timestamp_millis() - start >= 100);
            }

            #[test]
            fn test_slps_opcode() {
                let mut test_vm = VM::new();
                test_vm.registers[0] = 1;
                test_vm.set_program(vec![19, 0], Vec::new());
                let start = Utc::now().timestamp_millis();
                test_vm.run_once();
                assert!(Utc::now().timestamp_millis() - start >= 1000);
            }
        }
    }
}
