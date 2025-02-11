use core::{
    cmp::Ordering,
    convert::identity,
    fmt,
    ops::{Add, Mul, Sub},
};

use lazy_errors::{prelude::*, Result};

use crate::parser::Direction;

pub type UVec2 = Vec2<usize>;

pub type IVec2 = Vec2<isize>;

#[derive(Debug, Copy, Clone, Default, PartialEq, Hash, Eq)]
pub struct Vec2<T> {
    y: T,
    x: T,
}

impl IVec2 {
    /// Cardinal and intercardinal directions.
    /// TODO: Merge with `Direction` (Direction<4>)
    pub const DIRECTIONS: [IVec2; 8] = [
        Vec2::new(0, 1),
        Vec2::new(1, 0),
        Vec2::new(1, 1),
        Vec2::new(0, -1),
        Vec2::new(-1, 0),
        Vec2::new(1, -1),
        Vec2::new(-1, -1),
        Vec2::new(-1, 1),
    ];
    pub const E_X: IVec2 = Vec2::new(0, 1);
    pub const E_Y: IVec2 = Vec2::new(1, 0);
}

impl<T: Copy> Vec2<T> {
    pub const fn new(y: T, x: T) -> Self {
        Self { y, x }
    }

    pub const fn y(&self) -> T {
        self.y
    }

    pub const fn x(&self) -> T {
        self.x
    }

    /// TODO: This should be a [`TryFrom`] implementation,
    /// but apparently that's not possible.
    /// Probably related to: https://github.com/rust-lang/rust/issues/50133
    pub fn try_from<U>(v: Vec2<U>) -> Result<Self>
    where
        U: Copy,
        T: TryFrom<U>,
        <T as TryFrom<U>>::Error: Into<Stashable>,
    {
        let mut errs = ErrorStash::new(|| "Failed to convert 2D vector");

        let y: Result<T> = v
            .y()
            .try_into()
            .or_wrap_with(|| "Failed to convert y={y}");

        let x: Result<T> = v
            .x()
            .try_into()
            .or_wrap_with(|| "Failed to convert x={x}");

        let [y, x] = try2!([y, x].try_map_or_stash(identity, &mut errs));

        Ok(Self { y, x })
    }
}

impl From<Direction> for IVec2 {
    fn from(val: Direction) -> Self {
        match val {
            Direction::N => Vec2::new(-1, 0),
            Direction::E => Vec2::new(0, 1),
            Direction::S => Vec2::new(1, 0),
            Direction::W => Vec2::new(0, -1),
        }
    }
}

impl std::ops::Neg for IVec2 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Vec2 {
            y: -self.y,
            x: -self.x,
        }
    }
}

impl<T: Add<Output = T>> Add for Vec2<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let Self { y: y_l, x: x_l } = self;
        let Self { y: y_r, x: x_r } = rhs;
        Self::Output {
            y: y_l + y_r,
            x: x_l + x_r,
        }
    }
}

impl<T: Sub<Output = T>> Sub for Vec2<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let Self { y: y_l, x: x_l } = self;
        let Self { y: y_r, x: x_r } = rhs;
        Self::Output {
            y: y_l - y_r,
            x: x_l - x_r,
        }
    }
}

impl<T: Copy + Mul<Output = T>> Mul<T> for Vec2<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        let Self { y, x } = self;
        Self::Output {
            y: y * rhs,
            x: x * rhs,
        }
    }
}

impl<T: PartialOrd> PartialOrd for Vec2<T> {
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

impl<T: fmt::Display> fmt::Display for Vec2<T> {
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
        let p_l = Vec2::new(y, x);
        let p_r = Vec2::new(1, 1);
        assert_eq!(p_l.partial_cmp(&p_r), expectation);
    }
}
