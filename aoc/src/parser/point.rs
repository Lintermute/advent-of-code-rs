use std::fmt;

use lazy_errors::{prelude::*, Result};

use super::{Rect, Vector};

#[derive(Copy, Debug, Clone, Default, PartialEq, Hash, Eq)]
pub struct Point(Vector);

impl Point {
    pub const fn new(y: isize, x: isize) -> Self {
        Self(Vector::new(y, x))
    }

    pub fn from_unsigned(y: usize, x: usize) -> Result<Self> {
        Ok(Self(Vector::from_unsigned(y, x)?))
    }

    pub fn y(&self) -> isize {
        self.0.y()
    }

    pub fn x(&self) -> isize {
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

impl std::ops::Sub<Point> for Point {
    type Output = Vector;

    fn sub(self, rhs: Point) -> Self::Output {
        self.0 - rhs.0
    }
}

impl std::ops::Add<Vector> for Point {
    type Output = Self;

    fn add(self, rhs: Vector) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl std::ops::Sub<Vector> for Point {
    type Output = Self;

    fn sub(self, rhs: Vector) -> Self::Output {
        Self(self.0 - rhs)
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
