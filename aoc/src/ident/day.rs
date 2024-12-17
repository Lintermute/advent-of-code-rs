use std::str::FromStr;

use lazy_errors::{prelude::*, Result};

pub const D01: Day = Day(1);
pub const D02: Day = Day(2);
pub const D03: Day = Day(3);
pub const D04: Day = Day(4);
pub const D05: Day = Day(5);
pub const D06: Day = Day(6);
pub const D07: Day = Day(7);
pub const D08: Day = Day(8);
pub const D14: Day = Day(14);
pub const D15: Day = Day(15);
pub const D16: Day = Day(16);
pub const D17: Day = Day(17);

/// Day of an Advent of Code challenge in the range `1..=25`.
///
/// Most notably, this struct is part of [`Spec`].
///
/// See also: [`D01`], [`D02`], ….
///
/// Note: This type implements [`Copy`].
///
/// [`Spec`]: [`util::ident::Spec`]
#[derive(
    Debug,
    Clone,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Copy,
    derive_more::Display,
    derive_more::Into,
)]
pub struct Day(u8);

impl TryFrom<u8> for Day {
    type Error = Error;

    fn try_from(d: u8) -> Result<Self> {
        if !(1..=25).contains(&d) {
            return Err(err!("Day {d} is out of range [1,25]"));
        }

        Ok(Self(d))
    }
}

impl FromStr for Day {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let inner: u8 = s
            .parse()
            .or_wrap_with(|| format!("Not a day: '{s}'"))?;

        Self::try_from(inner)
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case(Day(1), "1", 1u8)]
    #[test_case(Day(25), "25", 25u8)]
    fn conversions_ok(day: Day, txt: &str, num: u8) -> Result<()> {
        assert_eq!(day, txt.parse()?);
        assert_eq!(day.to_string(), txt);
        assert_eq!(day, num.try_into()?);
        assert_eq!(u8::from(day), num);
        Ok(())
    }

    #[test_case(0u8)]
    #[test_case(26u8)]
    fn convert_try_from_err(num: u8) -> Result<()> {
        let _ = Day::try_from(num).unwrap_err();
        Ok(())
    }

    #[test_case("")]
    #[test_case("0")]
    #[test_case("26")]
    #[test_case("-1")]
    #[test_case("a")]
    fn convert_from_str_err(txt: &str) -> Result<()> {
        let _ = Day::from_str(txt).unwrap_err();
        Ok(())
    }
}
