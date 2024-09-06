use lazy_errors::{prelude::*, Result};

pub const P1: Part = Part::Part1;
pub const P2: Part = Part::Part2;

/// Represents either the first or the second part of an Advent of Code puzzle
/// for a certain date.
///
/// Most notably, this struct is part of [`Spec`].
///
/// Note: This type implements [`Copy`].
///
/// [`Spec`]: [`util::ident::Spec`]
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub enum Part {
    Part1,
    Part2,
}

impl TryFrom<u8> for Part {
    type Error = Error;

    fn try_from(p: u8) -> Result<Self> {
        match p {
            1 => Ok(Part::Part1),
            2 => Ok(Part::Part2),
            _ => Err(err!("Puzzle part {p} is out of range [1,2]")),
        }
    }
}

impl From<Part> for u8 {
    fn from(value: Part) -> Self {
        match value {
            Part::Part1 => 1,
            Part::Part2 => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case(Part::Part1, 1u8)]
    #[test_case(Part::Part2, 2u8)]
    fn conversions_ok(part: Part, num: u8) -> Result<()> {
        assert_eq!(part, num.try_into()?);
        assert_eq!(u8::from(part), num);
        Ok(())
    }

    #[test_case(0u8)]
    #[test_case(3u8)]
    fn conversions_err(num: u8) -> Result<()> {
        let _ = Part::try_from(num).unwrap_err();
        Ok(())
    }
}
