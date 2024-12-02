use itertools::Itertools;
use lazy_errors::{prelude::*, Result};

type Input = Vec<Vec<u8>>;

pub fn parse(input: &str) -> Result<Input> {
    use core::str::FromStr;

    let lists = input
        .lines()
        .map(|line| {
            let levels: Vec<u8> = line
                .split_whitespace()
                .map(u8::from_str)
                .try_collect()
                .or_wrap_with(|| "TODO")?;

            Ok(levels)
        })
        .collect::<Result<_>>()?;

    Ok(lists)
}

pub fn part1(reports: &Input) -> Result<usize> {
    let count = reports
        .iter()
        .filter(|levels| {
            let a = levels
                .iter()
                .tuple_windows()
                .all(|(l, r)| l < r);
            let b = levels
                .iter()
                .tuple_windows()
                .all(|(l, r)| l > r);
            let c = levels
                .iter()
                .tuple_windows()
                .map(|(l, r)| l.abs_diff(*r))
                .all(|d| d <= 3);

            (a || b) && c
        })
        .count();

    Ok(count)
}

pub fn part2(reports: &Input) -> Result<usize> {
    let count = reports
        .iter()
        .filter(|levels| {
            let a = levels
                .iter()
                .tuple_windows()
                .all(|(l, r)| l < r);
            let b = levels
                .iter()
                .tuple_windows()
                .all(|(l, r)| l > r);
            let c = levels
                .iter()
                .tuple_windows()
                .map(|(l, r)| l.abs_diff(*r))
                .all(|d| d <= 3);

            let o = (a || b) && c;
            if o {
                return true;
            }

            for i in 0..levels.len() {
                let mut lvl: Vec<u8> = (*levels).clone();
                lvl.remove(i);

                let a = lvl
                    .iter()
                    .tuple_windows()
                    .all(|(l, r)| l < r);
                let b = lvl
                    .iter()
                    .tuple_windows()
                    .all(|(l, r)| l > r);
                let c = lvl
                    .iter()
                    .tuple_windows()
                    .map(|(l, r)| l.abs_diff(*r))
                    .all(|d| d <= 3);

                let o = (a || b) && c;
                if o {
                    return true;
                }
            }

            false
        })
        .count();

    Ok(count)
}

#[cfg(test)]
mod tests {
    use crate::{
        fs::Config,
        ident::{D02, Y24},
    };

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
