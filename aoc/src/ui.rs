use std::{
    fmt::Debug,
    io::Stdout,
    sync::{Mutex, MutexGuard},
    time::{Duration, Instant},
};

use lazy_errors::{prelude::*, Result};
use ratatui::{
    crossterm::terminal::{disable_raw_mode, enable_raw_mode},
    prelude::*,
    widgets::List,
    TerminalOptions, Viewport,
};
use tokio::{
    sync::mpsc,
    task::{self, JoinHandle},
    time::{interval, Interval},
};

use crate::{
    ident::{Day, Id, Year, P1, P2},
    solver::{num_threads, Event, Parts, Solver, State, Step},
};

const TABLE_HEADER: &str = "\
Day ───┬ Download ┬ Prep ────┬ Part 1 ─────────────────┬ Part 2 ────────────────
";

const SPINNERS: [&str; 8] = ["⢎⡡", "⢎⡑", "⢎⠱", "⠎⡱", "⢊⡱", "⢌⡱", "⢆⡱", "⢎⡰"];

const ERR_TERM_IS_NONE: &str =
    "Failed to lookup terminal handle (internal error)";

static IS_TUI_OPEN: Mutex<bool> = Mutex::new(false);

type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;

/// The Terminal User Interface (TUI), renders puzzle states to the terminal.
///
/// This type has a fallible destructor.
/// When you're done using the object, you should call [`UiActor::close`]
/// and handle the error, if any.
/// If you didn't call [`UiActor::close`] explicitly,
/// it will be called automatically when the value is dropped;
/// in that case the thread will panic if [`UiActor::close`] returns an error.
struct UiActor {
    term:   Option<Terminal>, // Never `None` except usually in `drop()`
    ticks:  usize,
    states: Vec<PuzzleState>,
}

pub struct Ui {
    tx:   mpsc::Sender<Event>,
    join: JoinHandle<Result<Summary, Terminated>>,
}

#[derive(Debug)]
pub enum Action {
    Resize,
    Quit,
    Err(Error),
}

/// Basically a “row” on the TUI screen.
#[derive(Debug)]
struct PuzzleState {
    y:  Year,
    d:  Day,
    pd: State,
    p0: State,
    p1: State,
    p2: State,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub enum Summary {
    Success,
    SomeRunnersFailed,
}

#[derive(thiserror::Error, Debug)]
pub enum Terminated {
    #[error("Aborted by user input")]
    AbortedByUser,

    #[error(transparent)]
    InternalError(#[from] Error),
}

impl Ui {
    pub fn open(puzzles: Vec<(Solver, Parts)>) -> Result<Self> {
        // Even if event processing and screen rendering takes a lot of time,
        // it shouldn't block the executor tasks. Otherwise the puzzle solver
        // may be “blocked” (async wait) trying to send a `started(time_now)`
        // message while its timer is already running.
        // The execution time we measured would be incorrect in this case.
        let (tx, rx) = mpsc::channel(2 * num_threads());
        let join = task::spawn(init_and_run(puzzles, rx));
        Ok(Self { tx, join })
    }

    pub fn tx(&self) -> mpsc::Sender<Event> {
        self.tx.clone()
    }

    pub async fn join(self) -> Result<Summary, Terminated> {
        // Allow actor to shut down gracefully.
        drop(self.tx);

        // Now wait until it does so.
        match self.join.await {
            Ok(ok_or_err) => ok_or_err,
            Err(join_err) => {
                let context = Error::wrap_with(
                    join_err,
                    "Failed to wait for UI shutdown",
                );
                Err(Terminated::InternalError(context))
            }
        }
    }
}

impl UiActor {
    /// Opens the terminal user interface and initializes the screen.
    ///
    /// Since there's just a single terminal anyway,
    /// this method will return an error if called a second time
    /// before calling [`UiActor::close`] on the value returned by
    /// the first call to [`UiActor::open`].
    pub fn open(puzzles: &[(Solver, Parts)]) -> Result<Self> {
        let mut is_open = UiActor::is_open()?;
        if *is_open {
            return Err(err!("TUI is already open"));
        } else {
            *is_open = true;
        }

        let puzzles: Vec<PuzzleState> = puzzles
            .iter()
            .map(|(solver, parts)| {
                let (p1, p2) = match parts {
                    Parts::First => (State::Waiting, State::Skipped),
                    Parts::Second => (State::Skipped, State::Waiting),
                    Parts::Both => (State::Waiting, State::Waiting),
                };

                PuzzleState {
                    y: solver.year(),
                    d: solver.day(),
                    pd: State::Waiting,
                    p0: State::Waiting,
                    p1,
                    p2,
                }
            })
            .collect();

        let term = Some(setup_terminal(&puzzles, 0)?);

        Ok(UiActor {
            term,
            states: puzzles,
            ticks: 0,
        })
    }

    fn is_open() -> Result<MutexGuard<'static, bool>> {
        match IS_TUI_OPEN.lock() {
            Ok(is_open) => Ok(is_open),
            Err(_) => Err(err!("Failed to check whether TUI is open")),
        }
    }

    fn update(&mut self, event: Event) -> Result<()> {
        let Event {
            year: y,
            day: d,
            step,
            state,
        } = event;
        if let State::Done(_, Err(err)) = &state {
            // TODO: Use `insert_after` when something like that exists
            self.term
                .as_mut()
                .ok_or_else(|| Error::from_message(ERR_TERM_IS_NONE))?
                .insert_before(1, |buf| {
                    let action = match step {
                        Step::Download => {
                            format!("download {} input", Id((y, d)))
                        }
                        Step::Preproc => {
                            format!("preprocess {} input", Id((y, d)))
                        }
                        Step::Part1 => {
                            format!("solve {}", Id((y, d, P1)))
                        }
                        Step::Part2 => {
                            format!("solve {}", Id((y, d, P2)))
                        }
                    };
                    Line::from(format!("ERROR: Failed to {action}: {err}"))
                        .render(buf.area, buf);
                })
                .or_wrap_with(|| "Failed to display completed step")?;
        }

        let record = self
            .states
            .iter_mut()
            .find(|p| p.y == y && p.d == d)
            .ok_or_else(|| err!("Failed to find puzzle {}", Id((y, d))))?;

        match step {
            Step::Download => record.pd = state,
            Step::Preproc => record.p0 = state,
            Step::Part1 => record.p1 = state,
            Step::Part2 => record.p2 = state,
        }

        Ok(())
    }

    fn tick(&mut self) {
        self.ticks += 1;
    }

    fn render(&mut self) -> Result<()> {
        self.term
            .as_mut()
            .ok_or_else(|| Error::from_message(ERR_TERM_IS_NONE))?
            .draw(|frame| {
                let now = Instant::now();
                let spinner = SPINNERS[self.ticks % SPINNERS.len()];

                let mut lines: Vec<String> = vec![];

                lines.push(TABLE_HEADER.to_string());
                for PuzzleState {
                    y,
                    d,
                    pd,
                    p0,
                    p1,
                    p2,
                } in self.states.iter()
                {
                    let id = Id((*y, *d));
                    let dl = format_column_time(pd, now);
                    let p0 = format_column_time(p0, now);
                    let p1 = format_column_answer_and_time(p1, spinner, now);
                    let p2 = format_column_answer_and_time(p2, spinner, now);
                    lines.push(format!("{id} │ {dl} │ {p0} │ {p1} │ {p2}"));
                }

                let lines = List::new(lines);
                let area = frame.area();
                frame.render_widget(lines, area);
            })
            .or_wrap_with(|| "Failed to render updated terminal")?;

        Ok(())
    }

    fn resize(&mut self) -> Result<()> {
        let term = self
            .term
            .as_mut()
            .ok_or_else(|| Error::from_message(ERR_TERM_IS_NONE))?;

        term.autoresize()
            .or_wrap_with(|| "Failed to resize terminal")
    }

    fn close(mut self) -> Result<()> {
        let Some(term) = self.term.take() else {
            // This case should actually be impossible to reach.
            return Err(err!(
                "Refusing to close TUI that seems to be closed already"
            ));
        };

        // We have an instance, so the TUI must be open.
        // Also, it should practically be impossible for `is_open` to fail.
        // Let's reset the flag before trying to restore the screen state.
        // Restore may fail and the instance would be gone while is_open = true.
        *UiActor::is_open()? = false;
        restore_terminal(term)?;

        Ok(())
    }
}

impl Drop for UiActor {
    fn drop(&mut self) {
        take_mut::take_or_recover(
            &mut self.term,
            || None,
            |term| match term {
                None => {
                    // Default case: TUI was closed before being dropped.
                    None
                }
                Some(term) => {
                    eprintln!("TUI was not closed yet, restoring terminal...");
                    restore_terminal(term)
                        .expect("Failed to run automatic TUI shutdown");
                    None
                }
            },
        )
    }
}

async fn init_and_run(
    puzzles: Vec<(Solver, Parts)>,
    rx: mpsc::Receiver<Event>,
) -> Result<Summary, Terminated> {
    // WARNING! The terminal MUST be set up before trying to read key presses.
    // In other words, `UiActor::open` MUST have completed
    // BEFORE `relay_user_actions` is spawned.
    // Otherwise something sometimes locks up until a key is pressed.

    let mut ui = UiActor::open(&puzzles)?;

    let (tx_action, rx_action) = mpsc::channel(1);
    task::spawn(relay_user_actions(tx_action));

    let result = run_loop(rx, rx_action, ticker(), &mut ui).await;
    ui.close()?;
    result
}

async fn run_loop(
    mut rx_event: mpsc::Receiver<Event>,
    mut rx_action: mpsc::Receiver<Action>,
    mut ticker: Interval,
    ui: &mut UiActor,
) -> Result<Summary, Terminated> {
    let mut some_runners_failed = false;
    loop {
        tokio::select! {
            event_maybe = rx_event.recv() => {
                let Some(event) = event_maybe else {
                    break;
                };

                if matches!(event.state, State::Done(_, Err(_))) {
                    some_runners_failed = true;
                }

                ui.update(event)?;
            },
            Some(action) = rx_action.recv() => {
                match action {
                    Action::Resize => {
                        ui.resize()?;
                    }
                    Action::Quit => {
                        return Err(Terminated::AbortedByUser);
                    }
                    Action::Err(e) => {
                        return Err(Terminated::InternalError(e));
                    }
                }
            },
            _ = ticker.tick() => {
                ui.tick();
                ui.render()?;
            }
        }
    }

    // UI is “usually” outdated because it only gets rendered on ticks.
    // Render the final screen.
    ui.render()?;

    if some_runners_failed {
        Ok(Summary::SomeRunnersFailed)
    } else {
        Ok(Summary::Success)
    }
}

async fn relay_user_actions(tx: mpsc::Sender<Action>) -> Result<()> {
    use crossterm::event::{
        Event as CtEvent, EventStream as CtEventStream, KeyCode, KeyEvent,
        KeyEventKind, KeyModifiers,
    };
    use tokio_stream::StreamExt;

    let mut events = CtEventStream::new();
    while let Some(event) = events.next().await {
        let action = match event {
            Ok(CtEvent::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: _,
            })) => Some(Action::Quit),
            Ok(CtEvent::Resize(..)) => Some(Action::Resize),
            Ok(_) => None,
            Err(e) => Some(Action::Err(Error::wrap_with(
                e,
                "Failed to read user input",
            ))),
        };

        if let Some(action) = action {
            tx.send(action)
                .await
                .or_wrap_with(|| "Failed to send user action event to UI")?;
        }
    }

    Ok(())
}

fn ticker() -> Interval {
    // If and only if
    // (a) rendering is fast enough, and
    // (b) the program is not busy handling async events
    // the duration between ticks will be equal to the interval we specify here.
    // In all other cases, the number of elapsed milliseconds will increase by a
    // number larger than specified below. Nevertheless, to increase chances
    // that the least significant digit of the number of elapsed milliseconds
    // changes each tick, we use an interval with `interval % 10 != 0`.
    //
    // While the number of milliseconds should change rather fast on the
    // screen, the spinner should not appear frantic. The interval below
    // is a good tradeoff, IMO.
    let mut interval = interval(Duration::from_millis(97));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    interval
}

fn setup_terminal(puzzles: &[PuzzleState], x: u16) -> Result<Terminal> {
    let err_msg = "Failed to setup terminal";

    let rows: u16 = puzzles
        .len()
        .try_into()
        .or_wrap_with(|| "Too many puzzles")?;

    enable_raw_mode().or_wrap_with(|| err_msg)?;

    let term = match setup_terminal_backend(rows + x)
        .or_create_stash::<Stashable>(|| err_msg)
    {
        Ok(term) => term,
        Err(mut stash) => {
            disable_raw_mode().or_stash(&mut stash);
            return Err(stash.into());
        }
    };

    Ok(term)
}

fn setup_terminal_backend(rows: u16) -> Result<Terminal> {
    // Add space for the table headers.
    let rows = rows + 1;

    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);

    Terminal::with_options(backend, TerminalOptions {
        viewport: Viewport::Inline(rows),
    })
    .and_then(|mut term| {
        term.clear()?;
        Ok(term)
    })
    .or_wrap_with(|| "Failed to setup terminal backend")
}

fn restore_terminal(mut term: Terminal) -> Result<()> {
    let mut errs = ErrorStash::new(|| "Failed to restore terminal state");

    // Move cursor to the first line outside of what we rendered:
    let y = term.get_frame().area().bottom();
    term.set_cursor_position((0, y))
        .or_stash(&mut errs);

    // If we printed to the last row of the entire terminal,
    // ratatui's `bottom()` will NOT move to an empty row.
    // Thus, we need to write a trailing empty row manually,
    // otherwise the shell prompt would overwrite the last payload row.
    let may_have_printed_to_last_line = term
        .size()
        .map(|area| area.height == y)
        .unwrap_or(true);
    if may_have_printed_to_last_line {
        println!();
    }

    term.show_cursor().or_stash(&mut errs);

    disable_raw_mode().or_stash(&mut errs);

    errs.into()
}

fn format_column_time(state: &State, now: Instant) -> String {
    match state {
        State::Waiting => "        ".to_string(),
        State::Skipped => "     ---".to_string(),
        State::Started(t) => {
            let time = format_time(&now.duration_since(*t));
            format!(" {time}")
        }
        State::Done(t, Ok(_)) => {
            let time = format_time(t);
            format!(" {time}")
        }
        State::Done(_t, Err(_)) => "  ERROR!".to_string(),
    }
}

fn format_column_answer_and_time(
    state: &State,
    spinner: &str,
    now: Instant,
) -> String {
    match state {
        //                 12345678901234567890123
        State::Waiting => "                       ".to_string(),
        State::Skipped => "                    ---".to_string(),
        State::Started(t) => {
            let time = format_time(&now.duration_since(*t));
            format!("{spinner:>14}  {time}") // spinner is double-width
        }
        State::Done(t, Ok(None)) => {
            let time = format_time(t);
            format!("{time:>23}")
        }
        State::Done(t, Ok(Some(result))) => {
            let time = format_time(t);
            format!("{result:>14}  {time}")
        }
        State::Done(_t, Err(e)) => {
            let mut e = e.to_string();
            if e.len() > 16 {
                e.truncate(15);
                e.push('…');
            }
            format!("ERROR: {:16}", &e[0..e.len()])
        }
    }
}

fn format_time(duration: &Duration) -> String {
    if duration < &Duration::from_millis(10_000) {
        let ms = duration.as_millis();
        return format!("{ms:>4} ms");
    }

    for (factor, symbol) in [(1, "s"), (60, "m"), (3600, "h")] {
        if duration < &Duration::from_secs(100 * factor) {
            let ms = duration.as_millis();
            let f = u128::from(factor) * 1000;
            let n = ms / f;
            let r = ms % f;
            let r = (r * 10) / f;
            return format!("{n:>2}.{r} {symbol:<2}");
        }
    }

    String::from("  🧙   ")
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    //           12345678901234567890123
    #[test_case("                       ", |_, _| State::Waiting)]
    #[test_case("                    ---", |_, _| State::Skipped)]
    #[test_case("            ⢎⡡    42 ms",
        |t, d| State::Started(t - d))]
    #[test_case("                  42 ms",
        |_, d| State::Done(d, Ok(None)))]
    #[test_case("           123    42 ms",
        |_, d| State::Done(d, Ok(Some(Box::new(123)))))]
    #[test_case("    1234567890    42 ms",
        |_, d| State::Done(d, Ok(Some(Box::new(1234567890)))))]
    #[test_case("ERROR: Foobar failed...",
        |_, d| State::Done(d, Err(err!("Foobar failed..."))))]
    #[test_case("ERROR: Foobar failed n…",
        |_, d| State::Done(d, Err(err!("Foobar failed now"))))]
    fn format(expected: &str, state: impl FnOnce(Instant, Duration) -> State) {
        let dur = Duration::from_millis(42);
        let begin = Instant::now() - dur;
        let state = state(begin, dur);
        let actual = super::format_column_answer_and_time(&state, "⢎⡡", begin);
        assert_eq!(&actual, expected);
    }

    #[test_case("1234 ms", 1234)]
    #[test_case("   0 ms", 0)]
    #[test_case("9999 ms", 9999)]
    #[test_case("10.0 s ", 10_000)]
    #[test_case("10.0 s ", 10_001)]
    #[test_case("10.0 s ", 10_010)]
    #[test_case("10.0 s ", 10_099)]
    #[test_case("10.1 s ", 10_100)]
    #[test_case("10.9 s ", 10_999)]
    #[test_case("11.9 s ", 11_999)]
    #[test_case("12.0 s ", 12_000)]
    #[test_case("60.0 s ", 60_000)]
    #[test_case("99.9 s ", 99_999)]
    #[test_case(" 1.6 m ", 100_000)]
    #[test_case(" 1.9 m ", 119_999)]
    #[test_case(" 2.0 m ", 120_000)]
    #[test_case("99.9 m ", 5_999_999)]
    #[test_case(" 1.6 h ", 6_000_000)]
    #[test_case("99.9 h ", 359_999_999)]
    #[test_case("  🧙   ", 360_000_000)]
    fn format_time(expected: &str, millis: u64) {
        let t = Duration::from_millis(millis);
        let actual = super::format_time(&t);
        assert_eq!(expected, &actual);
    }
}
