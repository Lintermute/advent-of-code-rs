use std::fmt::Debug;

use lazy_errors::{prelude::*, Result};
use num::integer::average_floor;

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
pub struct Score(u16);

impl Score {
    pub fn new(score: u16) -> Score {
        Score(score)
    }
}

impl TryFrom<&str> for Score {
    type Error = Error;

    fn try_from(score: &str) -> Result<Self> {
        score
            .parse::<u16>()
            .or_wrap_with(|| format!("Invalid score: '{score}'"))
            .map(Score::new)
    }
}

impl Mean for Score {
    fn mean(&self, right: &Self) -> Self {
        let avg = average_floor(self.0, right.0);
        Score::new(avg)
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use crate::leaderboard::min_med_max::min_med_max_sorted;

    use super::*;

    #[test]
    fn parse_score_fails_if_empty() {
        assert!(Score::try_from("").is_err());
    }

    #[test]
    fn parse_score_succeeds() -> Result<()> {
        assert_eq!(Score::try_from("42")?, Score::new(42));
        Ok(())
    }

    #[test]
    fn parse_score_fails_if_nan() {
        assert!(Score::try_from("A").is_err());
    }

    #[test]
    fn parse_score_fails_if_negative() {
        assert!(Score::try_from("-1").is_err());
    }

    #[test_case(0, 0, 0; "Average of identical scores is the same")]
    #[test_case(1, 5, 3; "Computes the average")]
    #[test_case(1, 2, 1; "Chooses the worse (lower) score if in-between")]
    fn average(a: u16, b: u16, exp: u16) -> Result<()> {
        let a = Score::new(a);
        let b = Score::new(b);
        let exp = Score::new(exp);
        assert_eq!(exp, a.mean(&b));
        Ok(())
    }

    #[test]
    fn min_med_max() -> Result<()> {
        let mut vec = vec![
            Score::new(100),
            Score::new(42),
            Score::new(30),
            Score::new(7),
            Score::new(80),
        ];
        vec.sort_unstable();
        assert_eq!(
            min_med_max_sorted(&vec),
            Some((Score::new(7), Score::new(42), Score::new(100)))
        );
        Ok(())
    }
}
