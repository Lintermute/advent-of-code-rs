# advent-of-code-rs

Solutions to all Advent of Code puzzles I have solved and refactored so far.
Comes with a text-based runner
and can also print personal leaderboard statistics.

### Running the Advent of Code Puzzle Solvers

The runner supports flexible puzzle filters:

![`cargo run -- solve y21d01 y21d02p2 y21d03`](README-solve.gif)

Solvers are run in parallel (as long as logical CPU threads are available).

The `Prep` step is optional and runs preprocessing logic required
by both parts of a given day, such as parsing complex input data.
This logic is run at most once for each day.
Afterwards, parts 1 and 2 of that day will be run in parallel.

### Puzzle Input Downloading & Caching

Personal puzzle inputs will be downloaded automatically on-demand
if you have run `cargo run -- login` before and entered your
[adventofcode.com](https://adventofcode.com) session cookie value.
That command guides you through the necessary steps.
Your personal puzzle inputs will be cached in
`$CACHE_DIR/advent_of_code/personal_puzzle_inputs`, where `$CACHE_DIR` is:

- Linux: `$XDG_CACHE_HOME` or `$HOME/.cache` (e.g. `/home/you/.cache`)
- macOS: `$HOME/Library/Caches` (e.g. `/Users/you/Library/Caches`)
- Windows: `{FOLDERID_LocalAppData}` (e.g. `C:\Users\you\AppData\Local`)

You can also manually place your personal puzzle inputs in this directory
if you prefer not to enter your session cookie.
In that case, name your personal puzzle input files like
`y21d01_personal_puzzle_input.txt` (for Advent of Code year 2021, day 1).

### Displaying Personal Leaderboard Statistics

Once your leaderboards are downloaded,
you can filter, analyze, and display them:

	> cargo run -- stats y21d01 y21d06
	    Finished dev [unoptimized + debuginfo] target(s) in 0.06s
	     Running `target/debug/aoc-stats y21d01 y21d06`
	Advent of Code 2021 - Personal Leaderboard Statistics
	
	      -------Part 1--------   -------Part 2--------
	Day       Time  Rank  Score       Time  Rank  Score
	  6   00:14:37  5023      0   00:29:07  3395      0
	  1   00:20:32  6893      0   00:24:50  5662      0
	---------------------------------------------------
	MIN   00:14:37  5023      0   00:24:50  3395      0
	MED   00:17:35  5958      0   00:26:59  4529      0
	MAX   00:20:32  6893      0   00:29:07  5662      0

[Personal leaderboards](https://adventofcode.com/2021/leaderboard/self)
must currently be downloaded manually and saved to files named like
`y21_personal_leaderboard_statistics.txt` in the directory
`$DATA_DIR/advent_of_code/personal_leaderboard_statistics`,
where `$DATA_DIR` is:

- Linux: `$XDG_DATA_HOME` or `$HOME/.local/share`
  (e.g. `/home/you/.local/share`)
- macOS: `$HOME/Library/Application Support`
  (e.g. `/Users/you/Library/Application Support`)
- Windows: `{FOLDERID_RoamingAppData}` (e.g. `C:\Users\You\AppData\Roaming`)
