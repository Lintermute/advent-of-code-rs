use std::fmt::Display;

use crate::{
    ident::Day,
    leaderboard::{HeaderRow1, HeaderRow2, Leaderboard, Row, Stats},
};

const W_LABEL: usize = "Day".len();
const W_TIME: usize = "00:00:00".len();
const W_RANK_MIN: usize = "Rank".len();
const W_SCORE_MIN: usize = "Score".len();

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Adjusted<'a, T: ?Sized> {
    pub element: &'a T,
    pub widths:  &'a Widths,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Widths {
    pub label: usize,
    pub parts: [ColumnWidths; 2],
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct ColumnWidths {
    pub rank:  usize,
    pub score: usize,
    pub total: usize,
}

pub trait Formattable {
    fn adjust_to<'a>(&'a self, widths: &'a Widths) -> Adjusted<'a, Self> {
        Adjusted {
            element: self,
            widths,
        }
    }
}

impl<'a, T: 'a> Formattable for T where Adjusted<'a, T>: Display {}

impl Display for Leaderboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let year = self.year();
        let header1 = HeaderRow1 {};
        let header2 = HeaderRow2 {};

        writeln!(f, "Advent of Code {year} - Personal Leaderboard Statistics")?;
        writeln!(f)?;

        write!(f, "{}", header1.adjust_to(self.widths()))?;
        write!(f, "{}", header2.adjust_to(self.widths()))?;

        for row in self.days() {
            write!(f, "{}", row.adjust_to(self.widths()))?;
        }

        if let Some(totals) = self.totals() {
            let width_total = self.widths().total;
            writeln!(f, "{:-^width_total$}", "")?;
            for row in &totals.rows {
                write!(f, "{}", row.adjust_to(self.widths()))?;
            }
        }

        Ok(())
    }
}

impl<'a> Display for Adjusted<'a, HeaderRow1> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "   ")?;

        let widths = &self.widths.parts;
        write!(f, "   {:-^width$}", "Part 1", width = widths[0].total)?;
        write!(f, "   {:-^width$}", "Part 2", width = widths[1].total)?;

        writeln!(f)?;

        Ok(())
    }
}

impl<'a> Display for Adjusted<'a, HeaderRow2> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Day")?;

        let t = "Time";
        let r = "Rank";
        let s = "Score";

        let widths = &self.widths.parts;
        let w_r1 = widths[0].rank;
        let w_s1 = widths[0].score;
        let w_r2 = widths[1].rank;
        let w_s2 = widths[1].score;

        write!(f, "   {t:>8}  {r:>w_r1$}  {s:>w_s1$}")?;
        write!(f, "   {t:>8}  {r:>w_r2$}  {s:>w_s2$}")?;

        writeln!(f)?;

        Ok(())
    }
}

impl<'a, T: Display> Display for Adjusted<'a, Row<T>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:3}", self.element.label)?;

        let parts = self.element.parts.iter();
        let widths = self.widths.parts.iter();
        for (stats, widths) in parts.zip(widths) {
            let w_r = widths.rank;
            let w_s = widths.score;
            match stats {
                Some(Stats {
                    time: t,
                    rank: r,
                    score: s,
                }) => {
                    write!(f, "   {:>8}  {:>w_r$}  {:>w_s$}", t, r, s)?;
                }
                None => {
                    write!(f, "   {:>8}  {:>w_r$}  {:>w_s$}", '-', '-', '-')?
                }
            }
        }

        writeln!(f)?;

        Ok(())
    }
}

pub fn compute_display_widths(days: &[Row<Day>]) -> Widths {
    let r1 = W_RANK_MIN;
    let s1 = W_SCORE_MIN;
    let r2 = W_RANK_MIN;
    let s2 = W_SCORE_MIN;

    let parts = days
        .iter()
        .fold([(r1, s1), (r2, s2)], |maxes, row| {
            let [(r1, s1), (r2, s2)] = maxes;

            let (r1, s1) = max_widths(r1, s1, row.parts[0].as_ref());
            let (r2, s2) = max_widths(r2, s2, row.parts[1].as_ref());

            [(r1, s1), (r2, s2)]
        })
        .map(|part| {
            let (rank, score) = part;
            let total = W_TIME + 2 + rank + 2 + score;
            ColumnWidths { rank, score, total }
        });

    let total = parts
        .iter()
        .map(|ColumnWidths { total, .. }| 3 + total)
        .sum::<usize>()
        + W_LABEL;

    Widths {
        label: W_LABEL,
        parts,
        total,
    }
}

fn max_widths(
    rank: usize,
    score: usize,
    stats: Option<&Stats>,
) -> (usize, usize) {
    match stats {
        Some(stats) => {
            let r = stats.rank.to_string().len();
            let s = stats.score.to_string().len();
            (rank.max(r), score.max(s))
        }
        None => (rank, score),
    }
}

// Formatting is tested as part of the roundtrip tests in `leaderboard/mod.rs`.
