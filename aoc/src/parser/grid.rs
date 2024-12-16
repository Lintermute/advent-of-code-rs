use core::fmt;

use std::collections::HashSet;

use lazy_errors::Result;

use super::{Direction, Point, Rect, Vector};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Grid {
    bounds: Rect,
    tiles:  HashSet<Point>,
}

impl Grid {
    pub fn from<T: IntoIterator<Item = Point>>(bounds: Rect, iter: T) -> Self {
        let tiles = iter.into_iter().collect();
        Self { bounds, tiles }
    }

    pub fn from_str<'a, I>(
        input: &'a str,
        matcher: impl FnMut(&'a str) -> I + 'a,
    ) -> Result<Grid>
    where
        I: Iterator<Item = (usize, usize)> + 'a,
    {
        use itertools::Itertools;

        let bounds = super::parse_bounds(input)?;
        let tiles = super::parse_substrs(input.lines(), matcher)
            .map_ok(|(p, _): (Point, char)| p)
            .collect::<Result<_>>()?;

        Ok(Self { bounds, tiles })
    }

    pub fn neighbors(&self, p: &Point) -> Vec<(Point, Direction)> {
        Direction::ALL
            .iter()
            .flat_map(|&d| {
                let p = *p + Vector::from(d);
                if self.tiles.contains(&p) {
                    Some((p, d))
                } else {
                    None
                }
            })
            .collect()
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use itertools::Itertools;

        let y_min = self.bounds.pos().y();
        let y_len = self.bounds.len().y();
        let x_min = self.bounds.pos().x();
        let x_len = self.bounds.len().x();

        write!(
            f,
            "{}",
            (y_min..(y_min + y_len))
                .map(|y| {
                    (x_min..(x_min + x_len))
                        .map(|x| {
                            if self.tiles.contains(&Point::new(y, x)) {
                                '#'
                            } else {
                                ' '
                            }
                        })
                        .collect::<String>()
                })
                .join("\n")
        )
    }
}
