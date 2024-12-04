use lazy_errors::Result;

pub enum Instr {
    Do,
    Dont,
    Mul(u16, u16),
}

pub fn parse(input: &str) -> Result<Vec<Instr>> {
    use core::str::FromStr;

    use lazy_regex::regex;

    let instrs = regex!(r"do\(\)|don't\(\)|mul\(\d{1,3},\d{1,3}\)")
        .find_iter(input)
        .map(|m| match m.as_str() {
            "do()" => Instr::Do,
            "don't()" => Instr::Dont,
            mul => {
                let start = 4;
                let end = mul.len() - 1;
                let comma = mul.find(',').unwrap();

                let l = &mul[start..comma];
                let r = &mul[(comma + 1)..end];

                let l = u16::from_str(l).unwrap();
                let r = u16::from_str(r).unwrap();

                Instr::Mul(l, r)
            }
        })
        .collect();

    Ok(instrs)
}

pub fn part1(data: &[Instr]) -> Result<u64> {
    let sum = data
        .iter()
        .map(|i| match i {
            Instr::Mul(l, r) => {
                let l = u64::from(*l);
                let r = u64::from(*r);
                l * r
            }
            _ => 0,
        })
        .sum();

    Ok(sum)
}

pub fn part2(data: &[Instr]) -> Result<u64> {
    let (sum, _) = data
        .iter()
        .fold((0, true), |(sum, take), i| match i {
            Instr::Do => (sum, true),
            Instr::Dont => (sum, false),
            Instr::Mul(l, r) if take => {
                let l = u64::from(*l);
                let r = u64::from(*r);
                (sum + (l * r), true)
            }
            _ => (sum, take),
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

        let p0 = super::parse(&input)?;
        let p2 = super::part2(&p0)?;

        assert_eq!(p2, 48);
        Ok(())
    }
}
