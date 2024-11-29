use std::{
    cmp::Ordering,
    fmt,
    ops::{Add, Sub},
};

#[derive(Copy, Debug, Clone, Default, PartialEq, Hash, Eq)]
pub struct Vector {
    y: usize,
    x: usize,
}

impl Vector {
    pub const fn new(y: usize, x: usize) -> Self {
        Self { y, x }
    }

    pub fn y(&self) -> usize {
        self.y
    }

    pub fn x(&self) -> usize {
        self.x
    }
}

impl Add for Vector {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let Self { y: y_l, x: x_l } = self;
        let Self { y: y_r, x: x_r } = rhs;
        Self {
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
    fn partial_cmp(y: usize, x: usize, expectation: Option<Ordering>) {
        let p_l = Vector::new(y, x);
        let p_r = Vector::new(1, 1);
        assert_eq!(p_l.partial_cmp(&p_r), expectation);
    }
}
