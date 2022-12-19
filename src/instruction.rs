use nom::types::CompleteStr;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Opcode {
    HLT,
    LOAD,
    ADD,
    SUB,
    MUL,
    DIV,
    JMP,
    JMPF,
    JMPB,
    EQ,
    NEQ,
    GT,
    LT,
    GTQ,
    LTQ,
    JEQ,
    ALOC,
    PRTS,
    SLP,
    SLPS,
    IGL,
}

impl From<u8> for Opcode {
    fn from(v: u8) -> Self {
        match v {
            0 => Opcode::HLT,
            1 => Opcode::LOAD,
            2 => Opcode::ADD,
            3 => Opcode::SUB,
            4 => Opcode::MUL,
            5 => Opcode::DIV,
            6 => Opcode::JMP,
            7 => Opcode::JMPF,
            8 => Opcode::JMPB,
            9 => Opcode::EQ,
            10 => Opcode::NEQ,
            11 => Opcode::GT,
            12 => Opcode::LT,
            13 => Opcode::GTQ,
            14 => Opcode::LTQ,
            15 => Opcode::JEQ,
            16 => Opcode::ALOC,
            17 => Opcode::PRTS,
            18 => Opcode::SLP,
            19 => Opcode::SLPS,
            _ => Opcode::IGL
        }
    }
}

impl<'a> From<CompleteStr<'a>> for Opcode {
    fn from(v: CompleteStr<'a>) -> Self {
        match &v {
            CompleteStr("load") => Opcode::LOAD,
            CompleteStr("add") => Opcode::ADD,
            CompleteStr("sub") => Opcode::SUB,
            CompleteStr("mul") => Opcode::MUL,
            CompleteStr("div") => Opcode::DIV,
            CompleteStr("hlt") => Opcode::HLT,
            CompleteStr("jmp") => Opcode::JMP,
            CompleteStr("jmpf") => Opcode::JMPF,
            CompleteStr("jmpb") => Opcode::JMPB,
            CompleteStr("eq") => Opcode::EQ,
            CompleteStr("neq") => Opcode::NEQ,
            CompleteStr("gtq") => Opcode::GTQ,
            CompleteStr("gte") => Opcode::GTQ,
            CompleteStr("gt") => Opcode::GT,
            CompleteStr("ltq") => Opcode::LTQ,
            CompleteStr("lte") => Opcode::LTQ,
            CompleteStr("lt") => Opcode::LT,
            CompleteStr("jeq") => Opcode::JEQ,
            CompleteStr("jmpe") => Opcode::JEQ,
            CompleteStr("aloc") => Opcode::ALOC,
            CompleteStr("prts") => Opcode::PRTS,
            CompleteStr("slp") => Opcode::SLP,
            CompleteStr("wait") => Opcode::SLP,
            CompleteStr("slps") => Opcode::SLPS,
            CompleteStr("waits") => Opcode::SLP,
            _ => Opcode::IGL,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Instruction {
    opcode: Opcode
}

impl Instruction {
    pub fn new(opcode: Opcode) -> Self {
        Self {
            opcode: opcode
        }
    }   
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_create_hlt() {
        let opcode = Opcode::HLT;
        assert_eq!(opcode, Opcode::HLT);
    }

    #[test]
    fn test_create_instruction() {
        let instruction = Instruction::new(Opcode::HLT);
        assert_eq!(instruction.opcode, Opcode::HLT);
    }

    #[test]
    fn test_str_to_opcode() {
        let opcode = Opcode::from(CompleteStr("load"));
        assert_eq!(opcode, Opcode::LOAD);
        let opcode = Opcode::from(CompleteStr("illegal"));
        assert_eq!(opcode, Opcode::IGL);
    }
}