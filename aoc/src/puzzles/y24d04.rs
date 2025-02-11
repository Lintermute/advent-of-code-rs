use core::iter;

use std::collections::HashMap;

use lazy_errors::Result;

use crate::parser::{self, Point, Vec2};

pub fn parse(input: &str) -> Result<HashMap<Point, char>> {
    parser::parse_substrs(input.lines(), parser::chars).collect()
}

pub fn part1(data: &HashMap<Point, char>) -> Result<u32> {
    let sum = data
        .iter()
        .filter(|(_, &char)| char == 'X')
        .map(|(&p, _)| {
            let count = Vec2::DIRECTIONS
                .iter()
                .filter(|&&d| {
                    let pos = iter::successors(Some(p + d), |&p| Some(p + d));
                    iter::zip(pos, "MAS".chars())
                        .all(|(p, char)| data.get(&p) == Some(&char))
                })
                .count();

            assert!(
                Vec2::DIRECTIONS.len() <= usize::try_from(u32::MAX).unwrap()
            );
            count as u32
        })
        .sum();

    Ok(sum)
}

pub fn part2(data: &HashMap<Point, char>) -> Result<u32> {
    let tl = Vec2::new(-1, -1);
    let tr = Vec2::new(-1, 1);
    let bl = Vec2::new(1, -1);
    let br = Vec2::new(1, 1);

    let sum = data
        .iter()
        .filter(|(_, &char)| char == 'A')
        .map(|(&p, _)| {
            let tl = data.get(&(p + tl));
            let tr = data.get(&(p + tr));
            let bl = data.get(&(p + bl));
            let br = data.get(&(p + br));

            let a1 = matches!(tl, Some('M')) && matches!(br, Some('S'));
            let a2 = matches!(tl, Some('S')) && matches!(br, Some('M'));

            let b1 = matches!(bl, Some('M')) && matches!(tr, Some('S'));
            let b2 = matches!(bl, Some('S')) && matches!(tr, Some('M'));

            if (a1 || a2) && (b1 || b2) {
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
    use crate::{day::*, fs::Config, year::*};

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
