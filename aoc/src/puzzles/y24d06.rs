use core::iter;

use std::collections::HashSet;

use rayon::prelude::*;

use itertools::Itertools;
use lazy_errors::{prelude::*, Result};

use crate::parser::{self, Point, Rect, Vector};

pub fn parse(input: &str) -> Result<Grid> {
    let area = area(input)?;
    let guard = parse_guard(input)?;
    let stuff = parse_stuff(input)?;
    let trace = walk(guard, &area, &stuff)
        .map(|g| g.p)
        .collect();

    Ok(Grid {
        area,
        guard,
        stuff,
        trace,
    })
}

pub fn part1(grid: &Grid) -> Result<usize> {
    Ok(grid.trace.len())
}

pub fn part2(grid: &Grid) -> Result<usize> {
    let count = grid
        .trace
        .par_iter()
        .filter(|&&p| {
            let mut stuff = grid.stuff.clone();
            stuff.insert(p);

            let mut path_iter = walk(grid.guard, &grid.area, &stuff);
            !path_iter.all_unique()
        })
        .count();

    Ok(count)
}

#[derive(Debug)]
pub struct Grid {
    area:  Rect,
    guard: Guard,
    stuff: HashSet<Point>,
    trace: HashSet<Point>,
}

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
struct Guard {
    p: Point,
    d: Direction,
}

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
enum Direction {
    R,
    D,
    L,
    U,
}

impl Guard {
    fn walk(self, stuff: &HashSet<Point>, area: &Rect) -> Option<Self> {
        let mut p;
        let mut d = self.d;

        loop {
            p = self.p + Vector::from(d);
            if !stuff.contains(&p) {
                break;
            }
            d = d.rotate_clockwise();
        }

        if area.contains(&p) {
            Some(Self { p, d })
        } else {
            None
        }
    }
}

impl Direction {
    pub fn rotate_clockwise(self) -> Self {
        use Direction::*;
        match self {
            R => D,
            D => L,
            L => U,
            U => R,
        }
    }
}

impl core::str::FromStr for Direction {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        use Direction::*;
        match s {
            ">" => Ok(R),
            "v" => Ok(D),
            "<" => Ok(L),
            "^" => Ok(U),
            _ => Err(err!("Not a direction: '{s}'")),
        }
    }
}

impl From<Direction> for Vector {
    fn from(val: Direction) -> Self {
        match val {
            Direction::U => Vector::new(-1, 0),
            Direction::R => Vector::new(0, 1),
            Direction::D => Vector::new(1, 0),
            Direction::L => Vector::new(0, -1),
        }
    }
}

fn area(input: &str) -> Result<Rect> {
    let mut lens: Vec<usize> = input
        .lines()
        .map(|line| line.len())
        .collect();

    let y = lens.len();

    lens.dedup(); // Leaves good values after first bad one, but we don't care.

    let [x] = lens
        .try_into()
        .map_err(|v| err!("Line lengths differ: {v:?}"))?;

    let p = Point::new(0, 0);
    let v = Vector::from_unsigned(y, x)?;
    Ok(Rect::new(p, v))
}

fn parse_guard(input: &str) -> Result<Guard> {
    let guard = |line| str::match_indices(line, &['^', '>', 'v', '<']);
    let guard = |line| parser::pattern_matches(line, guard);

    let [(p, d)] = parser::parse_substrs(input.lines(), guard)
        .collect::<Result<Vec<_>>>()?
        .try_into()
        .map_err(|_| err!("Expected exactly one guard"))?;

    Ok(Guard { p, d })
}

fn parse_stuff(input: &str) -> Result<HashSet<Point>> {
    let stuff = |line| str::match_indices(line, &['#']);
    let stuff = |line| parser::pattern_matches(line, stuff);
    parser::parse_substrs(input.lines(), stuff)
        .map_ok(|(p, _): (Point, char)| p)
        .collect()
}

fn walk<'a>(
    guard: Guard,
    area: &'a Rect,
    stuff: &'a HashSet<Point>,
) -> impl Iterator<Item = Guard> + 'a {
    iter::successors(Some(guard), |g| g.walk(stuff, area))
}

#[cfg(test)]
mod tests {
    use crate::{day::*, fs::Config, year::*};

    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `read_workspace_dir_from_cargo`
    fn example_1() -> Result<()> {
        let config = Config::from_env_or_defaults()?;
        let input = config.read_example_puzzle_input(Y24, D06, "1")?;

        let p0 = super::parse(&input)?;
        let p1 = super::part1(&p0)?;
        let p2 = super::part2(&p0)?;

        assert_eq!(p1, 41);
        assert_eq!(p2, 6);
        Ok(())
    }
}
