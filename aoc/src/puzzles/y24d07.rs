use itertools::Itertools;
use lazy_errors::{prelude::*, Result};

use crate::parser;

type Input = Vec<Equation>;

pub struct Equation {
    result:   u64,
    operands: Vec<u16>,
}

impl core::str::FromStr for Equation {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let [result, operands] = s
            .split(": ")
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| err!("Failed to parse equation: '{s}'"))?;

        let result = result
            .parse()
            .or_wrap_with(|| format!("Failed to parse result: '{result}'"))?;

        let operands = operands
            .split(' ')
            .map(|operand| -> Result<u16> {
                operand.parse().or_wrap_with(|| {
                    format!("Failed to parse operand: '{operand}'")
                })
            })
            .collect::<Result<_>>()?;

        Ok(Equation { result, operands })
    }
}

pub fn parse(input: &str) -> Result<Input> {
    parser::parse_each(input.lines()).collect()
}

pub fn part1(input: &Input) -> Result<u64> {
    Ok(input
        .iter()
        .filter(|equation| {
            let n = equation.operands.len();
            let add = |l, r| l + r;
            let mul = |l, r| l * r;

            (1..n)
                .map(|_| vec![add, mul])
                .multi_cartesian_product()
                .map(|ops| {
                    let mut ops = ops.iter();
                    equation
                        .operands
                        .iter()
                        .copied()
                        .map(u64::from)
                        .reduce(|acc, e| {
                            let op = ops.next().unwrap();
                            op(acc, e)
                        })
                        .unwrap()
                })
                .any(|result| result == equation.result)
        })
        .map(|equation| equation.result)
        .sum())
}

pub fn part2(input: &Input) -> Result<u64> {
    use core::str::FromStr;

    Ok(input
        .iter()
        .filter(|equation| {
            let n = equation.operands.len();
            let add = |l, r| l + r;
            let mul = |l, r| l * r;
            let cat = |l, r| u64::from_str(&format!("{l}{r}")).unwrap();

            (1..n)
                .map(|_| vec![add, mul, cat])
                .multi_cartesian_product()
                .map(|ops| {
                    let mut ops = ops.iter();
                    equation
                        .operands
                        .iter()
                        .copied()
                        .map(u64::from)
                        .reduce(|acc, e| {
                            let op = ops.next().unwrap();
                            op(acc, e)
                        })
                        .unwrap()
                })
                .any(|result| result == equation.result)
        })
        .map(|equation| equation.result)
        .sum())
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
