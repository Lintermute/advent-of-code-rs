use std::time::{Duration, Instant};

use lazy_errors::{prelude::*, Result};
use tokio::{sync::mpsc, task};

use crate::{
    fs::Config,
    ident::{Day, Year},
    runner::Input,
    solver::{Event, Parts, Solver, State, Step},
};

pub struct Downloader;

impl Downloader {
    pub fn spawn(
        config: Config,
        puzzles: Vec<(Solver, Parts)>,
        tx_next: mpsc::Sender<(Solver, Parts, Input)>,
        tx_ui: mpsc::Sender<Event>,
    ) -> Self {
        task::spawn(run(config, puzzles, tx_next, tx_ui));
        Self {}
    }
}

async fn run(
    mut config: Config,
    puzzles: Vec<(Solver, Parts)>,
    tx_next: mpsc::Sender<(Solver, Parts, Input)>,
    tx_ui: mpsc::Sender<Event>,
) {
    // We have to notify the next stage ASAP if the file is cached.
    // Otherwise the next stage cannot even start solving cached inputs.

    let mut queue = vec![];

    for (solver, parts) in puzzles {
        let input: Result<Option<String>> =
            config.read_personal_puzzle_input(solver.year(), solver.day());

        enqueue_or_forward(solver, parts, input, &mut queue, &tx_next, &tx_ui)
            .await
            .expect("Failed to enqueue or forward solver");
    }

    for (solver, parts) in queue {
        // Serialize requests to keep load on adventofcode.com low.
        download_and_cache_and_forward(
            solver,
            parts,
            &mut config,
            &tx_next,
            &tx_ui,
        )
        .await
        .expect("Failed to download puzzle input");
    }
}

async fn enqueue_or_forward(
    solver: Solver,
    parts: Parts,
    input_maybe: Result<Option<String>>,
    queue: &mut Vec<(Solver, Parts)>,
    tx_next: &mpsc::Sender<(Solver, Parts, Input)>,
    tx_ui: &mpsc::Sender<Event>,
) -> Result<()> {
    let year = solver.year();
    let day = solver.day();

    match input_maybe {
        Ok(None) => queue.push((solver, parts)),
        Ok(Some(input)) => {
            send(skipped(year, day), tx_ui).await?;
            send((solver, parts, input), tx_next).await?;
        }
        Err(e) => {
            send(failed(year, day, Duration::ZERO, e), tx_ui).await?;
        }
    }

    Ok(())
}

async fn download_and_cache_and_forward(
    solver: Solver,
    parts: Parts,
    config: &mut Config,
    tx_next: &mpsc::Sender<(Solver, Parts, Input)>,
    tx_ui: &mpsc::Sender<Event>,
) -> Result<()> {
    let year = solver.year();
    let day = solver.day();

    let start_time = Instant::now();
    send(started(year, day, start_time), tx_ui).await?;

    let result = download_and_cache(year, day, config).await;
    let duration = start_time.elapsed();

    match result {
        Ok(input) => {
            send(succeeded(year, day, duration), tx_ui).await?;
            send((solver, parts, input), tx_next).await?;
        }
        Err(e) => {
            send(failed(year, day, duration, e), tx_ui).await?;
        }
    }

    Ok(())
}

async fn download_and_cache(
    year: Year,
    day: Day,
    config: &mut Config,
) -> Result<String> {
    let session_cookie = match config.read_session_cookie() {
        Ok(Some(cookie)) => cookie,
        Ok(None) => return Err(err!("Not logged in")),
        Err(e) => return Err(e),
    };

    let url = format!("https://adventofcode.com/{year}/day/{day}/input");
    let Ok(response) = reqwest::Client::new()
        .request(reqwest::Method::GET, url)
        .header("Cookie", format!("session={session_cookie}"))
        .send()
        .await
        .and_then(|r| r.error_for_status())
    else {
        // adventofcode.com sends HTTP 400 instead of HTTP 401,
        // so we can't distinguish “real” errors.
        return Err(err!("HTTP request failed. Are you logged in?"));
    };

    let input = response
        .text()
        .await
        .or_wrap_with(|| "Failed to convert input to text")?;

    config.save_personal_puzzle_input(year, day, &input)?;

    Ok(input)
}

fn skipped(year: Year, day: Day) -> Event {
    Event {
        year,
        day,
        step: Step::Download,
        state: State::Skipped,
    }
}

fn started(year: Year, day: Day, t: Instant) -> Event {
    Event {
        year,
        day,
        step: Step::Download,
        state: State::Started(t),
    }
}

fn succeeded(year: Year, day: Day, t: Duration) -> Event {
    Event {
        year,
        day,
        step: Step::Download,
        state: State::Done(t, Ok(None)),
    }
}

fn failed(year: Year, day: Day, t: Duration, e: Error) -> Event {
    Event {
        year,
        day,
        step: Step::Download,
        state: State::Done(t, Err(e)),
    }
}

async fn send<T>(data: T, tx: &mpsc::Sender<T>) -> Result<()>
where
    T: Send + Sync + 'static,
{
    tx.send(data)
        .await
        .or_wrap_with(|| "Failed to send data")
}
