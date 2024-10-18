use lazy_errors::{prelude::*, Result};

pub const Y21: Year = Year(2021);
pub const Y23: Year = Year(2023);

/// Year of an Advent of Code challenge, such as `2021`.
///
/// Most notably, this struct is part of [`Spec`].
///
/// See also: [`Y21`], ….
///
/// Note: This type implements [`Copy`].
///
/// [`Spec`]: [`util::ident::Spec`]
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    PartialOrd,
    Hash,
    Eq,
    Ord,
    derive_more::Display,
    derive_more::Into,
)]
pub struct Year(u16);

impl TryFrom<u16> for Year {
    type Error = Error;

    fn try_from(y: u16) -> Result<Self> {
        if !(2020..=2023).contains(&y) {
            return Err(err!("Year {y} is out of range [2020,2023]"));
        }

        Ok(Self(y))
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case(Year(2021), "2021", 2021u16)]
    #[test_case(Year(2023), "2023", 2023u16)]
    fn conversions_ok(year: Year, txt: &str, num: u16) -> Result<()> {
        assert_eq!(year.to_string(), txt);
        assert_eq!(year, num.try_into()?);
        assert_eq!(u16::from(year), num);
        Ok(())
    }

    #[test_case(2019u16)]
    #[test_case(2024u16)]
    fn conversions_err(num: u16) -> Result<()> {
        let _ = Year::try_from(num).unwrap_err();
        Ok(())
    }
}
