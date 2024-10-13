use std::{fs::DirEntry, str::FromStr};

use itertools::Itertools;
use lazy_errors::{prelude::*, Result};
use lazy_regex::regex_is_match;

use crate::{
    fs::{self, Config},
    ident::{Day, Filter, Id, Year},
    parser,
};

use super::{stats::Stats, HeaderRow1, HeaderRow2, Leaderboard, Row};

#[cfg(test)]
use super::{rank::Rank, score::Score, time::Time};

impl FromStr for HeaderRow1 {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if regex_is_match!(r"---Part 1---[\-]*\s+[\-]*---Part 2---", s) {
            Ok(HeaderRow1 {})
        } else {
            Err(err!("Not the first line of the table header: '{s}'"))
        }
    }
}

impl FromStr for HeaderRow2 {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if regex_is_match!(r"Day(\s+Time\s+Rank\s+Score){2}", s) {
            Ok(HeaderRow2 {})
        } else {
            Err(err!("Not the second line of the table header: '{s}'"))
        }
    }
}

impl<T, E> FromStr for Row<T>
where
    T: FromStr<Err = E>,
    E: Into<Stashable>,
{
    type Err = Error;

    fn from_str(line: &str) -> Result<Self> {
        let (label, time1, rank1, score1, time2, rank2, score2) = line
            .split_whitespace()
            .collect_tuple()
            .ok_or_else(|| err!("Failed to tokenize table row: '{}'", line))?;

        let label: T = label
            .parse()
            .or_wrap_with(|| format!("Failed to parse row label '{label}'"))?;

        let parts = [
            parse_part_cols(time1, rank1, score1)?,
            parse_part_cols(time2, rank2, score2)?,
        ];

        Ok(Row { label, parts })
    }
}

pub fn parse_leaderboard(
    year: Year,
    filter: &Filter,
    mut lines: impl Iterator<Item = Result<String>>,
) -> Result<Option<Leaderboard>> {
    let msg = || format!("Failed to parse {year} leaderboard");

    let _: HeaderRow1 = parser::try_parse_next(&mut lines).or_wrap_with(msg)?;
    let _: HeaderRow2 = parser::try_parse_next(&mut lines).or_wrap_with(msg)?;

    let days: Vec<Row<Day>> = parser::parse_all_ok(lines)
        .filter_ok(|row: &Row<Day>| filter.matches_year_day(year, row.label))
        .try_collect()
        .or_wrap_with(msg)?;

    Ok(Leaderboard::new(year, days))
}

pub fn parse_leaderboards_from_fs(
    config: &Config,
    filter: &Filter,
) -> Result<Vec<Leaderboard>> {
    parse_years_from_fs(config)?
        .into_iter()
        .filter(|&y| filter.matches_year(y))
        .flat_map(|y| parse_leaderboard_from_fs(y, config, filter).transpose())
        .try_collect()
}

fn parse_years_from_fs(config: &Config) -> Result<Vec<Year>> {
    let dir = config.personal_leaderboard_dir();

    let mut errs = ErrorStash::new(|| {
        format!("Failed to read leaderboards from '{}'", dir.display())
    });

    let entries: Vec<DirEntry> = try2!(std::fs::read_dir(&dir)
        .and_then(|files| files.try_collect())
        .or_wrap_with::<Stashable>(|| "Failed to read directory")
        .or_stash(&mut errs));

    let mut years: Vec<Year> = try2!(entries
        .iter()
        .map(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();

            lazy_regex::regex_captures!(
                r"^(y\d{2})_personal_leaderboard_statistics.txt$",
                &name
            )
            .ok_or_else(|| {
                err!(
                    "File name does not match pattern \
                     'yYY_personal_leaderboard_statistics.txt'"
                )
            })
            .and_then(|(_, y)| y.parse().map(|Id::<Year>(y)| y))
            .or_wrap_with::<Stashable>(|| {
                format!("Failed to parse file name '{name}'")
            })
        })
        .try_collect_or_stash(&mut errs));

    years.sort_unstable();

    Ok(years)
}

fn parse_leaderboard_from_fs(
    year: Year,
    config: &Config,
    filter: &Filter,
) -> Result<Option<Leaderboard>> {
    let lines = read_leaderboard_lines(year, config)?;
    parse_leaderboard(year, filter, lines)
}

fn read_leaderboard_lines(
    year: Year,
    config: &Config,
) -> Result<impl Iterator<Item = Result<String>>> {
    let path = config.personal_leaderboard_file(year);
    fs::open(path)
        .or_wrap_with(|| format!("Failed to open {year} leaderboard"))
        .map(fs::lines)
}

fn parse_part_cols(
    time: &str,
    rank: &str,
    score: &str,
) -> Result<Option<Stats>> {
    match (time, rank, score) {
        ("-", "-", "-") => Ok(None),
        others => Ok(Some(others.try_into()?)),
    }
}

/// Also tested as part of the roundtrip tests in `leaderboard/mod.rs`.
#[cfg(test)]
mod tests {
    use std::time::Duration;

    use test_case::test_case;

    use crate::{fs, ident::Day};

    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `RepoDir`/`create_test_config…`
    fn parse_from_fs_when_subdir_does_not_exist() -> Result<()> {
        // Both $REPO_DIR and $DATA_DIR exist,
        // but "${DATA_DIR}/personal_leaderboard_statistics" does not.
        let config = fs::create_test_config_for_dir_thats_empty()?;
        let path = config
            .personal_leaderboard_dir()
            .to_string_lossy()
            .to_string();

        let result = parse_leaderboards_from_fs(&config, &Filter::default());
        let msg = result.unwrap_err().to_string();

        dbg!(&msg);
        assert!(msg.contains("Failed to read leaderboards from"));
        assert!(msg.contains(&path));

        Ok(())
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `RepoDir`/`create_test_config…`
    fn parse_from_fs_when_dir_contains_invalid_files() -> Result<()> {
        let config = fs::create_test_config_for_dir_with_invalid_files()?;
        let path = config
            .personal_leaderboard_dir()
            .to_string_lossy()
            .to_string();

        let result = parse_leaderboards_from_fs(&config, &Filter::default());
        let msg = format!("{:#}", result.unwrap_err());

        dbg!(&msg);
        assert!(msg.contains(
            "File name does not match pattern \
             'yYY_personal_leaderboard_statistics.txt'"
        ));
        assert!(msg.contains(
            "Failed to parse file name 'this_file_makes_tests_fail'"
        ));
        assert!(msg.contains(&path));

        Ok(())
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `RepoDir`/`create_test_config`
    fn parse_from_fs_when_leaderboard_does_not_exist() -> Result<()> {
        let config = fs::create_test_config()?;
        let path = config
            .personal_leaderboard_dir()
            .to_string_lossy()
            .into_owned();

        let y = Year::try_from(2023)?;
        let result = parse_leaderboard_from_fs(y, &config, &Filter::default());
        let msg = result.unwrap_err().to_string();

        dbg!(&msg, &path);
        assert!(msg.contains("Failed to open 2023 leaderboard"));
        assert!(msg.contains(&path));

        Ok(())
    }

    #[test_case("", false, false; "Empty string does not belong to table head")]
    #[test_case(
        "      --------Part 1--------   --------Part 2--------", true, false;
        "Part X delimiter belongs to table head")]
    #[test_case(
        "Day       Time   Rank  Score       Time   Rank  Score", false, true;
        "Column names belong to table head")]
    #[test_case(
        "  1   00:20:32   6893      0   00:24:50   5662      0", false, false;
        "Stat rows do not belong to table head")]
    fn is_table_head(line: &str, is_line_1: bool, is_line_2: bool) {
        assert_parsing::<HeaderRow1>(line, is_line_1);
        assert_parsing::<HeaderRow2>(line, is_line_2);
    }

    #[test]
    fn parse_table_row() -> Result<()> {
        let Row {
            label: day,
            parts: [part1, part2],
        } = Row::<Day>::from_str(
            "  9   00:44:45   8618      0       >24h  10293      42",
        )?;

        let part1 = part1.unwrap();
        let part2 = part2.unwrap();

        assert_eq!(day, Day::try_from(9)?);

        assert_eq!(part1.time, Time::Exactly(Duration::new(45 + 44 * 60, 0)));
        assert_eq!(part1.rank, Rank::new(8618)?);
        assert_eq!(part1.score, Score::new(0));

        assert_eq!(part2.time, Time::Forever);
        assert_eq!(part2.rank, Rank::new(10293)?);
        assert_eq!(part2.score, Score::new(42));

        Ok(())
    }

    #[test_case(
        "  9   00:44:45   8618      0       >24h  10293      42 0",
        "tokenize"
    )]
    #[test_case(
        "  0   00:44:45   8618      0       >24h  10293      42",
        "label"
    )]
    #[test_case(
        "  9   0-:44:45   8618      0       >24h  10293      42",
        "Time"
    )]
    #[test_case(
        "  9   00:44:45      0      0       >24h  10293      42",
        "Rank"
    )]
    #[test_case(
        "  9   00:44:45   8618     -1       >24h  10293      42",
        "Score"
    )]
    #[test_case(
        "  9   00:44:45   8618      0       <24h  10293      42",
        "Time"
    )]
    #[test_case(
        "  9   00:44:45   8618      0       >24h     -1      42",
        "Rank"
    )]
    #[test_case(
        "  9   00:44:45   8618      0       >24h  10293      42000000000000000",
        "Score"
    )]
    fn parse_table_row_fails_gracefully(
        row: &str,
        error_token: &str,
    ) -> Result<()> {
        let err = Row::<Day>::from_str(row).unwrap_err();

        let msg = err.to_string().to_uppercase();
        let tok = error_token.to_uppercase();
        if !msg.contains(&tok) {
            Err(err!("Error message does not contain '{tok}': '{msg}'"))
        } else {
            Ok(())
        }
    }

    fn assert_parsing<T: FromStr<Err = Error> + std::fmt::Debug>(
        line: &str,
        is_valid: bool,
    ) {
        let t = T::from_str(line);

        if is_valid {
            t.unwrap();
        } else {
            let _ = t.unwrap_err();
        }
    }
}
