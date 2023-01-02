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
    ASKI,
    ASKS,
    GRPS,
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
            20 => Opcode::ASKI,
            21 => Opcode::ASKS,
            22 => Opcode::GRPS,
            _ => Opcode::IGL,
        }
    }
}

impl From<String> for Opcode {
    fn from(v: String) -> Self {
        match v.as_str() {
            "load" => Opcode::LOAD,
            "add" => Opcode::ADD,
            "sub" => Opcode::SUB,
            "mul" => Opcode::MUL,
            "div" => Opcode::DIV,
            "hlt" => Opcode::HLT,
            "jmp" => Opcode::JMP,
            "jmpf" => Opcode::JMPF,
            "jmpb" => Opcode::JMPB,
            "eq" => Opcode::EQ,
            "neq" => Opcode::NEQ,
            "gtq" => Opcode::GTQ,
            "gt" => Opcode::GT,
            "ltq" => Opcode::LTQ,
            "lt" => Opcode::LT,
            "jeq" => Opcode::JEQ,
            "aloc" => Opcode::ALOC,
            "prts" => Opcode::PRTS,
            "slp" => Opcode::SLP,
            "slps" => Opcode::SLPS,
            "aski" => Opcode::ASKI,
            "asks" => Opcode::ASKS,
            "grps" => Opcode::GRPS,
            _ => Opcode::IGL,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Instruction {
    opcode: Opcode,
}

impl Instruction {
    pub fn new(opcode: Opcode) -> Self {
        Self { opcode: opcode }
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
        let opcode = Opcode::from("load".to_owned());
        assert_eq!(opcode, Opcode::LOAD);
        let opcode = Opcode::from("illegal".to_owned());
        assert_eq!(opcode, Opcode::IGL);
    }
}
