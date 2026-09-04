//! Board and Vulnerability types for bridge.

use crate::{Deal, Direction};
use std::fmt;

/// Represents vulnerability state for a board
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Vulnerability {
    #[default]
    None,
    NorthSouth,
    EastWest,
    Both,
}

impl Vulnerability {
    /// Parse vulnerability from PBN string
    pub fn from_pbn(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "NONE" | "-" | "LOVE" => Some(Vulnerability::None),
            "NS" | "N-S" => Some(Vulnerability::NorthSouth),
            "EW" | "E-W" => Some(Vulnerability::EastWest),
            "BOTH" | "ALL" => Some(Vulnerability::Both),
            _ => None,
        }
    }

    /// Format vulnerability for PBN
    pub fn to_pbn(&self) -> &'static str {
        match self {
            Vulnerability::None => "None",
            Vulnerability::NorthSouth => "NS",
            Vulnerability::EastWest => "EW",
            Vulnerability::Both => "All",
        }
    }

    /// Check if a direction is vulnerable
    pub fn is_vulnerable(&self, direction: Direction) -> bool {
        match self {
            Vulnerability::None => false,
            Vulnerability::Both => true,
            Vulnerability::NorthSouth => {
                matches!(direction, Direction::North | Direction::South)
            }
            Vulnerability::EastWest => {
                matches!(direction, Direction::East | Direction::West)
            }
        }
    }

    /// Calculate vulnerability from board number using standard rotation
    pub fn from_board_number(board: u32) -> Self {
        match (board - 1) % 16 {
            0 | 7 | 10 | 13 => Vulnerability::None,
            1 | 4 | 11 | 14 => Vulnerability::NorthSouth,
            2 | 5 | 8 | 15 => Vulnerability::EastWest,
            3 | 6 | 9 | 12 => Vulnerability::Both,
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for Vulnerability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Vulnerability::None => write!(f, "None Vul"),
            Vulnerability::NorthSouth => write!(f, "N-S Vul"),
            Vulnerability::EastWest => write!(f, "E-W Vul"),
            Vulnerability::Both => write!(f, "Both Vul"),
        }
    }
}

/// Calculate dealer from board number using standard rotation
pub fn dealer_from_board_number(board: u32) -> Direction {
    match (board - 1) % 4 {
        0 => Direction::North,
        1 => Direction::East,
        2 => Direction::South,
        3 => Direction::West,
        _ => unreachable!(),
    }
}

/// A `%` directive or `;` comment preserved verbatim from a PBN record.
///
/// These are not board content, but they are file content. `%` is where Bridge
/// Composer keeps fonts, page setup and colours; `;` is where a hand author
/// leaves notes. Dropping them means a read/write cycle silently strips a
/// user's page layout, so a board carries the ones from its own record.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Directive {
    /// The line exactly as written, including its leading `%` or `;`.
    pub text: String,
    /// Name of the tag this line followed in the source record, or `None` if it
    /// preceded every tag. A writer re-emits the line after that tag, so it
    /// keeps its place even though the writer chooses its own tag order.
    pub after_tag: Option<String>,
}

/// Represents a complete bridge board with metadata
#[derive(Debug, Clone, Default)]
pub struct Board {
    pub number: Option<u32>,
    /// Raw `[Board]` identifier as written, e.g. "1" or "1-1". Preserves
    /// non-integer ids (common in lesson sets) that `number` cannot hold.
    pub board_id: Option<String>,
    pub event: Option<String>,
    pub site: Option<String>,
    pub date: Option<String>,
    pub dealer: Option<Direction>,
    pub vulnerable: Vulnerability,
    pub deal: Deal,
    pub player_names: Option<crate::PlayerNames>,
    pub auction: Option<crate::Auction>,
    pub contract: Option<String>,
    pub declarer: Option<Direction>,
    pub play: Option<crate::PlaySequence>,
    pub result: Option<i8>,
    pub commentary: Vec<String>,
    pub double_dummy_tricks: Option<String>,
    pub optimum_score: Option<String>,
    pub par_contract: Option<String>,
    /// Supplemental PBN tag pairs not modeled by a dedicated field, in the order
    /// encountered. The PBN spec explicitly permits arbitrary supplemental tags
    /// (e.g. `[SkillPath ...]`, bridge-mastery tags); this preserves them rather
    /// than discarding them, so parsers round-trip and consumers can inventory
    /// them. Standard tags that DO have a dedicated field never land here.
    pub extra_tags: Vec<(String, String)>,
    /// `%` directives and `;` comments from this board's record, in the order
    /// encountered, each anchored to the tag it followed. Preserved so a
    /// read/write cycle re-emits them in place rather than discarding them; see
    /// [`Directive`].
    pub directives: Vec<Directive>,
}

impl Board {
    /// Create a new empty board
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: set board number
    pub fn with_number(mut self, number: u32) -> Self {
        self.number = Some(number);
        self
    }

    /// Builder: set the raw board identifier (e.g. "1-1").
    pub fn with_board_id(mut self, id: impl Into<String>) -> Self {
        self.board_id = Some(id.into());
        self
    }

    /// Builder: set dealer
    pub fn with_dealer(mut self, dealer: Direction) -> Self {
        self.dealer = Some(dealer);
        self
    }

    /// Builder: set vulnerability
    pub fn with_vulnerability(mut self, vuln: Vulnerability) -> Self {
        self.vulnerable = vuln;
        self
    }

    /// Builder: set deal
    pub fn with_deal(mut self, deal: Deal) -> Self {
        self.deal = deal;
        self
    }

    /// Builder: set player names
    pub fn with_player_names(mut self, names: crate::PlayerNames) -> Self {
        self.player_names = Some(names);
        self
    }

    /// Builder: set auction
    pub fn with_auction(mut self, auction: crate::Auction) -> Self {
        self.auction = Some(auction);
        self
    }

    /// Builder: set contract string (e.g., "3NT", "4SX")
    pub fn with_contract(mut self, contract: String) -> Self {
        self.contract = Some(contract);
        self
    }

    /// Builder: set declarer
    pub fn with_declarer(mut self, declarer: Direction) -> Self {
        self.declarer = Some(declarer);
        self
    }

    /// Builder: set play sequence
    pub fn with_play(mut self, play: crate::PlaySequence) -> Self {
        self.play = Some(play);
        self
    }

    /// Builder: set result (tricks taken)
    pub fn with_result(mut self, result: i8) -> Self {
        self.result = Some(result);
        self
    }

    /// Builder: add commentary text
    pub fn with_commentary(mut self, text: String) -> Self {
        self.commentary.push(text);
        self
    }

    /// Builder: record a supplemental (non-standard) PBN tag pair.
    pub fn with_extra_tag(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra_tags.push((name.into(), value.into()));
        self
    }

    /// Look up a supplemental tag's value by name (first match).
    pub fn extra_tag(&self, name: &str) -> Option<&str> {
        self.extra_tags
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    /// Builder: preserve a `%` directive or `;` comment.
    ///
    /// `after_tag` names the tag the line followed in the source record, or is
    /// `None` if it came before every tag.
    pub fn with_directive(mut self, text: impl Into<String>, after_tag: Option<&str>) -> Self {
        self.directives.push(Directive {
            text: text.into(),
            after_tag: after_tag.map(str::to_string),
        });
        self
    }

    /// The preserved directives and comments that followed the tag `name`, in
    /// the order encountered.
    pub fn directives_after<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a str> {
        self.directives
            .iter()
            .filter(move |d| d.after_tag.as_deref() == Some(name))
            .map(|d| d.text.as_str())
    }

    /// The preserved directives and comments that came before every tag in the
    /// record, in the order encountered.
    pub fn leading_directives(&self) -> impl Iterator<Item = &str> {
        self.directives
            .iter()
            .filter(|d| d.after_tag.is_none())
            .map(|d| d.text.as_str())
    }

    /// Generate a title string for the board
    pub fn title(&self) -> String {
        let mut parts = Vec::new();

        if let Some(num) = self.number {
            parts.push(format!("Board {}", num));
        }

        if let Some(dealer) = self.dealer {
            parts.push(format!("{} Deals", dealer));
        }

        parts.push(self.vulnerable.to_string());

        parts.join(" - ")
    }

    /// Get HCP for a specific direction
    pub fn hcp(&self, direction: Direction) -> u8 {
        self.deal.hand(direction).hcp()
    }

    /// Get all HCP as [N, E, S, W]
    pub fn all_hcp(&self) -> [u8; 4] {
        [
            self.deal.north.hcp(),
            self.deal.east.hcp(),
            self.deal.south.hcp(),
            self.deal.west.hcp(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vulnerability_parsing() {
        assert_eq!(Vulnerability::from_pbn("None"), Some(Vulnerability::None));
        assert_eq!(Vulnerability::from_pbn("-"), Some(Vulnerability::None));
        assert_eq!(
            Vulnerability::from_pbn("NS"),
            Some(Vulnerability::NorthSouth)
        );
        assert_eq!(
            Vulnerability::from_pbn("E-W"),
            Some(Vulnerability::EastWest)
        );
        assert_eq!(Vulnerability::from_pbn("Both"), Some(Vulnerability::Both));
        assert_eq!(Vulnerability::from_pbn("All"), Some(Vulnerability::Both));
    }

    #[test]
    fn test_vulnerability_check() {
        assert!(!Vulnerability::None.is_vulnerable(Direction::North));
        assert!(Vulnerability::Both.is_vulnerable(Direction::North));
        assert!(Vulnerability::NorthSouth.is_vulnerable(Direction::South));
        assert!(!Vulnerability::NorthSouth.is_vulnerable(Direction::East));
    }

    #[test]
    fn test_vulnerability_from_board() {
        assert_eq!(Vulnerability::from_board_number(1), Vulnerability::None);
        assert_eq!(
            Vulnerability::from_board_number(2),
            Vulnerability::NorthSouth
        );
        assert_eq!(Vulnerability::from_board_number(3), Vulnerability::EastWest);
        assert_eq!(Vulnerability::from_board_number(4), Vulnerability::Both);
        assert_eq!(Vulnerability::from_board_number(17), Vulnerability::None);
    }

    #[test]
    fn test_dealer_from_board() {
        assert_eq!(dealer_from_board_number(1), Direction::North);
        assert_eq!(dealer_from_board_number(2), Direction::East);
        assert_eq!(dealer_from_board_number(3), Direction::South);
        assert_eq!(dealer_from_board_number(4), Direction::West);
        assert_eq!(dealer_from_board_number(5), Direction::North);
    }

    #[test]
    fn test_board_title() {
        let board = Board::new()
            .with_number(1)
            .with_dealer(Direction::North)
            .with_vulnerability(Vulnerability::None);

        assert_eq!(board.title(), "Board 1 - North Deals - None Vul");
    }
}

#[cfg(test)]
mod directive_tests {
    use super::*;

    #[test]
    fn directives_keep_their_place_in_the_record() {
        let board = Board::new()
            .with_directive("% Creator \"Bridge Composer\"", None)
            .with_directive("% 065A62DCF61869AE5D72DF8D408A", Some("Board"))
            .with_directive("; checked by hand", Some("Board"))
            .with_directive("% page setup", Some("Deal"));

        assert_eq!(
            board.leading_directives().collect::<Vec<_>>(),
            vec!["% Creator \"Bridge Composer\""]
        );
        assert_eq!(
            board.directives_after("Board").collect::<Vec<_>>(),
            vec!["% 065A62DCF61869AE5D72DF8D408A", "; checked by hand"]
        );
        assert_eq!(
            board.directives_after("Deal").collect::<Vec<_>>(),
            vec!["% page setup"]
        );
        assert!(board.directives_after("Result").next().is_none());
    }

    #[test]
    fn a_board_carries_no_directives_by_default() {
        assert!(Board::new().directives.is_empty());
    }
}
