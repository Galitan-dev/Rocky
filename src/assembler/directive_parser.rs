use super::{
    instruction_parser::AssemblerInstruction, label_parser::label_declaration,
    operand_parser::operand, utils::ws, Token,
};
use nom::{
    bytes::complete::tag,
    character::complete::alpha1,
    combinator::{map, opt},
    error::VerboseError,
    sequence::preceded,
    sequence::tuple,
    IResult,
};

fn directive_declaration<'a>(i: &'a str) -> IResult<&'a str, Token, VerboseError<&'a str>> {
    map(preceded(tag("."), alpha1), |name: &str| Token::Directive {
        name: name.to_string(),
    })(i)
}

fn directive_combined<'a>(
    i: &'a str,
) -> IResult<&'a str, AssemblerInstruction, VerboseError<&'a str>> {
    ws(map(
        tuple((
            opt(label_declaration),
            directive_declaration,
            opt(operand),
            opt(operand),
            opt(operand),
        )),
        |(l, name, o1, o2, o3)| AssemblerInstruction {
            label: l,
            directive: Some(name),
            operand1: o1,
            operand2: o2,
            operand3: o3,
            ..Default::default()
        },
    ))(i)
}

pub fn directive<'a>(i: &'a str) -> IResult<&'a str, AssemblerInstruction, VerboseError<&'a str>> {
    directive_combined(i)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_directive() {
        let result = directive(".test");
        assert_eq!(result.is_ok(), true);
        let (rest, token) = result.unwrap();
        assert_eq!(
            token,
            AssemblerInstruction {
                directive: Some(Token::Directive {
                    name: "test".to_string()
                }),
                ..Default::default()
            }
        );
        assert_eq!(rest, "");
    }

    #[test]
    fn test_string_directive() {
        let result = directive_combined("test: .asciiz 'Hello'");
        assert_eq!(result.is_ok(), true);
        let (_, directive) = result.unwrap();

        // Yes, this is the what the result should be
        let correct_instruction = AssemblerInstruction {
            label: Some(Token::LabelDeclaration {
                name: "test".to_string(),
            }),
            directive: Some(Token::Directive {
                name: "asciiz".to_string(),
            }),
            operand1: Some(Token::RkString {
                name: "Hello".to_string(),
            }),
            ..Default::default()
        };

        assert_eq!(directive, correct_instruction);
    }
}
