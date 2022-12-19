use nom::{types::CompleteStr, digit};
use super::{Token, register::register, label::label_usage};

named!(pub operand<CompleteStr, Token>,
    alt!(
        integer_operand |
        register |
        label_usage |
        rkstring
    )
);

named!(integer_operand<CompleteStr, Token>,
    ws!(
        do_parse!(
            tag!("#") >>
            reg_num: digit >>
            (
                Token::IntegerOperand {value: reg_num.parse::<i32>().unwrap()}
            )
        )
    )
);

named!(rkstring<CompleteStr, Token>,
    alt!(
        rkstring_single_quote |
        rkstring_double_quote
    )
);

named!(rkstring_single_quote<CompleteStr, Token>,
    do_parse!(
        alt!(tag!("'")) >>
        content: take_until!("'") >>
        tag!("'") >>
        (
            Token::RkString{ name: content.to_string() }
        )
    )
);

named!(rkstring_double_quote<CompleteStr, Token>,
    do_parse!(
        alt!(tag!("\"")) >>
        content: take_until!("\"") >>
        tag!("\"") >>
        (
            Token::RkString{ name: content.to_string() }
        )
    )
);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_integer_operand() {
        let result = integer_operand(CompleteStr("#10"));
        assert!(result.is_ok());
        let (rest, value) = result.unwrap();
        assert_eq!(rest, CompleteStr(""));
        assert_eq!(value, Token::IntegerOperand{value: 10});

        let result = integer_operand(CompleteStr("10"));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_string_operand() {
        assert!(rkstring(CompleteStr("'This is a test'")).is_ok());
        assert!(rkstring(CompleteStr("'This is a \"test\"'")).is_ok());
        assert!(rkstring(CompleteStr("\"This is a test\"")).is_ok());
        assert!(rkstring(CompleteStr("\"This is a 'test'\"")).is_ok());
        assert!(rkstring(CompleteStr("\"This is a test'")).is_err());
        assert!(rkstring(CompleteStr("'This is a test\"")).is_err());
    }

}