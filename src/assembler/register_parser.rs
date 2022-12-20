use super::Token;
use nom::{
    bytes::complete::tag, character::complete::digit1, combinator::map, error::VerboseError,
    sequence::preceded, IResult,
};

pub fn register<'a>(i: &'a str) -> IResult<&'a str, Token, VerboseError<&'a str>> {
    map(preceded(tag("$"), digit1), |reg_num: &str| {
        Token::Register {
            reg_num: reg_num.parse::<u8>().unwrap(),
        }
    })(i)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_register() {
        assert!(register("$0").is_ok());
        assert!(register("$10").is_ok());
        assert!(register("0").is_err());
        assert!(register("$a").is_err());
        assert!(register("$").is_err());
        assert!(register("0$").is_err());
    }
}
