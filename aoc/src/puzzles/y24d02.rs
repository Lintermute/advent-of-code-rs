use lazy_errors::{prelude::*, Result};

pub fn parse(input: &str) -> Result<Vec<Vec<i8>>> {
    use core::str::FromStr;

    input
        .lines()
        .map(|line| {
            line.split_whitespace()
                .map(i8::from_str)
                .collect::<Result<_, _>>()
                .or_wrap_with(|| format!("Failed to parse line: '{line}'"))
        })
        .collect()
}

pub fn part1(reports: &[Vec<i8>]) -> Result<usize> {
    let count = reports
        .iter()
        .filter(|levels| find_error(levels).is_none())
        .count();

    Ok(count)
}

pub fn part2(reports: &[Vec<i8>]) -> Result<usize> {
    let count = reports
        .iter()
        .filter(|&levels| {
            if let Some(err) = find_error(levels) {
                is_fixable_up_to(levels, err)
            } else {
                true
            }
        })
        .count();

    Ok(count)
}

fn find_error(levels: &[i8]) -> Option<usize> {
    let diff = levels[1] - levels[0];
    if !(1..=3).contains(&diff.abs()) {
        return Some(1);
    }

    for i in 2..levels.len() {
        let d = levels[i] - levels[i - 1];
        if !(1..=3).contains(&d.abs()) || d.signum() != diff.signum() {
            return Some(i);
        }
    }

    None
}

fn is_fixable_up_to(levels: &[i8], i: usize) -> bool {
    (0..=i).rev().any(|i| {
        let mut levels: Vec<i8> = levels.to_vec();
        levels.remove(i);
        find_error(&levels).is_none()
    })
}

#[cfg(test)]
mod tests {
    use crate::{day::*, fs::Config, year::*};

    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `read_workspace_dir_from_cargo`
    fn example_1() -> Result<()> {
        let config = Config::from_env_or_defaults()?;
        let input = config.read_example_puzzle_input(Y24, D02, "1")?;

        let p0 = parse(&input)?;
        let p1 = part1(&p0)?;
        let p2 = part2(&p0)?;

        dbg!(&p0);
        assert_eq!(p1, 2);
        assert_eq!(p2, 4);
        Ok(())
    }
}
