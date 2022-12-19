use nom::{types::CompleteStr, digit};
use super::Token;

named!(pub register <CompleteStr, Token>,
    ws!(
        do_parse!(
            tag!("$") >>
            reg_num: digit >>
            ( 
                Token::Register{
                  reg_num: reg_num.parse::<u8>().unwrap()
                } 
            ) 
        )
    )
);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_register() {
        assert!(register(CompleteStr("$0")).is_ok());
        assert!(register(CompleteStr("$10")).is_ok());
        assert!(register(CompleteStr("0")).is_err());
        assert!(register(CompleteStr("$a")).is_err());
        assert!(register(CompleteStr("$")).is_err());
        assert!(register(CompleteStr("0$")).is_err());
    }

}