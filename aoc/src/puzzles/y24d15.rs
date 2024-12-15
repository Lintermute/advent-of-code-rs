use core::iter;
use std::collections::{HashMap, HashSet};

use lazy_errors::{prelude::*, Result};

use crate::parser::{self, Grid2, Point, Rect, Vector};

pub struct Input {
    grid:  Grid2<Tile>,
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

        let lines = input
            .lines()
            .skip_while(|line| !line.is_empty());
        let moves = parser::parse_each(
            lines
                .skip(1)
                .flat_map(|line| line.chars().map(|ch| ch.to_string())),
        )
        .collect::<Result<_>>()?;
        Ok(Self { grid, moves })
    }
}

pub fn parse(input: &str) -> Result<Input> {
    input.parse()
}

pub fn part1(input: &Input) -> Result<u64> {
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

    grid.tiles
        .into_iter()
        .filter(|&(_, tile)| tile == Tile::Box)
        .map(|(p, _)| {
            let y = u64::try_from(p.y()).or_wrap()?;
            let x = u64::try_from(p.x()).or_wrap()?;
            Ok(100 * y + x)
        })
        .sum::<Result<_>>()
}

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub struct Object {
    p: Rect,
    k: Kind,
}

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum Kind {
    Wall,
    Box,
    Robot,
}

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum Thing {
    Box(Rect),
    Wall(Point),
    Robot(Point),
}

impl core::fmt::Display for Thing {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Thing::Box(_) => write!(f, "[]"),
            Thing::Wall(_) => write!(f, "#"),
            Thing::Robot(_) => write!(f, "@"), // Attention! Width 1!
        }
    }
}

// fn display_at(kind: &Kind, p: &Point) ->
impl core::fmt::Display for Kind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Kind::Box => write!(f, "*"),   // Attention! Width 2!
            Kind::Wall => write!(f, "#"),  // Attention! Width 1 or 2?!
            Kind::Robot => write!(f, "@"), // Attention! Width 1!
        }
    }
}

pub enum Geom {
    Point(Point),
    Rect(Rect),
}

pub fn part2(input: &Input) -> Result<u64> {
    // TODO: Use (kinds: Vec<Kind>, geoms: Vec<Rect/Point>, tiles: Vec<Point>
    // instead) (struct of arrays VS array of structs).
    // let mut things: Vec<Thing> = input
    //     .grid
    //     .tiles
    //     .into_iter()
    //     .flat_map(|(p, tile)| {
    //         let p1 = Point::new(p.y(), 2 * p.x());
    //         let p2 = Point::new(p.y(), 2 * p.x());
    //         match tile {
    //             Tile::Box => vec![Thing::Box(Rect::new(p1, Vector::new(1,
    // 2)))],             Tile::Wall => vec![Thing::Wall(p1),
    // Thing::Wall(p2)],             Tile::Robot => vec![Thing::Robot(p1)],
    //         }
    //     })
    //     .collect();

    // let mut tiles: HashMap<Point, usize> = things
    //     .iter()
    //     .enumerate()
    //     .flat_map(|(id, thing)| match thing {
    //         Thing::Box(rect) => rect
    //             .into_points()
    //             .map(|p| (p, id))
    //             .collect(),
    //         Thing::Wall(point) => vec![(*point, id)],
    //         Thing::Robot(point) => vec![(*point, id)],
    //     })
    //     .collect();

    // let mut robo = things
    //     .iter()
    //     .position(|thing| matches!(thing, Thing::Robot(_)))
    //     .unwrap();

    let (kinds, mut geoms): (Vec<Kind>, Vec<Geom>) = input
        .grid
        .tiles
        .iter()
        .map(|(p, tile)| {
            let p1 = Point::new(p.y(), 2 * p.x());
            // let p2 = Point::new(p.y(), 2 * p.x() + 1);
            let rect = Rect::new(p1, Vector::new(1, 2));
            match tile {
                Tile::Box => (Kind::Box, Geom::Rect(rect)),
                Tile::Wall => (Kind::Wall, Geom::Rect(rect)),
                Tile::Robot => (Kind::Robot, Geom::Point(p1)),
            }
        })
        .collect();

    let mut tiles: HashMap<Point, usize> = geoms
        .iter()
        .enumerate()
        .flat_map(|(id, geoms)| match geoms {
            Geom::Rect(rect) => rect
                .into_points()
                .map(|p| (p, id))
                .collect(),
            Geom::Point(p) => vec![(*p, id)],
        })
        .collect();

    let robo = kinds
        .iter()
        .position(|k| k == &Kind::Robot)
        .unwrap();
    let mut robo_p = match geoms[robo] {
        Geom::Point(p) => p,
        _ => panic!(),
    };

    // let first = robo_p;
    // for dir in input.moves.iter().take(100) {
    for dir in input.moves.iter() {
        // println!("{dir:?}");
        let dir = Vector::from(*dir);

        let first = robo_p + dir;

        let mut boxes: HashSet<usize> = HashSet::new();

        let mut pos = vec![robo_p];
        let mut found_space = false;
        loop {
            // dbg!(&pos);
            let collisions: Vec<usize> = pos
                .iter()
                .map(|&p| p + dir)
                .flat_map(|p| tiles.get(&p).copied())
                .collect();

            let new_collisions = collisions
                .iter()
                .filter(|&id| !boxes.contains(id))
                .collect::<Vec<_>>();

            if new_collisions.is_empty() {
                found_space = true;
                break;
            }

            if collisions
                .iter()
                .any(|&id| matches!(kinds[id], Kind::Wall))
            {
                break;
            }

            collisions
                .iter()
                .filter(|&&id| matches!(kinds[id], Kind::Box))
                .for_each(|&id| {
                    boxes.insert(id);
                });

            pos = collisions
                .iter()
                .filter(|&&id| matches!(kinds[id], Kind::Box))
                .flat_map(|&id| match geoms[id] {
                    Geom::Rect(rect) => rect.into_points().collect(),
                    Geom::Point(p) => vec![p],
                })
                .collect();
        }

        if found_space {
            // dbg!(&boxes);
            boxes
                .iter()
                .flat_map(|&id| match geoms[id] {
                    Geom::Rect(rect) => rect.into_points().collect(),
                    Geom::Point(p) => vec![p],
                })
                .for_each(|p| {
                    // dbg!(&p);
                    tiles.remove(&p);
                });

            for &id in boxes.iter() {
                geoms[id] = match geoms[id] {
                    Geom::Rect(rect) => {
                        let rect = rect.move_by(&dir);
                        for p in rect.into_points() {
                            // dbg!(&p);
                            tiles.insert(p, id);
                        }
                        Geom::Rect(rect)
                    }
                    // Geom::Point(p) => vec![p],
                    _ => panic!(),
                }
            }

            tiles.remove(&robo_p);
            tiles.insert(first, robo);
            robo_p = first;
        }

        // use itertools::Itertools;

        // let y_min = input.grid.bounds.pos().y();
        // let y_len = input.grid.bounds.len().y();
        // let x_min = input.grid.bounds.pos().x();
        // let x_len = input.grid.bounds.len().x() * 2;

        // println!(
        //     "{}",
        //     (y_min..(y_min + y_len))
        //         .map(|y| {
        //             (x_min..(x_min + x_len))
        //                 .map(|x| {
        //                     if let Some(&id) = tiles.get(&Point::new(y, x)) {
        //                         kinds[id].to_string()
        //                     } else {
        //                         String::from(" ")
        //                     }
        //                 })
        //                 .collect::<String>()
        //         })
        //         .join("\n")
        // );
    }

    kinds
        .into_iter()
        .enumerate()
        .filter(|&(_, kind)| matches!(kind, Kind::Box))
        .map(|(id, _)| match geoms[id] {
            Geom::Point(_) => panic!(),
            Geom::Rect(rect) => rect.pos(),
        })
        .map(|p| {
            let y = u64::try_from(p.y()).or_wrap()?;
            let x = u64::try_from(p.x()).or_wrap()?;
            Ok(100 * y + x)
        })
        .sum::<Result<_>>()
}

#[cfg(test)]
mod tests {
    use lazy_errors::Result;
    use test_case::test_case;

    use crate::{day::*, fs::Config, year::*};

    #[test_case(Y24, D15, "1", 2028)]
    #[test_case(Y24, D15, "2", 10092)]
    #[cfg_attr(miri, ignore)] // Because of `read_workspace_dir_from_cargo`
    fn p1_example(y: Year, d: Day, label: &str, expected: u64) -> Result<()> {
        let config = Config::from_env_or_defaults()?;
        let input = config.read_example_puzzle_input(y, d, label)?;
        let input = super::parse(&input)?;
        let result = super::part1(&input)?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[test_case(Y24, D15, "2", 9021)]
    #[cfg_attr(miri, ignore)] // Because of `read_workspace_dir_from_cargo`
    fn p2_example(y: Year, d: Day, label: &str, expected: u64) -> Result<()> {
        let config = Config::from_env_or_defaults()?;
        let input = config.read_example_puzzle_input(y, d, label)?;
        let input = super::parse(&input)?;
        let result = super::part2(&input)?;
        assert_eq!(result, expected);
        Ok(())
    }
}
