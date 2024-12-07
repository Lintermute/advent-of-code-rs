use core::iter;

use itertools::Itertools;
use lazy_errors::{prelude::*, Result};
use rayon::prelude::*;

use crate::parser;

pub struct Equation {
    result:  u64,
    numbers: Vec<u64>,
}

type Op = fn(u64, u64) -> u64;

impl core::str::FromStr for Equation {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let [result, numbers] = s
            .split(": ")
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| err!("Failed to parse equation: '{s}'"))?;

        let result = result
            .parse()
            .or_wrap_with(|| format!("Failed to parse result: '{result}'"))?;

        let numbers = numbers
            .split(' ')
            .map(|number| {
                number.parse().or_wrap_with(|| {
                    format!("Failed to parse number: '{number}'")
                })
            })
            .collect::<Result<_>>()?;

        Ok(Equation { result, numbers })
    }
}

pub fn parse(input: &str) -> Result<Vec<Equation>> {
    parser::par_parse_each(input.par_lines()).collect()
}

pub fn part1(input: &[Equation]) -> Result<u64> {
    Ok(input
        .par_iter()
        .filter(|Equation { result, numbers }| {
            let n = numbers.len();
            cartesian_products(n - 1, &[add, mul])
                .any(|ops| is_equal(numbers, &ops, *result))
        })
        .map(|equation| equation.result)
        .sum())
}

pub fn part2(input: &[Equation]) -> Result<u64> {
    Ok(input
        .par_iter()
        .filter(|Equation { result, numbers }| {
            let n = numbers.len();
            cartesian_products(n - 1, &[add, mul, cat])
                .any(|ops| is_equal(numbers, &ops, *result))
        })
        .map(|equation| equation.result)
        .sum())
}

fn add(l: u64, r: u64) -> u64 {
    l + r
}

fn mul(l: u64, r: u64) -> u64 {
    l * r
}

fn cat(l: u64, r: u64) -> u64 {
    format!("{l}{r}")
        .parse()
        .or_wrap_with::<Stashable>(|| format!("Failed to compute `{l} || {r}`"))
        .unwrap()
}

fn cartesian_products(
    n: usize,
    operations: &[Op],
) -> impl Iterator<Item = Vec<&Op>> {
    (0..n)
        .map(|_| operations)
        .multi_cartesian_product()
}

fn is_equal(numbers: &[u64], operations: &[&Op], expectation: u64) -> bool {
    let mut numbers = numbers.iter();
    let Some(mut acc) = numbers.next().copied() else {
        return false;
    };

    for (op, &number) in iter::zip(operations, numbers) {
        acc = op(acc, number);
        if acc > expectation {
            // Won't get smaller because `+`, `*`, `||` only increase values.
            return false;
        }
    }

    acc == expectation
}

#[cfg(test)]
mod tests {
    use crate::{day::*, fs::Config, year::*};

    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `read_workspace_dir_from_cargo`
    fn example_1() -> Result<()> {
        let config = Config::from_env_or_defaults()?;
        let input = config.read_example_puzzle_input(Y24, D07, "1")?;

        let p0 = super::parse(&input)?;
        let p1 = super::part1(&p0)?;
        let p2 = super::part2(&p0)?;

        assert_eq!(p1, 3749);
        assert_eq!(p2, 11387);
        Ok(())
    }
}
