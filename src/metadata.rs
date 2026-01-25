//! Metadata types for bridge boards and events.

use crate::Direction;
use std::fmt;

/// Player names for each seat at the table
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlayerNames {
    pub north: Option<String>,
    pub east: Option<String>,
    pub south: Option<String>,
    pub west: Option<String>,
}

impl PlayerNames {
    /// Create empty player names
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the player name for a direction
    pub fn get(&self, direction: Direction) -> Option<&str> {
        match direction {
            Direction::North => self.north.as_deref(),
            Direction::East => self.east.as_deref(),
            Direction::South => self.south.as_deref(),
            Direction::West => self.west.as_deref(),
        }
    }

    /// Set the player name for a direction
    pub fn set(&mut self, direction: Direction, name: impl Into<String>) {
        let name = name.into();
        let name = if name.is_empty() { None } else { Some(name) };
        match direction {
            Direction::North => self.north = name,
            Direction::East => self.east = name,
            Direction::South => self.south = name,
            Direction::West => self.west = name,
        }
    }

    /// Check if any player name is set
    pub fn has_any(&self) -> bool {
        self.north.as_ref().is_some_and(|s| !s.is_empty())
            || self.east.as_ref().is_some_and(|s| !s.is_empty())
            || self.south.as_ref().is_some_and(|s| !s.is_empty())
            || self.west.as_ref().is_some_and(|s| !s.is_empty())
    }

    /// Check if all player names are set
    pub fn is_complete(&self) -> bool {
        self.north.as_ref().is_some_and(|s| !s.is_empty())
            && self.east.as_ref().is_some_and(|s| !s.is_empty())
            && self.south.as_ref().is_some_and(|s| !s.is_empty())
            && self.west.as_ref().is_some_and(|s| !s.is_empty())
    }
}

/// Scoring method for a bridge game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScoringMethod {
    /// Matchpoints (pairs game)
    #[default]
    Matchpoints,
    /// International Match Points (teams)
    IMP,
    /// Board-a-Match (teams, win/lose/tie per board)
    BAM,
    /// Rubber bridge
    Rubber,
    /// Chicago (4-deal rubber)
    Chicago,
    /// Total points
    TotalPoints,
    /// Instant matchpoints (cross-IMP/Butler)
    InstantMP,
}

impl ScoringMethod {
    /// Parse from PBN Scoring tag value
    pub fn from_pbn(s: &str) -> Option<Self> {
        let s = s.trim().to_uppercase();
        // Handle modifiers like "MP;Butler" by taking the base method
        let base = s.split(';').next().unwrap_or(&s).trim();

        match base {
            "MP" | "MATCHPOINTS" | "MP%" => Some(ScoringMethod::Matchpoints),
            "IMP" | "IMPS" => Some(ScoringMethod::IMP),
            "BAM" | "BOARD-A-MATCH" => Some(ScoringMethod::BAM),
            "RUBBER" => Some(ScoringMethod::Rubber),
            "CHICAGO" | "CAVENDISH" => Some(ScoringMethod::Chicago),
            "TOTAL" | "TP" | "TOTALPOINTS" => Some(ScoringMethod::TotalPoints),
            "BUTLER" | "XIMP" | "INSTANTMP" => Some(ScoringMethod::InstantMP),
            _ => None,
        }
    }

    /// Convert to PBN notation
    pub fn to_pbn(&self) -> &'static str {
        match self {
            ScoringMethod::Matchpoints => "MP",
            ScoringMethod::IMP => "IMP",
            ScoringMethod::BAM => "BAM",
            ScoringMethod::Rubber => "Rubber",
            ScoringMethod::Chicago => "Chicago",
            ScoringMethod::TotalPoints => "Total",
            ScoringMethod::InstantMP => "Butler",
        }
    }

    /// Returns true if this is a teams scoring method
    pub fn is_teams(&self) -> bool {
        matches!(self, ScoringMethod::IMP | ScoringMethod::BAM)
    }

    /// Returns true if this is a pairs scoring method
    pub fn is_pairs(&self) -> bool {
        matches!(self, ScoringMethod::Matchpoints | ScoringMethod::InstantMP)
    }
}

impl fmt::Display for ScoringMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScoringMethod::Matchpoints => write!(f, "Matchpoints"),
            ScoringMethod::IMP => write!(f, "IMPs"),
            ScoringMethod::BAM => write!(f, "Board-a-Match"),
            ScoringMethod::Rubber => write!(f, "Rubber"),
            ScoringMethod::Chicago => write!(f, "Chicago"),
            ScoringMethod::TotalPoints => write!(f, "Total Points"),
            ScoringMethod::InstantMP => write!(f, "Butler/Instant MP"),
        }
    }
}

/// Room in a teams match
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Room {
    Open,
    Closed,
}

impl Room {
    /// Parse from PBN Room tag value
    pub fn from_pbn(s: &str) -> Option<Self> {
        match s.trim().to_uppercase().as_str() {
            "OPEN" | "1" => Some(Room::Open),
            "CLOSED" | "2" => Some(Room::Closed),
            _ => None,
        }
    }

    /// Convert to PBN notation
    pub fn to_pbn(&self) -> &'static str {
        match self {
            Room::Open => "Open",
            Room::Closed => "Closed",
        }
    }
}

impl fmt::Display for Room {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Room::Open => write!(f, "Open"),
            Room::Closed => write!(f, "Closed"),
        }
    }
}

/// Result of a played board (tricks taken by declarer)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardResult {
    /// Tricks taken by declarer (0-13)
    pub tricks: u8,
}

impl BoardResult {
    /// Create a new result
    pub fn new(tricks: u8) -> Self {
        Self { tricks: tricks.min(13) }
    }

    /// Parse from PBN Result tag value
    /// Accepts: "10", "=", "+2", "-1", etc.
    pub fn from_pbn(s: &str, contract_level: u8) -> Option<Self> {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }

        let tricks_needed = contract_level as i8 + 6;

        if s == "=" {
            return Some(Self::new(tricks_needed as u8));
        }

        if let Some(rest) = s.strip_prefix('+') {
            let overtricks: i8 = rest.parse().ok()?;
            return Some(Self::new((tricks_needed + overtricks) as u8));
        }

        if let Some(rest) = s.strip_prefix('-') {
            let undertricks: i8 = rest.parse().ok()?;
            return Some(Self::new((tricks_needed - undertricks).max(0) as u8));
        }

        // Plain number (absolute tricks)
        let tricks: u8 = s.parse().ok()?;
        Some(Self::new(tricks))
    }

    /// Format as PBN relative to contract level
    pub fn to_pbn(&self, contract_level: u8) -> String {
        let tricks_needed = contract_level + 6;
        if self.tricks == tricks_needed {
            "=".to_string()
        } else if self.tricks > tricks_needed {
            format!("+{}", self.tricks - tricks_needed)
        } else {
            format!("-{}", tricks_needed - self.tricks)
        }
    }

    /// Get the result relative to the contract
    pub fn relative_to(&self, contract_level: u8) -> i8 {
        self.tricks as i8 - (contract_level as i8 + 6)
    }

    /// Returns true if the contract was made
    pub fn made(&self, contract_level: u8) -> bool {
        self.tricks >= contract_level + 6
    }

    /// Returns the number of overtricks (positive) or undertricks (negative)
    pub fn overtricks(&self, contract_level: u8) -> i8 {
        self.relative_to(contract_level)
    }
}

impl fmt::Display for BoardResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} tricks", self.tricks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_names() {
        let mut names = PlayerNames::new();
        assert!(!names.has_any());

        names.set(Direction::North, "Alice");
        names.set(Direction::South, "Bob");
        assert!(names.has_any());
        assert!(!names.is_complete());

        assert_eq!(names.get(Direction::North), Some("Alice"));
        assert_eq!(names.get(Direction::South), Some("Bob"));
        assert_eq!(names.get(Direction::East), None);
    }

    #[test]
    fn test_scoring_method() {
        assert_eq!(ScoringMethod::from_pbn("MP"), Some(ScoringMethod::Matchpoints));
        assert_eq!(ScoringMethod::from_pbn("IMP"), Some(ScoringMethod::IMP));
        assert_eq!(ScoringMethod::from_pbn("MP;Butler"), Some(ScoringMethod::Matchpoints));

        assert!(ScoringMethod::IMP.is_teams());
        assert!(ScoringMethod::Matchpoints.is_pairs());
    }

    #[test]
    fn test_room() {
        assert_eq!(Room::from_pbn("Open"), Some(Room::Open));
        assert_eq!(Room::from_pbn("Closed"), Some(Room::Closed));
        assert_eq!(Room::from_pbn("1"), Some(Room::Open));
    }

    #[test]
    fn test_board_result() {
        // 3NT making exactly
        let result = BoardResult::from_pbn("=", 3).unwrap();
        assert_eq!(result.tricks, 9);
        assert!(result.made(3));
        assert_eq!(result.overtricks(3), 0);

        // 4S making with 2 overtricks
        let result = BoardResult::from_pbn("+2", 4).unwrap();
        assert_eq!(result.tricks, 12);
        assert!(result.made(4));
        assert_eq!(result.overtricks(4), 2);

        // 3NT down 1
        let result = BoardResult::from_pbn("-1", 3).unwrap();
        assert_eq!(result.tricks, 8);
        assert!(!result.made(3));
        assert_eq!(result.overtricks(3), -1);

        // Absolute tricks
        let result = BoardResult::from_pbn("10", 4).unwrap();
        assert_eq!(result.tricks, 10);
    }

    #[test]
    fn test_result_to_pbn() {
        let result = BoardResult::new(9);
        assert_eq!(result.to_pbn(3), "=");

        let result = BoardResult::new(12);
        assert_eq!(result.to_pbn(4), "+2");

        let result = BoardResult::new(8);
        assert_eq!(result.to_pbn(3), "-1");
    }
}
