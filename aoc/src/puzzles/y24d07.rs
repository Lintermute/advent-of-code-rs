use lazy_errors::Result;

type Input = Vec<usize>;

pub fn parse(_: &str) -> Result<Input> {
    Ok(vec![])
}

pub fn part1(_: &Input) -> Result<usize> {
    Ok(0)
}

pub fn part2(_: &Input) -> Result<usize> {
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
        let input = config.read_example_puzzle_input(Y24, D07, "1")?;

        let p0 = super::parse(&input)?;
        let p1 = super::part1(&p0)?;
        let p2 = super::part2(&p0)?;

        assert_eq!(p1, 0);
        assert_eq!(p2, 0);
        Ok(())
    }
}
