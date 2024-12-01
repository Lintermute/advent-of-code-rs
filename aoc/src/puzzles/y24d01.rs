use itertools::Itertools;
use lazy_errors::{prelude::*, Result};

pub fn parse(input: String) -> Result<(Vec<u64>, Vec<u64>)> {
    use core::str::FromStr;

    let lists: Vec<(u64, u64)> = input
        .lines()
        .map(|line| {
            let tokens: Vec<u64> = line
                .split_whitespace()
                .map(u64::from_str)
                .try_collect()
                .or_wrap_with(|| "TODO")?;
            let [left, right]: [_; 2] = tokens
                .try_into()
                .map_err(|_| err!("TODO"))?;
            Ok((left, right))
        })
        .collect::<Result<_>>()?;

    Ok(lists.into_iter().unzip())
}

pub fn part1((left, right): &(Vec<u64>, Vec<u64>)) -> Result<u64> {
    let left = left.iter().sorted_unstable();
    let right = right.iter().sorted_unstable();

    let sum = left
        .zip(right)
        .map(|(l, r)| l.abs_diff(*r))
        .sum();
    Ok(sum)
}

pub fn part2((left, right): &(Vec<u64>, Vec<u64>)) -> Result<u64> {
    let sum = left
        .iter()
        .map(|l| {
            let count = right.iter().filter(|&e| e == l).count();
            l * u64::try_from(count).unwrap()
        })
        .sum();
    Ok(sum)
}

#[cfg(test)]
mod tests {
    use crate::{
        fs::Config,
        ident::{D01, Y24},
    };

    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `read_workspace_dir_from_cargo`
    fn example_1() -> Result<()> {
        let config = Config::from_env_or_defaults()?;
        let input = config.read_example_puzzle_input(Y24, D01, "1")?;

        let p0 = parse(input)?;
        let p1 = part1(&p0)?;
        let p2 = part2(&p0)?;

        assert_eq!(p1, 11);
        assert_eq!(p2, 31);
        Ok(())
    }
}
