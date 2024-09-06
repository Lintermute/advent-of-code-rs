use std::fmt::Display;

use itertools::Itertools;

use crate::{
    ident::Day,
    leaderboard::{min_med_max::min_med_max_sorted, stats::Stats, Row},
};

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Totals {
    pub rows: [Row<TotalKind>; 3],
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub enum TotalKind {
    Min,
    Med,
    Max,
}

impl From<&[Row<Day>]> for Totals {
    fn from(rows: &[Row<Day>]) -> Self {
        // Compute min, median, and max,
        // for time, rank, and score,
        // for both parts.
        let totals_of_part: [Option<[Stats; 3]>; 2] = [0, 1].map(|part| {
            let (mut times, mut ranks, mut scores): (Vec<_>, Vec<_>, Vec<_>) =
                rows.iter()
                    .filter_map(|row| row.parts[part].as_ref())
                    .map(|record| (record.time, record.rank, record.score))
                    .multiunzip();

            times.sort_unstable();
            ranks.sort_unstable();
            scores.sort_unstable();

            let (t_min, t_med, t_max) = min_med_max_sorted(&times)?;
            let (r_min, r_med, r_max) = min_med_max_sorted(&ranks)?;
            let (s_min, s_med, s_max) = min_med_max_sorted(&scores)?;

            let min = Stats::new(t_min, r_min, s_min);
            let med = Stats::new(t_med, r_med, s_med);
            let max = Stats::new(t_max, r_max, s_max);

            Some([min, med, max])
        });

        // `totals_of_part` is basically an array of “columns”
        // (min, med, max for parts one and two,
        // each having a time, a rank, and a score).
        // To print that data, we have to “transpose” that matrix
        // into an array of rows:
        // The first row contains the minimum of time, rank, and score
        // of part one, followed by the values for part two.
        // The second row then contains the median of these values,
        // and the third row their maximum.
        let totals = [
            (0, TotalKind::Min),
            (1, TotalKind::Med),
            (2, TotalKind::Max),
        ]
        .map(|(index, label)| {
            let columns: [Option<Stats>; 2] = [0, 1].map(|part| {
                totals_of_part[part]
                    .as_ref()
                    .map(|stats| stats[index].clone())
            });

            Row {
                label,
                parts: columns,
            }
        });

        Self { rows: totals }
    }
}

impl Display for TotalKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TotalKind::*;

        let label = match self {
            Min => "MIN",
            Med => "MED",
            Max => "MAX",
        };

        write!(f, "{label}")
    }
}
