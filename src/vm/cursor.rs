use std::io::{Cursor, Read};

use crate::instruction::Opcode;

pub trait ProgramCursor {
    fn next_8_bits(&mut self) -> Option<u8>;
    fn next_16_bits(&mut self) -> Option<u16>;
    fn get_position(&self) -> usize;

    fn read_opcode(&mut self) -> Option<Opcode> {
        let byte = self.next_8_bits();
        let opcode = byte.map(|byte| Opcode::from(byte));
        // println!("{}: {:?} => {opcode:?}", self.get_position(), byte);
        return opcode;
    }

    fn read_index(&mut self) -> Option<usize> {
        self.next_16_bits().map(|index| index as usize)
    }

    fn read_register_index(&mut self) -> Option<usize> {
        self.next_8_bits().map(|index| index as usize)
    }
}

impl<T> ProgramCursor for Cursor<T>
where
    T: AsRef<[u8]>,
{
    fn next_8_bits(&mut self) -> Option<u8> {
        let mut buf = [0];
        let read = self.read(&mut buf).unwrap_or(0);
        if read == 1 {
            Some(buf[0])
        } else {
            None
        }
    }

    fn next_16_bits(&mut self) -> Option<u16> {
        let mut buf = [0; 2];
        let read = self.read(&mut buf).unwrap_or(0);
        if read == 2 {
            Some(((buf[0] as u16) << 8) | buf[1] as u16)
        } else {
            None
        }
    }

    fn get_position(&self) -> usize {
        self.position() as usize
    }
}
