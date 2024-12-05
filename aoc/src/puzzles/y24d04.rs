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
                    let y = p.y() + d.0;
                    let x = p.x() + d.1;
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

pub fn part2(data: &[(Point, char)]) -> Result<u64> {
    let sum = data
        .iter()
        .filter(|&&(_, ch)| ch == 'A')
        .map(|(point, _)| {
            let (y, x) = (point.y(), point.x());
            let tlm = (Point::new(y - 1, x - 1), 'M');
            let trm = (Point::new(y - 1, x + 1), 'M');
            let blm = (Point::new(y + 1, x - 1), 'M');
            let brm = (Point::new(y + 1, x + 1), 'M');
            let tls = (Point::new(y - 1, x - 1), 'S');
            let trs = (Point::new(y - 1, x + 1), 'S');
            let bls = (Point::new(y + 1, x - 1), 'S');
            let brs = (Point::new(y + 1, x + 1), 'S');

            let first = data.contains(&tlm) && data.contains(&brs);
            let first2 = data.contains(&tls) && data.contains(&brm);

            let second = data.contains(&blm) && data.contains(&trs);
            let second2 = data.contains(&bls) && data.contains(&trm);

            if (first || first2) && (second || second2) {
                1
            } else {
                0
            }
        })
        .sum();

    Ok(sum)
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
        assert_eq!(p2, 9);
        Ok(())
    }
}
