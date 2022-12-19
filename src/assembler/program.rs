use nom::types::CompleteStr;
use super::{instruction::{AssemblerInstruction, instruction}, symbols::SymbolTable};

#[derive(Debug, PartialEq)]
pub struct Program {
    pub instructions: Vec<AssemblerInstruction>
}

impl Program {
    pub fn to_bytes(&self, symbols: &SymbolTable) -> Vec<u8> {
        let mut program = vec![];
        for instruction in &self.instructions {
            program.append(&mut instruction.to_bytes(symbols));
        }
        program
    }
}

named!(pub program<CompleteStr, Program>,
    do_parse!(
        instructions: many1!(instruction) >>
        (
            Program {
                instructions: instructions
            }
        )
    )
);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_program() {
        let result = program(CompleteStr("load $0 #100\n"));
        assert_eq!(result.is_ok(), true);
        let (leftover, p) = result.unwrap();
        assert_eq!(leftover, CompleteStr(""));
        assert_eq!(p.instructions.len(), 1);
    }
    
    #[test]
    fn test_program_to_bytes() {
        let result = program(CompleteStr("load $0 #100\n"));
        assert_eq!(result.is_ok(), true);
        let (_, program) = result.unwrap();
        let bytecode = program.to_bytes(&SymbolTable::new());
        assert_eq!(bytecode.len(), 4);
        println!("{:?}", bytecode);
    }

    #[test]
    fn test_complete_program() {
        let test_program = CompleteStr(".data\nhello: .asciiz 'Hello everyone!'\n.code\nhlt");
        let result = program(test_program);
        assert_eq!(result.is_ok(), true);
    }

}