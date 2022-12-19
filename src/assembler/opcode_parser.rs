use nom::{types::CompleteStr, alpha1};
use crate::instruction::Opcode;
use super::Token;

named!(pub opcode<CompleteStr, Token>,
    do_parse!(
        opcode: alpha1 >> 
        (
            Token::Opcode {code: Opcode::from(CompleteStr(&opcode.to_lowercase()))}
        )
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode() {
        let result = opcode(CompleteStr("load"));
        assert_eq!(result.is_ok(), true);
        let (rest, token) = result.unwrap();
        assert_eq!(token, Token::Opcode { code: Opcode::LOAD });
        assert_eq!(rest, CompleteStr(""));
        let result = opcode(CompleteStr("aold"));
        let (_, token) = result.unwrap();
        assert_eq!(token, Token::Opcode { code: Opcode::IGL });
}
}