use std::{
    cmp::Ordering,
    fmt,
    ops::{Add, Sub},
};

use lazy_errors::{prelude::*, Result};

use super::Direction;

#[derive(Copy, Debug, Clone, Default, PartialEq, Hash, Eq)]
pub struct Vector {
    y: isize,
    x: isize,
}

impl Vector {
    pub const DIRECTIONS: [Vector; 8] = [
        Vector::new(0, 1),
        Vector::new(1, 0),
        Vector::new(1, 1),
        Vector::new(0, -1),
        Vector::new(-1, 0),
        Vector::new(1, -1),
        Vector::new(-1, -1),
        Vector::new(-1, 1),
    ];
    pub const E_X: Vector = Vector::new(0, 1);
    pub const E_Y: Vector = Vector::new(1, 0);

    pub const fn new(y: isize, x: isize) -> Self {
        Self { y, x }
    }

    pub fn from_unsigned(y: usize, x: usize) -> Result<Self> {
        let y =
            isize::try_from(y).or_wrap_with(|| format!("Overflow: y={y}"))?;
        let x =
            isize::try_from(x).or_wrap_with(|| format!("Overflow: x={x}"))?;
        Ok(Self::new(y, x))
    }

    pub fn y(&self) -> isize {
        self.y
    }

    pub fn x(&self) -> isize {
        self.x
    }
}

impl From<Direction> for Vector {
    fn from(val: Direction) -> Self {
        match val {
            Direction::N => Vector::new(-1, 0),
            Direction::E => Vector::new(0, 1),
            Direction::S => Vector::new(1, 0),
            Direction::W => Vector::new(0, -1),
        }
    }
}

impl std::ops::Neg for Vector {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Vector {
            y: -self.y,
            x: -self.x,
        }
    }
}

impl Add for Vector {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let Vector { y: y_l, x: x_l } = self;
        let Vector { y: y_r, x: x_r } = rhs;
        Vector {
            y: y_l + y_r,
            x: x_l + x_r,
        }
    }
}

impl Sub for Vector {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let Self { y: y_l, x: x_l } = self;
        let Self { y: y_r, x: x_r } = rhs;
        Self {
            y: y_l - y_r,
            x: x_l - x_r,
        }
    }
}

impl std::ops::Mul<isize> for Vector {
    type Output = Self;

    fn mul(self, rhs: isize) -> Self::Output {
        let Vector { y, x } = self;
        Vector {
            y: y * rhs,
            x: x * rhs,
        }
    }
}

impl PartialOrd for Vector {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.y == other.y && self.x == other.x {
            return Some(Ordering::Equal);
        }

        if self.y <= other.y && self.x <= other.x {
            return Some(Ordering::Less);
        }

        if self.y >= other.y && self.x >= other.x {
            return Some(Ordering::Greater);
        }

        None
    }
}

impl fmt::Display for Vector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        fmt::Display::fmt(&self.y, f)?;
        write!(f, ",")?;
        fmt::Display::fmt(&self.x, f)?;
        write!(f, ")")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case(0, 0, Some(Ordering::Less))]
    #[test_case(0, 1, Some(Ordering::Less))]
    #[test_case(0, 2, None)]
    #[test_case(1, 0, Some(Ordering::Less))]
    #[test_case(1, 1, Some(Ordering::Equal))]
    #[test_case(1, 2, Some(Ordering::Greater))]
    #[test_case(2, 0, None)]
    #[test_case(2, 1, Some(Ordering::Greater))]
    #[test_case(2, 2, Some(Ordering::Greater))]
    fn partial_cmp(y: isize, x: isize, expectation: Option<Ordering>) {
        let p_l = Vector::new(y, x);
        let p_r = Vector::new(1, 1);
        assert_eq!(p_l.partial_cmp(&p_r), expectation);
    }
}
