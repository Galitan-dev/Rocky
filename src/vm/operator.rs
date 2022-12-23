use std::{
    fmt::Debug,
    io::{self, stdout, Write},
    str::FromStr,
    thread,
    time::Duration,
};

use byteorder::{LittleEndian, WriteBytesExt};

use crate::{assembler::PIE_HEADER_LENGTH, instruction::Opcode};

use super::VM;

pub trait Operator {
    fn execute_instruction(&mut self) -> Option<u32>;
    fn calculate<F: FnOnce(i32, i32) -> (i32, Option<u32>)>(&mut self, op: F);
    fn compare<F: FnOnce(i32, i32) -> bool>(&mut self, comparator: F);
    fn sleep(&mut self, unit: i32);
    fn print(&mut self);
    fn jump<F: FnOnce(usize, usize, usize, bool) -> usize>(&mut self, jump: F);
    fn alloc(&mut self);
    fn load(&mut self);
    fn ask<T>(&mut self) -> Option<T>
    where
        T: FromStr,
        T::Err: Debug;
}

impl Operator for VM {
    fn execute_instruction(&mut self) -> Option<u32> {
        if self.program_counter >= self.program.len() {
            return Some(1);
        }

        let decoded_opcode = self.decode_opcode();
        match decoded_opcode {
            Opcode::HLT => {
                println!("HLT encountered");
                return Some(0);
            }
            Opcode::LOAD => self.load(),
            Opcode::ADD => self.calculate(|a, b| (a + b, None)),
            Opcode::SUB => self.calculate(|a, b| (a - b, None)),
            Opcode::MUL => self.calculate(|a, b| (a * b, None)),
            Opcode::DIV => self.calculate(|a, b| (a / b, Some((a % b) as u32))),
            Opcode::JMP => self.jump(|target, offset, _, _| PIE_HEADER_LENGTH + target + offset),
            Opcode::JMPF => self.jump(|target, _, current, _| current + target),
            Opcode::JMPB => self.jump(|target, _, current, _| current - target),
            Opcode::JEQ => self.jump(|target, offset, current, equal_flag| {
                if equal_flag {
                    PIE_HEADER_LENGTH + target + offset
                } else {
                    current
                }
            }),
            Opcode::EQ => self.compare(|a, b| a == b),
            Opcode::NEQ => self.compare(|a, b| a != b),
            Opcode::GT => self.compare(|a, b| a > b),
            Opcode::LT => self.compare(|a, b| a < b),
            Opcode::GTQ => self.compare(|a, b| a >= b),
            Opcode::LTQ => self.compare(|a, b| a <= b),
            Opcode::ALOC => self.alloc(),
            Opcode::PRTS => self.print(),
            Opcode::SLP => self.sleep(1),
            Opcode::SLPS => self.sleep(1000),
            Opcode::ASKI => {
                if let Some(integer) = self.ask::<i32>() {
                    let index = self.next_16_bits() as usize;
                    let mut wtr = Vec::new();
                    wtr.write_i32::<LittleEndian>(integer).unwrap();

                    self.memory_heap.edit(wtr, index);
                } else {
                    self.next_16_bits();
                }
            }
            Opcode::ASKS => {
                if let Some(string) = self.ask::<String>() {
                    let index = self.next_16_bits() as usize;

                    self.memory_heap.edit(string.as_bytes().to_vec(), index);
                } else {
                    self.next_16_bits();
                }
            }
            Opcode::IGL => {
                println!("Illegal instruction encountered");
                return Some(1);
            }
        }

        None
    }

    fn calculate<F: FnOnce(i32, i32) -> (i32, Option<u32>)>(&mut self, op: F) {
        let register1 = self.registers[self.next_8_bits() as usize];
        let register2 = self.registers[self.next_8_bits() as usize];
        let (result, remainder) = op(register1, register2);
        self.registers[self.next_8_bits() as usize] = result;
        if let Some(remainder) = remainder {
            self.remainder = remainder
        }
    }

    fn compare<F: FnOnce(i32, i32) -> bool>(&mut self, comparator: F) {
        let register1 = self.registers[self.next_8_bits() as usize];
        let register2 = self.registers[self.next_8_bits() as usize];
        let equal_flag = comparator(register1, register2);
        self.equal_flag = equal_flag;
        self.next_8_bits();
    }

    fn sleep(&mut self, unit: i32) {
        let register = self.next_8_bits() as usize;
        let milliseconds = self.registers[register] * unit;
        thread::sleep(Duration::from_millis(milliseconds as u64));
    }

    fn ask<T>(&mut self) -> Option<T>
    where
        T: FromStr,
        T::Err: Debug,
    {
        let prompt: &str;
        match self.read_data() {
            Ok(s) => prompt = s,
            Err(e) => {
                println!("Error decoding string for prts instruction: {e:#?}");
                return None;
            }
        };

        print!("{prompt}");
        if let Err(e) = stdout().flush() {
            println!("Error flusing stdout: {e:#?}");
            return None;
        }

        let mut user_input = String::new();
        let stdin = io::stdin();
        if let Err(e) = stdin.read_line(&mut user_input) {
            println!("Error reading user input: {e:#?}");
            return None;
        }

        user_input = user_input[0..user_input.len() - 1].to_string();

        match user_input.parse::<T>() {
            Ok(parsed) => Some(parsed),
            Err(e) => {
                println!("Error parsing user input: {e:#?}");
                None
            }
        }
    }

    fn print(&mut self) {
        match self.read_data() {
            Ok(s) => println!("{s}"),
            Err(e) => println!("Error decoding string for prts instruction: {:#?}", e),
        };
    }

    fn jump<F: FnOnce(usize, usize, usize, bool) -> usize>(&mut self, jump: F) {
        let value = self.registers[self.next_8_bits() as usize];
        self.program_counter = jump(
            value as usize,
            self.get_starting_offset(),
            self.program_counter,
            self.equal_flag,
        );
    }

    fn alloc(&mut self) {
        let register = self.next_8_bits() as usize;
        let bytes = self.registers[register];
        self.memory_heap.alloc(bytes as usize);
    }

    fn load(&mut self) {
        let register = self.next_8_bits() as usize;
        let number = self.next_16_bits() as u32;
        self.registers[register] = number as i32;
    }
}
