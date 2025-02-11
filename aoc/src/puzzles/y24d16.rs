use lazy_errors::{prelude::*, Result};
use pathfinding::prelude::*;

use crate::parser::{self, Direction, Grid, Point};

pub struct Input {
    grid: Grid,
    s:    Point,
    e:    Point,
}

impl core::str::FromStr for Input {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        let grid = parse_grid(input)?;
        let s = find_char(&grid, 'S')?;
        let e = find_char(&grid, 'E')?;
        Ok(Self { grid, s, e })
    }
}

pub fn parse(input: &str) -> Result<Input> {
    input.parse()
}

pub fn part1(input: &Input) -> Result<u64> {
    astar(
        &(input.s, Direction::E),
        |&(p, d)| successors(input, &p, d),
        |_| 0, // benchmarked: using `||end-p||` is slower, even cached
        |&(p, _d)| p == input.e,
    )
    .ok_or_else(|| err!("Failed to find any path"))
    .map(|(_path, cost)| cost)
}

pub fn part2(input: &Input) -> Result<usize> {
    astar_bag(
        &(input.s, Direction::E),
        |&(p, d)| successors(input, &p, d),
        |_| 0, // benchmarked: using `||end-p||` is slower, even cached
        |&(p, _d)| p == input.e,
    )
    .ok_or_else(|| err!("Failed to find any path"))
    .map(|(paths, _cost)| {
        paths
            .flat_map(|path| path.into_iter().map(|(p, _d)| p))
            .collect::<std::collections::HashSet<Point>>()
            .len()
    })
}

fn parse_grid(input: &str) -> Result<Grid> {
    let tiles = |line| str::match_indices(line, &['.', 'S', 'E']);
    let tiles = |line| parser::pattern_matches(line, tiles);

    Grid::from_str(input, tiles)
}

fn find_char(grid: &Grid, c: char) -> Result<Point> {
    grid.find_exactly_one(&c)
        .map(|e| e.area())
        .copied()
        .or_wrap_with(|| "Failed to find char '{c}' in grid")
}

fn successors(
    input: &Input,
    p: &Point,
    d: Direction,
) -> Vec<((Point, Direction), u64)> {
    input
        .grid
        .find_all_neighbors(p)
        .map(|(e, d2)| ((*e.area(), d2), 1 + rot_cost(d, d2)))
        .collect()
}

fn rot_cost(d1: Direction, d2: Direction) -> u64 {
    match (d1, d2) {
        (Direction::N, Direction::N) => 0,
        (Direction::N, Direction::E) => 1000,
        (Direction::N, Direction::S) => 2000,
        (Direction::N, Direction::W) => 1000,

        (Direction::E, Direction::N) => 1000,
        (Direction::E, Direction::E) => 0,
        (Direction::E, Direction::S) => 1000,
        (Direction::E, Direction::W) => 2000,

        (Direction::S, Direction::N) => 2000,
        (Direction::S, Direction::E) => 1000,
        (Direction::S, Direction::S) => 0,
        (Direction::S, Direction::W) => 1000,

        (Direction::W, Direction::N) => 1000,
        (Direction::W, Direction::E) => 2000,
        (Direction::W, Direction::S) => 1000,
        (Direction::W, Direction::W) => 0,
    }
}

#[cfg(test)]
mod tests {
    use lazy_errors::Result;
    use test_case::test_case;

    use crate::{day::*, fs::Config, year::*};

    #[test_case(Y24, D16, "1", 7036)]
    #[test_case(Y24, D16, "2", 11048)]
    #[cfg_attr(miri, ignore)] // Because of `read_workspace_dir_from_cargo`
    fn p1_example(y: Year, d: Day, label: &str, expected: u64) -> Result<()> {
        let config = Config::from_env_or_defaults()?;
        let input = config.read_example_puzzle_input(y, d, label)?;
        let input = super::parse(&input)?;
        let result = super::part1(&input)?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[test_case(Y24, D16, "1", 45)]
    #[test_case(Y24, D16, "2", 64)]
    #[cfg_attr(miri, ignore)] // Because of `read_workspace_dir_from_cargo`
    fn p2_example(y: Year, d: Day, label: &str, expected: usize) -> Result<()> {
        let config = Config::from_env_or_defaults()?;
        let input = config.read_example_puzzle_input(y, d, label)?;
        let input = super::parse(&input)?;
        let result = super::part2(&input)?;
        assert_eq!(result, expected);
        Ok(())
    }
}
