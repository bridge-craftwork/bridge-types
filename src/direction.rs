//! Direction (table position) type for bridge.

use std::fmt;

/// Represents the four positions at a bridge table
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    /// All directions in clockwise order starting from North
    pub const ALL: [Direction; 4] = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];

    /// Parse direction from a character
    pub fn from_char(c: char) -> Option<Self> {
        match c.to_ascii_uppercase() {
            'N' => Some(Direction::North),
            'E' => Some(Direction::East),
            'S' => Some(Direction::South),
            'W' => Some(Direction::West),
            _ => None,
        }
    }

    /// Get the direction as a single character
    pub fn to_char(&self) -> char {
        match self {
            Direction::North => 'N',
            Direction::East => 'E',
            Direction::South => 'S',
            Direction::West => 'W',
        }
    }

    /// Get the next direction clockwise
    pub fn next(&self) -> Direction {
        match self {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
        }
    }

    /// Get the previous direction (counter-clockwise)
    pub fn prev(&self) -> Direction {
        self.next().next().next()
    }

    /// Get the partner's direction (opposite)
    pub fn partner(&self) -> Direction {
        self.next().next()
    }

    /// Returns directions in clockwise order starting from this direction
    pub fn clockwise_from(&self) -> [Direction; 4] {
        [*self, self.next(), self.next().next(), self.next().next().next()]
    }

    /// Convert to index (North=0, East=1, South=2, West=3)
    pub fn to_index(&self) -> usize {
        match self {
            Direction::North => 0,
            Direction::East => 1,
            Direction::South => 2,
            Direction::West => 3,
        }
    }

    /// Convert from index
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Direction::North),
            1 => Some(Direction::East),
            2 => Some(Direction::South),
            3 => Some(Direction::West),
            _ => None,
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Direction::North => write!(f, "North"),
            Direction::East => write!(f, "East"),
            Direction::South => write!(f, "South"),
            Direction::West => write!(f, "West"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_next() {
        assert_eq!(Direction::North.next(), Direction::East);
        assert_eq!(Direction::East.next(), Direction::South);
        assert_eq!(Direction::South.next(), Direction::West);
        assert_eq!(Direction::West.next(), Direction::North);
    }

    #[test]
    fn test_direction_prev() {
        assert_eq!(Direction::North.prev(), Direction::West);
        assert_eq!(Direction::East.prev(), Direction::North);
    }

    #[test]
    fn test_direction_partner() {
        assert_eq!(Direction::North.partner(), Direction::South);
        assert_eq!(Direction::East.partner(), Direction::West);
        assert_eq!(Direction::South.partner(), Direction::North);
        assert_eq!(Direction::West.partner(), Direction::East);
    }

    #[test]
    fn test_direction_from_char() {
        assert_eq!(Direction::from_char('N'), Some(Direction::North));
        assert_eq!(Direction::from_char('e'), Some(Direction::East));
        assert_eq!(Direction::from_char('X'), None);
    }

    #[test]
    fn test_direction_clockwise() {
        assert_eq!(
            Direction::East.clockwise_from(),
            [Direction::East, Direction::South, Direction::West, Direction::North]
        );
    }

    #[test]
    fn test_direction_index() {
        for (i, dir) in Direction::ALL.iter().enumerate() {
            assert_eq!(dir.to_index(), i);
            assert_eq!(Direction::from_index(i), Some(*dir));
        }
    }
}
