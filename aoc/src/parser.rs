use std::str::FromStr;

use itertools::Itertools;
use lazy_errors::{prelude::*, Result};

/// Calls [`parse_all_ok`] and collects the result into a [`Vec`].
///
/// Note: As of 2024-04-11, you often have to provide the `S` type parameter
/// explicitly, otherwise Rust seems to assume that `T == S`.
#[allow(dead_code)] // TODO
pub fn try_parse_all<T, E, S, X>(
    iter: impl Iterator<Item = Result<S, X>>,
) -> Result<Vec<T>, Error>
where
    T: FromStr<Err = E>,
    E: Into<Stashable>,
    S: AsRef<str>,
    X: Into<Stashable>,
{
    parse_all_ok::<T, E, S, X>(iter).try_collect()
}

pub fn try_parse_next<T, E, S, X>(
    iter: &mut impl Iterator<Item = Result<S, X>>,
) -> Result<T, Error>
where
    T: FromStr<Err = E>,
    E: Into<Stashable>,
    S: AsRef<str>,
    X: Into<Stashable>,
{
    let next = match iter.next() {
        Some(Ok(next)) => Ok(next),
        Some(Err(e)) => Err(Error::wrap(e)),
        None => Err(err!("No data left")),
    };

    let next = next.or_wrap_with(|| "Failed to read input")?;
    parse(&next)
}

/// Calls [`parse`] on each element of the iterator.
pub fn parse_all<T, E, S>(
    iter: impl Iterator<Item = S>,
) -> impl Iterator<Item = Result<T, Error>>
where
    T: FromStr<Err = E>,
    E: Into<Stashable>,
    S: AsRef<str>,
{
    iter.map(|stringly| parse(stringly))
}

/// Calls [`parse`] on each `Ok` element of the iterator
/// and converts `Err` elements from [`std::io::Error`] to `E`.
///
/// Note: As of 2024-04-11, you often have to provide the `S` type parameter
/// explicitly, otherwise Rust seems to assume that `T == S`.
pub fn parse_all_ok<T, E, S, X>(
    iter: impl Iterator<Item = Result<S, X>>,
) -> impl Iterator<Item = Result<T, Error>>
where
    T: FromStr<Err = E>,
    E: Into<Stashable>,
    S: AsRef<str>,
    X: Into<Stashable>,
{
    iter.map(|r| {
        let stringly = r.or_wrap_with(|| "Failed to read input")?;
        parse(stringly)
    })
}

pub fn parse<T, E, S>(text: S) -> Result<T, Error>
where
    T: FromStr<Err = E>,
    E: Into<Stashable>,
    S: AsRef<str>,
{
    let text = text.as_ref();
    text.parse::<T>()
        .or_wrap_with(|| format!("Failed to parse input '{text}'"))
}

#[cfg(test)]
mod tests {
    use std::iter;

    use super::*;

    #[test]
    fn try_parse_all_when_all_ok() {
        let input: Vec<Result<&str>> = vec![Ok("1"), Ok("2"), Ok("42")];

        let actual: Vec<u8> = try_parse_all(input.into_iter()).unwrap();

        assert_eq!(actual, vec![1, 2, 42]);
    }

    #[test]
    fn try_parse_all_when_some_err() {
        let input: Vec<Result<&str>> = vec![Ok("1"), Err(err!("MOCK ERROR"))];

        let actual: Result<Vec<u8>> = try_parse_all(input.into_iter());

        let actual = actual.unwrap_err().to_string();
        assert_eq!(actual, "Failed to read input: MOCK ERROR");
    }

    #[test]
    fn try_parse_all_when_some_ok_but_invalid() {
        let input: Vec<Result<&str>> = vec![Ok("1"), Ok("1337")];

        let actual: Result<Vec<u8>> = try_parse_all(input.into_iter());

        let actual = actual.unwrap_err().to_string();
        assert_eq!(
            actual,
            "Failed to parse input '1337': \
            number too large to fit in target type"
        );
    }

    #[test]
    fn try_parse_next_when_next_ok() {
        let input: Vec<Result<&str>> = vec![Ok("42"), Err(err!("MOCK ERROR"))];
        let mut iter = input.into_iter();

        let actual: u8 = try_parse_next(&mut iter).unwrap();

        assert_eq!(actual, 42);
    }

    #[test]
    fn try_parse_next_when_next_err() {
        let input: Vec<Result<&str>> = vec![Err(err!("MOCK ERROR")), Ok("42")];
        let mut iter = input.into_iter();

        let actual: Result<u8> = try_parse_next(&mut iter);

        let actual = actual.unwrap_err().to_string();
        assert_eq!(actual, "Failed to read input: MOCK ERROR");
    }

    #[test]
    fn try_parse_next_when_next_ok_but_invalid() {
        let input: Vec<Result<&str>> = vec![Ok("1337"), Ok("42")];
        let mut iter = input.into_iter();

        let actual: Result<u8> = try_parse_next(&mut iter);

        let actual = actual.unwrap_err().to_string();
        assert_eq!(
            actual,
            "Failed to parse input '1337': \
            number too large to fit in target type"
        );
    }

    #[test]
    fn try_parse_next_when_empty() {
        let mut iter = iter::empty::<Result<&str, std::io::Error>>();
        let actual: Result<u8> = try_parse_next(&mut iter);

        let actual = actual.unwrap_err().to_string();
        assert_eq!(actual, "Failed to read input: No data left");
    }

    #[test]
    fn parse_all() {
        let input: Vec<&str> = vec!["1", "1337"];

        let actual = super::parse_all(input.into_iter());

        let actual: Vec<Result<u8>> = actual.collect();
        let actual: Vec<Result<u8, String>> = actual
            .into_iter()
            .map(|r| r.map_err(|e| e.to_string()))
            .collect();

        assert_eq!(actual, vec![
            Ok(1),
            Err(String::from(
                "Failed to parse input '1337': \
                number too large to fit in target type"
            ))
        ]);
    }
}
