use lazy_errors::Result;

use crate::parser::{self, Point};

pub fn parse(input: &str) -> Result<Vec<(Point, char)>> {
    use itertools::Itertools;

    let pattern = |line| str::match_indices(line, &['X', 'M', 'A', 'S']);
    let matcher = |line| parser::pattern_matches(line, pattern);
    let parsed: Vec<(Point, char)> =
        parser::parse_substrs(input.lines(), matcher).try_collect()?;

    Ok(parsed)
}

pub fn part1(data: &[(Point, char)]) -> Result<u64> {
    let sum = data
        .iter()
        .filter(|&&(_, ch)| ch == 'X')
        .map(|(point, _)| {
            let mut count = 0;
            let dirs: &[(isize, isize)] = &[
                (0, 1),
                (1, 0),
                (1, 1),
                (0, -1),
                (-1, 0),
                (1, -1),
                (-1, -1),
                (-1, 1),
            ];
            for d in dirs {
                let mut p = *point;
                let mut found = true;
                for next in "MAS".chars() {
                    let y = isize::try_from(p.y()).unwrap() + d.0;
                    let x = isize::try_from(p.x()).unwrap() + d.1;
                    let Ok(y) = usize::try_from(y) else {
                        found = false;
                        break;
                    };
                    let Ok(x) = usize::try_from(x) else {
                        found = false;
                        break;
                    };
                    p = Point::new(y, x);
                    let ch = (p, next);
                    if !data.contains(&ch) {
                        found = false;
                        break;
                    }
                }

                if found {
                    count += 1
                }
            }
            count
        })
        .sum();

    Ok(sum)
}

pub fn part2(_data: &[(Point, char)]) -> Result<u64> {
    Ok(0)
}

#[cfg(test)]
mod tests {
    use crate::{
        fs::Config,
        ident::{D04, Y24},
    };

    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `read_workspace_dir_from_cargo`
    fn example_1() -> Result<()> {
        let config = Config::from_env_or_defaults()?;
        let input = config.read_example_puzzle_input(Y24, D04, "1")?;

        let p0 = super::parse(&input)?;
        let p1 = super::part1(&p0)?;
        let p2 = super::part2(&p0)?;

        assert_eq!(p1, 18);
        assert_eq!(p2, 0);
        Ok(())
    }
}
