use lazy_errors::{prelude::*, Result};

use crate::parser::{self, Grid, Point, Rect, Vector};

const BOUNDS: Rect = Rect::new(Point::new(0, 0), Vector::new(103, 101));

pub struct Robot {
    p: Point,
    v: Vector,
}

pub fn parse(input: &str) -> Result<Vec<Robot>> {
    parser::parse_each(input.lines()).collect()
}

pub fn part1(input: &[Robot]) -> Result<usize> {
    part1_impl(input, &BOUNDS)
}

pub fn part2(input: &[Robot]) -> Result<usize> {
    (0..usize::MAX)
        .find(|&steps| {
            let grid = Grid::from(BOUNDS, move_robots(input, steps, &BOUNDS));
            parser::contains_2d(&grid.to_string(), indoc::indoc! {"\
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
        })
        .ok_or_else(|| err!("Easter eggs? On christmas?!"))
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
        let v = Vector::new(y, x);

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

fn part1_impl(input: &[Robot], bounds: &Rect) -> Result<usize> {
    let y_mid = bounds.len().y() / 2;
    let x_mid = bounds.len().x() / 2;

    let quads = move_robots(input, 100, bounds)
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

fn move_robots<'a>(
    robots: &'a [Robot],
    steps: usize,
    bounds: &Rect,
) -> impl Iterator<Item = Point> + 'a {
    use num::Integer;

    let steps = isize::try_from(steps).unwrap();
    let y_len = bounds.len().y();
    let x_len = bounds.len().x();

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

    use crate::{
        day::*,
        fs::Config,
        parser::{Point, Rect, Vector},
        year::*,
        Part,
    };

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
                let bounds = Rect::new(Point::new(0, 0), Vector::new(7, 11));
                let result = super::part1_impl(&input, &bounds)?;
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
