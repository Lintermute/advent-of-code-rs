use core::iter;

use lazy_errors::{prelude::*, Result};

use crate::parser::{self, Grid2, Vector};

pub struct Input {
    //
    grid:  Grid2<Tile>,
    // robot: Point,
    moves: Vec<Direction>,
}

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum Tile {
    Wall,
    Box,
    Robot,
}

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
enum Direction {
    R,
    D,
    L,
    U,
}

impl core::str::FromStr for Tile {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        use Tile::*;
        match s {
            "#" => Ok(Wall),
            "O" => Ok(Box),
            "@" => Ok(Robot),
            _ => Err(err!("Failed to parse tile '{s}'")),
        }
    }
}

impl core::fmt::Display for Tile {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Tile::Wall => write!(f, "#"),
            Tile::Box => write!(f, "O"),
            Tile::Robot => write!(f, "@"),
        }
    }
}

impl core::str::FromStr for Direction {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        use Direction::*;
        match s {
            ">" => Ok(R),
            "v" => Ok(D),
            "<" => Ok(L),
            "^" => Ok(U),
            _ => Err(err!("Failed to parse direction '{s}'")),
        }
    }
}

impl From<Direction> for Vector {
    fn from(val: Direction) -> Self {
        match val {
            Direction::U => Vector::new(-1, 0),
            Direction::R => Vector::new(0, 1),
            Direction::D => Vector::new(1, 0),
            Direction::L => Vector::new(0, -1),
        }
    }
}

impl core::str::FromStr for Input {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        let mut lines = input.lines();

        let grid = |line| str::match_indices(line, &['#', 'O', '@']);
        let grid = |line| parser::pattern_matches(line, grid);

        let block = &input[0..input.find("\n\n").unwrap()];
        let bounds = parser::parse_bounds(block)?;
        let tiles = parser::parse_substrs(
            (&mut lines).take_while(|line| !line.is_empty()),
            grid,
        );
        let grid = Grid2::try_from(bounds, tiles)?;

        // let lines = lines.clone();
        let lines = input
            .lines()
            .skip_while(|line| !line.is_empty());
        // let moves = parser::parse_each(
        //     lines
        //         .skip(1)
        //         .flat_map(|line| line.chars()),
        // );
        let moves = parser::parse_each(
            lines
                .skip(1)
                .flat_map(|line| line.chars().map(|ch| ch.to_string())),
        )
        // .map()
        .collect::<Result<_>>()?;
        Ok(Self { grid, moves })
    }
}

pub fn parse(input: &str) -> Result<Input> {
    input.parse()
}

pub fn part1(input: &Input) -> Result<u64> {
    // let _ = input;
    // println!("{}", input.grid);
    // dbg!(&input.moves);

    let mut grid = input.grid.clone();
    let mut robo = input
        .grid
        .position(&Tile::Robot)
        .unwrap();
    for dir in &input.moves {
        // println!("{dir:?}");
        let dir = Vector::from(*dir);

        let first = robo + dir;
        let pos = iter::successors(Some(first), |&pos| Some(pos + dir));

        // // let mut first_box = None;
        // for p in pos {
        //     match grid.tiles.get(&p) {
        //         Some(Tile::Box) => {
        //             // if first_box.is_none() {
        //             //     first_box = Some(p);
        //             // }
        //         }
        //         Some(Tile::Wall) => {
        //             break;
        //         }
        //         None => {
        //             if p != first {
        //                 grid.tiles.remove(&first);
        //                 grid.tiles.insert(p, Tile::Box);
        //             }
        //             grid.tiles.remove(&robo);
        //             grid.tiles.insert(first, Tile::Robot);
        //             robo = first;
        //         }
        //         _ => panic!(),
        //     }
        //     println!("{}\n", grid);
        // }

        // let mut first_box = None;
        let p = pos
            .map(|p| (p, grid.tiles.get(&p)))
            .take_while(|(_, tile)| !matches!(tile, Some(Tile::Wall)))
            .find(|(_, tile)| tile.is_none())
            .map(|(p, _)| p);

        if let Some(p) = p {
            if p != first {
                grid.tiles.remove(&first);
                grid.tiles.insert(p, Tile::Box);
            }
            grid.tiles.remove(&robo);
            grid.tiles.insert(first, Tile::Robot);
            robo = first;
        }
        // println!("{}\n", grid);
    }

    Ok(grid
        .tiles
        .into_iter()
        .filter(|&(_, tile)| tile == Tile::Box)
        .map(|(p, _)| {
            let y = u64::try_from(p.y()).or_wrap()?;
            let x = u64::try_from(p.x()).or_wrap()?;
            Ok(100 * y + x)
        })
        .sum::<Result<_>>()?)
}

pub fn part2(input: &Input) -> Result<u64> {
    let _ = input;
    Ok(0)
}

#[cfg(test)]
mod tests {
    use lazy_errors::Result;
    use test_case::test_case;

    use crate::{day::*, fs::Config, year::*, Part};

    #[test_case(Y24, D15, "1", Part::Part1, 2028)]
    #[test_case(Y24, D15, "2", Part::Part1, 10092)]
    #[cfg_attr(miri, ignore)] // Because of `read_workspace_dir_from_cargo`
    fn example(
        y: Year,
        d: Day,
        label: &str,
        p: Part,
        expected: u64,
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
