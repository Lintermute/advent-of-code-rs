use lazy_errors::{prelude::*, Result};

pub fn parse(input: &str) -> Result<(Vec<u64>, Vec<u64>)> {
    use core::str::FromStr;

    let lists: Vec<(u64, u64)> = input
        .lines()
        .map(|line| {
            let tokens: Vec<u64> = line
                .split_whitespace()
                .map(u64::from_str)
                .collect::<Result<_, _>>()
                .or_wrap_with(|| format!("Failed to parse line '{line}'"))?;

            let [left, right]: [_; 2] =
                tokens
                    .try_into()
                    .map_err(|tokens: Vec<_>| {
                        let n = tokens.len();
                        err!("Expected exactly two numbers, got {n}")
                    })?;

            Ok((left, right))
        })
        .collect::<Result<_>>()?;

    Ok(lists.into_iter().unzip())
}

pub fn part1((left, right): &(Vec<u64>, Vec<u64>)) -> Result<u64> {
    let mut left = left.clone();
    let mut right = right.clone();

    left.sort_unstable();
    right.sort_unstable();

    let sum = left
        .into_iter()
        .zip(right)
        .map(|(l, r)| l.abs_diff(r))
        .sum();

    Ok(sum)
}

pub fn part2((left, right): &(Vec<u64>, Vec<u64>)) -> Result<u64> {
    use itertools::Itertools;
    let l = left.iter().counts();
    let r = right.iter().counts();

    let sum = l
        .into_iter()
        .map(|(value, count1)| -> Result<u64> {
            if let Some(&count2) = r.get(value) {
                let c1 = u64::try_from(count1).or_wrap()?;
                let c2 = u64::try_from(count2).or_wrap()?;
                Ok(value * c1 * c2)
            } else {
                Ok(0)
            }
        })
        .sum::<Result<_>>()?;

    Ok(sum)
}

#[cfg(test)]
mod tests {
    use crate::{day::*, fs::Config, year::*};

    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `read_workspace_dir_from_cargo`
    fn example_1() -> Result<()> {
        let config = Config::from_env_or_defaults()?;
        let input = config.read_example_puzzle_input(Y24, D01, "1")?;

        let p0 = super::parse(&input)?;
        let p1 = super::part1(&p0)?;
        let p2 = super::part2(&p0)?;

        assert_eq!(p1, 11);
        assert_eq!(p2, 31);
        Ok(())
    }
}
