use nom::{
    bytes::complete::tag,
    character::complete::{alphanumeric1, multispace1},
    combinator::{map, opt},
    error::VerboseError,
    sequence::{delimited, terminated, tuple},
    IResult,
};

use super::{utils::ws, Token};

pub fn label_declaration<'a>(i: &'a str) -> IResult<&'a str, Token, VerboseError<&'a str>> {
    ws(map(
        terminated(alphanumeric1, tuple((tag(":"), opt(multispace1)))),
        |name: &str| Token::LabelDeclaration {
            name: name.to_string(),
        },
    ))(i)
}

pub fn label_usage<'a>(i: &'a str) -> IResult<&'a str, Token, VerboseError<&'a str>> {
    ws(map(
        delimited(tag("@"), alphanumeric1, opt(multispace1)),
        |name: &str| Token::LabelUsage {
            name: name.to_string(),
        },
    ))(i)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_label_declaration() {
        let result = label_declaration("test:");
        println!("{result:?}");
        assert_eq!(result.is_ok(), true);
        let (_, token) = result.unwrap();
        assert_eq!(
            token,
            Token::LabelDeclaration {
                name: "test".to_string()
            }
        );
        let result = label_declaration("test");
        assert_eq!(result.is_ok(), false);
    }

    #[test]
    fn test_parse_label_usage() {
        let result = label_usage("@test");
        assert_eq!(result.is_ok(), true);
        let (_, token) = result.unwrap();
        assert_eq!(
            token,
            Token::LabelUsage {
                name: "test".to_string()
            }
        );
        let result = label_usage("test");
        assert_eq!(result.is_ok(), false);
    }
}
