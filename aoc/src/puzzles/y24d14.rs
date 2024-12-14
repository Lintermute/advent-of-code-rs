use lazy_errors::{prelude::*, Result};
use num::Integer;

use crate::parser::{self, Point, Rect, Vector};

pub struct Input {
    robots: Vec<Robot>,
}

pub struct Robot {
    p: Vector, // TODO
    v: Vector,
}

impl core::str::FromStr for Robot {
    type Err = Error;

    fn from_str(line: &str) -> Result<Self> {
        let [p, v] = line
            .strip_prefix("p=")
            .unwrap()
            .split(" v=")
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| -> Error { err!("Invalid line: '{line}'") })?;

        let p = parse_coords(p)?;
        let v = parse_coords(v)?;

        Ok(Self { p, v })
    }
}

fn parse_coords(s: &str) -> Result<Vector> {
    use core::str::FromStr;

    let [x, y] = s
        .split(',')
        .map(|k| isize::from_str(k).or_wrap_with(|| "TODO"))
        .collect::<Result<Vec<_>>>()?
        .try_into()
        .map_err(|_| -> Error { err!("Invalid line: '{s}'") })?;

    Ok(Vector::new(y, x))
}

pub fn parse(input: &str) -> Result<Input> {
    use itertools::Itertools;
    let robots = parser::parse_each(input.lines()).try_collect()?;
    Ok(Input { robots })
}

pub fn part1(input: &Input) -> Result<usize> {
    solve(input, Rect::new(Point::new(0, 0), Vector::new(103, 101)))
}

pub fn solve(input: &Input, bounds: Rect) -> Result<usize> {
    let _ = input;

    let y_max = bounds.len().y();
    let x_max = bounds.len().x();

    let quads = input
        .robots
        .iter()
        .map(|r| {
            let p = r.p + (r.v * 100);
            let (y, x) = (p.y(), p.x());
            let (y, x) = (y.mod_floor(&y_max), x.mod_floor(&x_max));
            Point::new(y, x)
        })
        .map(|p| {
            let y = p.y().cmp(&(y_max / 2));
            let x = p.x().cmp(&(x_max / 2));

            use core::cmp::Ordering::*;
            match (y, x) {
                (Less, Less) => (1, 0, 0, 0),
                (Less, Greater) => (0, 1, 0, 0),
                (Greater, Less) => (0, 0, 1, 0),
                (Greater, Greater) => (0, 0, 0, 1),
                _ => (0, 0, 0, 0),
            }
        })
        .reduce(|(acc1, acc2, acc3, acc4), (q1, q2, q3, q4)| {
            (acc1 + q1, acc2 + q2, acc3 + q3, acc4 + q4)
        })
        .unwrap();

    Ok(quads.0 * quads.1 * quads.2 * quads.3)
}

pub fn part2(input: &Input) -> Result<usize> {
    use std::collections::HashSet;

    for i in [7051] {
        let grid: HashSet<Point> = input
            .robots
            .iter()
            .map(|r| {
                // let p = r.p + (r.v * 47);
                let p = r.p + (r.v * i);
                let (y, x) = (p.y(), p.x());
                let (y, x) = (y.mod_floor(&103), x.mod_floor(&101));
                Point::new(y, x)
            })
            .collect();
        print!("\x1B[2J\x1B[1;1H");
        for y in 0..103 {
            for x in 0..101 {
                let char = match grid.get(&Point::new(y, x)) {
                    Some(_) => '#',
                    None => ' ',
                };
                print!("{char}");
            }
            println!()
        }

        println!("{i}");

        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    Ok(0)
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use crate::{day::*, fs::Config, year::*, Part};

    use super::*;

    #[test_case(Y24, D14, "1", Part::Part1, 12)]
    #[cfg_attr(miri, ignore)] // Because of `read_workspace_dir_from_cargo`
    fn example(
        y: Year,
        d: Day,
        label: &str,
        p: Part,
        expected: usize,
    ) -> Result<()> {
        let config = Config::from_env_or_defaults()?;

        let input = config.read_example_puzzle_input(y, d, label)?;
        let input = super::parse(&input)?;
        let result = match p {
            Part::Part1 => {
                solve(&input, Rect::new(Point::new(0, 0), Vector::new(7, 11)))?
            }
            Part::Part2 => super::part2(&input)?,
        };

        assert_eq!(result, expected);
        Ok(())
    }
}
