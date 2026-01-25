//! Auction types for bridge bidding.

use std::collections::HashMap;
use std::fmt;

use crate::{Direction, Strain};

/// A single call in an auction
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Call {
    Pass,
    Bid { level: u8, strain: Strain },
    Double,
    Redouble,
}

impl Call {
    /// Create a bid call
    pub fn bid(level: u8, strain: Strain) -> Self {
        Call::Bid { level, strain }
    }

    /// Parse a call from PBN notation (e.g., "1C", "3NT", "Pass", "X", "XX")
    pub fn from_pbn(s: &str) -> Option<Self> {
        let s = s.trim();
        match s.to_uppercase().as_str() {
            "PASS" | "P" | "-" => Some(Call::Pass),
            "X" | "DBL" | "DOUBLE" => Some(Call::Double),
            "XX" | "RDBL" | "REDOUBLE" => Some(Call::Redouble),
            _ => {
                // Parse "1C", "2H", "3NT", etc.
                let mut chars = s.chars();
                let level = chars.next()?.to_digit(10)? as u8;
                if !(1..=7).contains(&level) {
                    return None;
                }
                let strain_str: String = chars.collect();
                let strain = Strain::from_str(&strain_str)?;
                Some(Call::Bid { level, strain })
            }
        }
    }

    /// Convert to PBN notation
    pub fn to_pbn(&self) -> String {
        match self {
            Call::Pass => "Pass".to_string(),
            Call::Double => "X".to_string(),
            Call::Redouble => "XX".to_string(),
            Call::Bid { level, strain } => format!("{}{}", level, strain.to_char()),
        }
    }

    /// Returns true if this is a Pass
    pub fn is_pass(&self) -> bool {
        matches!(self, Call::Pass)
    }

    /// Returns true if this is a Bid
    pub fn is_bid(&self) -> bool {
        matches!(self, Call::Bid { .. })
    }

    /// Returns true if this is a Double
    pub fn is_double(&self) -> bool {
        matches!(self, Call::Double)
    }

    /// Returns true if this is a Redouble
    pub fn is_redouble(&self) -> bool {
        matches!(self, Call::Redouble)
    }
}

impl fmt::Display for Call {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Call::Pass => write!(f, "Pass"),
            Call::Double => write!(f, "X"),
            Call::Redouble => write!(f, "XX"),
            Call::Bid { level, strain } => write!(f, "{}{}", level, strain),
        }
    }
}

/// A call with an optional annotation (alert, explanation)
#[derive(Debug, Clone)]
pub struct AnnotatedCall {
    pub call: Call,
    /// Optional annotation text (alert, explanation)
    pub annotation: Option<String>,
}

impl AnnotatedCall {
    /// Create an unannotated call
    pub fn new(call: Call) -> Self {
        Self {
            call,
            annotation: None,
        }
    }

    /// Create a call with annotation
    pub fn with_annotation(call: Call, annotation: impl Into<String>) -> Self {
        Self {
            call,
            annotation: Some(annotation.into()),
        }
    }

    /// Check if this call has an annotation
    pub fn has_annotation(&self) -> bool {
        self.annotation.is_some()
    }
}

/// A complete auction (bidding sequence)
#[derive(Debug, Clone)]
pub struct Auction {
    /// The dealer (first to call)
    pub dealer: Direction,
    /// Sequence of calls
    pub calls: Vec<AnnotatedCall>,
    /// Notes referenced by =N= in PBN (e.g., [Note "1:Forcing"])
    pub notes: HashMap<u8, String>,
}

impl Auction {
    /// Create a new auction with the given dealer
    pub fn new(dealer: Direction) -> Self {
        Self {
            dealer,
            calls: Vec::new(),
            notes: HashMap::new(),
        }
    }

    /// Add an unannotated call
    pub fn add_call(&mut self, call: Call) {
        self.calls.push(AnnotatedCall::new(call));
    }

    /// Add a call with optional annotation
    pub fn add_annotated_call(&mut self, call: Call, annotation: Option<String>) {
        self.calls.push(AnnotatedCall { call, annotation });
    }

    /// Add a note (for =N= references)
    pub fn add_note(&mut self, number: u8, text: impl Into<String>) {
        self.notes.insert(number, text.into());
    }

    /// Get the note text for a given number
    pub fn get_note(&self, number: u8) -> Option<&str> {
        self.notes.get(&number).map(|s| s.as_str())
    }

    /// Returns true if the auction ended in a passed-out deal (4 passes)
    pub fn is_passed_out(&self) -> bool {
        self.calls.len() >= 4
            && self.calls.iter().all(|ac| ac.call.is_pass())
    }

    /// Get the direction of the player who made the Nth call (0-indexed)
    pub fn caller(&self, index: usize) -> Direction {
        let mut dir = self.dealer;
        for _ in 0..index {
            dir = dir.next();
        }
        dir
    }

    /// Returns the final contract, if any (None if passed out)
    pub fn final_contract(&self) -> Option<FinalContract> {
        let mut last_bid: Option<(u8, Strain, Direction)> = None;
        let mut doubled = false;
        let mut redoubled = false;
        let mut current_player = self.dealer;

        for annotated in &self.calls {
            match &annotated.call {
                Call::Bid { level, strain } => {
                    last_bid = Some((*level, *strain, current_player));
                    doubled = false;
                    redoubled = false;
                }
                Call::Double => {
                    doubled = true;
                    redoubled = false;
                }
                Call::Redouble => {
                    doubled = false;
                    redoubled = true;
                }
                Call::Pass => {}
            }
            current_player = current_player.next();
        }

        last_bid.map(|(level, strain, declarer)| FinalContract {
            level,
            strain,
            doubled,
            redoubled,
            declarer,
        })
    }

    /// Check if this is an uncontested auction (only one partnership bids)
    /// Returns the bidding partnership if uncontested
    pub fn bidding_side(&self) -> Option<(Direction, Direction)> {
        let mut ns_bid = false;
        let mut ew_bid = false;

        let mut current = self.dealer;
        for annotated in &self.calls {
            if annotated.call.is_bid() || annotated.call.is_double() || annotated.call.is_redouble() {
                match current {
                    Direction::North | Direction::South => ns_bid = true,
                    Direction::East | Direction::West => ew_bid = true,
                }
            }
            current = current.next();
        }

        match (ns_bid, ew_bid) {
            (true, false) => Some((Direction::North, Direction::South)),
            (false, true) => Some((Direction::East, Direction::West)),
            _ => None,
        }
    }

    /// Returns the number of calls in the auction
    pub fn len(&self) -> usize {
        self.calls.len()
    }

    /// Returns true if the auction is empty
    pub fn is_empty(&self) -> bool {
        self.calls.is_empty()
    }
}

/// The final contract resulting from an auction
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalContract {
    pub level: u8,
    pub strain: Strain,
    pub doubled: bool,
    pub redoubled: bool,
    pub declarer: Direction,
}

impl FinalContract {
    /// Create a new final contract
    pub fn new(level: u8, strain: Strain, declarer: Direction) -> Self {
        Self {
            level,
            strain,
            doubled: false,
            redoubled: false,
            declarer,
        }
    }

    /// Builder: set doubled
    pub fn doubled(mut self) -> Self {
        self.doubled = true;
        self.redoubled = false;
        self
    }

    /// Builder: set redoubled
    pub fn redoubled(mut self) -> Self {
        self.doubled = false;
        self.redoubled = true;
        self
    }

    /// Convert to PBN notation (e.g., "4S", "3NTX", "6HXX")
    pub fn to_pbn(&self) -> String {
        let mut s = format!("{}{}", self.level, self.strain.to_char());
        if self.redoubled {
            s.push_str("XX");
        } else if self.doubled {
            s.push('X');
        }
        s
    }

    /// Parse from PBN notation
    pub fn from_pbn(s: &str, declarer: Direction) -> Option<Self> {
        let s = s.trim();
        if s.is_empty() || s.eq_ignore_ascii_case("Pass") {
            return None;
        }

        let level = s.chars().next()?.to_digit(10)? as u8;
        if !(1..=7).contains(&level) {
            return None;
        }

        let rest = &s[1..];
        let (strain_part, doubled, redoubled) = if let Some(stripped) = rest.strip_suffix("XX") {
            (stripped, false, true)
        } else if let Some(stripped) = rest.strip_suffix('X') {
            (stripped, true, false)
        } else {
            (rest, false, false)
        };

        let strain = Strain::from_str(strain_part)?;

        Some(FinalContract {
            level,
            strain,
            doubled,
            redoubled,
            declarer,
        })
    }
}

impl fmt::Display for FinalContract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.level, self.strain)?;
        if self.redoubled {
            write!(f, "XX")?;
        } else if self.doubled {
            write!(f, "X")?;
        }
        write!(f, " by {}", self.declarer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_parsing() {
        assert_eq!(Call::from_pbn("Pass"), Some(Call::Pass));
        assert_eq!(Call::from_pbn("P"), Some(Call::Pass));
        assert_eq!(Call::from_pbn("X"), Some(Call::Double));
        assert_eq!(Call::from_pbn("XX"), Some(Call::Redouble));
        assert_eq!(
            Call::from_pbn("1C"),
            Some(Call::Bid { level: 1, strain: Strain::Clubs })
        );
        assert_eq!(
            Call::from_pbn("3NT"),
            Some(Call::Bid { level: 3, strain: Strain::NoTrump })
        );
        assert_eq!(
            Call::from_pbn("7S"),
            Some(Call::Bid { level: 7, strain: Strain::Spades })
        );
    }

    #[test]
    fn test_call_display() {
        assert_eq!(Call::Pass.to_string(), "Pass");
        assert_eq!(Call::Double.to_string(), "X");
        assert_eq!(Call::Redouble.to_string(), "XX");
        assert_eq!(Call::bid(1, Strain::Clubs).to_string(), "1♣");
        assert_eq!(Call::bid(3, Strain::NoTrump).to_string(), "3NT");
    }

    #[test]
    fn test_auction_passed_out() {
        let mut auction = Auction::new(Direction::North);
        auction.add_call(Call::Pass);
        auction.add_call(Call::Pass);
        auction.add_call(Call::Pass);
        auction.add_call(Call::Pass);

        assert!(auction.is_passed_out());
        assert!(auction.final_contract().is_none());
    }

    #[test]
    fn test_auction_simple_contract() {
        let mut auction = Auction::new(Direction::North);
        auction.add_call(Call::bid(1, Strain::NoTrump));
        auction.add_call(Call::Pass);
        auction.add_call(Call::bid(3, Strain::NoTrump));
        auction.add_call(Call::Pass);
        auction.add_call(Call::Pass);
        auction.add_call(Call::Pass);

        let contract = auction.final_contract().unwrap();
        assert_eq!(contract.level, 3);
        assert_eq!(contract.strain, Strain::NoTrump);
        assert!(!contract.doubled);
        assert_eq!(contract.declarer, Direction::South);
    }

    #[test]
    fn test_auction_doubled_contract() {
        let mut auction = Auction::new(Direction::East);
        auction.add_call(Call::bid(4, Strain::Spades));
        auction.add_call(Call::Double);
        auction.add_call(Call::Pass);
        auction.add_call(Call::Pass);
        auction.add_call(Call::Pass);

        let contract = auction.final_contract().unwrap();
        assert_eq!(contract.level, 4);
        assert_eq!(contract.strain, Strain::Spades);
        assert!(contract.doubled);
        assert!(!contract.redoubled);
        assert_eq!(contract.declarer, Direction::East);
    }

    #[test]
    fn test_final_contract_display() {
        let contract = FinalContract::new(4, Strain::Spades, Direction::South).doubled();
        assert_eq!(contract.to_string(), "4♠X by South");

        let contract = FinalContract::new(6, Strain::Hearts, Direction::North).redoubled();
        assert_eq!(contract.to_string(), "6♥XX by North");
    }

    #[test]
    fn test_final_contract_pbn() {
        let contract = FinalContract::from_pbn("4SX", Direction::South).unwrap();
        assert_eq!(contract.level, 4);
        assert_eq!(contract.strain, Strain::Spades);
        assert!(contract.doubled);
        assert_eq!(contract.declarer, Direction::South);

        assert_eq!(contract.to_pbn(), "4SX");
    }

    #[test]
    fn test_annotated_call() {
        let call = AnnotatedCall::with_annotation(
            Call::bid(2, Strain::Clubs),
            "Stayman"
        );
        assert!(call.has_annotation());
        assert_eq!(call.annotation.as_deref(), Some("Stayman"));
    }

    #[test]
    fn test_auction_notes() {
        let mut auction = Auction::new(Direction::North);
        auction.add_note(1, "Forcing");
        auction.add_note(2, "Natural, 4+ cards");

        assert_eq!(auction.get_note(1), Some("Forcing"));
        assert_eq!(auction.get_note(2), Some("Natural, 4+ cards"));
        assert_eq!(auction.get_note(3), None);
    }

    #[test]
    fn test_auction_bidding_side() {
        let mut auction = Auction::new(Direction::North);
        auction.add_call(Call::bid(1, Strain::Spades));
        auction.add_call(Call::Pass);
        auction.add_call(Call::bid(2, Strain::Spades));
        auction.add_call(Call::Pass);
        auction.add_call(Call::Pass);
        auction.add_call(Call::Pass);

        // Only N-S bid
        assert_eq!(
            auction.bidding_side(),
            Some((Direction::North, Direction::South))
        );
    }
}
