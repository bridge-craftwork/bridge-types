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

mod auction;
mod board;
mod card;
mod deal;
mod direction;
mod hand;
mod metadata;
mod play;
mod scoring;

pub use auction::{AnnotatedCall, Auction, Call, FinalContract};
pub use board::{dealer_from_board_number, Board, Vulnerability};
pub use card::{Card, Rank, Suit};
pub use deal::Deal;
pub use direction::Direction;
pub use hand::Hand;
pub use metadata::{BoardResult, PlayerNames, Room, ScoringMethod};
pub use play::{PlaySequence, Trick};
pub use scoring::{calculate_matchpoints, Contract, Doubled, Strain};
