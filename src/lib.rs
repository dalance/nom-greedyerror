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

pub trait AsStr {
    fn as_str(&self) -> &str;
}

impl<'a> AsStr for &'a str {
    #[inline(always)]
    fn as_str(&self) -> &str {
        self
    }
}

impl AsStr for str {
    #[inline(always)]
    fn as_str(&self) -> &str {
        self
    }
}

impl<T: AsStr, X> AsStr for LocatedSpanEx<T, X> {
    #[inline]
    fn as_str(&self) -> &str {
        self.fragment.as_str()
    }
}

/// transforms a `GreedyError` into a trace with input position information
pub fn convert_error<T: AsStr, U: AsStr>(input: T, e: GreedyError<U>) -> String {
    use nom::Offset;
    use std::iter::repeat;

    let lines: Vec<_> = input.as_str().lines().map(String::from).collect();

    let mut result = String::new();

    for (i, (substring, kind)) in e.errors.iter().enumerate() {
        let mut offset = input.as_str().offset(substring.as_str());

        if lines.is_empty() {
            match kind {
                GreedyErrorKind::Char(c) => {
                    result += &format!("{}: expected '{}', got empty input\n\n", i, c);
                }
                GreedyErrorKind::Context(s) => {
                    result += &format!("{}: in {}, got empty input\n\n", i, s);
                }
                GreedyErrorKind::Nom(e) => {
                    result += &format!("{}: in {:?}, got empty input\n\n", i, e);
                }
            }
        } else {
            let mut line = 0;
            let mut column = 0;

            for (j, l) in lines.iter().enumerate() {
                if offset <= l.len() {
                    line = j;
                    column = offset;
                    break;
                } else {
                    offset = offset - l.len() - 1;
                }
            }

            match kind {
                GreedyErrorKind::Char(c) => {
                    result += &format!("{}: at line {}:\n", i, line);
                    result += &lines[line];
                    result += "\n";

                    if column > 0 {
                        result += &repeat(' ').take(column).collect::<String>();
                    }
                    result += "^\n";
                    result += &format!(
                        "expected '{}', found {}\n\n",
                        c,
                        substring.as_str().chars().next().unwrap()
                    );
                }
                GreedyErrorKind::Context(s) => {
                    result += &format!("{}: at line {}, in {}:\n", i, line, s);
                    result += &lines[line];
                    result += "\n";
                    if column > 0 {
                        result += &repeat(' ').take(column).collect::<String>();
                    }
                    result += "^\n\n";
                }
                GreedyErrorKind::Nom(e) => {
                    result += &format!("{}: at line {}, in {:?}:\n", i, line, e);
                    result += &lines[line];
                    result += "\n";
                    if column > 0 {
                        result += &repeat(' ').take(column).collect::<String>();
                    }
                    result += "^\n\n";
                }
            }
        }
    }

    result
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
    fn test_position() {
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

    #[test]
    fn test_convert_error() {
        let error = parser::<GreedyError<Span>>(Span::new("abc012:::"));
        let msg = r##"0: at line 0, in Alpha:
abc012:::
      ^

1: at line 0, in Alt:
abc012:::
^

"##;
        match error {
            Err(nom::Err::Error(e)) => assert_eq!(convert_error("abc012:::", e), String::from(msg)),
            _ => (),
        };
    }
}
