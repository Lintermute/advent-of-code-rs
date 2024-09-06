use std::str::FromStr;

use lazy_errors::{prelude::*, Result};

use crate::ident::{Day, Part, Year};

/// Represents a [`Year`], [`Day`], or [`Part`] in a distinctive form,
/// such as `y21` for `2021`, `d21` for `21`, and `p1` for `1`.
/// This type is intended for reading and writing strings.
///
/// Note: This type implements [`Copy`].
#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub struct Id<T>(pub T);

trait Identifiable {
    fn key() -> &'static str;
    fn len() -> usize;
}

impl Identifiable for Year {
    fn key() -> &'static str {
        "y"
    }

    fn len() -> usize {
        3
    }
}

impl Identifiable for Day {
    fn key() -> &'static str {
        "d"
    }

    fn len() -> usize {
        3
    }
}

impl Identifiable for Part {
    fn key() -> &'static str {
        "p"
    }

    fn len() -> usize {
        2
    }
}

impl TryFrom<u8> for Id<Year> {
    type Error = Error;

    fn try_from(k: u8) -> Result<Self> {
        let digits = u16::from(k);
        Year::try_from(2000 + digits).map(Id)
    }
}

impl<T> TryFrom<u8> for Id<T>
where
    T: TryFrom<u8, Error = Error>,
{
    type Error = Error;

    fn try_from(k: u8) -> Result<Self> {
        Ok(Id(T::try_from(k)?))
    }
}

impl From<Id<Year>> for u8 {
    fn from(val: Id<Year>) -> Self {
        let yyyy = u16::from(val.0);
        let yy = yyyy % 100;
        yy as u8
    }
}

impl<T> From<Id<T>> for u8
where
    T: Into<u8>,
{
    fn from(val: Id<T>) -> Self {
        val.0.into()
    }
}

impl<T: Identifiable> FromStr for Id<T>
where
    Id<T>: TryFrom<u8, Error = Error>,
{
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let key = T::key();
        let len_key = key.len();
        let len_str = T::len();
        let len_num = len_str - len_key;

        if !(s.len() == len_str && s.starts_with(key)) {
            return Err(err!(
                "Failed to parse: '{s}' does not match \
                 '{key}[0-9]{{{len_num}}}'"
            ));
        }

        let n = &s[len_key..len_str];
        let n = u8::from_str(n)
            .or_wrap_with(|| format!("Invalid number: '{n}'"))?;

        Self::try_from(n)
    }
}

impl<T: Identifiable> std::fmt::Display for Id<T>
where
    Id<T>: Into<u8>,
    T: Copy, // `Id` is already copy
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let key = T::key();
        let len = T::len() - key.len();
        let num: u8 = (*self).into();
        write!(f, "{key}{num:0len$}")
    }
}

impl FromStr for Id<(Year, Day, Part)> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.len() != 8 {
            return Err(err!("Expected pattern yYYdDDpP, got '{s}'"));
        }

        let Id::<Year>(y) = s[0..3].parse()?;
        let Id::<Day>(d) = s[3..6].parse()?;
        let Id::<Part>(p) = s[6..8].parse()?;
        Ok(Id((y, d, p)))
    }
}

impl std::fmt::Display for Id<(Year, Day, Part)> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (y, d, p) = self.0;
        write!(f, "{}{}{}", Id(y), Id(d), Id(p))
    }
}

impl FromStr for Id<(Year, Day)> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.len() != 6 {
            return Err(err!("Expected pattern yYYdDD, got '{s}'"));
        }

        let Id::<Year>(y) = s[0..3].parse()?;
        let Id::<Day>(d) = s[3..6].parse()?;
        Ok(Id((y, d)))
    }
}

impl std::fmt::Display for Id<(Year, Day)> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (y, d) = self.0;
        write!(f, "{}{}", Id(y), Id(d))
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case("y21d03p2", 2021, 3, 2)]
    #[test_case("y22d01p1", 2022, 1, 1)]
    fn conversions_ymd_ok(s: &str, y: u16, d: u8, p: u8) -> Result<()> {
        let id = Id::from_str(s)?;
        assert_eq!(&id.to_string(), s);

        let Id((year, day, part)) = id;
        assert_eq!(u16::from(year), y);
        assert_eq!(u8::from(day), d);
        assert_eq!(u8::from(part), p);
        Ok(())
    }

    #[test_case("y19d01p1")]
    #[test_case("y20d26p1")]
    #[test_case("y20d01p3")]
    #[test_case("y21d01")]
    fn conversions_ymd_err(s: &str) -> Result<()> {
        let _ = Id::<(Year, Day, Part)>::from_str(s).unwrap_err();
        Ok(())
    }

    #[test_case("y21d03", 2021, 3)]
    #[test_case("y22d01", 2022, 1)]
    fn conversions_ym_ok(s: &str, y: u16, d: u8) -> Result<()> {
        let id = Id::from_str(s)?;
        assert_eq!(&id.to_string(), s);

        let Id((year, day)) = id;
        assert_eq!(u16::from(year), y);
        assert_eq!(u8::from(day), d);
        Ok(())
    }

    #[test_case("y19d01")]
    #[test_case("y20d26")]
    #[test_case("y21d01p1")]
    fn conversions_ym_err(s: &str) -> Result<()> {
        let _ = Id::<(Year, Day)>::from_str(s).unwrap_err();
        Ok(())
    }

    #[test_case("y21", 2021)]
    #[test_case("y22", 2022)]
    fn conversions_y_ok(s: &str, y: u16) -> Result<()> {
        let id = Id::<Year>::from_str(s)?;
        assert_eq!(&id.to_string(), s);

        let Id(year) = id;
        assert_eq!(u16::from(year), y);
        Ok(())
    }

    #[test_case("y19")]
    #[test_case("y24")]
    #[test_case("y2019")]
    #[test_case("y21d01")]
    #[test_case("y-1")]
    #[test_case("year")]
    fn conversions_y_err(s: &str) -> Result<()> {
        let _ = Id::<Year>::from_str(s).unwrap_err();
        Ok(())
    }
}
