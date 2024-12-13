use itertools::Itertools;
use lazy_errors::{prelude::*, Result};
use rayon::prelude::*;

use crate::parser::{Point, Vector};

pub struct Input {
    machines: Vec<Machine>,
}

#[derive(Debug)]
pub struct Machine {
    a: Vector,
    b: Vector,
    p: Point,
}

impl core::str::FromStr for Input {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        let machines = input
            .split("\n\n")
            .map(|block| Machine::from_str(block).or_wrap_with(|| "TODO"))
            .collect::<Result<_>>()?;

        Ok(Self { machines })
    }
}

impl core::str::FromStr for Machine {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        let mut lines = input.lines();

        let a = lines
            .next()
            .map(|line| -> Result<_> {
                let [x, y] = line
                    .strip_prefix("Button A: X+")
                    .unwrap()
                    .split(", Y+")
                    .map(|k| isize::from_str(k).or_wrap_with(|| "TODO"))
                    .collect::<Result<Vec<_>>>()?
                    .try_into()
                    .map_err(|_| -> Error { err!("Invalid line: '{line}'") })?;
                Ok(Vector::new(y, x))
            })
            .unwrap()
            .unwrap();

        let b = lines
            .next()
            .map(|line| -> Result<_> {
                let [x, y] = line
                    .strip_prefix("Button B: X+")
                    .unwrap()
                    .split(", Y+")
                    .map(|k| isize::from_str(k).or_wrap_with(|| "TODO"))
                    .collect::<Result<Vec<_>>>()?
                    .try_into()
                    .map_err(|_| -> Error { err!("Invalid line: '{line}'") })?;
                Ok(Vector::new(y, x))
            })
            .unwrap()
            .unwrap();

        let [x, y] = lines
            .next()
            .unwrap()
            .strip_prefix("Prize: X=")
            .unwrap()
            .split(", Y=")
            .map(|k| isize::from_str(k).or_wrap_with(|| "TODO"))
            .collect::<Result<Vec<_>>>()?
            .try_into()
            .map_err(|_| err!("Invalid line: '{input}' TODO"))?;
        let p = Point::new(y, x);

        Ok(Self { a, b, p })
    }
}

pub fn parse(input: &str) -> Result<Input> {
    input.parse()
}

// TODO: Return value
pub fn part1(input: &Input) -> Result<isize> {
    Ok(input
        .machines
        .iter()
        .filter_map(|&Machine { a, b, p }| {
            let p = Vector::from(p);
            (0..=100)
                .cartesian_product(0..=100)
                .filter_map(|(c_a, c_b)| {
                    if a * c_a + b * c_b != p {
                        return None;
                    }

                    Some((3 * c_a) + c_b)
                })
                .min()
        })
        .sum())
}

// TODO: Return value
pub fn part2(input: &Input) -> Result<isize> {
    Ok(input
        .machines
        .par_iter()
        .filter_map(|&Machine { a, b, p }| {
            let p = Vector::from(p) + 10_000_000_000_000;
            (0..10_000)
                .cartesian_product(0..10_000)
                .filter_map(|(c_a, c_b)| {
                    // rayon::yield_now();
                    if a * c_a + b * c_b != p {
                        return None;
                    }

                    Some((3 * c_a) + c_b)
                })
                .min()
        })
        .sum())
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use crate::{day::*, fs::Config, year::*, Part};

    use super::*;

    #[test_case(Y24, D13, "1", Part::Part1, 480)]
    #[cfg_attr(miri, ignore)] // Because of `read_workspace_dir_from_cargo`
    fn example(
        y: Year,
        d: Day,
        label: &str,
        p: Part,
        expected: isize,
    ) -> Result<()> {
        let config = Config::from_env_or_defaults()?;

        let input = config.read_example_puzzle_input(y, d, label)?;
        let input = super::parse(&input)?;
        let result = match p {
            Part::Part1 => super::part1(&input)?,
            Part::Part2 => super::part2(&input)?,
        };

        assert_eq!(result, expected);
        Ok(())
    }
}
