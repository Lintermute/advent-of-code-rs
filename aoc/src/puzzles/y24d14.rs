use lazy_errors::{prelude::*, Result};

use crate::parser::{
    self,
    vec2::{IVec2, UVec2},
    Grid, Point, Rect,
};

const Y_LEN: isize = 103;
const X_LEN: isize = 101;
const GRID_BOUNDS: UVec2 = UVec2::new(103, 101);

pub struct Robot {
    p: Point,
    v: IVec2,
}

pub fn parse(input: &str) -> Result<Vec<Robot>> {
    parser::parse_each(input.lines()).collect()
}

pub fn part1(robots: &[Robot]) -> Result<usize> {
    part1_impl(robots, Y_LEN, X_LEN)
}

pub fn part2(robots: &[Robot]) -> Result<isize> {
    use itertools::Itertools;

    let bounds: Rect = Rect::new(Point::new(0, 0), GRID_BOUNDS)?;

    for steps in 0..isize::MAX {
        let robot_points = move_robots(robots, steps, Y_LEN, X_LEN);
        let grid = Grid::from_points(bounds, robot_points.unique())?;

        if parser::contains_2d(&grid.to_string(), indoc::indoc! {"\
                    ###############################
                    #                             #
                    #                             #
                    #                             #
                    #                             #
                    #              #              #
                    #             ###             #
                    #            #####            #
                    #           #######           #
                    #          #########          #
                    #            #####            #
                    #           #######           #
                    #          #########          #
                    #         ###########         #
                    #        #############        #
                    #          #########          #
                    #         ###########         #
                    #        #############        #
                    #       ###############       #
                    #      #################      #
                    #        #############        #
                    #       ###############       #
                    #      #################      #
                    #     ###################     #
                    #    #####################    #
                    #             ###             #
                    #             ###             #
                    #             ###             #
                    #                             #
                    #                             #
                    #                             #
                    #                             #
                    ###############################
            "})
        {
            return Ok(steps);
        }
    }

    Err(err!("Easter eggs? On christmas?!"))
}

impl core::str::FromStr for Robot {
    type Err = Error;

    fn from_str(line: &str) -> Result<Self> {
        let err = || err!("Failed to parse line '{line}'");

        let [p, v] = line
            .strip_prefix("p=")
            .ok_or_else(err)?
            .split(" v=")
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| err())?;

        let (y, x) = parse_coords(p)?;
        let p = Point::new(y, x);

        let (y, x) = parse_coords(v)?;
        let v = IVec2::new(y, x);

        Ok(Self { p, v })
    }
}

fn parse_coords(s: &str) -> Result<(isize, isize)> {
    use core::str::FromStr;

    let err = || err!("Failed to parse coordinates '{s}'");

    let [x, y] = s
        .split(',')
        .map(|k| isize::from_str(k).or_wrap_with(err))
        .collect::<Result<Vec<_>>>()?
        .try_into()
        .map_err(|_| err())?;

    Ok((y, x))
}

fn part1_impl(input: &[Robot], y_len: isize, x_len: isize) -> Result<usize> {
    let y_mid = y_len / 2;
    let x_mid = x_len / 2;

    let quads = move_robots(input, 100, y_len, x_len)
        .map(|p| {
            let y = p.y().cmp(&y_mid);
            let x = p.x().cmp(&x_mid);

            use core::cmp::Ordering::*;
            match (y, x) {
                (Less, Less) => (1, 0, 0, 0),
                (Less, Greater) => (0, 1, 0, 0),
                (Greater, Less) => (0, 0, 1, 0),
                (Greater, Greater) => (0, 0, 0, 1),
                _ => (0, 0, 0, 0),
            }
        })
        .reduce(|(acc1, acc2, acc3, acc4), (q1, q2, q3, q4)| {
            (acc1 + q1, acc2 + q2, acc3 + q3, acc4 + q4)
        })
        .unwrap();

    Ok(quads.0 * quads.1 * quads.2 * quads.3)
}

fn move_robots(
    robots: &[Robot],
    steps: isize,
    y_len: isize,
    x_len: isize,
) -> impl Iterator<Item = Point> + use<'_> {
    use num::Integer;
    robots.iter().map(move |r| {
        let p = r.p + (r.v * steps);
        let (y, x) = (p.y(), p.x());
        let (y, x) = (y.mod_floor(&y_len), x.mod_floor(&x_len));
        Point::new(y, x)
    })
}

#[cfg(test)]
mod tests {
    use lazy_errors::Result;
    use test_case::test_case;

    use crate::{day::*, fs::Config, year::*, Part};

    #[test_case(Y24, D14, "1", Part::Part1, 12)]
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
                let result = super::part1_impl(&input, 7, 11)?;
                assert_eq!(result, expected);
            }
            Part::Part2 => {
                let result = super::part2(&input)?;
                let result = usize::try_from(result).unwrap();
                assert_eq!(result, expected);
            }
        };

        Ok(())
    }
}
