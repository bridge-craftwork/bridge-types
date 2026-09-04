//! Auction types for bridge bidding.

use std::collections::HashMap;
use std::fmt;

use crate::{Direction, Strain};

/// A single call in an auction
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Call {
    Pass,
    Bid {
        level: u8,
        strain: Strain,
    },
    Double,
    Redouble,
    /// Indicates auction continues (used in teaching materials where student fills in next bid)
    /// Represented as "+" in PBN, typically displayed as "?"
    Continue,
    /// A blank placeholder for fill-in-the-blank exercises
    /// Represented as one or more underscores in PBN (e.g., "____" or "_____")
    /// Typically displayed as a horizontal line for students to write their answer
    Blank,
}

impl Call {
    /// Create a bid call
    pub fn bid(level: u8, strain: Strain) -> Self {
        Call::Bid { level, strain }
    }

    /// Parse a call from PBN notation (e.g., "1C", "3NT", "Pass", "X", "XX", "+", "____")
    pub fn from_pbn(s: &str) -> Option<Self> {
        let s = s.trim();
        match s.to_uppercase().as_str() {
            "PASS" | "P" | "-" => Some(Call::Pass),
            "X" | "DBL" | "DOUBLE" => Some(Call::Double),
            "XX" | "RDBL" | "REDOUBLE" => Some(Call::Redouble),
            "+" => Some(Call::Continue),
            _ => {
                // Check for underscore sequences (blanks for fill-in exercises)
                if s.chars().all(|c| c == '_') && !s.is_empty() {
                    return Some(Call::Blank);
                }

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
            Call::Continue => "+".to_string(),
            Call::Blank => "_____".to_string(),
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

    /// Returns true if this is a Continue marker
    pub fn is_continue(&self) -> bool {
        matches!(self, Call::Continue)
    }

    /// Returns true if this is a Blank (fill-in exercise placeholder)
    pub fn is_blank(&self) -> bool {
        matches!(self, Call::Blank)
    }
}

impl fmt::Display for Call {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Call::Pass => write!(f, "Pass"),
            Call::Double => write!(f, "X"),
            Call::Redouble => write!(f, "XX"),
            Call::Continue => write!(f, "?"),
            Call::Blank => write!(f, "_____"),
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

/// How a section's token sequence ended.
///
/// The auction and play sections may each close with a marker token. The two
/// markers are mutually exclusive: the standard forbids `*` where `+` is used.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SectionEnd {
    /// No marker. The section ran to its natural end — all passes, or thirteen
    /// tricks — or simply stops where the data stops.
    #[default]
    Unmarked,
    /// `*`: no further calls or cards will or can be given. The section is
    /// closed, though not necessarily complete — a board may record only
    /// `[Play "W"]` and a `*` to say the play is not on record.
    Terminated,
    /// `+`: the next call or card is to be made another time, as when a game is
    /// saved for another program to continue.
    Continued,
}

impl SectionEnd {
    /// The marker this end is written as, or `None` when there is none.
    pub fn to_pbn(self) -> Option<&'static str> {
        match self {
            SectionEnd::Unmarked => None,
            SectionEnd::Terminated => Some("*"),
            SectionEnd::Continued => Some("+"),
        }
    }

    /// Read a marker token, or `None` if the token is not one.
    pub fn from_pbn(token: &str) -> Option<Self> {
        match token {
            "*" => Some(SectionEnd::Terminated),
            "+" => Some(SectionEnd::Continued),
            _ => None,
        }
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
    /// The marker the auction closed with, if any. Preserved so a section that
    /// is nothing but `*` still says so when written back.
    pub end: SectionEnd,
}

impl Auction {
    /// Create a new auction with the given dealer
    pub fn new(dealer: Direction) -> Self {
        Self {
            dealer,
            calls: Vec::new(),
            notes: HashMap::new(),
            end: SectionEnd::default(),
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
        self.calls.len() >= 4 && self.calls.iter().all(|ac| ac.call.is_pass())
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
    ///
    /// The declarer is the *first* player of the contract side to have named
    /// the final strain — not whoever made the last bid. Over
    /// `1S - Pass - 4S` the opener declares, not the raiser. Declarer fixes
    /// the opening leader (declarer's LHO), so this distinction matters to
    /// anything that goes on to play or solve the deal.
    pub fn final_contract(&self) -> Option<FinalContract> {
        /// Index a strain without requiring `Hash`/`Ord` on it.
        fn strain_index(strain: Strain) -> usize {
            match strain {
                Strain::Clubs => 0,
                Strain::Diamonds => 1,
                Strain::Hearts => 2,
                Strain::Spades => 3,
                Strain::NoTrump => 4,
            }
        }
        fn is_ns(seat: Direction) -> bool {
            matches!(seat, Direction::North | Direction::South)
        }

        let mut last_bid: Option<(u8, Strain, Direction)> = None;
        let mut doubled = false;
        let mut redoubled = false;
        let mut current_player = self.dealer;
        // [side][strain] -> the first seat of that side to name that strain.
        let mut first_named: [[Option<Direction>; 5]; 2] = [[None; 5], [None; 5]];

        for annotated in &self.calls {
            match &annotated.call {
                Call::Bid { level, strain } => {
                    let side = usize::from(is_ns(current_player));
                    first_named[side][strain_index(*strain)].get_or_insert(current_player);
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
                Call::Pass | Call::Continue | Call::Blank => {}
            }
            current_player = current_player.next();
        }

        last_bid.map(|(level, strain, bidder)| FinalContract {
            level,
            strain,
            doubled,
            redoubled,
            // The final bidder's own side named the strain at least once (that
            // bid), so the lookup always hits; fall back to the bidder rather
            // than discard an otherwise valid contract.
            declarer: first_named[usize::from(is_ns(bidder))][strain_index(strain)]
                .unwrap_or(bidder),
        })
    }

    /// Check if this is an uncontested auction (only one partnership bids)
    /// Returns the bidding partnership if uncontested
    pub fn bidding_side(&self) -> Option<(Direction, Direction)> {
        let mut ns_bid = false;
        let mut ew_bid = false;

        let mut current = self.dealer;
        for annotated in &self.calls {
            // Active calls that count as bidding (not Pass, Continue, or Blank)
            if annotated.call.is_bid() || annotated.call.is_double() || annotated.call.is_redouble()
            {
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
            Some(Call::Bid {
                level: 1,
                strain: Strain::Clubs
            })
        );
        assert_eq!(
            Call::from_pbn("3NT"),
            Some(Call::Bid {
                level: 3,
                strain: Strain::NoTrump
            })
        );
        assert_eq!(
            Call::from_pbn("7S"),
            Some(Call::Bid {
                level: 7,
                strain: Strain::Spades
            })
        );
    }

    #[test]
    fn test_call_display() {
        assert_eq!(Call::Pass.to_string(), "Pass");
        assert_eq!(Call::Double.to_string(), "X");
        assert_eq!(Call::Redouble.to_string(), "XX");
        assert_eq!(Call::Continue.to_string(), "?");
        assert_eq!(Call::bid(1, Strain::Clubs).to_string(), "1♣");
        assert_eq!(Call::bid(3, Strain::NoTrump).to_string(), "3NT");
    }

    #[test]
    fn test_call_continue() {
        assert_eq!(Call::from_pbn("+"), Some(Call::Continue));
        assert!(Call::Continue.is_continue());
        assert_eq!(Call::Continue.to_pbn(), "+");
    }

    #[test]
    fn test_call_blank() {
        // Single underscore
        assert_eq!(Call::from_pbn("_"), Some(Call::Blank));
        // Multiple underscores (common in teaching materials)
        assert_eq!(Call::from_pbn("____"), Some(Call::Blank));
        assert_eq!(Call::from_pbn("_____"), Some(Call::Blank));
        assert!(Call::Blank.is_blank());
        assert_eq!(Call::Blank.to_pbn(), "_____");
        assert_eq!(Call::Blank.to_string(), "_____");
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
        // North named notrump first, so North declares — not South, who made
        // the last bid.
        assert_eq!(contract.declarer, Direction::North);
    }

    /// Build an auction from PBN call tokens, e.g. `"1S Pass 4S Pass Pass Pass"`.
    fn auction_from_pbn(dealer: Direction, calls: &str) -> Auction {
        let mut auction = Auction::new(dealer);
        for token in calls.split_whitespace() {
            auction.add_call(Call::from_pbn(token).expect("valid call token"));
        }
        auction
    }

    #[test]
    fn test_declarer_is_first_to_name_strain_not_last_bidder() {
        // North opens 1S, South raises to 4S. North named spades first, so
        // North declares and East is on lead.
        let auction = auction_from_pbn(Direction::North, "1S Pass 4S Pass Pass Pass");

        let contract = auction.final_contract().unwrap();
        assert_eq!(contract.level, 4);
        assert_eq!(contract.strain, Strain::Spades);
        assert_eq!(contract.declarer, Direction::North);
        // Opening leader is declarer's LHO — the reason this matters.
        assert_eq!(contract.declarer.next(), Direction::East);
    }

    #[test]
    fn test_declarer_reference_board_from_bbo() {
        // A real board from BBO, cross-checked against Bridge Base's own BSOL
        // analysis. Dealer South; East bids the final 3NT, but West named
        // notrump first with the 1NT opener, so West declares. West holds 16
        // HCP (A5.AK97.732.KQ72), which independently confirms the opener.
        let auction = auction_from_pbn(
            Direction::South,
            "Pass 1NT Pass 2C Pass 2H Pass 3NT Pass Pass Pass",
        );

        let contract = auction.final_contract().unwrap();
        assert_eq!(contract.level, 3);
        assert_eq!(contract.strain, Strain::NoTrump);
        assert!(!contract.doubled);
        assert!(!contract.redoubled);
        assert_eq!(contract.declarer, Direction::West);
        assert_eq!(contract.declarer.next(), Direction::North);
    }

    #[test]
    fn test_declarer_ignores_opponents_naming_same_strain() {
        // North deals: 1C by North, 1H by East, Pass, 4H by West. The
        // declaring side is E-W, and East named hearts first for that side.
        // North's earlier clubs bid must not confuse the lookup.
        let auction = auction_from_pbn(Direction::North, "1C 1H Pass 4H Pass Pass Pass");

        let contract = auction.final_contract().unwrap();
        assert_eq!(contract.level, 4);
        assert_eq!(contract.strain, Strain::Hearts);
        assert_eq!(contract.declarer, Direction::East);
    }

    #[test]
    fn test_declarer_with_doubled_contract_after_competitive_auction() {
        // East deals. E 1H, S 1S, W 2H, N 4S, E Double. N-S declare in
        // spades; South named spades first, so South declares doubled.
        let auction = auction_from_pbn(Direction::East, "1H 1S 2H 4S X Pass Pass Pass");

        let contract = auction.final_contract().unwrap();
        assert_eq!(contract.level, 4);
        assert_eq!(contract.strain, Strain::Spades);
        assert!(contract.doubled);
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
        let call = AnnotatedCall::with_annotation(Call::bid(2, Strain::Clubs), "Stayman");
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

    #[test]
    fn section_end_markers_round_trip() {
        assert_eq!(SectionEnd::from_pbn("*"), Some(SectionEnd::Terminated));
        assert_eq!(SectionEnd::from_pbn("+"), Some(SectionEnd::Continued));
        assert_eq!(SectionEnd::from_pbn("Pass"), None);
        assert_eq!(SectionEnd::Terminated.to_pbn(), Some("*"));
        assert_eq!(SectionEnd::Continued.to_pbn(), Some("+"));
        assert_eq!(SectionEnd::Unmarked.to_pbn(), None);
        // A section with no marker is the default, so nothing that does not set
        // one starts claiming the auction was closed.
        assert_eq!(SectionEnd::default(), SectionEnd::Unmarked);
        assert_eq!(Auction::new(Direction::North).end, SectionEnd::Unmarked);
    }
}
