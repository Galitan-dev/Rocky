use super::{label_parser::label_usage, register_parser::register, utils::ws, Token};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until1},
    character::complete::digit1,
    combinator::map,
    error::VerboseError,
    sequence::{delimited, preceded},
    IResult,
};

pub fn operand<'a>(i: &'a str) -> IResult<&'a str, Token, VerboseError<&'a str>> {
    ws(alt((integer_operand, rkstring, register, label_usage)))(i)
}

fn integer_operand<'a>(i: &'a str) -> IResult<&'a str, Token, VerboseError<&'a str>> {
    map(preceded(tag("#"), digit1), |reg_num: &str| {
        Token::IntegerOperand {
            value: reg_num.parse::<i32>().unwrap(),
        }
    })(i)
}

fn rkstring<'a>(i: &'a str) -> IResult<&'a str, Token, VerboseError<&'a str>> {
    map(
        alt((
            delimited(tag("'"), take_until1("'"), tag("'")),
            delimited(tag("\""), take_until1("\""), tag("\"")),
        )),
        |name: &str| Token::RkString {
            name: name.to_string(),
        },
    )(i)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_integer_operand() {
        let result = integer_operand("#10");
        assert!(result.is_ok());
        let (rest, value) = result.unwrap();
        assert_eq!(rest, "");
        assert_eq!(value, Token::IntegerOperand { value: 10 });

        let result = integer_operand("10");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_string_operand() {
        assert!(rkstring("'This is a test'").is_ok());
        assert!(rkstring("'This is a \"test\"'").is_ok());
        assert!(rkstring("\"This is a test\"").is_ok());
        assert!(rkstring("\"This is a 'test'\"").is_ok());
        assert!(rkstring("\"This is a test'").is_err());
        assert!(rkstring("'This is a test\"").is_err());
    }

    #[test]
    fn test_parse_alt_operand() {
        assert!(operand("'This is a test' ").is_ok());
        assert!(operand(" $1").is_ok());
        assert!(operand("#1 ").is_ok());
    }
}
