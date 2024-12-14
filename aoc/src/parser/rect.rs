use std::fmt;

use super::{Point, Vector};

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
#[derive(Copy, Debug, Clone, Default, PartialEq, Hash, Eq)]
pub struct Rect {
    p: Point,
    v: Vector,
}

impl Rect {
    pub const fn new(p: Point, v: Vector) -> Self {
        Self { p, v }
    }

    pub fn pos(&self) -> Point {
        self.p
    }

    pub fn len(&self) -> Vector {
        self.v
    }

    pub fn contains(&self, &p: &Point) -> bool {
        if self.v.y() == 0 || self.v.x() == 0 {
            return false;
        }

        let p: Vector = p.into();
        let o: Vector = self.p.into();
        let v: Vector = self.v - Vector::new(1, 1);
        let q: Vector = o + v;

        o <= p && p <= q
    }

    /// Expands the rectangle in all four directions,
    /// without checking for overflows and
    /// without using saturating arithmetic.
    pub fn grow(&self) -> Self {
        let y = self.p.y();
        let x = self.p.x();
        let dy = self.v.y();
        let dx = self.v.x();

        let (y, dy) = match dy {
            0 => (y, 1),
            dy => (y - 1, dy + 2),
        };

        let (x, dx) = match dx {
            0 => (x, 1),
            dx => (x - 1, dx + 2),
        };

        let p = Point::new(y, x);
        let v = Vector::new(dy, dx);
        Rect::new(p, v)
    }
}

impl fmt::Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let y = self.p.y();
        let x = self.p.x();
        let dy = self.v.y();
        let dx = self.v.x();

        let y_end = y + dy;
        let x_end = x + dx;

        write!(f, "({y}..{y_end},{x}..{x_end})")
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case(1, 1, 0, 0, 1, 1, false)]
    #[test_case(1, 1, 1, 1, 1, 1, true)]
    #[test_case(0, 0, 1, 1, 1, 1, false)]
    #[test_case(0, 0, 2, 2, 1, 1, true)]
    #[test_case(0, 0, 3, 3, 1, 1, true)]
    #[test_case(0, 5, 3, 5, 2, 4, false)]
    #[test_case(6, 2, 3, 5, 6, 6, true)]
    fn contains(
        ry: isize,
        rx: isize,
        rdy: isize,
        rdx: isize,
        px: isize,
        py: isize,
        expectation: bool,
    ) {
        let rect = Rect::new(Point::new(ry, rx), Vector::new(rdy, rdx));
        let p = Point::new(px, py);
        assert_eq!(rect.contains(&p), expectation);
    }

    #[test_case(1, 1, 0, 0, 1, 1, 1, 1)]
    #[test_case(1, 1, 1, 1, 0, 0, 3, 3)]
    #[allow(clippy::too_many_arguments)]
    fn grow(
        y: isize,
        x: isize,
        dy: isize,
        dx: isize,
        grown_y: isize,
        grown_x: isize,
        grown_dy: isize,
        grown_dx: isize,
    ) {
        let input = Rect::new(Point::new(y, x), Vector::new(dy, dx));
        let output = Rect::new(
            Point::new(grown_y, grown_x),
            Vector::new(grown_dy, grown_dx),
        );
        assert_eq!(input.grow(), output);
    }
}
