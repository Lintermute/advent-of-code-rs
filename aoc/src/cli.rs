use crate::ident::{Filter, FilterTerm};

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Command {
    Login,
    Logout,
    Solve(Filter),
    Stats(Filter),
    // Render(Id<(Year, Day, Part)>),
    Render(Filter),
}

#[derive(clap::Parser, Debug, Clone, PartialEq, Hash, Eq)]
struct CliArgs {
    #[command(subcommand)]
    command: Option<CliCommand>,
}

/// Solve Advent of Code puzzles and print your personal leaderboard statistics.
#[derive(clap::Subcommand, Debug, Clone, PartialEq, Hash, Eq)]
enum CliCommand {
    /// Save your adventofcode.com session cookie
    /// to download puzzle inputs automatically.
    ///
    /// To have your personal puzzle inputs downloaded automatically,
    /// you need to enter the value of your adventofcode.com session cookie.
    /// Run this command to guide you through the steps.
    ///
    /// If you don't want to enter your session cookie into this program,
    /// you'll need to download your personal puzzle inputs manually.
    /// For example, the input for year 2021 day 1 MUST be named
    /// `y21d01_personal_puzzle_input.txt` and that file must be located in
    /// `$CACHE_DIR/advent_of_code/personal_puzzle_inputs`, where `$CACHE_DIR`
    /// is `$XDG_CACHE_HOME` or `$HOME/.cache` on Linux
    /// (e.g. `/home/you/.cache`),
    /// `$HOME/Library/Caches` on macOS (e.g. `/Users/you/Library/Caches`), or
    /// `{FOLDERID_LocalAppData}` on Windows
    /// (e.g. `C:\\Users\\you\\AppData\\Local`).
    Login,
    /// Remove your adventofcode.com session cookie from the file system.
    ///
    /// Of course, this command will NOT remove the session cookie from any
    /// of your browsers. If will only remove the session cookie from
    /// the configuration directory of this program.
    Logout,
    /// Solve Advent of Code puzzles (default command).
    Solve(Puzzles),
    /// Print your personal leaderboard statistics.
    ///
    /// To run this command, you'll have to copy and paste your
    /// personal leaderboard from adventofcode.com manually.
    /// Please put your personal leaderboard files into the directory
    /// `$DATA_DIR/advent_of_code/personal_leaderboard_statistics`, where
    /// `$DATA_DIR` is `$XDG_DATA_HOME` or `$HOME/.local/share` on Linux,
    /// `$HOME/Library/Application Support` on macOS, and
    /// `{FOLDERID_RoamingAppData}` (i.e. `C:\Users\You\AppData\Roaming`)
    /// on Windows.
    /// The files must be named `y21_personal_leaderboard_statistics.txt`
    /// for year 2021, for example.
    Stats(Puzzles),
    // TODO
    Render(Puzzles),
}

#[derive(clap::Args, Debug, Clone, PartialEq, Hash, Eq)]
struct Puzzles {
    /// Puzzles to select (defaults to all).
    ///
    /// You can pass one or more puzzle filters to select a certain
    /// subset of puzzles. A filter consists of year, day, and part number,
    /// and looks like `y21d01p2`. Missing components are treated as wildcard.
    /// For example:
    /// `y21d01p2` selects year 2021 day 1 part 2.
    /// `y21d01` selects both parts.
    /// `y21` selects all puzzles from year 2021.
    ///
    /// A puzzle will be selected if it matches at least one filter.
    /// For example, `y21 d01` selects all puzzles from year 2021,
    /// as well as day 1 of any other year.
    puzzles: Vec<FilterTerm>,
}

impl From<Puzzles> for Filter {
    fn from(val: Puzzles) -> Self {
        val.puzzles.into()
    }
}

pub fn parse_args_from_env_or_exit() -> Command {
    parse_or_exit(std::env::args_os())
}

fn parse_or_exit<IntoIter, T>(args: IntoIter) -> Command
where
    IntoIter: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    use clap::Parser;
    let args = CliArgs::parse_from(args);
    match args.command {
        None => Command::Solve(Filter::default()),
        Some(CliCommand::Login) => Command::Login,
        Some(CliCommand::Logout) => Command::Logout,
        Some(CliCommand::Solve(puzzles)) => {
            Command::Solve(Filter::from(puzzles))
        }
        Some(CliCommand::Render(puzzles)) => {
            // TODO
            Command::Render(Filter::from(puzzles))
        }
        Some(CliCommand::Stats(puzzles)) => {
            Command::Stats(Filter::from(puzzles))
        }
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    // First parameter references name of the program/binary.
    // It does not need to be checked. Thus, we use an empty string here.

    #[test]
    fn parse_login() {
        match super::parse_or_exit(["", "login"]) {
            Command::Login => (),
            others => panic!("Unexpected result: {others:?}"),
        };
    }

    #[test]
    fn parse_logout() {
        match super::parse_or_exit(["", "logout"]) {
            Command::Logout => (),
            others => panic!("Unexpected result: {others:?}"),
        };
    }

    #[test_case(
        &[""],
        vec![];
        "`solve` is default subcommand"
    )]
    #[test_case(
        &["", "solve"],
        vec![];
        "Defaults to no filters (implicit wildcard)"
    )]
    #[test_case(
        &["", "solve", "*"],
        vec!["*".parse().unwrap()];
        "Supports explicit wildcard (no filters)"
    )]
    #[test_case(
        &["", "solve", "y21d01"],
        vec!["y21d01".parse().unwrap()];
        "Can select puzzle of the day"
    )]
    #[test_case(
        &["", "solve", "y21d01p2"],
        vec!["y21d01p2".parse().unwrap()];
        "Can skip parts of the daily puzzle"
    )]
    #[test_case(
        &["", "solve", "y21"],
        vec!["y21".parse().unwrap()];
        "Year can be singled out"
    )]
    #[test_case(
        &["", "solve", "y21", "d03"],
        vec![
            "y21".parse().unwrap(),
            "d03".parse().unwrap(),
        ];
        "Supports multiple filters"
    )]
    fn parse_solve(args: &[&str], expected: Vec<FilterTerm>) {
        let expected = Filter::from(expected);
        let actual = match super::parse_or_exit(args) {
            Command::Solve(actual) => actual,
            others => panic!("Unexpected result: {others:?}"),
        };

        assert_eq!(actual, expected);
    }

    #[test_case(
        &["", "stats"],
        vec![];
        "Defaults to no filters (implicit wildcard)"
    )]
    #[test_case(
        &["", "stats", "y21d01"],
        vec!["y21d01".parse().unwrap()];
        "Supports single filter"
    )]
    #[test_case(
        &["", "stats", "y21", "d01"],
        vec![
            "y21".parse().unwrap(),
            "d01".parse().unwrap(),
        ];
        "Supports multiple filters"
    )]
    fn parse_stats(args: &[&str], expected: Vec<FilterTerm>) {
        let expected = Filter::from(expected);
        let actual = match super::parse_or_exit(args) {
            Command::Stats(actual) => actual,
            others => panic!("Unexpected result: {others:?}"),
        };

        assert_eq!(actual, expected);
    }
}
