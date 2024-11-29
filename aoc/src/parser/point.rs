use std::fmt;

use lazy_errors::{prelude::*, Result};

use super::{Rect, Vector};

#[derive(Copy, Debug, Clone, Default, PartialEq, Hash, Eq)]
pub struct Point(Vector);

impl Point {
    pub const fn new(y: usize, x: usize) -> Self {
        Self(Vector::new(y, x))
    }

    pub fn y(&self) -> usize {
        self.0.y()
    }

    pub fn x(&self) -> usize {
        self.0.x()
    }
}

impl TryFrom<Rect> for Point {
    type Error = Error;

    fn try_from(rect: Rect) -> Result<Self> {
        if rect.len() != Vector::new(1, 1) {
            return Err(err!("Not a point: '{rect}'"));
        }

        Ok(rect.pos())
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl From<Point> for Vector {
    fn from(value: Point) -> Self {
        value.0
    }
}
