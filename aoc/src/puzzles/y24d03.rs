use lazy_errors::Result;

pub enum Instruction {
    Mul(u16, u16),
    Do,
    Dont,
}
pub fn parse(input: &str) -> Result<Vec<Instruction>> {
    use itertools::Itertools;
    use lazy_regex::regex;

    use crate::parser::{parse_substrs, regex_matches, Rect};
    use core::str::FromStr;

    let instrs = |line| {
        regex_matches(
            line,
            regex!(r"(mul\(\d{1,3},\d{1,3}\)|do\(\)|don't\(\))"),
        )
    };
    parse_substrs(input.lines(), instrs)
        .map_ok(|(_, y): (Rect, String)| y)
        .map_ok(|x| {
            if x == "do()" {
                Instruction::Do
            } else if x == "don't()" {
                Instruction::Dont
            } else {
                let p = x.find(',').unwrap();
                let n = x.len();
                let l = &x[4..p];
                let r = &x[p + 1..n - 1];

                // dbg!(l, r);
                let l = u16::from_str(l).unwrap();
                let r = u16::from_str(r).unwrap();
                Instruction::Mul(l, r)
            }
        })
        .collect()
}

pub fn part1(_data: &[Instruction]) -> Result<u64> {
    let sum = _data
        .iter()
        .map(|i| match i {
            Instruction::Mul(l, r) => {
                let l = u64::from(*l);
                let r = u64::from(*r);
                l * r
            }
            _ => 0,
        })
        .sum();

    Ok(sum)
}

pub fn part2(_data: &[Instruction]) -> Result<u64> {
    let (sum, _) = _data
        .iter()
        .fold((0, true), |(sum, take), i| match i {
            Instruction::Mul(l, r) if take => {
                let l = u64::from(*l);
                let r = u64::from(*r);
                (sum + l * r, true)
            }
            Instruction::Mul(..) => (sum, take),
            Instruction::Do => (sum, true),
            Instruction::Dont => (sum, false),
        });

    Ok(sum)
}

#[cfg(test)]
mod tests {
    use crate::{
        fs::Config,
        ident::{D03, Y24},
    };

    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `read_workspace_dir_from_cargo`
    fn example_1() -> Result<()> {
        let config = Config::from_env_or_defaults()?;
        let input = config.read_example_puzzle_input(Y24, D03, "1")?;
        // let input = config
        //     .read_personal_puzzle_input(Y24, D03)?
        //     .unwrap();

        let p0 = super::parse(&input)?;
        let p1 = super::part1(&p0)?;

        assert_eq!(p1, 161);
        Ok(())
    }

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `read_workspace_dir_from_cargo`
    fn example_2() -> Result<()> {
        let config = Config::from_env_or_defaults()?;
        let input = config.read_example_puzzle_input(Y24, D03, "2")?;
        // let input = config
        //     .read_personal_puzzle_input(Y24, D03)?
        //     .unwrap();

        let p0 = super::parse(&input)?;
        let p2 = super::part2(&p0)?;

        assert_eq!(p2, 48);
        Ok(())
    }
}
