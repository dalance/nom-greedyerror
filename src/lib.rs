use nom::error::{ErrorKind, ParseError};
use nom_locate::LocatedSpanEx;

#[derive(Clone, Debug, PartialEq)]
pub struct GreedyError<I> {
    pub errors: Vec<(I, GreedyErrorKind)>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GreedyErrorKind {
    Context(&'static str),
    Char(char),
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
