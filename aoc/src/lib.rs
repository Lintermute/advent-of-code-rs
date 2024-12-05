#![forbid(unsafe_code)]

pub mod ident;
pub mod puzzles;
pub mod runner;
pub mod solver;

mod cli;
mod downloader;
mod fs;
mod leaderboard;
mod parser;
mod ui;

pub use fs::Config;
pub use ident::{D01, D02, D03, D04, D15, P1, P2, Y21, Y23, Y24};

use std::{
    io::Write,
    process::{ExitCode, Termination},
};

use lazy_errors::{prelude::*, Result};
use runner::Runner;
use tokio::sync::mpsc;

use downloader::Downloader;
use ident::{Filter, Id};
use solver::{Event, Parts, Solver};
use ui::{Summary, Terminated, Ui};

use puzzles::*;

const SOLVERS: &[Solver] = &[
    solver!(Y21, D01, y21d01::part1, y21d01::part2, y21d01::parse),
    solver!(Y21, D02, y21d02::part1, y21d02::part2),
    solver!(Y21, D03, y21d03::part1, y21d03::part2, y21d03::parse),
    solver!(Y23, D03, y23d03::part1, y23d03::part2, y23d03::parse),
    solver!(Y23, D15, y23d15::part1, y23d15::part2, y23d15::parse),
    solver!(Y24, D01, y24d01::part1, y24d01::part2, y24d01::parse),
    solver!(Y24, D02, y24d02::part1, y24d02::part2, y24d02::parse),
    solver!(Y24, D03, y24d03::part1, y24d03::part2, y24d03::parse),
    solver!(Y24, D04, y24d04::part1, y24d04::part2, y24d04::parse),
];

#[derive(Debug)]
pub enum ExitStatus {
    AllRunnersSucceeded,
    SomeRunnersFailed,
    AbortedByUser,
    InternalError(Error),
}

impl Termination for ExitStatus {
    fn report(self) -> ExitCode {
        match self {
            ExitStatus::AllRunnersSucceeded => ExitCode::SUCCESS,
            ExitStatus::SomeRunnersFailed => ExitCode::from(1),
            ExitStatus::AbortedByUser => ExitCode::from(2),
            ExitStatus::InternalError(_) => ExitCode::from(4),
        }
    }
}

impl From<Result<Summary, Terminated>> for ExitStatus {
    fn from(value: Result<Summary, Terminated>) -> Self {
        match value {
            Ok(Summary::Success) => ExitStatus::AllRunnersSucceeded,
            Ok(Summary::SomeRunnersFailed) => ExitStatus::SomeRunnersFailed,
            Err(Terminated::AbortedByUser) => ExitStatus::AbortedByUser,
            Err(Terminated::InternalError(e)) => ExitStatus::InternalError(e),
        }
    }
}

pub async fn main() -> ExitStatus {
    let result = try_main().await;
    let status = ExitStatus::from(result);

    if let ExitStatus::InternalError(err) = &status {
        eprintln!(); // Add some space between table and error log
        eprintln!("Internal error: {err:#}");
    }

    status
}

async fn try_main() -> Result<Summary, Terminated> {
    use cli::Command;
    use std::io::stdout;

    let config = Config::from_env_or_defaults()?;
    match cli::parse_args_from_env_or_exit() {
        Command::Login => login(config),
        Command::Logout => logout(config),
        Command::Solve(filter) => run_solvers(config, &filter).await,
        Command::Stats(filter) => print_stats(&config, &filter, stdout()),
    }
}

fn login(mut config: Config) -> Result<Summary, Terminated> {
    let y = Y21;
    let d = D01;
    let id = Id((y, d));
    let path = config.personal_puzzle_inputs_dir();
    let path = path.to_string_lossy();

    print!(
        "\
To have your personal puzzle inputs downloaded automatically,
you need to enter the value of your adventofcode.com session cookie.
Here's how:

1. Open your web browser and log in to adventofcode.com
2. Open your web browser's developer tools
3. Locate the cookie named `session`
4. Copy the value of that cookie and paste it into the prompt below

If you don't want to enter your session cookie into this program,
you'll need to download your personal puzzle inputs manually.
For example, the input for year {y} day {d} MUST be named
`{id}_personal_puzzle_input.txt` and you MUST put those files
in the following directory:
{path}

Enter the value of your session cookie now or press CTRL-C to cancel:
> "
    );
    // The `> ` (without newline!) needs a flush to be printed.
    // But it's not a critical error if flushing fails, so drop the `Result`.
    _ = std::io::stdout().flush();

    let mut session_cookie = String::new();
    std::io::stdin()
        .read_line(&mut session_cookie)
        .or_wrap_with(|| "Failed to read user input")?;

    config.save_session_cookie(session_cookie.trim())?;
    Ok(Summary::Success)
}

fn logout(mut config: Config) -> Result<Summary, Terminated> {
    config.delete_session_cookie()?;
    Ok(Summary::Success)
}

async fn run_solvers(
    config: Config,
    filter: &Filter,
) -> Result<Summary, Terminated> {
    let puzzles = filter_puzzles(SOLVERS, filter);

    let ui = Ui::open(puzzles.clone())?;
    spawn_actors(config, puzzles, ui.tx());
    ui.join().await
}

fn print_stats(
    config: &Config,
    filters: &Filter,
    mut w: impl Write,
) -> Result<Summary, Terminated> {
    let mut delim = "";
    for board in leaderboard::parse_leaderboards_from_fs(config, filters)? {
        write!(w, "{delim}").or_wrap()?;
        write!(w, "{board}").or_wrap()?;
        delim = "\n=====================================================\n\n";
    }

    Ok(Summary::Success)
}

fn filter_puzzles(solvers: &[Solver], filter: &Filter) -> Vec<(Solver, Parts)> {
    solvers
        .iter()
        .filter_map(|solver| {
            let year = solver.year();
            let day = solver.day();

            let has_p1 = filter.matches_year_day_part(year, day, P1);
            let has_p2 = filter.matches_year_day_part(year, day, P2);

            let parts = match (has_p1, has_p2) {
                (false, false) => return None,
                (true, false) => Parts::First,
                (false, true) => Parts::Second,
                (true, true) => Parts::Both,
            };

            Some((solver.clone(), parts))
        })
        .collect()
}

fn spawn_actors(
    config: Config,
    puzzles: Vec<(Solver, Parts)>,
    tx_ui: mpsc::Sender<Event>,
) {
    let solver = Runner::spawn(tx_ui.clone());
    let _downloader = Downloader::spawn(config, puzzles, solver.tx(), tx_ui);
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use indoc::indoc;
    use itertools::{izip, Itertools};
    use test_case::test_case;
    use tokio_stream::{wrappers::ReceiverStream, StreamExt};

    use ident::{Day, FilterTerm, Id, Year, D04, D05, D06};
    use solver::{State, Step};

    use super::*;

    fn mock_prep_ok(_input: &str) -> Result<String> {
        Ok(String::from("MOCK_PARSED_INPUT"))
    }

    fn mock_prep_err(_input: &str) -> Result<String> {
        Err(err!("Parser failed, so both solvers cannot run"))
    }

    fn mock_prep_panic(_input: &str) -> Result<String> {
        panic!("Mock panic")
    }

    fn mock_ok_1(data: &str) -> Result<String> {
        Ok(data.replace("PARSED_INPUT", "PUZZLE_ANSWER"))
    }

    fn mock_ok_2(data: &str) -> Result<usize> {
        // Let's block the thread and async runtime and make our tests slow.
        // Yeah, this is bad, but I'd like to see the tick events get fired.
        std::thread::sleep(std::time::Duration::from_millis(500));

        Ok(data.len())
    }

    fn mock_err(_: &str) -> Result<usize> {
        Err(err!("This mock solver must fail"))
    }

    fn mock_panic(_: &str) -> Result<usize> {
        panic!("Mock panic")
    }

    #[test_case(&["y21d03p1"], &[(Y21, D03, Parts::First)])]
    #[test_case(&["y21d01p2"], &[(Y21, D01, Parts::Second)])]
    #[test_case(&["y21d02"], &[(Y21, D02, Parts::Both)])]
    #[test_case(&["y21d02p1", "y21d02p2"], &[(Y21, D02, Parts::Both)])]
    #[test_case(&["y21d01p1", "y21d01p1"], &[(Y21, D01, Parts::First)])]
    #[test_case(&["d02p2"], &[
        (Y21, D02, Parts::Second),
        (Y24, D02, Parts::Second),
    ])]
    #[test_case(&[], &[
        (Y21, D01, Parts::Both),
        (Y21, D02, Parts::Both),
        (Y21, D03, Parts::Both),
        (Y23, D03, Parts::Both),
        (Y23, D15, Parts::Both),
        (Y24, D01, Parts::Both),
        (Y24, D02, Parts::Both),
        (Y24, D03, Parts::Both),
        (Y24, D04, Parts::Both),
    ])]
    fn init_from_filter(filters: &[&str], expected: &[(Year, Day, Parts)]) {
        let filter = Filter::from(
            filters
                .iter()
                .map(|text| text.parse().unwrap())
                .collect_vec(),
        );

        let puzzles = super::filter_puzzles(SOLVERS, &filter);

        assert_eq!(expected.len(), puzzles.len());
        for (expected, puzzle) in izip!(expected, puzzles) {
            let (solver, parts) = puzzle;
            let y = solver.year();
            let d = solver.day();
            let p = parts;
            assert_eq!(expected, &(y, d, p));
        }
    }

    #[tokio::test]
    #[cfg_attr(miri, ignore)] // Because of `RepoDir`/`create_config_for`
    async fn run_loop_sends_events_to_ui() -> Result<()> {
        let solvers = &[
            solver!(Y21, D01, mock_ok_1, mock_ok_2, mock_prep_ok),
            solver!(Y21, D02, mock_err, mock_ok_1, mock_prep_ok),
            solver!(Y21, D03, mock_panic, mock_panic, mock_prep_err),
            solver!(Y21, D04, mock_ok_1, mock_panic, mock_prep_ok),
            solver!(Y21, D05, mock_panic, mock_err, mock_prep_ok),
            solver!(Y21, D06, mock_panic, mock_panic, mock_prep_panic),
        ];

        // If we don't create the puzzle input files for those solvers,
        // the system would try to download them automatically.

        let tempdir = fs::tempdir()?;

        let mut path = tempdir.path().to_path_buf();
        path.push("personal_puzzle_inputs");
        std::fs::create_dir(&path).unwrap();

        for d in ["d01", "d02", "d03", "d04", "d05", "d06"] {
            let mut path = path.clone();
            path.push(format!("y21{d}_personal_puzzle_input.txt"));
            std::fs::write(&path, "").unwrap();
        }

        let config = fs::create_config_for(&tempdir)?;

        let filter = Filter::from(vec![
            FilterTerm::from_str("y21d01")?,
            FilterTerm::from_str("y21d02")?,
            FilterTerm::from_str("y21d03")?,
            FilterTerm::from_str("y21d04")?,
            FilterTerm::from_str("y21d05")?,
            FilterTerm::from_str("y21d06")?,
        ]);

        let mut got_d01dl_skipped = false;
        let mut got_d01p0_start = false;
        let mut got_d01p0_done = false;
        let mut got_d01p1_start = false;
        let mut got_d01p1_done = false;
        let mut got_d01p2_start = false;
        let mut got_d01p2_done = false;
        let mut got_d02dl_skipped = false;
        let mut got_d02p0_start = false;
        let mut got_d02p0_done = false;
        let mut got_d02p1_start = false;
        let mut got_d02p1_done = false;
        let mut got_d02p2_start = false;
        let mut got_d02p2_done = false;
        let mut got_d03dl_skipped = false;
        let mut got_d03p0_start = false;
        let mut got_d03p0_done = false;
        let mut got_d04dl_skipped = false;
        let mut got_d04p0_start = false;
        let mut got_d04p0_done = false;
        let mut got_d04p1_start = false;
        let mut got_d04p1_done = false;
        let mut got_d04p2_start = false;
        let mut got_d04p2_done = false;
        let mut got_d05dl_skipped = false;
        let mut got_d05p0_start = false;
        let mut got_d05p0_done = false;
        let mut got_d05p1_start = false;
        let mut got_d05p1_done = false;
        let mut got_d05p2_start = false;
        let mut got_d05p2_done = false;
        let mut got_d06dl_skipped = false;
        let mut got_d06p0_start = false;
        let mut got_d06p0_done = false;

        let puzzles = super::filter_puzzles(solvers, &filter);

        for e in spawn_actors_and_await_events(config, puzzles).await {
            use State::*;
            use Step::*;
            match e {
                Event {
                    year: Y21,
                    day: D01,
                    step: Download,
                    state: Skipped,
                } => {
                    assert!(!got_d01dl_skipped);
                    got_d01dl_skipped = true;
                }

                Event {
                    year: Y21,
                    day: D01,
                    step: Preproc,
                    state: Started(_),
                } => {
                    assert!(!got_d01p0_start);
                    got_d01p0_start = true;
                }
                Event {
                    year: Y21,
                    day: D01,
                    step: Preproc,
                    state: Done(_, Ok(None)),
                } => {
                    assert!(!got_d01p0_done);
                    got_d01p0_done = true;
                }

                Event {
                    year: Y21,
                    day: D01,
                    step: Part1,
                    state: Started(_),
                } => {
                    assert!(got_d01p0_done);
                    assert!(!got_d01p1_start);
                    got_d01p1_start = true;
                }
                Event {
                    year: Y21,
                    day: D01,
                    step: Part1,
                    state: Done(_, Ok(Some(answer))),
                } => {
                    assert!(got_d01p0_done);
                    assert!(!got_d01p1_done);
                    assert_eq!(answer.to_string(), "MOCK_PUZZLE_ANSWER");
                    got_d01p1_done = true;
                }

                Event {
                    year: Y21,
                    day: D01,
                    step: Part2,
                    state: Started(_),
                } => {
                    assert!(got_d01p0_done);
                    assert!(!got_d01p2_start);
                    got_d01p2_start = true;
                }
                Event {
                    year: Y21,
                    day: D01,
                    step: Part2,
                    state: Done(_, Ok(Some(answer))),
                } => {
                    assert!(got_d01p0_done);
                    assert!(!got_d01p2_done);
                    assert_eq!(answer.to_string(), "17");
                    got_d01p2_done = true;
                }

                Event {
                    year: Y21,
                    day: D02,
                    step: Download,
                    state: Skipped,
                } => {
                    assert!(!got_d02dl_skipped);
                    got_d02dl_skipped = true;
                }

                Event {
                    year: Y21,
                    day: D02,
                    step: Preproc,
                    state: Started(_),
                } => {
                    assert!(!got_d02p0_start);
                    got_d02p0_start = true;
                }
                Event {
                    year: Y21,
                    day: D02,
                    step: Preproc,
                    state: Done(_, Ok(None)),
                } => {
                    assert!(!got_d02p0_done);
                    got_d02p0_done = true;
                }

                Event {
                    year: Y21,
                    day: D02,
                    step: Part1,
                    state: Started(_),
                } => {
                    assert!(got_d02p0_done);
                    assert!(!got_d02p1_start);
                    got_d02p1_start = true;
                }
                Event {
                    year: Y21,
                    day: D02,
                    step: Part1,
                    state: Done(_, Err(err)),
                } => {
                    assert!(got_d02p0_done);
                    assert!(!got_d02p1_done);
                    assert_eq!(err.to_string(), "This mock solver must fail");
                    got_d02p1_done = true;
                }

                Event {
                    year: Y21,
                    day: D02,
                    step: Part2,
                    state: Started(_),
                } => {
                    assert!(got_d02p0_done);
                    assert!(!got_d02p2_start);
                    got_d02p2_start = true;
                }
                Event {
                    year: Y21,
                    day: D02,
                    step: Part2,
                    state: Done(_, Ok(Some(answer))),
                } => {
                    assert!(got_d02p0_done);
                    assert!(!got_d02p2_done);
                    assert_eq!(answer.to_string(), "MOCK_PUZZLE_ANSWER");
                    got_d02p2_done = true;
                }

                Event {
                    year: Y21,
                    day: D03,
                    step: Download,
                    state: Skipped,
                } => {
                    assert!(!got_d03dl_skipped);
                    got_d03dl_skipped = true;
                }

                Event {
                    year: Y21,
                    day: D03,
                    step: Preproc,
                    state: Started(_),
                } => {
                    assert!(!got_d03p0_start);
                    got_d03p0_start = true;
                }
                Event {
                    year: Y21,
                    day: D03,
                    step: Preproc,
                    state: Done(_, Err(err)),
                } => {
                    assert!(!got_d03p0_done);
                    assert_eq!(
                        err.to_string(),
                        "Parser failed, so both solvers cannot run"
                    );
                    got_d03p0_done = true;
                }

                Event {
                    year: Y21,
                    day: D04,
                    step: Download,
                    state: Skipped,
                } => {
                    assert!(!got_d04dl_skipped);
                    got_d04dl_skipped = true;
                }

                Event {
                    year: Y21,
                    day: D04,
                    step: Preproc,
                    state: Started(_),
                } => {
                    assert!(!got_d04p0_start);
                    got_d04p0_start = true;
                }
                Event {
                    year: Y21,
                    day: D04,
                    step: Preproc,
                    state: Done(_, Ok(None)),
                } => {
                    assert!(!got_d04p0_done);
                    got_d04p0_done = true;
                }

                Event {
                    year: Y21,
                    day: D04,
                    step: Part1,
                    state: Started(_),
                } => {
                    assert!(got_d04p0_done);
                    assert!(!got_d04p1_start);
                    got_d04p1_start = true;
                }
                Event {
                    year: Y21,
                    day: D04,
                    step: Part1,
                    state: Done(_, Ok(Some(answer))),
                } => {
                    assert!(!got_d04p1_done);
                    assert_eq!(answer.to_string(), "MOCK_PUZZLE_ANSWER");
                    got_d04p1_done = true;
                }

                Event {
                    year: Y21,
                    day: D04,
                    step: Part2,
                    state: Started(_),
                } => {
                    assert!(!got_d04p2_start);
                    got_d04p2_start = true;
                }
                Event {
                    year: Y21,
                    day: D04,
                    step: Part2,
                    state: Done(_, Err(err)),
                } => {
                    assert!(!got_d04p2_done);

                    let msg = err.to_string();
                    assert_eq!(msg, "PANIC");

                    got_d04p2_done = true;
                }

                Event {
                    year: Y21,
                    day: D05,
                    step: Download,
                    state: Skipped,
                } => {
                    assert!(!got_d05dl_skipped);
                    got_d05dl_skipped = true;
                }

                Event {
                    year: Y21,
                    day: D05,
                    step: Preproc,
                    state: Started(_),
                } => {
                    assert!(!got_d05p0_start);
                    got_d05p0_start = true;
                }
                Event {
                    year: Y21,
                    day: D05,
                    step: Preproc,
                    state: Done(_, Ok(None)),
                } => {
                    assert!(!got_d05p0_done);
                    got_d05p0_done = true;
                }

                Event {
                    year: Y21,
                    day: D05,
                    step: Part1,
                    state: Started(_),
                } => {
                    assert!(!got_d05p1_start);
                    got_d05p1_start = true;
                }
                Event {
                    year: Y21,
                    day: D05,
                    step: Part1,
                    state: Done(_, Err(err)),
                } => {
                    assert!(!got_d05p1_done);

                    let msg = err.to_string();
                    assert_eq!(msg, "PANIC");

                    got_d05p1_done = true;
                }

                Event {
                    year: Y21,
                    day: D05,
                    step: Part2,
                    state: Started(_),
                } => {
                    assert!(got_d05p0_done);
                    assert!(!got_d05p2_start);
                    got_d05p2_start = true;
                }
                Event {
                    year: Y21,
                    day: D05,
                    step: Part2,
                    state: Done(_, Err(err)),
                } => {
                    assert!(!got_d05p2_done);
                    assert_eq!(err.to_string(), "This mock solver must fail");
                    got_d05p2_done = true;
                }

                Event {
                    year: Y21,
                    day: D06,
                    step: Download,
                    state: Skipped,
                } => {
                    assert!(!got_d06dl_skipped);
                    got_d06dl_skipped = true;
                }

                Event {
                    year: Y21,
                    day: D06,
                    step: Preproc,
                    state: Started(_),
                } => {
                    assert!(!got_d06p0_start);
                    got_d06p0_start = true;
                }
                Event {
                    year: Y21,
                    day: D06,
                    step: Preproc,
                    state: Done(_, Err(err)),
                } => {
                    assert!(!got_d06p0_done);

                    let msg = err.to_string();
                    assert_eq!(msg, "PANIC");

                    got_d06p0_done = true;
                }

                others => panic!("Unexpected event: {others:?}"),
            }
        }

        assert!(got_d01dl_skipped);
        assert!(got_d01p0_start);
        assert!(got_d01p0_done);
        assert!(got_d01p1_start);
        assert!(got_d01p1_done);
        assert!(got_d01p2_start);
        assert!(got_d01p2_done);
        assert!(got_d02dl_skipped);
        assert!(got_d02p0_start);
        assert!(got_d02p0_done);
        assert!(got_d02p1_start);
        assert!(got_d02p1_done);
        assert!(got_d02p2_start);
        assert!(got_d02p2_done);
        assert!(got_d03dl_skipped);
        assert!(got_d03p0_start);
        assert!(got_d03p0_done);
        assert!(got_d04dl_skipped);
        assert!(got_d04p0_start);
        assert!(got_d04p0_done);
        assert!(got_d04p1_start);
        assert!(got_d04p1_done);
        assert!(got_d04p2_start);
        assert!(got_d04p2_done);
        assert!(got_d05dl_skipped);
        assert!(got_d05p0_start);
        assert!(got_d05p0_done);
        assert!(got_d05p1_start);
        assert!(got_d05p1_done);
        assert!(got_d05p2_start);
        assert!(got_d05p2_done);
        assert!(got_d06dl_skipped);
        assert!(got_d06p0_start);
        assert!(got_d06p0_done);

        Ok(())
    }

    // TODO: Add macro to generate cases from `const SOLVERS` automatically.
    #[test_case("y21d01p1")]
    #[test_case("y21d01p2")]
    #[test_case("y21d02p1")]
    #[test_case("y21d02p2")]
    #[test_case("y21d03p1")]
    #[test_case("y21d03p2")]
    #[test_case("y23d03p1")]
    #[test_case("y23d03p2")]
    #[test_case("y23d15p1")]
    #[test_case("y23d15p2")]
    #[test_case("y24d01p1")]
    #[test_case("y24d01p2")]
    #[test_case("y24d02p1")]
    #[test_case("y24d02p2")]
    #[test_case("y24d03p1")]
    #[test_case("y24d03p2")]
    #[test_case("y24d04p1")]
    #[test_case("y24d04p2")]
    #[tokio::test]
    #[ignore] // Requires manually saving the personal puzzles answers before
    async fn solve_personal_inputs(filter: &str) -> Result<()> {
        let Id((y, d, p)) = filter.parse()?;
        let filter = Filter::from(vec![filter.parse()?]);

        let config = Config::from_env_or_defaults()?; // Use the real ones here

        let expected_answer = config.personal_puzzle_answer(y, d, p)?;

        let puzzles = super::filter_puzzles(SOLVERS, &filter);

        let events = spawn_actors_and_await_events(config, puzzles).await;
        let answer = events
            .iter()
            .find_map(|e| match e {
                Event {
                    year,
                    day,
                    step,
                    state: State::Done(_, Ok(answer)),
                } if *year == y && *day == d && *step == p.into() => {
                    answer.as_ref()
                }
                _ => None,
            })
            .unwrap();
        assert_eq!(answer.to_string(), expected_answer);

        Ok(())
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `RepoDir`/`create_config_for`
    fn print_leaderboard_y21() -> Result<()> {
        let expected = indoc! {"\
            Advent of Code 2021 - Personal Leaderboard Statistics

                  --------Part 1--------   --------Part 2--------
            Day       Time   Rank  Score       Time   Rank  Score
             25       >24h  13830      0       >24h  10293      0
             24       >24h   6382      0       >24h   6250      0
             23       >24h  11373      0       >24h  11998      0
             22       >24h  20610      0       >24h  14796      0
             21       >24h  24976      0       >24h  18838      0
             20       >24h  22022      0       >24h  21682      0
             19       >24h  16028      0       >24h  15785      0
             18   05:45:04   4263      0   05:57:35   4200      0
             17   01:42:33   5577      0   01:47:33   4755      0
             16       >24h  32382      0       >24h  30839      0
             15   00:41:26   2841      0   01:13:18   2453      0
             14   00:44:13   6857      0   09:30:55  17848      0
             13   00:29:47   3233      0   00:39:18   3149      0
             12   01:13:40   5662      0   01:25:08   4642      0
             11   00:30:47   2625      0   00:40:21   3213      0
             10   00:18:20   4023      0   00:33:42   4230      0
              9   00:44:45   8618      0   03:46:18  13025      0
              8   00:27:01   7501      0   02:15:34   6812      0
              7   00:21:00   8179      0   00:25:22   6415      0
              6   00:14:37   5023      0   00:29:07   3395      0
              5   00:45:25   6042      0   01:01:39   5242      0
              4   01:07:48   6677      0   01:25:47   6346      0
              3   00:24:26   8496      0   01:04:05   7054      0
              2   03:39:44  34128      0   03:50:44  32547      0
              1   00:20:32   6893      0   00:24:50   5662      0
            -----------------------------------------------------
            MIN   00:14:37   2625      0   00:24:50   2453      0
            MED   01:07:48   6893      0   02:15:34   6415      0
            MAX       >24h  34128      0       >24h  32547      0
        "};

        verify_stats(&["y21"], expected)
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `RepoDir`/`create_config_for`
    fn print_leaderboards_y20d01_y21d05_y21d06() -> Result<()> {
        // Note: Each board will be formatted with it's own width,
        // determined by the length (in chars) of its largest rank.
        let expected = indoc! {"\
            Advent of Code 2020 - Personal Leaderboard Statistics

                  --------Part 1---------   -------Part 2--------
            Day       Time    Rank  Score       Time  Rank  Score
              1       >24h  187123      0          -     -      -

            =====================================================

            Advent of Code 2021 - Personal Leaderboard Statistics

                  -------Part 1--------   -------Part 2--------
            Day       Time  Rank  Score       Time  Rank  Score
              6   00:14:37  5023      0   00:29:07  3395      0
              5   00:45:25  6042      0   01:01:39  5242      0
            ---------------------------------------------------
            MIN   00:14:37  5023      0   00:29:07  3395      0
            MED   00:30:01  5533      0   00:45:23  4319      0
            MAX   00:45:25  6042      0   01:01:39  5242      0
        "};

        verify_stats(&["y20d01", "y21d05", "y21d06"], expected)
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `RepoDir`/`create_config_for`
    fn print_all_leaderboards() -> Result<()> {
        let expected = indoc! {"\
            Advent of Code 2020 - Personal Leaderboard Statistics

                  --------Part 1---------   -------Part 2--------
            Day       Time    Rank  Score       Time  Rank  Score
              1       >24h  187123      0          -     -      -

            =====================================================

            Advent of Code 2021 - Personal Leaderboard Statistics

                  --------Part 1--------   --------Part 2--------
            Day       Time   Rank  Score       Time   Rank  Score
             25       >24h  13830      0       >24h  10293      0
             24       >24h   6382      0       >24h   6250      0
             23       >24h  11373      0       >24h  11998      0
             22       >24h  20610      0       >24h  14796      0
             21       >24h  24976      0       >24h  18838      0
             20       >24h  22022      0       >24h  21682      0
             19       >24h  16028      0       >24h  15785      0
             18   05:45:04   4263      0   05:57:35   4200      0
             17   01:42:33   5577      0   01:47:33   4755      0
             16       >24h  32382      0       >24h  30839      0
             15   00:41:26   2841      0   01:13:18   2453      0
             14   00:44:13   6857      0   09:30:55  17848      0
             13   00:29:47   3233      0   00:39:18   3149      0
             12   01:13:40   5662      0   01:25:08   4642      0
             11   00:30:47   2625      0   00:40:21   3213      0
             10   00:18:20   4023      0   00:33:42   4230      0
              9   00:44:45   8618      0   03:46:18  13025      0
              8   00:27:01   7501      0   02:15:34   6812      0
              7   00:21:00   8179      0   00:25:22   6415      0
              6   00:14:37   5023      0   00:29:07   3395      0
              5   00:45:25   6042      0   01:01:39   5242      0
              4   01:07:48   6677      0   01:25:47   6346      0
              3   00:24:26   8496      0   01:04:05   7054      0
              2   03:39:44  34128      0   03:50:44  32547      0
              1   00:20:32   6893      0   00:24:50   5662      0
            -----------------------------------------------------
            MIN   00:14:37   2625      0   00:24:50   2453      0
            MED   01:07:48   6893      0   02:15:34   6415      0
            MAX       >24h  34128      0       >24h  32547      0
        "};

        verify_stats(&[], expected)
    }

    async fn spawn_actors_and_await_events(
        config: Config,
        puzzles: Vec<(Solver, Parts)>,
    ) -> Vec<Event> {
        let (tx, rx) = mpsc::channel(1);
        let rx = ReceiverStream::new(rx);

        spawn_actors(config, puzzles, tx);

        rx.collect().await
    }

    fn verify_stats(filters: &[&str], expected_output: &str) -> Result<()> {
        let filter = Filter::from(
            filters
                .iter()
                .map(|text| text.parse().unwrap())
                .collect_vec(),
        );

        let tempdir = fs::tempdir()?;

        let mut stats_dir = tempdir.path().to_path_buf();
        stats_dir.push("personal_leaderboard_statistics");
        std::fs::create_dir(&stats_dir).unwrap();

        let mut y20_stats_file = stats_dir.clone();
        y20_stats_file.push("y20_personal_leaderboard_statistics.txt");
        std::fs::write(&y20_stats_file, indoc! {"\
                  --------Part 1---------   -------Part 2--------
            Day       Time    Rank  Score       Time  Rank  Score
              1       >24h  187123      0          -     -      -
        "})
        .unwrap();

        let mut y21_stats_file = stats_dir.clone();
        y21_stats_file.push("y21_personal_leaderboard_statistics.txt");
        std::fs::write(&y21_stats_file, indoc! {"\
                  --------Part 1--------   --------Part 2--------
            Day       Time   Rank  Score       Time   Rank  Score
             25       >24h  13830      0       >24h  10293      0
             24       >24h   6382      0       >24h   6250      0
             23       >24h  11373      0       >24h  11998      0
             22       >24h  20610      0       >24h  14796      0
             21       >24h  24976      0       >24h  18838      0
             20       >24h  22022      0       >24h  21682      0
             19       >24h  16028      0       >24h  15785      0
             18   05:45:04   4263      0   05:57:35   4200      0
             17   01:42:33   5577      0   01:47:33   4755      0
             16       >24h  32382      0       >24h  30839      0
             15   00:41:26   2841      0   01:13:18   2453      0
             14   00:44:13   6857      0   09:30:55  17848      0
             13   00:29:47   3233      0   00:39:18   3149      0
             12   01:13:40   5662      0   01:25:08   4642      0
             11   00:30:47   2625      0   00:40:21   3213      0
             10   00:18:20   4023      0   00:33:42   4230      0
              9   00:44:45   8618      0   03:46:18  13025      0
              8   00:27:01   7501      0   02:15:34   6812      0
              7   00:21:00   8179      0   00:25:22   6415      0
              6   00:14:37   5023      0   00:29:07   3395      0
              5   00:45:25   6042      0   01:01:39   5242      0
              4   01:07:48   6677      0   01:25:47   6346      0
              3   00:24:26   8496      0   01:04:05   7054      0
              2   03:39:44  34128      0   03:50:44  32547      0
              1   00:20:32   6893      0   00:24:50   5662      0
        "})
        .unwrap();

        let config = fs::create_config_for(&tempdir)?;
        let mut buffer = Vec::new();
        super::print_stats(&config, &filter, &mut buffer)
            .or_wrap_with(|| "print_stats() failed")?;
        let actual_output = String::from_utf8(buffer).unwrap();

        assert_eq!(actual_output, expected_output);
        Ok(())
    }
}
