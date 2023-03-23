# nom-greedyerror
Custom error type of [nom](https://github.com/Geal/nom) to improve accuracy of error position.

[![Actions Status](https://github.com/dalance/nom-greedyerror/workflows/Rust/badge.svg)](https://github.com/dalance/nom-greedyerror/actions)
[![Crates.io](https://img.shields.io/crates/v/nom-greedyerror.svg)](https://crates.io/crates/nom-greedyerror)
[![Docs.rs](https://docs.rs/nom-greedyerror/badge.svg)](https://docs.rs/nom-greedyerror)

The default error types of nom ( `(I, ErrorKind)` and `VerboseError` ) take a last challenged error at `alt` combinator.
Alternatively `GreedyError` of nom-greedyerror take a deepest error.

For example, the following parser accepts string like `abc012abc` or `012abc012`.

```rust
alt((
    tuple((alpha1, digit1, alpha1)),
    tuple((digit1, alpha1, digit1)),
))(input)
```

If `abc012:::` is provided, we expect that the parse error happens at:

```
abc012:::
      ^
```

But `VerboseError` reports the parse error at:

```
abc012:::
^
```

This is because the last challenged parser is `tuple((digit1, alpha1, digit1))` and it is failed.
`GreedyError` reports the parse error at the expected position.

## Requirement

nom must be 5.0.0 or later.

## Usage

```Cargo.toml
[dependencies]
nom-greedyerror = "0.5.0"
```

## Example

```rust
use nom::branch::alt;
use nom::character::complete::{alpha1, digit1};
use nom::error::{ErrorKind, ParseError, VerboseError};
use nom::sequence::tuple;
use nom::Err::Error;
use nom::IResult;
use nom_greedyerror::{error_position, GreedyError, Position};
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
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
