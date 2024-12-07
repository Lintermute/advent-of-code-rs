use lazy_errors::{prelude::*, Result};
use rayon::prelude::*;

use crate::parser;

pub struct Input {}

impl core::str::FromStr for Input {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        _ = s;
        Ok(Input {})
    }
}

pub fn parse(input: &str) -> Result<Vec<Input>> {
    parser::par_parse_each(input.par_lines()).collect()
}

pub fn part1(input: &[Input]) -> Result<u64> {
    _ = input;
    Ok(0)
}

pub fn part2(input: &[Input]) -> Result<u64> {
    _ = input;
    Ok(0)
}

#[cfg(test)]
mod tests {
    use crate::{day::*, fs::Config, year::*};

    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `read_workspace_dir_from_cargo`
    fn example_1() -> Result<()> {
        let config = Config::from_env_or_defaults()?;
        let input = config.read_example_puzzle_input(Y24, D08, "1")?;

        let p0 = super::parse(&input)?;
        let p1 = super::part1(&p0)?;
        let p2 = super::part2(&p0)?;

        assert_eq!(p1, 0);
        assert_eq!(p2, 0);
        Ok(())
    }
}
