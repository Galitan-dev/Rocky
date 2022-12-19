use nom::{alpha1, types::CompleteStr};

use super::{Token, instruction::AssemblerInstruction, operand::operand, label::label_declaration};

named!(directive_declaration<CompleteStr, Token>,
    do_parse!(
        tag!(".") >>
        name: alpha1 >>
        (
          Token::Directive{name: name.to_string()}
        )
    )
);
  
named!(directive_combined<CompleteStr, AssemblerInstruction>,
    ws!(
        do_parse!(
            l: opt!(label_declaration) >>
            name: directive_declaration >>
            o1: opt!(operand) >>
            o2: opt!(operand) >>
            o3: opt!(operand) >>
            (
                AssemblerInstruction {
                    directive: Some(name),
                    label: l,
                    operand1: o1,
                    operand2: o2,
                    operand3: o3,
                    ..Default::default()
                }
            )
        )
    )
);

named!(pub directive<CompleteStr, AssemblerInstruction>,
    do_parse!(
        ins: alt!(
            directive_combined
        ) >>
        (
            ins
        )
    )
);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_directive() {
        let result = directive(CompleteStr(".test"));
        assert_eq!(result.is_ok(), true);
        let (rest, token) = result.unwrap();
        assert_eq!(token, AssemblerInstruction { directive: Some(Token::Directive { name: "test".to_string() }), ..Default::default() });
        assert_eq!(rest, CompleteStr(""));
    }

    #[test]
fn test_string_directive() {
    let result = directive_combined(CompleteStr("test: .asciiz 'Hello'"));
    assert_eq!(result.is_ok(), true);
    let (_, directive) = result.unwrap();

    // Yes, this is the what the result should be
    let correct_instruction =
        AssemblerInstruction {
            label: Some(Token::LabelDeclaration {
                name: "test".to_string()
            }),
            directive: Some(Token::Directive {
                name: "asciiz".to_string()
            }),
            operand1: Some(Token::RkString { 
                name: "Hello".to_string() 
            }),
            ..Default::default()
        };

    assert_eq!(directive, correct_instruction);
}

}
