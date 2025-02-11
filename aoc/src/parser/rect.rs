use std::fmt;

use lazy_errors::{prelude::*, Result};

use crate::parser::{
    grid::Position,
    vec2::{IVec2, UVec2},
    Direction, Point,
};

const MSG_PANIC_AS_SIGNED: &str = "Rect: Failed to convert UVec2 to IVec2";
const MSG_PANIC_INTERNAL_ERROR: &str = "Rect: Internal Error";

/// A rectangle that is aligned to the 2D grid,
/// i.e. it's edges are parallel to the grid's axes.
///
/// `p` points to the edge with minimum `x` and `y` values.
/// That is, since axes start at the top-left,
/// `p` is the top-left edge of the rectangle.
/// `v` is a vector that measures the length of the diagonal.
/// In the most simple case, a point/“dot”, `v` is `(1,1)`.
/// Note that this description is sufficient to describe the size
/// because the rectangle is aligned to the grid.
#[derive(Debug, Copy, Clone, Default, PartialEq, Hash, Eq)]
pub struct Rect {
    p: Point,
    v: UVec2,
}

impl Rect {
    pub fn new(p: Point, v: UVec2) -> Result<Self> {
        let mut errs = ErrorStash::new(|| "Failed to create Rect");

        if v.y() == 0 {
            errs.push("v.y is 0");
        }

        if v.x() == 0 {
            errs.push("v.x is 0");
        }

        try2!(errs.ok());

        // Ensure that `p+v` does not exceed `isize` bounds:
        try2!(try_pos_max(&p, &v).or_stash(&mut errs));

        Ok(Self { p, v })
    }

    pub fn pos(&self) -> &Point {
        &self.p
    }

    pub fn len(&self) -> &UVec2 {
        &self.v
    }

    pub fn contains(&self, p: &Point) -> bool {
        let v: UVec2 = self.v - UVec2::new(1, 1);
        let v: IVec2 = as_ivec2_or_panic(v);

        let o: IVec2 = self.p.into();
        let p: IVec2 = (*p).into();
        let q: IVec2 = o + v;

        o <= p && p <= q
    }

    pub fn contains_all(&self, area: impl Into<Vec<Point>>) -> bool {
        area.into()
            .iter()
            .all(|p| self.contains(p))
    }

    /// Expands the rectangle in all four directions.
    pub fn grow(&self) -> Result<Self> {
        let y = self.p.y();
        let x = self.p.x();
        let dy = self.v.y();
        let dx = self.v.x();

        let (y, dy) = (y - 1, dy + 2);
        let (x, dx) = (x - 1, dx + 2);

        let p = Point::new(y, x);
        let v = UVec2::new(dy, dx);
        Rect::new(p, v)
    }

    pub fn edge(&self, d: Direction) -> Self {
        let Rect { p, v } = self;

        let v_hori = UVec2::new(1, v.x());
        let v_vert = UVec2::new(v.y(), 1);

        let (p, v) = match d {
            Direction::N => (*p, v_hori),
            Direction::W => (*p, v_vert),
            Direction::E => {
                let vx = as_isize_or_panic(self.len().x());
                let p = Point::new(p.y(), p.x() + vx - 1);
                (p, v_vert)
            }
            Direction::S => {
                let vy = as_isize_or_panic(self.len().y());
                let p = Point::new(p.y() + vy - 1, p.x());
                (p, v_hori)
            }
        };

        // Creating the `edge` Rect cannot fail
        // because its points are already part of the `self` Rect.
        Rect::new(p, v).expect(MSG_PANIC_INTERNAL_ERROR)
    }

    pub fn to_points(self) -> Vec<Point> {
        self.into()
    }

    fn pos_max(&self) -> Point {
        // This invariant must have been tested in all constructors.
        try_pos_max(&self.p, &self.v).expect(MSG_PANIC_INTERNAL_ERROR)
    }
}

impl From<Point> for Rect {
    fn from(p: Point) -> Self {
        // All origins `p` are valid and `v = (1,1)` is so for all `p`.
        Self::new(p, UVec2::new(1, 1)).expect(MSG_PANIC_INTERNAL_ERROR)
    }
}

impl core::ops::Add<IVec2> for Rect {
    type Output = Self;

    fn add(self, rhs: IVec2) -> Self::Output {
        let Self { p, v } = self;
        let p = p + rhs;
        Self::new(p, v).expect("Rect: Overflow when adding Vec2")
    }
}

impl Position for Rect {
    fn position(&self) -> &Point {
        &self.p
    }
}

impl fmt::Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (y, x) = (self.pos().y(), self.pos().x());

        let p_max = self.pos_max();
        let (y_max, x_max) = (p_max.y(), p_max.x());

        write!(f, "({y}..={y_max},{x}..={x_max})")
    }
}

impl From<Rect> for Vec<Point> {
    fn from(val: Rect) -> Self {
        let y_min = val.p.y();
        let x_min = val.p.x();

        let p_max = val.pos_max();
        let (y_max, x_max) = (p_max.y(), p_max.x());

        (y_min..=y_max)
            .flat_map(|y| (x_min..=x_max).map(move |x| Point::new(y, x)))
            .collect()
    }
}

/// Returns `pos_max`, i.e. the [`Point`] that's farthest away
/// from the origin ([`Rect::pos()`]).
///
/// [`Point`] is backed by [`isize`] while
/// the size of the `Rect` is represented as [`IVec2`],
/// which is backed by [`usize`].
/// Thus, computing `pos_max` can cause a overflow.
/// In that case, this method will return `Err`.
///
/// This method thus also serves to ensure the invariant that
/// all points within the rectangle can actually be represented.
/// Call this method before returning new instances of [`Rect`]
/// to make sure the invariant holds.
fn try_pos_max(p: &Point, v: &UVec2) -> Result<Point> {
    let (y, x) = (p.y(), p.x());
    let (vy, vx) = (v.y(), v.x());

    let y = y
        .checked_add_unsigned(vy - 1)
        .ok_or_else(|| err!("Overflow on y+vy = {y}+{vy}"))?;

    let x = x
        .checked_add_unsigned(vx - 1)
        .ok_or_else(|| err!("Overflow on x+vx = {x}+{vx}"))?;

    Ok(Point::new(y, x))
}

fn as_ivec2_or_panic(v: UVec2) -> IVec2 {
    IVec2::try_from(v).expect(MSG_PANIC_AS_SIGNED)
}

fn as_isize_or_panic(k: usize) -> isize {
    isize::try_from(k).expect(MSG_PANIC_AS_SIGNED)
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case(1, 1, 1, 1, 1, 1, true)]
    #[test_case(0, 0, 1, 1, 1, 1, false)]
    #[test_case(0, 0, 2, 2, 1, 1, true)]
    #[test_case(0, 0, 3, 3, 1, 1, true)]
    #[test_case(0, 5, 3, 5, 2, 4, false)]
    #[test_case(6, 2, 3, 5, 6, 6, true)]
    fn contains(
        ry: isize,
        rx: isize,
        rdy: usize,
        rdx: usize,
        px: isize,
        py: isize,
        expectation: bool,
    ) -> Result<()> {
        let rect = Rect::new(Point::new(ry, rx), UVec2::new(rdy, rdx))?;
        let p = Point::new(px, py);
        assert_eq!(rect.contains(&p), expectation);
        Ok(())
    }

    #[test_case(1, 1, 1, 1, 0, 0, 3, 3)]
    #[allow(clippy::too_many_arguments)]
    fn grow(
        y: isize,
        x: isize,
        dy: usize,
        dx: usize,
        grown_y: isize,
        grown_x: isize,
        grown_dy: usize,
        grown_dx: usize,
    ) -> Result<()> {
        let input = Rect::new(Point::new(y, x), UVec2::new(dy, dx))?;
        let output = Rect::new(
            Point::new(grown_y, grown_x),
            UVec2::new(grown_dy, grown_dx),
        )?;
        assert_eq!(input.grow()?, output);
        Ok(())
    }
}
