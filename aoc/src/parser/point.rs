use std::fmt;

use lazy_errors::{prelude::*, Result};

use crate::parser::{
    grid::Position,
    vec2::{IVec2, UVec2},
    Rect, Vec2,
};

#[derive(Debug, Copy, Clone, Default, PartialEq, Hash, Eq)]
pub struct Point(IVec2);

impl Point {
    pub const ZERO: Point = Point::new(0, 0);

    pub const fn new(y: isize, x: isize) -> Self {
        Self(Vec2::new(y, x))
    }

    pub fn from_unsigned(y: usize, x: usize) -> Result<Self> {
        let v = UVec2::new(y, x);
        let v = IVec2::try_from(v)?;
        Ok(Self(v))
    }

    pub const fn y(&self) -> isize {
        self.0.y()
    }

    pub const fn x(&self) -> isize {
        self.0.x()
    }
}

impl TryFrom<Rect> for Point {
    type Error = Error;

    fn try_from(rect: Rect) -> Result<Self> {
        if rect.len() != &Vec2::new(1, 1) {
            return Err(err!("Not a point: '{rect}'"));
        }

        Ok(*rect.pos())
    }
}

impl std::ops::Sub<Point> for Point {
    type Output = IVec2;

    fn sub(self, rhs: Point) -> Self::Output {
        self.0 - rhs.0
    }
}

impl std::ops::Sub<&Point> for &Point {
    type Output = IVec2;

    fn sub(self, rhs: &Point) -> Self::Output {
        self.0 - rhs.0
    }
}

impl std::ops::Add<IVec2> for Point {
    type Output = Self;

    fn add(self, rhs: IVec2) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl std::ops::Sub<IVec2> for Point {
    type Output = Self;

    fn sub(self, rhs: IVec2) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl Position for Point {
    fn position(&self) -> &Point {
        self
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl From<Point> for IVec2 {
    fn from(value: Point) -> Self {
        value.0
    }
}

impl From<Point> for Vec<Point> {
    fn from(val: Point) -> Self {
        vec![val]
    }
}
