use std::{
    fmt::{Debug, Display},
    time::{Duration, Instant},
};

use lazy_errors::Result;
use tokio::sync::mpsc;

use crate::ident::{Day, Part, Year};

/// Creates a [`Solver`] for a certain Advent of Code puzzle.
///
/// To solve a puzzle, a separate solver function is required for each part.
/// In its simplest form, this macro creates an object that will run
/// the two solver functions in parallel. Each of those functions
/// will then need to parse the input data by itself:
///
/// ```
/// use aoc::{day::*, solver, year::*};
///
/// fn y21d01p1(_input: &str) -> Result<u8, String> {
///     // Read personal puzzle input from file.
///     Ok(42)
/// }
///
/// fn y21d01p2(_input: &str) -> Result<u8, String> {
///     // Read personal puzzle input from file here, too.
///     Ok(42 + 42)
/// }
///
/// let s = solver!(Y21, D01, y21d01p1, y21d01p2);
/// ```
///
/// For some puzzles, it makes sense to parse the input only once:
///
/// ```
/// use aoc::{day::*, solver, year::*};
///
/// fn y21d01_preproc(_input: &str) -> Result<u8, String> {
///     // Read personal puzzle input from file.
///     Ok(42)
/// }
///
/// fn y21d01p1(input: &u8) -> Result<u8, String> {
///     Ok(*input)
/// }
///
/// fn y21d01p2(input: &u8) -> Result<u8, String> {
///     Ok(*input + *input)
/// }
///
/// let s = solver!(Y21, D01, y21d01p1, y21d01p2, y21d01_preproc);
/// ```
// Implementation Notes:
//
// It may look like too many details leak into the public API of this module.
// It would be nice if there is a way to reduce that surface.
// However, I did not find a better way that satisifies all constraints:
// - The result of `solver!` must be `const`, so we can have `const SOLVERS:
//   &[Solver] = &[ … ];` in main.rs. Note that closures are `const` IFF they do
//   not capture any environment. By using `$year` and `$day` literally in the
//   closure, we avoid capturing any environment.
// - It must be possible for `$parser` to return _any_ `Result<I>` and for the
//   solvers to accept any `T` if `I: AsRef<T>`. At the same time, the value
//   created by `solver!` must not have generic type parameters, because they'd
//   differ between puzzles and must fit the signature above. By passing the
//   result of the parser to the solvers in this closure, we do not need to
//   specify any generic types, and instead just have a simple, common
//   `fn(Parts, Input, Tx)` signature in _any_ solver object (that does all the
//   parsing and solving).
#[macro_export]
macro_rules! solver {
    ($year:ident, $day:ident, $solver1:path, $solver2:path) => {{
        let runner: $crate::solver::RunnerFn = |parts, input, tx| {
            $crate::runner::skip_preproc($year, $day, &tx)?;
            let p1 = || $solver1(&input);
            let p2 = || $solver2(&input);
            $crate::runner::solve($year, $day, p1, p2, parts, &tx)
        };
        $crate::solver::Solver::new($year, $day, runner)
    }};

    ($year:ident, $day:ident, $solver1:path, $solver2:path, $parser:expr) => {{
        let runner: $crate::solver::RunnerFn = |parts, input, tx| {
            match $crate::runner::preprocess($year, $day, $parser, input, &tx)?
            {
                None => Ok(()), // Parsing failed; will be handled by UI
                Some(input) => {
                    let p1 = || $solver1(&input);
                    let p2 = || $solver2(&input);
                    $crate::runner::solve($year, $day, p1, p2, parts, &tx)
                }
            }
        };
        $crate::solver::Solver::new($year, $day, runner)
    }};
}

/// Represents the solver functions (parts one and two),
/// and optionally a puzzle input parser function,
/// for a certain Advent of Code puzzle.
///
/// You generally don't want to create values of this type manually
/// for reasons explained below. Please use the [`solver!`] macro.
///
/// This type supports solver functions with arbitrary result types,
/// as long as they return a [`Result`] containing a [`PuzzleAnswer`].
/// If a dedicated puzzle input parsing function is used,
/// that function may return any type `Result<T>`
/// as long as both solver functions accept `&T` as parameter.
/// To achieve this kind of flexibility,
/// one would usually make `Solver` accept generic type parameters.
/// However, to store a [`Vec`] or slice of [`Solver`]s,
/// generic type parameters would require us to [`Box`] each [`Solver`].
/// This would not be possible at compile time.
/// Since all solver and parsing functions are available at compile time,
/// we made sure we also can define a `const &[Solver]`.
/// We achieve this by relying on `fn(...) -> Result<...>` function pointers
/// that don't accept/return generic data, such as [`RunnerFn`].
/// Instead of actual parser/solver functions with generic type parameters,
/// [`Solver`] stores a [`RunnerFn`] function pointer.
/// Thus, its possible to create [`Solver`] values at compile time.
/// Each [`RunnerFn`] is actually a closure that
/// (1) sends the parser function's generic result to the solver functions, and
/// (2) sends the puzzle answers through a channel for the UI to interpret.
/// As long as closures don't capture variables, Rust allows them to be coerced
/// into function pointers. The [`solver!`] macro does just that.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct Solver {
    year:   Year,
    day:    Day,
    runner: RunnerFn,
}

/// Indicates whether to run only the first or only the second part
/// of an Advent of Code puzzle, or both.
///
/// Note: This type implements `Copy`.
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
pub enum Parts {
    First,
    Second,
    Both,
}

#[derive(Debug)]
pub struct Event {
    pub year:  Year,
    pub day:   Day,
    pub step:  Step,
    pub state: State,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub enum Step {
    Download,
    Preproc,
    Part1,
    Part2,
}

#[derive(Debug)]
pub enum State {
    Waiting,
    Skipped,
    Started(Instant),
    Done(Duration, Result<Option<Box<dyn PuzzleAnswer>>>),
}

/// Result of successfully solving an Advent of Code puzzle.
/// This trait is basically an alias that allows
/// solvers to return virtually any “regular” data type as answer,
/// such as `usize` or `String`, and even custom structs.
pub trait PuzzleAnswer: Display + Debug + Send + Sync + 'static {}

impl<T> PuzzleAnswer for T where T: Display + Debug + Send + Sync + 'static {}

#[doc(hidden)]
pub type RunnerFn = fn(Parts, &str, mpsc::Sender<Event>) -> Result<()>;

impl From<Part> for Step {
    fn from(value: Part) -> Self {
        match value {
            Part::Part1 => Step::Part1,
            Part::Part2 => Step::Part2,
        }
    }
}

impl Solver {
    #[doc(hidden)]
    pub const fn new(year: Year, day: Day, runner: RunnerFn) -> Self {
        Self { year, day, runner }
    }

    pub fn year(&self) -> Year {
        self.year
    }

    pub fn day(&self) -> Day {
        self.day
    }

    /// Run the solver, i.e. preprocessing, part one, and part two,
    /// depending on the `parts` filter and whether a separate
    /// preprocessing stage exists for this solver.
    ///
    /// This method is deliberately _not_ async, even though it sends
    /// state updates via the supplied channel. Instead, we use a blocking
    /// send and run it on a dedicated (rayon) thread. This approach reduces
    /// the risk of overcommitting the CPU and producing incorrect runtime
    /// measurements. As soon as a solver is started, it should de-facto have
    /// dedicated access to a (single) logical hardware thread.
    pub fn solve(
        &self,
        parts: Parts,
        input: &str,
        tx: mpsc::Sender<Event>,
    ) -> Result<()> {
        let f = self.runner;
        f(parts, input, tx)
    }
}

/// Returns the number of threads to use as returned from
/// [`std::thread::available_parallelism`],
/// or `1` if that function cannot determine that number.
///
/// Returning `1` in case of any issues increases the chance we may notice it.
pub fn num_threads() -> usize {
    std::thread::available_parallelism()
        .map(std::num::NonZeroUsize::get)
        .unwrap_or(1)
}
