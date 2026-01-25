//! Core bridge types for card games.
//!
//! This crate provides fundamental bridge data types including:
//! - `Card`, `Suit`, `Rank` - basic card representation
//! - `Hand` - a player's 13 cards with evaluation methods
//! - `Deal` - all four hands at the table
//! - `Direction` - table positions (North, East, South, West)
//! - `Board` - complete board setup with vulnerability
//! - `Contract`, `Strain` - contract and scoring types
//! - `Auction`, `Call` - bidding sequence types
//! - `Trick`, `PlaySequence` - card play tracking
//! - `PlayerNames`, `ScoringMethod`, `Room` - metadata types

mod card;
mod direction;
mod hand;
mod deal;
mod board;
mod scoring;
mod auction;
mod play;
mod metadata;

pub use card::{Card, Suit, Rank};
pub use direction::Direction;
pub use hand::Hand;
pub use deal::Deal;
pub use board::{Board, Vulnerability, dealer_from_board_number};
pub use scoring::{Contract, Strain, Doubled, calculate_matchpoints};
pub use auction::{Call, AnnotatedCall, Auction, FinalContract};
pub use play::{Trick, PlaySequence};
pub use metadata::{PlayerNames, ScoringMethod, Room, BoardResult};
