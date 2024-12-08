use core::iter;

use std::collections::HashMap;

use itertools::Itertools;
use lazy_errors::{prelude::*, Result};
use lazy_regex::regex;

use crate::parser::{self, Point, Rect, Vector};

pub struct Input {
    area:     Rect,
    antennas: HashMap<char, Vec<Point>>,
}

impl core::str::FromStr for Input {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        let area = area(input)?;
        let antennas = parser::parse_substrs(input.lines(), |line| {
            parser::regex_matches(line, regex!(r"[^\.]"))
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .map(|(point, char)| (char, point))
        .into_group_map();

        Ok(Self { area, antennas })
    }
}

pub fn parse(input: &str) -> Result<Input> {
    input.parse()
}

pub fn part1(input: &Input) -> Result<usize> {
    use itertools::Itertools;
    Ok(input
        .antennas
        .iter()
        .flat_map(|(_kind, positions)| {
            positions
                .iter()
                .cartesian_product(positions)
                .filter(|(&a, &b)| a != b)
                .flat_map(|(&a, &b)| {
                    let dist = b - a;
                    let iter_a = iter::successors(Some(b), move |&p| {
                        let p = p + dist;
                        if input.area.contains(&p) {
                            Some(p)
                        } else {
                            None
                        }
                    })
                    .skip(1)
                    .take(1);
                    let iter_b = iter::successors(Some(a), move |&p| {
                        let p = p - dist;
                        if input.area.contains(&p) {
                            Some(p)
                        } else {
                            None
                        }
                    })
                    .skip(1)
                    .take(1);

                    iter_a.chain(iter_b)
                })
        })
        .unique()
        .count())
}

pub fn part2(input: &Input) -> Result<usize> {
    Ok(input
        .antennas
        .iter()
        .flat_map(|(_kind, positions)| {
            positions
                .iter()
                .cartesian_product(positions)
                .filter(|(&a, &b)| a != b)
                .flat_map(|(&a, &b)| {
                    let dist = b - a;
                    let iter_a = iter::successors(Some(b), move |&p| {
                        let p = p + dist;
                        if input.area.contains(&p) {
                            Some(p)
                        } else {
                            None
                        }
                    });
                    let iter_b = iter::successors(Some(a), move |&p| {
                        let p = p - dist;
                        if input.area.contains(&p) {
                            Some(p)
                        } else {
                            None
                        }
                    });

                    iter_a.chain(iter_b)
                })
        })
        .unique()
        .count())
}

// TODO: Dedup
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
