use nom::{types::CompleteStr};
use super::{Token, opcode::opcode, operand::operand, label::label_declaration, directive::directive, symbols::SymbolTable};


#[derive(Debug, PartialEq)]
pub struct AssemblerInstruction {
    pub opcode: Option<Token>,
    pub label: Option<Token>,
    pub directive: Option<Token>,
    pub operand1: Option<Token>,
    pub operand2: Option<Token>,
    pub operand3: Option<Token>,
}

impl Default for AssemblerInstruction{
    fn default() -> Self {
        Self { 
            opcode: None, 
            label: None, 
            directive: None, 
            operand1: None, 
            operand2: None, 
            operand3: None,
        }
    }
}

impl AssemblerInstruction {
    pub fn to_bytes(&self, symbols: &SymbolTable) -> Vec<u8> {
        let mut results: Vec<u8> = vec![];
        if let Some(ref token) = self.opcode {
            match token {
                Token::Opcode { code } =>  results.push(*code as u8),
                _ => println!("Non-opcode found in opcode field")
            }
        }
        for operand in &[&self.operand1, &self.operand2, &self.operand3] {
            if let Some(token) = operand {
                AssemblerInstruction::extract_operand(token, &mut results, symbols);
            }
        }

        results
    }


    pub fn is_label(&self) -> bool {
        self.label.is_some()
    }

    pub fn label_name(&self) -> Option<String> {
        match &self.label {
            Some(l) => match l {
                Token::LabelDeclaration { name } => Some(name.clone()),
                _ => None,
            },
            None => None,
        }
    }

    fn extract_operand(t: &Token, results: &mut Vec<u8>, symbols: &SymbolTable) {
        match t {
            Token::Register { reg_num } => {
                results.push(*reg_num);
            }
            Token::IntegerOperand { value } => {
                let converted = *value as u16;
                let byte1 = converted;
                let byte2 = converted >> 8;
                results.push(byte2 as u8);
                results.push(byte1 as u8);
            },
            Token::LabelUsage { name } => {
                let value = symbols.symbol_value(name).unwrap();
                let converted = value as u16;
                let byte1 = converted;
                let byte2 = converted >> 8;
                results.push(byte2 as u8);
                results.push(byte1 as u8);
            },
            _ => {
                println!("Opcode found in operand field");
                std::process::exit(1);
            }
        };
    }

    pub fn is_directive(&self) -> bool {
        self.directive.is_some()
    }

    pub fn directive_name(&self) -> Option<String> {
        match &self.directive {
            Some(d) => match d {
                Token::Directive { name } => Some(name.clone()),
                _ => None,
            },
            None => None,
        }
    }

    pub fn has_operands(&self) -> bool {
        self.operand1.is_some() || self.operand2.is_some() || self.operand3.is_some()
    }

    pub fn string_constant(&self) -> Option<String> {
        match &self.operand1 {
            Some(d) => match d {
                Token::RkString { name } => Some(name.clone()),
                _ => None,
            },
            None => None,
        }
    }

    pub fn is_opcode(&self) -> bool {
        self.opcode.is_some()
    }
}

named!(instruction_combined<CompleteStr, AssemblerInstruction>,
    ws!(do_parse!(
        l: opt!(label_declaration) >>
        o: opcode >>
        o1: opt!(operand) >>
        o2: opt!(operand) >>
        o3: opt!(operand) >>
        _o4: opt!(operand) >>
        (
            AssemblerInstruction {
                opcode: Some(o),
                label: l,
                operand1: o1,
                operand2: o2,
                operand3: o3,
                ..Default::default()
            }
        )
    ))
);

named!(pub instruction<CompleteStr, AssemblerInstruction>,
    do_parse!(
        ins: alt!(
            instruction_combined |
            directive
        ) >>
        (
            ins
        )
    )
);

#[cfg(test)]
mod test {
    use super::*;
    use super::super::Opcode;

    #[test]
    fn test_parse_instruction_form_one() {
        let result = instruction(CompleteStr("hlt\n"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                AssemblerInstruction {
                    opcode: Some(Token::Opcode { code: Opcode::HLT }),
                    ..Default::default()
                }
            ))
        );
    }

    #[test]
    fn test_parse_instruction_form_one_uppercase() {
        let result = instruction(CompleteStr("HLT\n"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                AssemblerInstruction {
                    opcode: Some(Token::Opcode { code: Opcode::HLT }),
                    ..Default::default()
                }
            ))
        );
    }

    #[test]
    fn test_parse_instruction_form_two() {
        let result = instruction(CompleteStr("load $0 #100\n"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                AssemblerInstruction {
                    opcode: Some(Token::Opcode { code: Opcode::LOAD }),
                    operand1: Some(Token::Register { reg_num: 0 }),
                    operand2: Some(Token::IntegerOperand { value: 100 }),
                    ..Default::default()
                }
            ))
        );
    }

    #[test]
    fn test_parse_instruction_form_three() {
        let result = instruction(CompleteStr("add $0 $1 $2\n"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                AssemblerInstruction {
                    opcode: Some(Token::Opcode { code: Opcode::ADD }),
                    operand1: Some(Token::Register { reg_num: 0 }),
                    operand2: Some(Token::Register { reg_num: 1 }),
                    operand3: Some(Token::Register { reg_num: 2 }),
                    ..Default::default()
                }
            ))
        );
    }
}