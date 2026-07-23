//! Deal type representing all four hands at a bridge table.

use std::collections::HashSet;

use crate::{Card, Direction, Hand};

/// Represents a complete bridge deal (all four hands)
#[derive(Debug, Clone, Default)]
pub struct Deal {
    pub north: Hand,
    pub east: Hand,
    pub south: Hand,
    pub west: Hand,
}

impl Deal {
    /// Create a new empty deal
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a hand by direction
    pub fn hand(&self, direction: Direction) -> &Hand {
        match direction {
            Direction::North => &self.north,
            Direction::East => &self.east,
            Direction::South => &self.south,
            Direction::West => &self.west,
        }
    }

    /// Get a mutable hand by direction
    pub fn hand_mut(&mut self, direction: Direction) -> &mut Hand {
        match direction {
            Direction::North => &mut self.north,
            Direction::East => &mut self.east,
            Direction::South => &mut self.south,
            Direction::West => &mut self.west,
        }
    }

    /// Set a hand for a direction
    pub fn set_hand(&mut self, direction: Direction, hand: Hand) {
        match direction {
            Direction::North => self.north = hand,
            Direction::East => self.east = hand,
            Direction::South => self.south = hand,
            Direction::West => self.west = hand,
        }
    }

    /// Format deal in PBN notation: "N:spades.hearts.diamonds.clubs spades.hearts... ..."
    pub fn to_pbn(&self, first: Direction) -> String {
        let mut parts = Vec::with_capacity(4);
        let mut dir = first;
        for _ in 0..4 {
            parts.push(self.hand(dir).to_pbn());
            dir = dir.next();
        }
        format!("{}:{}", first.to_char(), parts.join(" "))
    }

    /// Parse deal from PBN notation
    pub fn from_pbn(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.len() < 3 {
            return None;
        }

        let first_dir = Direction::from_char(s.chars().next()?)?;
        if s.chars().nth(1)? != ':' {
            return None;
        }

        let hands_str = &s[2..];
        let hand_strs: Vec<&str> = hands_str.split_whitespace().collect();
        if hand_strs.len() != 4 {
            return None;
        }

        let mut deal = Deal::new();
        let mut dir = first_dir;
        for hand_str in hand_strs {
            let hand = Hand::from_pbn(hand_str)?;
            deal.set_hand(dir, hand);
            dir = dir.next();
        }

        Some(deal)
    }

    /// Get total HCP for a partnership
    pub fn partnership_hcp(&self, direction: Direction) -> u8 {
        self.hand(direction).hcp() + self.hand(direction.partner()).hcp()
    }

    /// Returns the total number of cards in the deal
    pub fn total_cards(&self) -> usize {
        self.north.len() + self.east.len() + self.south.len() + self.west.len()
    }

    /// Returns true if the deal has any cards (not empty/placeholder)
    pub fn has_cards(&self) -> bool {
        self.total_cards() > 0
    }

    /// Returns true if the deal is complete (52 cards total)
    pub fn is_complete(&self) -> bool {
        self.total_cards() == 52
    }

    /// Returns true if the deal has no duplicate cards
    pub fn is_valid(&self) -> bool {
        let mut seen = HashSet::new();
        for dir in Direction::ALL {
            for card in self.hand(dir).cards() {
                if !seen.insert(*card) {
                    return false; // Duplicate card found
                }
            }
        }
        true
    }

    /// Returns the number of tricks in this deal (total cards / 4)
    ///
    /// A complete deal has 13 tricks. Partial deals may have fewer.
    pub fn trick_count(&self) -> usize {
        self.total_cards() / 4
    }

    /// Collects all cards in the deal into a vector
    pub fn all_cards(&self) -> Vec<Card> {
        let mut cards = Vec::with_capacity(52);
        for dir in Direction::ALL {
            cards.extend(self.hand(dir).cards().iter().copied());
        }
        cards
    }

    /// Find duplicate cards in the deal, if any
    pub fn find_duplicates(&self) -> Vec<Card> {
        let mut seen = HashSet::new();
        let mut duplicates = Vec::new();
        for dir in Direction::ALL {
            for card in self.hand(dir).cards() {
                if !seen.insert(*card) {
                    duplicates.push(*card);
                }
            }
        }
        duplicates
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deal_from_pbn() {
        let pbn = "N:K843.T542.J6.863 AQJ7.K.Q75.AT942 962.AJ7.KT82.J75 T5.Q9863.A943.KQ";
        let deal = Deal::from_pbn(pbn).unwrap();

        assert_eq!(deal.north.hcp(), 4);
        assert_eq!(deal.east.hcp(), 16);

        let pbn_out = deal.to_pbn(Direction::North);
        assert_eq!(pbn, pbn_out);
    }

    #[test]
    fn test_deal_hand_access() {
        let pbn = "N:K843.T542.J6.863 AQJ7.K.Q75.AT942 962.AJ7.KT82.J75 T5.Q9863.A943.KQ";
        let deal = Deal::from_pbn(pbn).unwrap();

        assert_eq!(deal.hand(Direction::North).hcp(), 4);
        assert_eq!(deal.hand(Direction::East).hcp(), 16);
        assert_eq!(deal.hand(Direction::South).hcp(), 9);
        assert_eq!(deal.hand(Direction::West).hcp(), 11);
    }

    #[test]
    fn test_partnership_hcp() {
        let pbn = "N:K843.T542.J6.863 AQJ7.K.Q75.AT942 962.AJ7.KT82.J75 T5.Q9863.A943.KQ";
        let deal = Deal::from_pbn(pbn).unwrap();

        assert_eq!(deal.partnership_hcp(Direction::North), 13); // N(4) + S(9)
        assert_eq!(deal.partnership_hcp(Direction::East), 27); // E(16) + W(11)
    }

    #[test]
    fn test_deal_completeness() {
        let pbn = "N:K843.T542.J6.863 AQJ7.K.Q75.AT942 962.AJ7.KT82.J75 T5.Q9863.A943.KQ";
        let deal = Deal::from_pbn(pbn).unwrap();

        assert_eq!(deal.total_cards(), 52);
        assert!(deal.has_cards());
        assert!(deal.is_complete());
    }

    #[test]
    fn test_empty_deal() {
        let deal = Deal::new();

        assert_eq!(deal.total_cards(), 0);
        assert!(!deal.has_cards());
        assert!(!deal.is_complete());
    }

    #[test]
    fn test_deal_validity() {
        // Valid deal - no duplicates
        let pbn = "N:K843.T542.J6.863 AQJ7.K.Q75.AT942 962.AJ7.KT82.J75 T5.Q9863.A943.KQ";
        let deal = Deal::from_pbn(pbn).unwrap();
        assert!(deal.is_valid());
        assert!(deal.find_duplicates().is_empty());
    }

    #[test]
    fn test_deal_with_duplicates() {
        use crate::{Card, Rank, Suit};

        // Create a deal with duplicate cards
        let mut deal = Deal::new();
        let mut north = Hand::new();
        let mut east = Hand::new();

        // Add same card to both hands
        north.add_card(Card::new(Suit::Spades, Rank::Ace));
        east.add_card(Card::new(Suit::Spades, Rank::Ace)); // Duplicate!

        deal.north = north;
        deal.east = east;

        assert!(!deal.is_valid());
        let dups = deal.find_duplicates();
        assert_eq!(dups.len(), 1);
        assert_eq!(dups[0], Card::new(Suit::Spades, Rank::Ace));
    }

    #[test]
    fn test_trick_count() {
        // Complete deal = 13 tricks
        let pbn = "N:K843.T542.J6.863 AQJ7.K.Q75.AT942 962.AJ7.KT82.J75 T5.Q9863.A943.KQ";
        let deal = Deal::from_pbn(pbn).unwrap();
        assert_eq!(deal.trick_count(), 13);

        // Empty deal = 0 tricks
        let empty = Deal::new();
        assert_eq!(empty.trick_count(), 0);
    }

    #[test]
    fn test_all_cards() {
        let pbn = "N:K843.T542.J6.863 AQJ7.K.Q75.AT942 962.AJ7.KT82.J75 T5.Q9863.A943.KQ";
        let deal = Deal::from_pbn(pbn).unwrap();
        let cards = deal.all_cards();
        assert_eq!(cards.len(), 52);
    }
}
