use nom7::branch::alt;
use nom7::character::complete::{alpha1, digit1};
use nom7::error::{ErrorKind, ParseError, VerboseError};
use nom7::sequence::tuple;
use nom7::Err::Error;
use nom7::IResult;
use nom_greedyerror::{error_position, GreedyError, Position};
use nom_locate4::LocatedSpan;

type Span<'a> = LocatedSpan<&'a str>;

fn parser<'a, E: ParseError<Span<'a>>>(
    input: Span<'a>,
) -> IResult<Span<'a>, (Span<'a>, Span<'a>, Span<'a>), E> {
    alt((
        tuple((alpha1, digit1, alpha1)),
        tuple((digit1, alpha1, digit1)),
    ))(input)
}

fn main() {
    // VerboseError failed at
    //   abc012:::
    //   ^
    let error = parser::<VerboseError<Span>>(Span::new("abc012:::"));
    dbg!(&error);
    match error {
        Err(Error(e)) => assert_eq!(e.errors.first().map(|x| x.0.position()), Some(0)),
        _ => (),
    };

    // GreedyError failed at
    //   abc012:::
    //         ^
    let error = parser::<GreedyError<Span, ErrorKind>>(Span::new("abc012:::"));
    dbg!(&error);
    match error {
        Err(Error(e)) => assert_eq!(error_position(&e), Some(6)),
        _ => (),
    };
}
