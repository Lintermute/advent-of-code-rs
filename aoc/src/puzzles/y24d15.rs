use lazy_errors::{prelude::*, Result};

pub struct Input {
    //
}

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
    use lazy_errors::Result;
    use test_case::test_case;

    use crate::{day::*, fs::Config, year::*, Part};

    #[test_case(Y24, D15, "1", Part::Part1, 0)]
    #[cfg_attr(miri, ignore)] // Because of `read_workspace_dir_from_cargo`
    fn example(
        y: Year,
        d: Day,
        label: &str,
        p: Part,
        expected: usize,
    ) -> Result<()> {
        let config = Config::from_env_or_defaults()?;
        let input = config.read_example_puzzle_input(y, d, label)?;

        let input = super::parse(&input)?;
        match p {
            Part::Part1 => {
                let result = super::part1(&input)?;
                assert_eq!(result, expected);
            }
            Part::Part2 => {
                let result = super::part2(&input)?;
                assert_eq!(result, expected);
            }
        };

        Ok(())
    }
}
