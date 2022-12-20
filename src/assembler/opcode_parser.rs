use super::Token;
use crate::instruction::Opcode;
use nom::{character::complete::alpha1, combinator::map, error::VerboseError, IResult};

pub fn opcode<'a>(i: &'a str) -> IResult<&'a str, Token, VerboseError<&'a str>> {
    map(alpha1, |code: &str| Token::Opcode {
        code: Opcode::from(code.to_lowercase()),
    })(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode() {
        let result = opcode("load");
        assert_eq!(result.is_ok(), true);
        let (rest, token) = result.unwrap();
        assert_eq!(token, Token::Opcode { code: Opcode::LOAD });
        assert_eq!(rest, "");
        let result = opcode("aold");
        let (_, token) = result.unwrap();
        assert_eq!(token, Token::Opcode { code: Opcode::IGL });
    }
}
