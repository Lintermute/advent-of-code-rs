use core::iter;

use std::collections::HashMap;

use itertools::Itertools;
use lazy_errors::{prelude::*, Result};
use lazy_regex::regex;

use crate::parser::{self, Point, Rect};

// TODO: Integrate this into the Grid helper (basically “reverse lookup” cache).
pub struct Input {
    bounds:   Rect,
    antennas: HashMap<char, Vec<Point>>,
}

impl core::str::FromStr for Input {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        let bounds = parser::parse_bounds(input)?;
        let antennas = parser::parse_substrs(input.lines(), |line| {
            parser::regex_matches(line, regex!(r"[^\.]"))
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .map(|(point, char)| (char, point))
        .into_group_map();

        Ok(Self { bounds, antennas })
    }
}

pub fn parse(input: &str) -> Result<Input> {
    input.parse()
}

pub fn part1(input: &Input) -> Result<usize> {
    solve(input, |a, b, bounds| {
        harmonics(a, b, bounds).skip(1).take(1)
    })
}

pub fn part2(input: &Input) -> Result<usize> {
    solve(input, harmonics)
}

fn solve<'a, F, I>(input: &'a Input, nodes: F) -> Result<usize>
where
    F: Fn(Point, Point, &'a Rect) -> I,
    I: Iterator<Item = Point> + 'a,
{
    Ok(input
        .antennas
        .iter()
        .flat_map(|(_kind, positions)| {
            combinations(positions)
                .flat_map(|(&a, &b)| nodes(a, b, &input.bounds))
        })
        .unique()
        .count())
}

fn harmonics(
    a: Point,
    b: Point,
    bounds: &Rect,
) -> impl Iterator<Item = Point> + '_ {
    let direction = b - a;
    iter::successors(Some(b), move |&p| {
        let p = p + direction;
        if bounds.contains(&p) {
            Some(p)
        } else {
            None
        }
    })
}

fn combinations(points: &[Point]) -> impl Iterator<Item = (&Point, &Point)> {
    points
        .iter()
        .cartesian_product(points)
        .filter(|(a, b)| a != b)
}

#[cfg(test)]
mod tests {
    use crate::{day::*, fs::Config, year::*};

    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `read_workspace_dir_from_cargo`
    fn example_1() -> Result<()> {
        let config = Config::from_env_or_defaults()?;
        let input = config.read_example_puzzle_input(Y24, D08, "1")?;

        let p0 = super::parse(&input)?;
        let p1 = super::part1(&p0)?;
        let p2 = super::part2(&p0)?;

        assert_eq!(p1, 14);
        assert_eq!(p2, 34);
        Ok(())
    }
}
