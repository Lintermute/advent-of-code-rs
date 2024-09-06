use std::fmt::Debug;

use lazy_errors::{prelude::*, Result};
use num::integer::average_ceil;

use crate::leaderboard::min_med_max::Mean;

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
)]
pub struct Rank(u32);

impl Rank {
    pub fn new(rank: u32) -> Result<Rank> {
        if rank == 0 {
            return Err(err!("Rank must be greater than zero"));
        }

        Ok(Rank(rank))
    }
}

impl TryFrom<&str> for Rank {
    type Error = Error;

    fn try_from(rank: &str) -> Result<Self> {
        rank.parse::<u32>()
            .or_wrap_with(|| format!("Invalid rank: '{rank}'"))
            .and_then(Rank::new)
    }
}

impl Mean for Rank {
    fn mean(&self, right: &Self) -> Self {
        let avg = average_ceil(self.0, right.0);

        // "Cannot" fail
        Rank::new(avg).expect("Average of valid Ranks to be valid")
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use crate::leaderboard::min_med_max::min_med_max_sorted;

    use super::*;

    #[test]
    fn create_rank_0() {
        assert!(Rank::new(0).is_err());
    }

    #[test]
    fn create_rank_1() {
        assert!(Rank::new(1).is_ok());
    }

    #[test]
    fn parse_rank_succeeds() -> Result<()> {
        assert_eq!(Rank::try_from("42")?, Rank::new(42)?);
        Ok(())
    }

    #[test_case("")]
    #[test_case("A")]
    #[test_case("-1")]
    #[test_case("0"; "Rank must be greater than zero")]
    #[test_case(" 42"; "Leading whitespace is not allowed")]
    #[test_case("42 "; "Trailing whitespace is not allowed")]
    fn parse_rank_fails(text: &str) {
        assert!(Rank::try_from(text).is_err());
    }

    #[test_case(1, 1, 1; "Average of identical ranks is the same")]
    #[test_case(1, 5, 3; "Computes the average")]
    #[test_case(1, 2, 2; "Chooses the worse rank if in-between")]
    fn average(a: u32, b: u32, exp: u32) -> Result<()> {
        let a = Rank::new(a)?;
        let b = Rank::new(b)?;
        let exp = Rank::new(exp)?;
        assert_eq!(exp, a.mean(&b));
        Ok(())
    }

    #[test]
    fn min_med_max() -> Result<()> {
        let mut vec = vec![
            Rank::new(100)?,
            Rank::new(42)?,
            Rank::new(30)?,
            Rank::new(7)?,
            Rank::new(80)?,
        ];
        vec.sort_unstable();
        assert_eq!(
            min_med_max_sorted(&vec),
            Some((Rank::new(7)?, Rank::new(42)?, Rank::new(100)?))
        );
        Ok(())
    }
}
