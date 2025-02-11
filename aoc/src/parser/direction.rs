use lazy_errors::{prelude::*, Result};

/// Cardinal directions.
/// TODO: Merge with `IVec2::DIRECTIONS` (CardinalDir vs. DiagonalDir?)
#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum Direction {
    N,
    E,
    S,
    W,
}

impl Direction {
    pub const ALL: [Direction; 4] =
        [Direction::N, Direction::E, Direction::S, Direction::W];
}

impl Direction {
    pub fn rotate_clockwise(self) -> Self {
        use Direction::*;
        match self {
            E => S,
            S => W,
            W => N,
            N => E,
        }
    }
}

impl core::str::FromStr for Direction {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            ">" => Ok(Direction::E),
            "v" => Ok(Direction::S),
            "<" => Ok(Direction::W),
            "^" => Ok(Direction::N),
            _ => Err(err!("Not a direction: '{s}'")),
        }
    }
}
