use std::str::FromStr;

use itertools::Itertools;
use lazy_errors::{prelude::*, Result};

use crate::parser;

enum Command {
    F(isize), // Forwards,
    D(isize), // Downwards,
    U(isize), // Upwards
}

enum Action {
    Forward(isize),
    Aim(isize),
}

struct Diff(isize, isize);

pub fn part1(input: &str) -> Result<isize> {
    let result = parser::parse_all(input.lines()).try_fold::<_, _, Result<_>>(
        (0, 0),
        |(x, y), diff| {
            let Diff(xi, yi) = diff?;
            Ok((x + xi, y + yi))
        },
    )?;

    Ok(result.0 * result.1)
}

pub fn part2(input: &str) -> Result<isize> {
    let mut actions = parser::parse_all(input.lines());
    let (x, y, _a) = actions.try_fold::<_, _, Result<_>>(
        (0, 0, 0),
        |(x, y, a), action| match action? {
            Action::Aim(ai) => Ok((x, y, a + ai)),
            Action::Forward(xi) => Ok((x + xi, y + a * xi, a)),
        },
    )?;

    Ok(x * y)
}

impl FromStr for Diff {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match str::parse(s)? {
            Command::F(val) => Diff(val, 0),
            Command::D(val) => Diff(0, val),
            Command::U(val) => Diff(0, -val),
        })
    }
}

impl FromStr for Action {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match str::parse(s)? {
            Command::F(val) => Action::Forward(val),
            Command::D(val) => Action::Aim(val),
            Command::U(val) => Action::Aim(-val),
        })
    }
}

impl FromStr for Command {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let (cmd, val) = s
            .split(' ')
            .collect_tuple()
            .ok_or_else(|| err!("Not a command: '{s}'"))?;

        let val = val
            .parse()
            .or_wrap_with(|| "Invalid number: '{val}'")?;

        let cmd = match cmd {
            "forward" => Command::F(val),
            "down" => Command::D(val),
            "up" => Command::U(val),
            _ => panic!("Unexpected input: {cmd}"),
        };

        Ok(cmd)
    }
}
