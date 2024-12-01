use std::{
    panic::{catch_unwind, UnwindSafe},
    time::{Duration, Instant},
};

use lazy_errors::{prelude::*, Result};
use tokio::{
    sync::{mpsc, oneshot},
    task,
};

use crate::{
    ident::{Day, Id, Part, Year},
    solver::{
        num_threads, Event, Input, Parts, PuzzleAnswer, Solver, State, Step,
    },
};

pub struct Runner {
    tx: mpsc::Sender<(Solver, Parts, Input)>,
}

impl Runner {
    pub fn spawn(tx_ui: mpsc::Sender<Event>) -> Self {
        // Ensure there is enough work available.
        let (tx, rx) = mpsc::channel(num_threads());
        task::spawn(run_actor(rx, tx_ui));
        Self { tx }
    }

    pub fn tx(&self) -> mpsc::Sender<(Solver, Parts, Input)> {
        self.tx.clone()
    }
}

async fn run_actor(
    mut rx: mpsc::Receiver<(Solver, Parts, Input)>,
    tx: mpsc::Sender<Event>,
) {
    while let Some((solver, parts, input)) = rx.recv().await {
        let tx = tx.clone();
        task::spawn(await_rayon_thread(move || solver.solve(parts, input, tx)));
    }
}

async fn await_rayon_thread<F>(f: F)
where
    F: (FnOnce() -> Result<()>) + Send + 'static,
{
    let (tx, rx) = oneshot::channel();

    rayon::spawn_fifo(|| {
        let result = f();

        // If the receiver (the async context of this function)
        // is suddenly gone, we're probably shutting down anyways,
        // so drop any error in that case.
        let _ = tx.send(result);
    });

    rx.await
        .expect("Failed to wait for solver thread")
        .expect("Failed to run solver thread")
}

#[doc(hidden)]
pub fn skip_preproc(y: Year, d: Day, tx: &mpsc::Sender<Event>) -> Result<()> {
    send(skipped(y, d, Step::Preproc), tx)
}

#[doc(hidden)]
pub fn preprocess<I, E>(
    y: Year,
    d: Day,
    parser: fn(Input) -> Result<I, E>,
    input: Input,
    tx: &mpsc::Sender<Event>,
) -> Result<Option<I>>
where
    E: Into<Stashable>,
{
    let start_time = Instant::now();
    send(started(y, d, Step::Preproc, start_time), tx)?;

    let parsed_input = match catch_unwind(|| parser(input)) {
        Ok(result) => result.or_wrap(),
        Err(_panic) => Err(err!("PANIC")),
    };

    let duration = start_time.elapsed();

    match parsed_input {
        Ok(data) => {
            send(preproc_succeeded(y, d, duration), tx)?;
            Ok(Some(data))
        }
        Err(e) => {
            send(preproc_failed(y, d, duration, e), tx)?;
            Ok(None) // Failure will be displayed by UI
        }
    }
}

#[doc(hidden)]
pub fn solve<A1, A2, E1, E2>(
    y: Year,
    d: Day,
    p1: impl Fn() -> Result<A1, E1> + Send + UnwindSafe,
    p2: impl Fn() -> Result<A2, E2> + Send + UnwindSafe,
    parts: Parts,
    tx: &mpsc::Sender<Event>,
) -> Result<()>
where
    A1: PuzzleAnswer,
    A2: PuzzleAnswer,
    E1: Into<Stashable>,
    E2: Into<Stashable>,
{
    let p1 = || solve_part(y, d, Part::Part1, p1, tx);
    let p2 = || solve_part(y, d, Part::Part2, p2, tx);

    let (p1, p2) = match parts {
        Parts::First => (p1(), Ok(())),
        Parts::Second => (Ok(()), p2()),
        Parts::Both => rayon::join(p1, p2),
    };

    let mut errs = ErrorStash::new(|| {
        let id = Id((y, d));
        format!("Internal error while solving {id}")
    });
    p1.or_stash(&mut errs);
    p2.or_stash(&mut errs);

    errs.into()
}

fn solve_part<A, E>(
    y: Year,
    d: Day,
    p: Part,
    f: impl Fn() -> Result<A, E> + UnwindSafe,
    tx: &mpsc::Sender<Event>,
) -> Result<()>
where
    A: PuzzleAnswer,
    E: Into<Stashable>,
{
    let time = Instant::now();
    send(started(y, d, p.into(), time), tx)?;

    let result = match catch_unwind(f) {
        Ok(result) => result.or_wrap(),
        Err(_panic) => Err(err!("PANIC")),
    };

    let duration = time.elapsed();
    send(solver_done(y, d, p, result, duration), tx)?;

    Ok(())
}

fn skipped(year: Year, day: Day, step: Step) -> Event {
    Event {
        year,
        day,
        step,
        state: State::Skipped,
    }
}

fn started(year: Year, day: Day, step: Step, t: Instant) -> Event {
    Event {
        year,
        day,
        step,
        state: State::Started(t),
    }
}

fn preproc_succeeded(year: Year, day: Day, t: Duration) -> Event {
    Event {
        year,
        day,
        step: Step::Preproc,
        state: State::Done(t, Ok(None)),
    }
}

fn preproc_failed(year: Year, day: Day, t: Duration, e: Error) -> Event {
    Event {
        year,
        day,
        step: Step::Preproc,
        state: State::Done(t, Err(e)),
    }
}

fn solver_done<A: PuzzleAnswer>(
    year: Year,
    day: Day,
    part: Part,
    result: Result<A>,
    t: Duration,
) -> Event {
    let result = result.map(|answer| Some(Box::new(answer) as _));
    Event {
        year,
        day,
        step: part.into(),
        state: State::Done(t, result),
    }
}

fn send<T>(data: T, tx: &mpsc::Sender<T>) -> Result<()>
where
    T: Send + Sync + 'static,
{
    tx.blocking_send(data)
        .or_wrap_with(|| "Failed to send data")
}
