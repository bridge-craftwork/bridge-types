//! Play sequence types for tracking card play in bridge.

use crate::{Card, Direction, Suit};

/// A single trick (4 cards played, one from each direction)
#[derive(Debug, Clone)]
pub struct Trick {
    /// The player who led to this trick
    pub leader: Direction,
    /// Cards played in order (leader first, then clockwise)
    pub cards: [Option<Card>; 4],
    /// The suit that was led
    pub lead_suit: Option<Suit>,
    /// The winner of this trick (if complete)
    pub winner: Option<Direction>,
}

impl Trick {
    /// Create a new trick with the given leader
    pub fn new(leader: Direction) -> Self {
        Self {
            leader,
            cards: [None, None, None, None],
            lead_suit: None,
            winner: None,
        }
    }

    /// Play a card to this trick (next position)
    pub fn play(&mut self, card: Card) -> bool {
        for i in 0..4 {
            if self.cards[i].is_none() {
                self.cards[i] = Some(card);
                if i == 0 {
                    self.lead_suit = Some(card.suit);
                }
                return true;
            }
        }
        false // Trick is already complete
    }

    /// Get the card played by a specific direction
    pub fn card_by(&self, direction: Direction) -> Option<Card> {
        let leader_idx = self.leader.to_index();
        let target_idx = direction.to_index();
        let position = (target_idx + 4 - leader_idx) % 4;
        self.cards[position]
    }

    /// Check if the trick is complete (all 4 cards played)
    pub fn is_complete(&self) -> bool {
        self.cards.iter().all(|c| c.is_some())
    }

    /// Get the number of cards played so far
    pub fn cards_played(&self) -> usize {
        self.cards.iter().filter(|c| c.is_some()).count()
    }

    /// Get the direction that played at position N (0 = leader)
    pub fn player_at(&self, position: usize) -> Direction {
        let mut dir = self.leader;
        for _ in 0..position {
            dir = dir.next();
        }
        dir
    }

    /// Determine the winner of the trick (if complete)
    /// `trump` is the trump suit, if any
    pub fn determine_winner(&mut self, trump: Option<Suit>) {
        if !self.is_complete() {
            return;
        }

        let lead_suit = self.lead_suit.unwrap();
        let mut winning_pos = 0;
        let mut winning_card = self.cards[0].unwrap();

        for i in 1..4 {
            let card = self.cards[i].unwrap();
            let dominated = if let Some(trump_suit) = trump {
                // Trump beats non-trump
                if card.suit == trump_suit && winning_card.suit != trump_suit {
                    true
                } else if card.suit != trump_suit && winning_card.suit == trump_suit {
                    false
                } else if card.suit == winning_card.suit {
                    card.rank > winning_card.rank
                } else {
                    false
                }
            } else {
                // No trump: must follow suit to win
                card.suit == lead_suit
                    && (winning_card.suit != lead_suit || card.rank > winning_card.rank)
            };

            if dominated {
                winning_pos = i;
                winning_card = card;
            }
        }

        self.winner = Some(self.player_at(winning_pos));
    }
}

/// A complete play sequence (up to 13 tricks)
#[derive(Debug, Clone)]
pub struct PlaySequence {
    /// The opening leader
    pub opening_leader: Direction,
    /// The trump suit, if any (None for NT)
    pub trump: Option<Suit>,
    /// The tricks played
    pub tricks: Vec<Trick>,
}

impl PlaySequence {
    /// Create a new play sequence
    pub fn new(opening_leader: Direction, trump: Option<Suit>) -> Self {
        Self {
            opening_leader,
            trump,
            tricks: Vec::new(),
        }
    }

    /// Create a new play sequence for a NT contract
    pub fn new_nt(opening_leader: Direction) -> Self {
        Self::new(opening_leader, None)
    }

    /// Start a new trick with the given leader
    pub fn start_trick(&mut self, leader: Direction) {
        self.tricks.push(Trick::new(leader));
    }

    /// Play a card to the current trick
    pub fn play_card(&mut self, card: Card) -> bool {
        if self.tricks.is_empty() {
            self.tricks.push(Trick::new(self.opening_leader));
        }

        let trick = self.tricks.last_mut().unwrap();
        let played = trick.play(card);

        // If trick is complete, determine winner and start next trick
        if played && trick.is_complete() {
            trick.determine_winner(self.trump);
        }

        played
    }

    /// Get the current trick (or None if all tricks complete)
    pub fn current_trick(&self) -> Option<&Trick> {
        self.tricks.last().filter(|t| !t.is_complete())
    }

    /// Get the number of complete tricks
    pub fn tricks_complete(&self) -> usize {
        self.tricks.iter().filter(|t| t.is_complete()).count()
    }

    /// Get the number of tricks won by a direction
    pub fn tricks_won_by(&self, direction: Direction) -> usize {
        self.tricks
            .iter()
            .filter(|t| t.winner == Some(direction))
            .count()
    }

    /// Get the number of tricks won by a partnership
    pub fn tricks_won_by_partnership(&self, direction: Direction) -> usize {
        self.tricks_won_by(direction) + self.tricks_won_by(direction.partner())
    }

    /// Check if play is complete (13 tricks played)
    pub fn is_complete(&self) -> bool {
        self.tricks.len() == 13 && self.tricks.iter().all(|t| t.is_complete())
    }

    /// Get the next player to play
    pub fn next_to_play(&self) -> Option<Direction> {
        if self.is_complete() {
            return None;
        }

        if let Some(trick) = self.tricks.last() {
            if trick.is_complete() {
                // Winner of last trick leads
                trick.winner
            } else {
                // Next player in current trick
                Some(trick.player_at(trick.cards_played()))
            }
        } else {
            Some(self.opening_leader)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Rank;

    #[test]
    fn test_trick_play() {
        let mut trick = Trick::new(Direction::West);
        assert!(!trick.is_complete());

        trick.play(Card::new(Suit::Spades, Rank::Ace));
        trick.play(Card::new(Suit::Spades, Rank::King));
        trick.play(Card::new(Suit::Spades, Rank::Queen));
        trick.play(Card::new(Suit::Spades, Rank::Jack));

        assert!(trick.is_complete());
        assert_eq!(trick.lead_suit, Some(Suit::Spades));
    }

    #[test]
    fn test_trick_winner_nt() {
        let mut trick = Trick::new(Direction::North);
        trick.play(Card::new(Suit::Hearts, Rank::King));
        trick.play(Card::new(Suit::Hearts, Rank::Ace));
        trick.play(Card::new(Suit::Hearts, Rank::Two));
        trick.play(Card::new(Suit::Clubs, Rank::Ace)); // Discard

        trick.determine_winner(None);
        assert_eq!(trick.winner, Some(Direction::East)); // East played Ace of hearts
    }

    #[test]
    fn test_trick_winner_with_trump() {
        let mut trick = Trick::new(Direction::North);
        trick.play(Card::new(Suit::Hearts, Rank::Ace));
        trick.play(Card::new(Suit::Spades, Rank::Two)); // Trump!
        trick.play(Card::new(Suit::Hearts, Rank::King));
        trick.play(Card::new(Suit::Hearts, Rank::Queen));

        trick.determine_winner(Some(Suit::Spades));
        assert_eq!(trick.winner, Some(Direction::East)); // East trumped
    }

    #[test]
    fn test_play_sequence() {
        let mut play = PlaySequence::new_nt(Direction::West);

        // First trick
        play.play_card(Card::new(Suit::Spades, Rank::Ace));
        play.play_card(Card::new(Suit::Spades, Rank::Two));
        play.play_card(Card::new(Suit::Spades, Rank::Three));
        play.play_card(Card::new(Suit::Spades, Rank::Four));

        assert_eq!(play.tricks_complete(), 1);
        assert_eq!(play.tricks[0].winner, Some(Direction::West));
    }

    #[test]
    fn test_card_by_direction() {
        let mut trick = Trick::new(Direction::North);
        trick.play(Card::new(Suit::Hearts, Rank::Ace));
        trick.play(Card::new(Suit::Hearts, Rank::King));
        trick.play(Card::new(Suit::Hearts, Rank::Queen));
        trick.play(Card::new(Suit::Hearts, Rank::Jack));

        assert_eq!(
            trick.card_by(Direction::North),
            Some(Card::new(Suit::Hearts, Rank::Ace))
        );
        assert_eq!(
            trick.card_by(Direction::East),
            Some(Card::new(Suit::Hearts, Rank::King))
        );
        assert_eq!(
            trick.card_by(Direction::South),
            Some(Card::new(Suit::Hearts, Rank::Queen))
        );
        assert_eq!(
            trick.card_by(Direction::West),
            Some(Card::new(Suit::Hearts, Rank::Jack))
        );
    }
}
