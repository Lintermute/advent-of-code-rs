use lazy_errors::{prelude::*, Result};

pub struct Input {}

impl core::str::FromStr for Input {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        let _ = input;
        Ok(Self {})
    }
}

pub fn parse(input: &str) -> Result<Input> {
    input.parse()
}

pub fn part1(input: &Input) -> Result<usize> {
    let _ = input;
    Ok(0)
}

pub fn part2(input: &Input) -> Result<usize> {
    let _ = input;
    Ok(0)
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use crate::{day::*, fs::Config, year::*};

    use super::*;

    #[test_case(Y24, D12, "1", 0, 0)]
    #[cfg_attr(miri, ignore)] // Because of `read_workspace_dir_from_cargo`
    fn example_1(
        y: Year,
        d: Day,
        label: &str,
        expected_p1: usize,
        expected_p2: usize,
    ) -> Result<()> {
        let config = Config::from_env_or_defaults()?;
        let input = config.read_example_puzzle_input(y, d, label)?;

        let p0 = super::parse(&input)?;
        let p1 = super::part1(&p0)?;
        let p2 = super::part2(&p0)?;

        assert_eq!(p1, expected_p1);
        assert_eq!(p2, expected_p2);
        Ok(())
    }
}
