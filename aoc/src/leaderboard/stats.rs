use lazy_errors::{prelude::*, Result};

use crate::leaderboard::{rank::Rank, score::Score, time::Time};

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Stats {
    pub time:  Time,
    pub rank:  Rank,
    pub score: Score,
}

impl Stats {
    pub fn new(time: Time, rank: Rank, score: Score) -> Self {
        Self { time, rank, score }
    }
}

impl TryFrom<(&str, &str, &str)> for Stats {
    type Error = Error;

    fn try_from((time, rank, score): (&str, &str, &str)) -> Result<Self> {
        Ok(Self {
            time:  time.try_into()?,
            rank:  rank.try_into()?,
            score: score.try_into()?,
        })
    }
}
