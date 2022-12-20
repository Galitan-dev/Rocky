use nom::{
    character::complete::multispace1, combinator::opt, error::ParseError, sequence::delimited,
    AsChar, IResult, InputTakeAtPosition, Parser,
};

pub fn ws<I, O, E: ParseError<I>, P>(parser: P) -> impl FnMut(I) -> IResult<I, O, E>
where
    P: Parser<I, O, E>,
    I: Clone + InputTakeAtPosition,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
{
    delimited(opt(multispace1), parser, opt(multispace1))
}
