pub mod grid;
pub mod vec2;

mod direction;
mod point;
mod rect;

use core::str::FromStr;

use lazy_errors::{prelude::*, Result};
use lazy_regex::regex::Regex;
use rayon::iter::ParallelIterator;

pub use direction::Direction;
pub use grid::Grid;
pub use point::Point;
pub use rect::Rect;
pub use vec2::Vec2;

use vec2::UVec2;

// TODO: Check TODOs in callers -- then delete this entirely.
pub fn parse_bounds(input: &str) -> Result<Rect> {
    let mut lens: Vec<usize> = input
        .lines()
        .map(|line| line.len())
        .collect();

    let y = lens.len();

    lens.dedup(); // Leaves good values after first bad one, but we don't care.

    let [x] = lens
        .try_into()
        .map_err(|v| err!("Line lengths differ: {v:?}"))?;

    let p = Point::new(0, 0);
    let v = UVec2::new(y, x);
    Rect::new(p, v)
}

/// Parallel variant of [`parse_each`] based on [`rayon::ParallelIterator`].
pub fn par_parse_each<T, E, S>(
    iter: impl ParallelIterator<Item = S>,
) -> impl ParallelIterator<Item = Result<T, Error>>
where
    T: FromStr<Err = E> + Send,
    E: Into<Stashable>,
    S: AsRef<str>,
{
    iter.map(|stringly| parse(stringly))
}

/// Calls [`parse`] on each element of the iterator.
pub fn parse_each<T, E, S>(
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
pub fn parse_each_ok<T, E, S, X>(
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

pub fn parse_next_ok<T, E, S, X>(
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

pub fn parse_substrs<'a, A, T, E, L, M, I>(
    lines: L,
    mut matcher: M,
) -> impl Iterator<Item = Result<(A, T)>> + use<'a, A, T, E, L, M, I>
where
    A: TryFrom<Rect> + TryFrom<Point>,
    <A as TryFrom<Rect>>::Error: Into<Stashable>,
    <A as TryFrom<Point>>::Error: Into<Stashable>,
    T: FromStr<Err = E>,
    E: Into<Stashable>,
    L: Iterator<Item = &'a str>,
    M: FnMut(&'a str) -> I,
    I: Iterator<Item = (usize, usize)>,
{
    lines
        .enumerate()
        .flat_map(move |(y, line)| {
            matcher(line).map(move |(x, dx)| parse_substr(y, x, dx, line))
        })
}

pub fn chars(input: &str) -> impl Iterator<Item = (usize, usize)> {
    let n = input.len();
    (0..n).map(|i| (i, 1))
}

// TODO: Use `Pattern` instead of `M` when feature `pattern` (#27721) is stable.
#[allow(dead_code)]
pub fn pattern_matches<'a, M, I>(
    input: &'a str,
    matcher: M,
) -> impl Iterator<Item = (usize, usize)> + 'a
where
    M: FnOnce(&'a str) -> I,
    I: Iterator<Item = (usize, &'a str)> + 'a,
{
    matcher(input).map(|(pos, substr)| (pos, substr.len()))
}

pub fn regex_matches<'a>(
    input: &'a str,
    regex: &'a Regex,
) -> impl Iterator<Item = (usize, usize)> + 'a {
    regex
        .find_iter(input)
        .map(move |m| (m.start(), m.len()))
}

#[allow(dead_code)]
pub fn regex_captures<'a>(
    input: &'a str,
    regex: &'a Regex,
) -> impl Iterator<Item = (usize, usize)> + 'a {
    regex
        .captures_iter(input)
        .filter_map(move |cap| {
            let m = cap.get(1)?;
            Some((m.start(), m.len()))
        })
}

pub fn parse_substr<A, T, E>(
    y: usize,
    x: usize,
    dx: usize,
    line: &str,
) -> Result<(A, T)>
where
    A: TryFrom<Rect> + TryFrom<Point>,
    <A as TryFrom<Rect>>::Error: Into<Stashable>,
    <A as TryFrom<Point>>::Error: Into<Stashable>,
    T: FromStr<Err = E>,
    E: Into<Stashable>,
{
    let x_end = x + dx;

    let msg = || format!("Failed to parse {x}..{x_end} in '{line}'");

    if x_end > line.len() {
        let e = Error::from_message("Substring is out of bounds");
        return Err(Error::wrap_with(e, msg()));
    }

    let parsed = parse(&line[x..x_end]).or_wrap_with(msg)?;

    let p = Point::from_unsigned(y, x)?;
    let a: Result<A> = match dx {
        0 => Err(err!("Substring is empty")),
        1 => A::try_from(p).or_wrap(),
        _ => {
            let v = UVec2::new(1, dx);
            let r = Rect::new(p, v);
            r.and_then(|r| A::try_from(r).or_wrap())
        }
    };

    let a = a
        .or_wrap_with::<Stashable>(|| "Failed to create area")
        .or_wrap_with(msg)?;

    Ok((a, parsed))
}

pub fn contains_2d(haystack: &str, needle: &str) -> bool {
    let haystack: Vec<&str> = haystack.lines().collect();
    let needle: Vec<&str> = needle.lines().collect();

    haystack
        .iter()
        .enumerate()
        .flat_map(|(y, line)| {
            line.match_indices(needle[0])
                .map(move |(x, _match)| (y, x))
        })
        .any(|(y, x)| {
            haystack
                .iter()
                .skip(y)
                .zip(&needle)
                .all(|(haystack, needle)| haystack[x..].starts_with(needle))
        })
}

fn parse<T, E, S>(text: S) -> Result<T, Error>
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
    use core::{fmt, iter, marker::PhantomData};

    use indoc::indoc;
    use itertools::Itertools;
    use lazy_regex::regex;
    use test_case::test_case;

    use super::*;

    #[test]
    fn parse_each() {
        let input: Vec<&str> = vec!["1", "1337"];

        let actual = super::parse_each(input.into_iter());

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

    #[test]
    fn parse_next_ok_when_next_ok() {
        let input: Vec<Result<&str>> = vec![Ok("42"), Err(err!("MOCK ERROR"))];
        let mut iter = input.into_iter();

        let actual: u8 = super::parse_next_ok(&mut iter).unwrap();

        assert_eq!(actual, 42);
    }

    #[test]
    fn parse_next_ok_when_next_err() {
        let input: Vec<Result<&str>> = vec![Err(err!("MOCK ERROR")), Ok("42")];
        let mut iter = input.into_iter();

        let actual: Result<u8> = super::parse_next_ok(&mut iter);

        let actual = actual.unwrap_err().to_string();
        assert_eq!(actual, "Failed to read input: MOCK ERROR");
    }

    #[test]
    fn parse_next_ok_when_next_ok_but_invalid() {
        let input: Vec<Result<&str>> = vec![Ok("1337"), Ok("42")];
        let mut iter = input.into_iter();

        let actual: Result<u8> = super::parse_next_ok(&mut iter);

        let actual = actual.unwrap_err().to_string();
        assert_eq!(
            actual,
            "Failed to parse input '1337': \
            number too large to fit in target type"
        );
    }

    #[test]
    fn parse_next_ok_when_empty() {
        let mut iter = iter::empty::<Result<&str, std::io::Error>>();
        let actual: Result<u8> = super::parse_next_ok(&mut iter);

        let actual = actual.unwrap_err().to_string();
        assert_eq!(actual, "Failed to read input: No data left");
    }

    #[test]
    fn parse_substrs() -> Result<()> {
        let input = indoc! {"\
                foo
                4
                42
                foo4bar2
            "};
        let pattern = |line| str::match_indices(line, &['4', '2']);
        let matcher = |line| super::pattern_matches(line, pattern);
        let parsed: Vec<(Point, u8)> =
            super::parse_substrs(input.lines(), matcher).try_collect()?;

        assert_eq!(parsed, vec![
            (Point::new(1, 0), 4),
            (Point::new(2, 0), 4),
            (Point::new(2, 1), 2),
            (Point::new(3, 3), 4),
            (Point::new(3, 7), 2),
        ]);

        Ok(())
    }

    #[test_case("0110", &['1'], &[(1,1), (2,1)])]
    #[test_case("0112", &['1', '2'], &[(1,1), (2,1), (3,1)])]
    fn pattern_matches_chars(
        line: &str,
        patterns: &[char],
        expected: &[(usize, usize)],
    ) -> Result<()> {
        // TODO: Refactor when feature `pattern` (#27721) is stable.
        let matches: Vec<_> =
            super::pattern_matches(line, |s| s.match_indices(patterns))
                .collect();

        assert_eq!(&matches, expected);
        Ok(())
    }

    // TODO: Deduplicate by adding a generic parameter
    // when feature `pattern` (#27721) is stable.
    #[test_case("011000", "11", &[(1,2)])]
    #[test_case("011100", "11", &[(1,2)])]
    #[test_case("011110", "11", &[(1,2), (3,2)])]
    #[test_case("011011", "11", &[(1,2), (4,2)])]
    fn pattern_matches_str(
        line: &str,
        pattern: &str,
        expected: &[(usize, usize)],
    ) -> Result<()> {
        // TODO: Refactor when feature `pattern` (#27721) is stable.
        let matches: Vec<_> =
            super::pattern_matches(line, |s| s.match_indices(pattern))
                .collect();

        assert_eq!(&matches, expected);
        Ok(())
    }

    #[test_case("0000", &[],      &[])]
    #[test_case("0001", &[(1,3)], &[(3,1)])]
    #[test_case("0010", &[(0,3)], &[(2,1)])]
    #[test_case("0011", &[(0,4)], &[(2,2)])]
    #[test_case("0100", &[],      &[])]
    fn regex_matches_vs_regex_captures(
        line: &str,
        expected_matches: &[(usize, usize)],
        expected_captures: &[(usize, usize)],
    ) -> Result<()> {
        let regex = regex!(r"00([1-9]+)");

        let matches: Vec<_> = super::regex_matches(line, regex).collect();
        assert_eq!(&matches, expected_matches);

        let captures: Vec<_> = super::regex_captures(line, regex).collect();
        assert_eq!(&captures, expected_captures);

        Ok(())
    }

    #[test_case(0, 0, 1, "42", Point::new(0, 0), 4)]
    #[test_case(0, 1, 1, "42", Point::new(0, 1), 2)]
    #[test_case(1337, 3, 1, "foo9bar", Point::new(1337, 3), 9)]
    #[test_case(
        0, 0, 1, "42",
        Rect::new(Point::new(0, 0), Vec2::new(1, 1))?,
        4)]
    #[test_case(
        0, 1, 1, "42",
        Rect::new(Point::new(0, 1), Vec2::new(1, 1))?,
        2)]
    #[test_case(
        0, 0, 2, "42",
        Rect::new(Point::new(0, 0), Vec2::new(1, 2))?,
        42
    )]
    #[test_case(
        1337, 3, 2, "foo42bar",
        Rect::new(Point::new(1337, 3), Vec2::new(1, 2))?,
        42
    )]
    fn parse_substr<A>(
        y: usize,
        x: usize,
        dx: usize,
        line: &str,
        expected_shape: A,
        expected_num: u8,
    ) -> Result<()>
    where
        A: PartialEq + fmt::Debug,
        A: TryFrom<Rect> + TryFrom<Point>,
        <A as TryFrom<Rect>>::Error: Into<Stashable>,
        <A as TryFrom<Point>>::Error: Into<Stashable>,
    {
        let (rect, num): (A, u8) = super::parse_substr(y, x, dx, line)?;

        assert_eq!(rect, expected_shape);
        assert_eq!(num, expected_num);

        Ok(())
    }

    #[test_case(0, 0, 0, "42", PhantomData::<Point>, "empty")]
    #[test_case(0, 2, 1, "42", PhantomData::<Point>, "out of bounds")]
    #[test_case(0, 0, 1, "-1", PhantomData::<Point>, "invalid digit")]
    #[test_case(0, 0, 0, "42", PhantomData::<Rect>, "empty")]
    #[test_case(0, 0, 3, "42", PhantomData::<Rect>, "out of bounds")]
    #[test_case(0, 1, 2, "42", PhantomData::<Rect>, "out of bounds")]
    #[test_case(0, 2, 1, "42", PhantomData::<Rect>, "out of bounds")]
    #[test_case(0, 0, 2, "-1", PhantomData::<Rect>, "invalid digit")]
    fn parse_substr_err<A>(
        y: usize,
        x: usize,
        dx: usize,
        line: &str,
        _: PhantomData<A>,
        expected_msg: &str,
    ) -> Result<()>
    where
        A: PartialEq + fmt::Debug,
        A: TryFrom<Rect> + TryFrom<Point>,
        <A as TryFrom<Rect>>::Error: Into<Stashable>,
        <A as TryFrom<Point>>::Error: Into<Stashable>,
    {
        let result: Result<(A, u8)> = super::parse_substr(y, x, dx, line);
        let err = result.unwrap_err();
        let msg = err.to_string();
        dbg!(&msg);
        assert!(msg.contains(expected_msg));
        Ok(())
    }
}
