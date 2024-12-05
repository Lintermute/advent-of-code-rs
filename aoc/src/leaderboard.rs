mod formatting;
mod min_med_max;
mod parsing;
mod rank;
mod score;
mod stats;
mod time;
mod totals;

pub use parsing::parse_leaderboards_from_fs;

use crate::{
    ident::{Day, Year},
    leaderboard::{formatting::Widths, stats::Stats, totals::Totals},
};

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Leaderboard {
    year:   Year,
    days:   Vec<Row<Day>>,
    totals: Option<Totals>,
    widths: Widths,
}

/// The table's first header row,
/// i.e. columns containing the `--- Part X ---` strings.
#[derive(Debug)]
pub struct HeaderRow1 {}

/// The table's second header row,
/// i.e. columns containing the `Time`, `Rank`, and `Score` strings.
#[derive(Debug)]
pub struct HeaderRow2 {}

/// A “regular” row of the table, usually prefixed by the label `Day`.
/// When the leaderboard contains more than a single day,
/// there will also be `Row<TotalKind>` lines
/// which will be prefixed by `MIN`, `MED`, and `MAX`.
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Row<T> {
    pub label: T,
    pub parts: [Option<Stats>; 2],
}

impl Leaderboard {
    pub fn new(year: Year, days: Vec<Row<Day>>) -> Option<Leaderboard> {
        if days.is_empty() {
            return None;
        }

        let totals = if days.len() < 2 {
            None
        } else {
            Some(Totals::from(days.as_ref()))
        };

        let widths = formatting::compute_display_widths(&days);

        Some(Leaderboard {
            year,
            days,
            totals,
            widths,
        })
    }

    pub fn year(&self) -> Year {
        self.year
    }

    pub fn days(&self) -> &[Row<Day>] {
        &self.days
    }

    pub fn totals(&self) -> Option<&Totals> {
        self.totals.as_ref()
    }

    pub fn widths(&self) -> &Widths {
        &self.widths
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use lazy_errors::Result;

    use crate::ident::{year::Y21, Filter};

    use super::*;

    #[test]
    fn read_and_print_empty_stats() -> Result<()> {
        let input = indoc! {"\
                  --------Part 1--------   --------Part 2--------
            Day       Time   Rank  Score       Time   Rank  Score
        "};

        assert_roundtrip(2021, input, None)
    }

    #[test]
    fn read_and_print_single_day_still_does_not_show_stats() -> Result<()> {
        let input = indoc! {"\
                  --------Part 1--------   --------Part 2--------
            Day       Time   Rank  Score       Time   Rank  Score
              1   00:20:32   6893      0   00:24:50   5662      0
        "};

        let expected = indoc! {"\
            Advent of Code 2021 - Personal Leaderboard Statistics

                  -------Part 1--------   -------Part 2--------
            Day       Time  Rank  Score       Time  Rank  Score
              1   00:20:32  6893      0   00:24:50  5662      0
        "};

        assert_roundtrip(2021, input, expected)
    }

    #[test]
    fn read_and_print_empty_part2() -> Result<()> {
        let input = indoc! {"\
                  --------Part 1---------   -------Part 2--------
            Day       Time    Rank  Score       Time  Rank  Score
              2       >24h  187123      0          -     -      -
              1   00:20:32    6893      0          -     -      -
        "};

        let expected = indoc! {"\
            Advent of Code 2021 - Personal Leaderboard Statistics

                  --------Part 1---------   -------Part 2--------
            Day       Time    Rank  Score       Time  Rank  Score
              2       >24h  187123      0          -     -      -
              1   00:20:32    6893      0          -     -      -
            -----------------------------------------------------
            MIN   00:20:32    6893      0          -     -      -
            MED       >24h   97008      0          -     -      -
            MAX       >24h  187123      0          -     -      -
        "};

        assert_roundtrip(2021, input, expected)
    }

    #[test]
    fn parse_leaderboard_fails_when_header1_is_missing() -> Result<()> {
        let input = indoc! {"\
            Day       Time   Rank  Score       Time   Rank  Score
              1   00:20:32   6893      0   00:24:50   5662      0
        "};

        assert_err(input, "first line of the table header")
    }

    #[test]
    fn parse_leaderboard_fails_when_header2_is_missing() -> Result<()> {
        let input = indoc! {"\
                  --------Part 1--------   --------Part 2--------
              1   00:20:32   6893      0   00:24:50   5662      0
        "};

        assert_err(input, "second line of the table header")
    }

    #[test]
    fn parse_leaderboard_fails_when_row_is_invalid() -> Result<()> {
        let input = indoc! {"\
                  --------Part 1--------   --------Part 2--------
            Day       Time   Rank  Score       Time   Rank  Score
              0   00:00:00      0      0   00:00:00      0      0
        "};

        assert_err(input, "row label '0'")
    }

    fn assert_roundtrip<'a>(
        year: u16,
        input: &str,
        expected_output: impl Into<Option<&'a str>>,
    ) -> Result<()> {
        let year = Year::try_from(year)?;
        let filter = Filter::default();
        let lines = input.lines().map(|s| Ok(s.to_owned()));
        let board = parsing::parse_leaderboard(year, &filter, lines)?;

        match expected_output.into() {
            None => assert_eq!(board, None),
            Some(text) => {
                let board = board.unwrap();
                assert_eq!(text, &format!("{board}"));
            }
        }

        Ok(())
    }

    fn assert_err(input: &str, desc: &str) -> Result<()> {
        let year = Y21;
        let filter = Filter::default();
        let lines = input.lines().map(|s| Ok(s.to_owned()));
        let result = parsing::parse_leaderboard(year, &filter, lines);
        let err = result.unwrap_err();
        let msg = err.to_string();

        dbg!(&msg);
        assert!(msg.starts_with("Failed to parse 2021 leaderboard"));
        assert!(msg.contains(desc));

        Ok(())
    }
}
