use core::iter;

use lazy_errors::{prelude::*, Result};

use crate::parser::{self, grid::RenderOn, Direction, Grid, Point, Rect, Vec2};

pub struct Input {
    grid:  Grid<Point, Tile>,
    moves: Vec<Direction>,
}

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum Tile {
    Wall,
    Box,
    Robot,
}

impl core::str::FromStr for Tile {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "#" => Ok(Tile::Wall),
            "O" => Ok(Tile::Box),
            "@" => Ok(Tile::Robot),
            _ => Err(err!("Failed to parse tile '{s}'")),
        }
    }
}

impl RenderOn<Point> for Tile {
    fn render_on(&self, _: &Point) -> Result<Vec<Vec<char>>> {
        let char = match self {
            Tile::Wall => '#',
            Tile::Box => 'O',
            Tile::Robot => '@',
        };

        Ok(vec![vec![char]])
    }
}

impl RenderOn<Rect> for Tile {
    fn render_on(&self, r: &Rect) -> Result<Vec<Vec<char>>> {
        let v = r.len();
        let (y, x) = (v.y(), v.x());

        if y != 1 {
            return Err(err!(
                "Individual tiles can be rendered on a single line only"
            ));
        }

        let repr = match (self, x) {
            (Tile::Wall, l) => vec!['#'; l],
            (Tile::Robot, 1) => vec!['@'],
            (Tile::Box, 1) => vec!['O'],
            (Tile::Box, 2) => vec!['[', ']'],
            _ => return Err(err!("Cannot render {self:?} using width {x}")),
        };

        Ok(vec![repr])
    }
}

impl core::str::FromStr for Input {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        let mut lines = input.lines();

        let lines_grid = lines
            .by_ref()
            .take_while(|line| !line.is_empty());

        let grid = |line| str::match_indices(line, &['#', 'O', '@']);
        let grid = |line| parser::pattern_matches(line, grid);
        let grid = Grid::from_lines(lines_grid, grid)?;

        let _ = lines.by_ref().skip(1); // the empty line

        let chars_as_strings =
            lines.flat_map(|line| line.chars().map(|c| c.to_string()));

        let moves =
            parser::parse_each(chars_as_strings).collect::<Result<_>>()?;

        Ok(Self { grid, moves })
    }
}

pub fn parse(input: &str) -> Result<Input> {
    input.parse()
}

pub fn part1(input: &Input) -> Result<u64> {
    let mut grid = input.grid.clone();

    for dir in &input.moves {
        let robot = find_robot(&grid)?;
        let dir = Vec2::from(*dir);

        let first = robot + dir;
        let steps = iter::successors(Some(first), |&p| Some(p + dir));

        let empty = steps
            .map(|p| (p, grid.get_data_at(&p)))
            .take_while(|(_, tile)| !matches!(tile, Some(Tile::Wall)))
            .find(|(_, e)| e.is_none())
            .map(|(p, _)| p);

        if let Some(empty) = empty {
            if empty != first {
                // At least one box was moved → “swap” the first and last one.
                grid.extract_at(&first)
                    .ok_or_else(|| -> Error {
                        err!("Found a last element but no first one")
                    })?;
                grid.insert(empty, Tile::Box)?;
            }

            grid.move_by_from(dir, &robot)?;
        }
    }

    let sum: isize = grid
        .find_all(&Tile::Box)
        .map(|e| {
            let p = e.area();
            100 * p.y() + p.x()
        })
        .sum();

    sum.try_into().or_wrap()
}

pub fn part2(input: &Input) -> Result<u64> {
    let tiles = input.grid.iter().map(|e| {
        let p = e.area();
        let kind = e.data();

        let p = Point::new(p.y(), 2 * p.x());
        let wide = Rect::new(p, Vec2::new(1, 2))?;
        let slim = Rect::from(p);
        Ok(match kind {
            Tile::Box => (wide, Tile::Box),
            Tile::Wall => (wide, Tile::Wall),
            Tile::Robot => (slim, Tile::Robot),
        })
    });

    let bounds = input.grid.bounds();
    let (p, v) = (*bounds.pos(), *bounds.len());
    let v = Vec2::new(v.y(), 2 * v.x());
    let mut grid = Grid::try_from(Rect::new(p, v)?, tiles)?;
    for d in &input.moves {
        move_or_ignore(*d, &mut grid)?;
    }

    let sum: isize = grid
        .find_all(&Tile::Box)
        .map(|e| {
            let p = e.area().pos();
            100 * p.y() + p.x()
        })
        .sum();

    sum.try_into().or_wrap()
}

fn move_or_ignore(d: Direction, grid: &mut Grid<Rect, Tile>) -> Result<()> {
    let robot = find_robot(grid)?;
    let v = Vec2::from(d);

    let mut check_seen: Vec<Point> = robot.to_points();
    let mut check_root: Vec<Point> = robot.to_points();
    while !check_root.is_empty() {
        let check_curr = check_root.iter().map(|&p| p + v);

        let mut check_next = vec![];
        for e in grid.get_at_any(check_curr) {
            if e.data() == &Tile::Wall {
                // At least one tile is blocked, so nothing moves.
                return Ok(());
            }

            let area = e.area();
            check_next.extend(area.edge(d).to_points());
            check_seen.push(*area.pos());
        }

        check_root = check_next;
    }

    grid.move_by_from_any(v, &check_seen)
}

fn find_robot<A>(grid: &Grid<A, Tile>) -> Result<A>
where
    A: Into<Vec<Point>> + Clone + std::hash::Hash + Eq,
{
    let robot = grid
        .find_exactly_one(&Tile::Robot)
        .or_wrap_with(|| "Failed to find robot")?;

    Ok(robot.area().clone())
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
