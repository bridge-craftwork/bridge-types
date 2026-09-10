//! Hand type with evaluation methods for bridge.

use crate::{Card, Rank, Suit};

/// Represents a single player's hand of cards
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hand {
    cards: Vec<Card>,
}

impl Hand {
    /// Create a new empty hand
    pub fn new() -> Self {
        Hand { cards: Vec::new() }
    }

    /// Create a hand from a vector of cards
    pub fn from_cards(cards: Vec<Card>) -> Self {
        Hand { cards }
    }

    /// Add a card to the hand
    pub fn add_card(&mut self, card: Card) {
        self.cards.push(card);
    }

    /// Get all cards in the hand
    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    /// Get the number of cards in the hand
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// Check if the hand is empty
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Count cards of a specific suit
    pub fn suit_length(&self, suit: Suit) -> usize {
        self.cards.iter().filter(|c| c.suit == suit).count()
    }

    /// Get all cards of a specific suit
    pub fn cards_in_suit(&self, suit: Suit) -> Vec<Card> {
        self.cards
            .iter()
            .filter(|c| c.suit == suit)
            .copied()
            .collect()
    }

    /// Calculate total High Card Points (HCP)
    /// A=4, K=3, Q=2, J=1
    pub fn hcp(&self) -> u8 {
        self.cards.iter().map(|c| c.hcp()).sum()
    }

    /// Get the suit lengths in standard order [S, H, D, C]
    pub fn suit_lengths(&self) -> [usize; 4] {
        [
            self.suit_length(Suit::Spades),
            self.suit_length(Suit::Hearts),
            self.suit_length(Suit::Diamonds),
            self.suit_length(Suit::Clubs),
        ]
    }

    /// Get the distribution pattern as a sorted array [longest to shortest]
    pub fn distribution(&self) -> [usize; 4] {
        let mut lengths = self.suit_lengths();
        lengths.sort_by(|a, b| b.cmp(a));
        lengths
    }

    /// Get the shape as a sorted string (e.g., "5-4-3-1")
    pub fn shape(&self) -> String {
        let dist = self.distribution();
        format!("{}-{}-{}-{}", dist[0], dist[1], dist[2], dist[3])
    }

    /// Check if hand is balanced (4-3-3-3, 4-4-3-2, or 5-3-3-2)
    pub fn is_balanced(&self) -> bool {
        let dist = self.distribution();
        matches!(dist, [4, 3, 3, 3] | [4, 4, 3, 2] | [5, 3, 3, 2])
    }

    /// Count controls (A=2, K=1)
    pub fn controls(&self) -> u8 {
        self.cards
            .iter()
            .map(|c| match c.rank {
                Rank::Ace => 2,
                Rank::King => 1,
                _ => 0,
            })
            .sum()
    }

    /// Count honors (A, K, Q, J, T) in a specific suit
    pub fn honors_in_suit(&self, suit: Suit) -> u8 {
        self.cards
            .iter()
            .filter(|c| c.suit == suit && c.rank >= Rank::Ten)
            .count() as u8
    }

    /// Sort the hand by suit (spades first) and rank (high to low)
    pub fn sort(&mut self) {
        self.cards.sort_by(|a, b| match b.suit.cmp(&a.suit) {
            std::cmp::Ordering::Equal => b.rank.cmp(&a.rank),
            other => other,
        });
    }

    /// Get a sorted copy of the hand
    pub fn sorted(&self) -> Hand {
        let mut hand = self.clone();
        hand.sort();
        hand
    }

    /// Check if hand matches an exact shape pattern (S-H-D-C order)
    pub fn matches_exact_shape(&self, pattern: &[u8; 4]) -> bool {
        let lengths = self.suit_lengths();
        lengths[0] == pattern[0] as usize
            && lengths[1] == pattern[1] as usize
            && lengths[2] == pattern[2] as usize
            && lengths[3] == pattern[3] as usize
    }

    /// Check if hand matches a wildcard shape pattern
    /// None means "any length" for that suit
    pub fn matches_wildcard_shape(&self, pattern: &[Option<u8>; 4]) -> bool {
        let lengths = self.suit_lengths();
        for i in 0..4 {
            if let Some(required) = pattern[i] {
                if lengths[i] != required as usize {
                    return false;
                }
            }
        }
        true
    }

    /// Check if hand matches a distribution pattern (suit-order independent)
    pub fn matches_distribution(&self, pattern: &[u8; 4]) -> bool {
        let mut dist = self.distribution();
        let mut pat = *pattern;
        dist.sort_unstable();
        pat.sort_unstable();
        let dist_u8: [u8; 4] = [dist[0] as u8, dist[1] as u8, dist[2] as u8, dist[3] as u8];
        dist_u8 == pat
    }

    /// Calculate losers for entire hand
    pub fn losers(&self) -> u8 {
        self.losers_in_suit(Suit::Spades)
            + self.losers_in_suit(Suit::Hearts)
            + self.losers_in_suit(Suit::Diamonds)
            + self.losers_in_suit(Suit::Clubs)
    }

    /// Calculate losers in a specific suit
    pub fn losers_in_suit(&self, suit: Suit) -> u8 {
        let mut cards: Vec<Card> = self
            .cards
            .iter()
            .filter(|c| c.suit == suit)
            .copied()
            .collect();

        let len = cards.len();
        if len == 0 {
            return 0;
        }

        cards.sort_by_key(|c| std::cmp::Reverse(c.rank));

        if len == 1 {
            if cards[0].rank == Rank::Ace {
                0
            } else {
                1
            }
        } else if len == 2 {
            let has_ace = cards.iter().any(|c| c.rank == Rank::Ace);
            let has_king = cards.iter().any(|c| c.rank == Rank::King);

            if has_ace && has_king {
                0
            } else if has_ace || has_king {
                1
            } else {
                2
            }
        } else {
            let mut losers = 3;
            for card in cards.iter().take(3.min(len)) {
                if matches!(card.rank, Rank::Ace | Rank::King | Rank::Queen) {
                    losers -= 1;
                }
            }
            losers
        }
    }

    /// Check if hand contains a specific card
    pub fn has_card(&self, card: Card) -> bool {
        self.cards.contains(&card)
    }

    /// Count number of tens in hand
    pub fn tens(&self) -> u8 {
        self.cards.iter().filter(|c| c.rank == Rank::Ten).count() as u8
    }

    /// Count number of jacks in hand
    pub fn jacks(&self) -> u8 {
        self.cards.iter().filter(|c| c.rank == Rank::Jack).count() as u8
    }

    /// Count number of queens in hand
    pub fn queens(&self) -> u8 {
        self.cards.iter().filter(|c| c.rank == Rank::Queen).count() as u8
    }

    /// Count number of kings in hand
    pub fn kings(&self) -> u8 {
        self.cards.iter().filter(|c| c.rank == Rank::King).count() as u8
    }

    /// Count number of aces in hand
    pub fn aces(&self) -> u8 {
        self.cards.iter().filter(|c| c.rank == Rank::Ace).count() as u8
    }

    /// Count top 2 honors (A, K) in hand
    pub fn top2(&self) -> u8 {
        self.cards
            .iter()
            .filter(|c| matches!(c.rank, Rank::Ace | Rank::King))
            .count() as u8
    }

    /// Count top 3 honors (A, K, Q) in hand
    pub fn top3(&self) -> u8 {
        self.cards
            .iter()
            .filter(|c| matches!(c.rank, Rank::Ace | Rank::King | Rank::Queen))
            .count() as u8
    }

    /// Count top 4 honors (A, K, Q, J) in hand
    pub fn top4(&self) -> u8 {
        self.cards
            .iter()
            .filter(|c| matches!(c.rank, Rank::Ace | Rank::King | Rank::Queen | Rank::Jack))
            .count() as u8
    }

    /// Count top 5 honors (A, K, Q, J, T) in hand
    pub fn top5(&self) -> u8 {
        self.cards
            .iter()
            .filter(|c| {
                matches!(
                    c.rank,
                    Rank::Ace | Rank::King | Rank::Queen | Rank::Jack | Rank::Ten
                )
            })
            .count() as u8
    }

    /// Calculate C13 points (A=6, K=4, Q=2, J=1)
    pub fn c13(&self) -> u8 {
        self.cards
            .iter()
            .map(|c| match c.rank {
                Rank::Ace => 6,
                Rank::King => 4,
                Rank::Queen => 2,
                Rank::Jack => 1,
                _ => 0,
            })
            .sum()
    }

    /// Calculate suit quality metric (Bridge World Oct 1982)
    /// Returns quality value multiplied by 100 for integer math
    pub fn suit_quality(&self, suit: Suit) -> i32 {
        let mut cards: Vec<Card> = self
            .cards
            .iter()
            .filter(|c| c.suit == suit)
            .copied()
            .collect();

        let length = cards.len() as i32;
        if length == 0 {
            return 0;
        }

        cards.sort_by_key(|c| std::cmp::Reverse(c.rank));

        let has_ace = cards.iter().any(|c| c.rank == Rank::Ace);
        let has_king = cards.iter().any(|c| c.rank == Rank::King);
        let has_queen = cards.iter().any(|c| c.rank == Rank::Queen);
        let has_jack = cards.iter().any(|c| c.rank == Rank::Jack);
        let has_ten = cards.iter().any(|c| c.rank == Rank::Ten);
        let has_nine = cards.iter().any(|c| c.rank == Rank::Nine);
        let has_eight = cards.iter().any(|c| c.rank == Rank::Eight);

        let mut quality = 0;
        let mut higher_honors = 0;
        let suit_factor = length * 10;

        if has_ace {
            quality += 4 * suit_factor;
            higher_honors += 1;
        }
        if has_king {
            quality += 3 * suit_factor;
            higher_honors += 1;
        }
        if has_queen {
            quality += 2 * suit_factor;
            higher_honors += 1;
        }
        if has_jack {
            quality += suit_factor;
            higher_honors += 1;
        }

        if length > 6 {
            let mut replace_count = 3;
            if has_queen {
                replace_count -= 2;
            }
            if has_jack {
                replace_count -= 1;
            }
            if replace_count > (length - 6) {
                replace_count = length - 6;
            }
            quality += replace_count * suit_factor;
        } else {
            if has_ten {
                if (higher_honors > 1) || has_jack {
                    quality += suit_factor;
                } else {
                    quality += suit_factor / 2;
                }
            }
            if has_nine && ((higher_honors == 2) || has_ten || has_eight) {
                quality += suit_factor / 2;
            }
        }

        quality
    }

    /// Calculate CCCC hand evaluation (Bridge World Oct 1982)
    /// Returns evaluation multiplied by 100 for integer math
    pub fn cccc(&self) -> i32 {
        let mut eval = 0;
        let mut shape_points = 0;

        for suit in [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs] {
            let mut cards: Vec<Card> = self
                .cards
                .iter()
                .filter(|c| c.suit == suit)
                .copied()
                .collect();

            let length = cards.len();

            if length < 3 {
                shape_points += (3 - length) as i32 * 100;
            }

            if length == 0 {
                continue;
            }

            cards.sort_by_key(|c| std::cmp::Reverse(c.rank));

            let has_ace = cards.iter().any(|c| c.rank == Rank::Ace);
            let has_king = cards.iter().any(|c| c.rank == Rank::King);
            let has_queen = cards.iter().any(|c| c.rank == Rank::Queen);
            let has_jack = cards.iter().any(|c| c.rank == Rank::Jack);
            let has_ten = cards.iter().any(|c| c.rank == Rank::Ten);
            let has_nine = cards.iter().any(|c| c.rank == Rank::Nine);

            let mut higher_honors = 0;

            if has_ace {
                eval += 300;
                higher_honors += 1;
            }

            if has_king {
                eval += 200;
                if length == 1 {
                    eval -= 150;
                }
                higher_honors += 1;
            }

            if has_queen {
                eval += 100;
                if length == 1 {
                    eval -= 75;
                }
                if length == 2 {
                    eval -= 25;
                }
                if higher_honors == 0 {
                    eval -= 25;
                }
                higher_honors += 1;
            }

            if has_jack {
                if higher_honors == 2 {
                    eval += 50;
                }
                if higher_honors == 1 {
                    eval += 25;
                }
                higher_honors += 1;
            }

            if has_ten {
                if higher_honors == 2 {
                    eval += 25;
                }
                if (higher_honors == 1) && has_nine {
                    eval += 25;
                }
            }

            eval += self.suit_quality(suit);
        }

        if shape_points == 0 {
            eval -= 50;
        } else {
            eval += shape_points - 100;
        }

        eval
    }

    /// Parse hand from PBN notation (e.g., "AKQ.JT9.876.5432")
    /// Suits are in order: Spades.Hearts.Diamonds.Clubs
    ///
    /// "Not all 4 hands need to be given. A hand whose cards are not given, is
    /// indicated by `-`" (PBN 2.1 §3.4.11), so a lone dash reads as an empty
    /// hand. Rejecting it would fail the whole `[Deal]` rather than the one
    /// hand — the standard's own example,
    /// `W:KQT2.AT.J6542.85 - A8654.KQ5.T.QJT6 -`, would parse as no cards at
    /// all, and two-hand teaching records are written exactly this way.
    pub fn from_pbn(s: &str) -> Option<Self> {
        let s = s.trim();
        if s == "-" {
            return Some(Hand::new());
        }

        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 4 {
            return None;
        }

        let mut hand = Hand::new();

        for (suit_idx, part) in parts.iter().enumerate() {
            let suit = match suit_idx {
                0 => Suit::Spades,
                1 => Suit::Hearts,
                2 => Suit::Diamonds,
                3 => Suit::Clubs,
                _ => return None,
            };

            if *part == "-" {
                continue;
            }

            for c in part.chars() {
                let rank = Rank::from_char(c)?;
                hand.add_card(Card::new(suit, rank));
            }
        }

        Some(hand)
    }

    /// Format hand in PBN notation
    pub fn to_pbn(&self) -> String {
        let mut result = Vec::with_capacity(4);

        for suit in [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs] {
            let mut cards: Vec<Card> = self.cards_in_suit(suit);
            cards.sort_by_key(|c| std::cmp::Reverse(c.rank));
            let holding: String = cards.iter().map(|c| c.rank.to_char()).collect();
            result.push(holding);
        }

        result.join(".")
    }
}

impl Default for Hand {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hcp_calculation() {
        let mut hand = Hand::new();
        hand.add_card(Card::new(Suit::Spades, Rank::Ace));
        hand.add_card(Card::new(Suit::Hearts, Rank::King));
        hand.add_card(Card::new(Suit::Diamonds, Rank::Queen));
        hand.add_card(Card::new(Suit::Clubs, Rank::Jack));
        hand.add_card(Card::new(Suit::Spades, Rank::Seven));

        assert_eq!(hand.hcp(), 10);
    }

    #[test]
    fn test_suit_length() {
        let mut hand = Hand::new();
        hand.add_card(Card::new(Suit::Spades, Rank::Ace));
        hand.add_card(Card::new(Suit::Spades, Rank::King));
        hand.add_card(Card::new(Suit::Spades, Rank::Queen));
        hand.add_card(Card::new(Suit::Hearts, Rank::Ace));
        hand.add_card(Card::new(Suit::Hearts, Rank::King));

        assert_eq!(hand.suit_length(Suit::Spades), 3);
        assert_eq!(hand.suit_length(Suit::Hearts), 2);
        assert_eq!(hand.suit_length(Suit::Diamonds), 0);
        assert_eq!(hand.suit_length(Suit::Clubs), 0);
    }

    #[test]
    fn test_balanced_hand() {
        let hand = Hand::from_pbn("AK32.QJT9.876.54").unwrap();
        assert!(hand.is_balanced());
        assert_eq!(hand.distribution(), [4, 4, 3, 2]);
    }

    #[test]
    fn test_controls() {
        let mut hand = Hand::new();
        hand.add_card(Card::new(Suit::Spades, Rank::Ace));
        hand.add_card(Card::new(Suit::Hearts, Rank::King));
        hand.add_card(Card::new(Suit::Diamonds, Rank::Ace));

        assert_eq!(hand.controls(), 5);
    }

    #[test]
    fn test_pbn_round_trip() {
        let original = "K843.T542.J6.863";
        let hand = Hand::from_pbn(original).unwrap();
        assert_eq!(hand.to_pbn(), original);
        assert_eq!(hand.hcp(), 4);
    }

    #[test]
    fn test_pbn_with_void() {
        let hand = Hand::from_pbn("AKQJT98765432...").unwrap();
        assert_eq!(hand.suit_length(Suit::Spades), 13);
        assert_eq!(hand.suit_length(Suit::Hearts), 0);
        assert_eq!(hand.suit_length(Suit::Diamonds), 0);
        assert_eq!(hand.suit_length(Suit::Clubs), 0);
    }

    #[test]
    fn test_losers() {
        // Standard LTC: A/K/Q each reduce losers by 1 in top 3 positions
        // AKQ = 0 losers, KQx = 1 loser, Qxx = 2 losers, xx = 2 losers
        let hand = Hand::from_pbn("AKQ.KQ4.Q32.87").unwrap();
        assert_eq!(hand.losers_in_suit(Suit::Spades), 0);
        assert_eq!(hand.losers_in_suit(Suit::Hearts), 1);
        assert_eq!(hand.losers_in_suit(Suit::Diamonds), 2);
        assert_eq!(hand.losers_in_suit(Suit::Clubs), 2);
        assert_eq!(hand.losers(), 5);
    }

    #[test]
    fn an_unknown_hand_is_a_dash() {
        // Both halves matter: the hand is empty, and its neighbours in the deal
        // still parse. Two-hand bidding records are written this way.
        assert_eq!(Hand::from_pbn("-").unwrap().len(), 0);

        let deal = crate::Deal::from_pbn("W:- K43.AQJ54.63.T95 - A5.KT83.K752.843").unwrap();
        assert_eq!(deal.west.len(), 0);
        assert_eq!(deal.east.len(), 0);
        assert_eq!(deal.north.len(), 13);
        assert_eq!(deal.south.len(), 13);
    }
}
