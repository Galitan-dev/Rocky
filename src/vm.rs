use crate::{assembler::PIE_HEADER_PREFIX, instruction::Opcode};
use chrono::{DateTime, Utc};
use std::{thread, time::Duration};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum VMEventType {
    Start,
    GracefulStop { code: u32 },
    Crash,
}

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct VMEvent {
    event: VMEventType,
    at: DateTime<Utc>,
    application_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct VM {
    pub registers: [i32; 32],
    pub pc: usize,
    pub program: Vec<u8>,
    pub ro_data: Vec<u8>,
    heap: Vec<u8>,
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
            pc: 0,
            program: Vec::new(),
            ro_data: Vec::new(),
            heap: Vec::new(),
            events: Vec::new(),
            id: Uuid::new_v4(),
        }
    }

    // Loops as long as instructions can be executed.
    pub fn run(&mut self) -> Vec<VMEvent> {
        self.events.push(VMEvent {
            event: VMEventType::Start,
            at: Utc::now(),
            application_id: self.id.clone(),
        });

        if !self.verify_header() {
            self.events.push(VMEvent {
                event: VMEventType::Crash,
                at: Utc::now(),
                application_id: self.id.clone(),
            });
            println!("Header was incorrect");
            return self.events.clone();
        }

        self.pc = 64;

        let mut is_done = None;
        while is_done.is_none() {
            is_done = self.execute_instruction();
        }

        self.events.push(VMEvent {
            event: VMEventType::GracefulStop {
                code: is_done.unwrap(),
            },
            at: Utc::now(),
            application_id: self.id.clone(),
        });

        self.events.clone()
    }

    // Executes one instruction. Meant to allow for more controlled execution of the VM
    pub fn run_once(&mut self) {
        self.execute_instruction();
    }

    fn execute_instruction(&mut self) -> Option<u32> {
        if self.pc >= self.program.len() {
            return Some(1);
        }

        let opcode = self.decode_opcode();
        match opcode {
            Opcode::HLT => {
                println!("HLT encountered");
                return Some(0);
            }
            Opcode::LOAD => {
                let register = self.next_8_bits() as usize;
                let number = self.next_16_bits() as u32;
                self.registers[register] = number as i32;
            }
            Opcode::ADD => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 + register2;
            }
            Opcode::SUB => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 - register2;
            }
            Opcode::MUL => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 * register2;
            }
            Opcode::DIV => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 / register2;
                self.remainder = (register1 % register2) as u32
            }
            Opcode::JMP => {
                let target = self.registers[self.next_8_bits() as usize];
                self.pc = target as usize;
            }
            Opcode::JMPF => {
                let value = self.registers[self.next_8_bits() as usize];
                self.pc += value as usize;
            }
            Opcode::JMPB => {
                let value = self.registers[self.next_8_bits() as usize];
                self.pc -= value as usize;
            }
            Opcode::EQ => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 == register2;
                self.next_8_bits();
            }
            Opcode::NEQ => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 != register2;
                self.next_8_bits();
            }
            Opcode::GT => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 > register2;
                self.next_8_bits();
            }
            Opcode::LT => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 < register2;
                self.next_8_bits();
            }
            Opcode::GTQ => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 >= register2;
                self.next_8_bits();
            }
            Opcode::LTQ => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 <= register2;
                self.next_8_bits();
            }
            Opcode::JEQ => {
                let register = self.next_8_bits() as usize;
                let target = self.registers[register];
                if self.equal_flag {
                    self.pc = target as usize;
                }
            }
            Opcode::ALOC => {
                let register = self.next_8_bits() as usize;
                let bytes = self.registers[register];
                let new_end = self.heap.len() as i32 + bytes;
                self.heap.resize(new_end as usize, 0);
            }
            Opcode::PRTS => {
                let starting_offset = self.next_16_bits() as usize;
                let mut ending_offset = starting_offset;
                let slice = self.ro_data.as_slice();
                while slice[ending_offset] != 0 {
                    ending_offset += 1;
                }
                let result = std::str::from_utf8(&slice[starting_offset..ending_offset]);
                match result {
                    Ok(s) => {
                        println!("{}", s);
                    }
                    Err(e) => {
                        println!("Error decoding string for prts instruction: {:#?}", e)
                    }
                };
            }
            Opcode::SLP => {
                let register = self.next_8_bits() as usize;
                let milliseconds = self.registers[register];
                thread::sleep(Duration::from_millis(milliseconds as u64));
            }
            Opcode::SLPS => {
                let register = self.next_8_bits() as usize;
                let seconds = self.registers[register];
                thread::sleep(Duration::from_secs(seconds as u64));
            }
            Opcode::IGL => {
                println!("Illegal instruction encountered");
                return Some(1);
            }
        }

        None
    }

    fn decode_opcode(&mut self) -> Opcode {
        let opcode = Opcode::from(self.program[self.pc]);
        self.pc += 1;
        return opcode;
    }

    fn next_8_bits(&mut self) -> u8 {
        let result = self.program[self.pc];
        self.pc += 1;
        result
    }

    fn next_16_bits(&mut self) -> u16 {
        let result = ((self.program[self.pc] as u16) << 8) | self.program[self.pc + 1] as u16;
        self.pc += 2;
        result
    }

    pub fn add_byte(&mut self, byte: u8) {
        self.program.push(byte);
    }

    pub fn add_bytes(&mut self, mut b: Vec<u8>) {
        self.program.append(&mut b);
    }

    fn verify_header(&self) -> bool {
        if self.program[0..4] != PIE_HEADER_PREFIX {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod test {
    use crate::assembler::PIE_HEADER_LENGTH;

    use super::*;

    fn prepend_header(mut b: Vec<u8>) -> Vec<u8> {
        let mut prepension = vec![];
        for byte in PIE_HEADER_PREFIX.into_iter() {
            prepension.push(byte.clone());
        }
        while prepension.len() < PIE_HEADER_LENGTH {
            prepension.push(0);
        }
        prepension.append(&mut b);
        prepension
    }

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
            test_vm.program = prepend_header(test_vm.program);
            test_vm.run();
            assert_eq!(test_vm.pc, 65);
        }

        #[test]
        fn test_igl() {
            let mut test_vm = VM::new();
            test_vm.program = vec![200];
            test_vm.program = prepend_header(test_vm.program);
            test_vm.run();
            assert_eq!(test_vm.pc, 65);
        }

        #[test]
        fn test_load() {
            let mut test_vm = VM::new();
            test_vm.program = vec![1, 0, 1, 244];
            test_vm.program = prepend_header(test_vm.program);
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
                test_vm.program = vec![2, 0, 1, 0];
                test_vm.program = prepend_header(test_vm.program);
                test_vm.run();
                assert_eq!(test_vm.registers[0], 75);
            }

            #[test]
            fn test_sub() {
                let mut test_vm = VM::new();
                test_vm.registers[0] = 50;
                test_vm.registers[1] = 25;
                test_vm.program = vec![3, 0, 1, 0];
                test_vm.program = prepend_header(test_vm.program);
                test_vm.run();
                assert_eq!(test_vm.registers[0], 25);
            }

            #[test]
            fn test_mul() {
                let mut test_vm = VM::new();
                test_vm.registers[0] = 50;
                test_vm.registers[1] = 5;
                test_vm.program = vec![4, 0, 1, 0];
                test_vm.program = prepend_header(test_vm.program);
                test_vm.run();
                assert_eq!(test_vm.registers[0], 250);
            }

            #[test]
            fn test_div() {
                let mut test_vm = VM::new();
                test_vm.registers[0] = 50;
                test_vm.registers[1] = 6;
                test_vm.program = vec![5, 0, 1, 0];
                test_vm.program = prepend_header(test_vm.program);
                test_vm.run();
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
                test_vm.program = vec![6, 0];
                test_vm.run_once();
                assert_eq!(test_vm.pc, 5);
            }

            #[test]
            fn test_jmpf() {
                let mut test_vm = VM::new();
                test_vm.registers[0] = 2;
                test_vm.program = vec![7, 0];
                test_vm.run_once();
                assert_eq!(test_vm.pc, 4);
            }

            #[test]
            fn test_jmpb() {
                let mut test_vm = VM::new();
                test_vm.registers[0] = 2;
                test_vm.program = vec![8, 0];
                test_vm.run_once();
                assert_eq!(test_vm.pc, 0);
            }
        }

        mod logic {
            use super::*;

            #[test]
            fn test_eq() {
                let mut test_vm = VM::new();
                test_vm.registers[0] = 10;
                test_vm.registers[1] = 10;
                test_vm.program = vec![9, 0, 1, 0, 9, 0, 1, 0];
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
                test_vm.program = vec![10, 0, 1, 0, 10, 0, 1, 0];
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
                test_vm.program = vec![11, 0, 1, 0, 11, 0, 1, 0, 11, 0, 1, 0];
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
                test_vm.program = vec![12, 0, 1, 0, 12, 0, 1, 0, 12, 0, 1, 0];
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
                test_vm.program = vec![13, 0, 1, 0, 13, 0, 1, 0, 13, 0, 1, 0];
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
                test_vm.program = vec![14, 0, 1, 0, 14, 0, 1, 0, 14, 0, 1, 0];
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
                test_vm.program = vec![15, 0, 0, 0, 15, 0];
                test_vm.run_once();
                assert_eq!(test_vm.pc, 4);
                test_vm.equal_flag = false;
                test_vm.run_once();
                assert_eq!(test_vm.pc, 6);
            }
        }

        #[test]
        fn test_aloc() {
            let mut test_vm = VM::new();
            test_vm.registers[0] = 1024;
            test_vm.program = vec![16, 0, 0, 0];
            test_vm.run_once();
            assert_eq!(test_vm.heap.len(), 1024);
        }

        #[test]
        fn test_prts_opcode() {
            let mut test_vm = VM::new();
            test_vm.ro_data.append(&mut vec![72, 101, 108, 108, 111, 0]);
            test_vm.program = vec![17, 0, 0, 0];
            test_vm.run_once();
        }

        mod time {
            use super::*;

            #[test]
            fn test_slp_opcode() {
                let mut test_vm = VM::new();
                test_vm.registers[0] = 100;
                test_vm.program = vec![18, 0];
                let start = Utc::now().timestamp_millis();
                test_vm.run_once();
                assert!(Utc::now().timestamp_millis() - start >= 100);
            }

            #[test]
            fn test_slps_opcode() {
                let mut test_vm = VM::new();
                test_vm.registers[0] = 1;
                test_vm.program = vec![19, 0];
                let start = Utc::now().timestamp_millis();
                test_vm.run_once();
                assert!(Utc::now().timestamp_millis() - start >= 1000);
            }
        }
    }
}
