use std::str::FromStr;

use lazy_errors::{prelude::*, Result};

use crate::ident::{Day, Id, Part, Year};

/// Wraps a [`FilterTerm`] slice and matches [`Year`]/[`Day`]/[`Part`]
/// (or combinations thereof) if any [`FilterTerm`] matches.
///
/// Note that it is perfectly fine for values of this type
/// to identify a puzzle that does not exist or that does not exist yet,
/// such as the puzzle that will be released tomorrow.
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Filter {
    partial_ids: Box<[FilterTerm]>,
}

/// A “partial ID” that identifies puzzles by year and/or day and/or part.
///
/// This type allows users to specify a single puzzle or a “range” of puzzles.
/// Missing fields are treated as wildcards.
///
/// Note that it is perfectly fine for values of this type
/// to identify a puzzle that does not exist or that does not exist yet,
/// such as the puzzle that will be released tomorrow.
///
/// Note: This type implements `Copy`.
#[derive(Debug, Copy, Clone, Default, PartialEq, Hash, Eq)]
pub struct FilterTerm {
    year: Option<Year>,
    day:  Option<Day>,
    part: Option<Part>,
}

impl Default for Filter {
    fn default() -> Self {
        let wildcard = FilterTerm::default();
        Filter {
            partial_ids: Box::new([wildcard]),
        }
    }
}

impl From<Vec<FilterTerm>> for Filter {
    fn from(partial_ids: Vec<FilterTerm>) -> Self {
        if partial_ids.is_empty() {
            return Self::default();
        }

        Self {
            partial_ids: partial_ids.into_boxed_slice(),
        }
    }
}

impl FromStr for FilterTerm {
    type Err = Error;

    /// Creates a [`FilterTerm`] from a [`&str`](&str)
    /// of the format `yYYdDDpP`, where
    /// `YY` are the two last digits of a [`Year`],
    /// `DD` is the the zero-padded number of the [`Day`], and
    /// `P` is the part of the puzzle (either `1` or `2`).
    /// Any of year, day, or part may be missing.
    /// Missing components will be treated as wildcards.
    ///
    /// ```
    /// # use std::str::FromStr;
    /// # use aoc::ident::FilterTerm;
    ///
    /// assert!(FilterTerm::from_str("y21d02p2").is_ok());
    /// assert!(FilterTerm::from_str("y21d02").is_ok());
    /// assert!(FilterTerm::from_str("d02").is_ok());
    /// ```
    ///
    /// Please note that we reject the empty string.
    /// This is a deliberate design decision because
    /// [`FilterTerm`] may implement [`std::fmt::Display`] later.
    /// An all-wildcard filter could be printed as `*`.
    /// Thus, please parse `"*"` or call [`FilterTerm::default`]
    /// to get an all-wildcard filter.
    ///
    /// ```
    /// # use aoc::ident::FilterTerm;
    ///
    /// let a = "".parse::<FilterTerm>(); // Returns `Err`
    /// let b = "*".parse::<FilterTerm>().unwrap();
    /// let c = FilterTerm::new(None, None, None);
    /// let d = FilterTerm::default();
    /// assert!(a.is_err());
    /// assert_eq!(b, c);
    /// assert_eq!(c, d);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use lazy_regex::regex_captures;

        if s.is_empty() {
            return Err(err!("Input is empty (please use '*' as a wildcard)"));
        }

        if s == "*" {
            return Ok(FilterTerm::default());
        }

        let Some((_, y, d, p)) =
            regex_captures!(r"^(y\d{2})?(d\d{2})?(p\d{1})?$", s)
        else {
            return Err(err!("Input '{s}' does not match pattern yYYdDDpP"));
        };

        let y = parse_as_optional_id(y)?;
        let d = parse_as_optional_id(d)?;
        let p = parse_as_optional_id(p)?;

        Ok(FilterTerm::new(y, d, p))
    }
}

impl Filter {
    pub fn matches_year(&self, y: Year) -> bool {
        self.partial_ids
            .iter()
            .any(|s| s.matches_year(y))
    }

    pub fn matches_year_day(&self, y: Year, d: Day) -> bool {
        self.partial_ids
            .iter()
            .any(|s| s.matches_year_day(y, d))
    }

    pub fn matches_year_day_part(&self, y: Year, d: Day, p: Part) -> bool {
        self.partial_ids
            .iter()
            .any(|s| s.matches_year_day_part(y, d, p))
    }
}

impl FilterTerm {
    pub fn new<Y, D, P>(year: Y, day: D, part: P) -> Self
    where
        Y: Into<Option<Year>>,
        D: Into<Option<Day>>,
        P: Into<Option<Part>>,
    {
        FilterTerm {
            year: year.into(),
            day:  day.into(),
            part: part.into(),
        }
    }

    pub fn matches_year(&self, y: Year) -> bool {
        matches(&self.year, &y)
    }

    pub fn matches_year_day(&self, y: Year, d: Day) -> bool {
        self.matches_year(y) && matches(&self.day, &d)
    }

    pub fn matches_year_day_part(&self, y: Year, d: Day, p: Part) -> bool {
        self.matches_year_day(y, d) && matches(&self.part, &p)
    }
}

fn parse_as_optional_id<T>(s: &str) -> Result<Option<T>>
where
    Id<T>: FromStr<Err = Error>,
{
    if s.is_empty() {
        return Ok(None);
    }

    let Id::<T>(inner) = s.parse()?;
    Ok(Some(inner))
}

fn matches<T: Eq>(a: &Option<T>, b: &T) -> bool {
    match a.as_ref() {
        Some(a) => a == b,
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case("y21d02p1", 2021, 2, 1)]
    #[test_case("y21d02p2", 2021, 2, 2)]
    #[test_case("y21d02", 2021, 2, None)]
    #[test_case("y21", 2021, None, None)]
    #[test_case("p1", None, None, 1)]
    #[test_case("p2", None, None, 2)]
    #[test_case("d01", None, 1, None)]
    #[test_case("d01p1", None, 1, 1)]
    #[test_case("y21p1", 2021, None, 1)]
    fn conversions_ok<Y, D, P>(
        text: &str,
        year: Y,
        day: D,
        part: P,
    ) -> Result<()>
    where
        Y: Into<Option<u16>>,
        D: Into<Option<u8>>,
        P: Into<Option<u8>>,
    {
        let expected = from(year, day, part);
        assert_eq!(expected, text.parse()?);
        Ok(())
    }

    #[test_case("d00", "Day 0 is out of range")]
    #[test_case("d26", "Day 26 is out of range")]
    #[test_case("p0", "Puzzle part 0 is out of range")]
    #[test_case("p3", "Puzzle part 3 is out of range")]
    #[test_case("y19", "Year 2019 is out of range")]
    #[test_case("y25", "Year 2025 is out of range")]
    #[test_case("y25d26p1", "Year 2025 is out of range")]
    #[test_case("", "Input is empty (please use '*' as a wildcard")]
    #[test_case("yyydddpp", "Input 'yyydddpp' does not match pattern")]
    #[test_case(" y21", "Input ' y21' does not match pattern")]
    #[test_case("p1 ", "Input 'p1 ' does not match pattern")]
    #[test_case("p1d01y21", "Input 'p1d01y21' does not match pattern")]
    #[test_case(
        "y21d01p1y21d01p2",
        "Input 'y21d01p1y21d01p2' does not match pattern"
    )]
    fn conversions_err(spec: &str, expected_error_prefix: &str) -> Result<()> {
        let err = spec.parse::<FilterTerm>().unwrap_err();
        let actual_error_message = err.to_string();

        dbg!(&actual_error_message);
        dbg!(&expected_error_prefix);
        assert!(actual_error_message.starts_with(expected_error_prefix));
        Ok(())
    }

    fn from<Y, D, P>(year: Y, day: D, part: P) -> FilterTerm
    where
        Y: Into<Option<u16>>,
        D: Into<Option<u8>>,
        P: Into<Option<u8>>,
    {
        let year = year
            .into()
            .map(|y| Year::try_from(y).unwrap());

        let day = day
            .into()
            .map(|d| Day::try_from(d).unwrap());

        let part = part
            .into()
            .map(|p| Part::try_from(p).unwrap());

        FilterTerm::new(year, day, part)
    }
}
