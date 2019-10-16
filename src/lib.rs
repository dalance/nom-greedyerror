use nom::error::{ErrorKind, ParseError};
use nom_locate::LocatedSpanEx;

/// This error type accumulates errors and their position when backtracking
/// through a parse tree. This take a deepest error at `alt` combinator.
#[derive(Clone, Debug, PartialEq)]
pub struct GreedyError<I> {
    /// list of errors accumulated by `GreedyError`, containing the affected
    /// part of input data, and some context
    pub errors: Vec<(I, GreedyErrorKind)>,
}

#[derive(Clone, Debug, PartialEq)]
/// error context for `GreedyError`
pub enum GreedyErrorKind {
    /// static string added by the `context` function
    Context(&'static str),
    /// indicates which character was expected by the `char` function
    Char(char),
    /// error kind given by various nom parsers
    Nom(ErrorKind),
}

impl<I> ParseError<I> for GreedyError<I>
where
    I: Position,
{
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        GreedyError {
            errors: vec![(input, GreedyErrorKind::Nom(kind))],
        }
    }

    fn append(input: I, kind: ErrorKind, mut other: Self) -> Self {
        other.errors.push((input, GreedyErrorKind::Nom(kind)));
        other
    }

    fn from_char(input: I, c: char) -> Self {
        GreedyError {
            errors: vec![(input, GreedyErrorKind::Char(c))],
        }
    }

    fn add_context(input: I, ctx: &'static str, mut other: Self) -> Self {
        other.errors.push((input, GreedyErrorKind::Context(ctx)));
        other
    }

    fn or(self, other: Self) -> Self {
        let pos_self = if let Some(x) = self.errors.first() {
            x.0.position()
        } else {
            0
        };
        let pos_other = if let Some(x) = other.errors.first() {
            x.0.position()
        } else {
            0
        };
        if pos_other > pos_self {
            other
        } else {
            self
        }
    }
}

pub fn error_position<T: Position>(e: &GreedyError<T>) -> Option<usize> {
    e.errors.first().map(|x| x.0.position())
}

pub trait Position {
    fn position(&self) -> usize;
}

impl<T, U> Position for LocatedSpanEx<T, U> {
    fn position(&self) -> usize {
        self.offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::branch::alt;
    use nom::character::complete::{alpha1, digit1};
    use nom::error::{ParseError, VerboseError};
    use nom::sequence::tuple;
    use nom::IResult;
    use nom_locate::LocatedSpan;

    type Span<'a> = LocatedSpan<&'a str>;

    fn parser<'a, E: ParseError<Span<'a>>>(
        input: Span<'a>,
    ) -> IResult<Span<'a>, (Span<'a>, Span<'a>, Span<'a>), E> {
        alt((
            tuple((alpha1, digit1, alpha1)),
            tuple((digit1, alpha1, digit1)),
        ))(input)
    }

    #[test]
    fn test() {
        // VerboseError failed at
        //   abc012:::
        //   ^
        let error = parser::<VerboseError<Span>>(Span::new("abc012:::"));
        match error {
            Err(nom::Err::Error(e)) => {
                assert_eq!(e.errors.first().map(|x| x.0.position()), Some(0))
            }
            _ => (),
        };

        // GreedyError failed at
        //   abc012:::
        //         ^
        let error = parser::<GreedyError<Span>>(Span::new("abc012:::"));
        match error {
            Err(nom::Err::Error(e)) => assert_eq!(error_position(&e), Some(6)),
            _ => (),
        };
    }
}
