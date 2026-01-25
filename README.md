# bridge-types

Core data types for contract bridge applications in Rust.

## Overview

`bridge-types` provides fundamental data structures for representing bridge games, including cards, hands, deals, auctions, and scoring. It serves as the foundation for other bridge-related crates like `bridge-encodings` and `bridge-solver`.

## Features

- **Card Types**: `Card`, `Suit`, `Rank` with standard bridge ordering
- **Hand Evaluation**: `Hand` with HCP, controls, losers, shape analysis
- **Deal Management**: `Deal` for all four hands with validation
- **Table Positions**: `Direction` (North, East, South, West)
- **Board Setup**: `Board`, `Vulnerability`, dealer calculation
- **Bidding**: `Call`, `AnnotatedCall`, `Auction`, `FinalContract`
- **Play Tracking**: `Trick`, `PlaySequence` with winner calculation
- **Scoring**: `Contract`, `Strain`, duplicate scoring calculation
- **Metadata**: `PlayerNames`, `ScoringMethod`, `Room`, `BoardResult`

## Installation

```toml
[dependencies]
bridge-types = { git = "https://github.com/Rick-Wilson/bridge-types" }
```

## Quick Start

```rust
use bridge_types::{Card, Suit, Rank, Hand, Direction, Vulnerability};

// Create a card
let ace_of_spades = Card::new(Suit::Spades, Rank::Ace);

// Parse a hand from PBN notation
let hand = Hand::from_pbn("AKQ.JT9.8765.432").unwrap();
println!("HCP: {}", hand.hcp());  // 9

// Check vulnerability
let vul = Vulnerability::from_board_number(5);
assert!(vul.is_vulnerable(Direction::North));
```

## Type Summary

### Card Types

```rust
pub enum Suit { Clubs, Diamonds, Hearts, Spades }
pub enum Rank { Two, Three, ..., King, Ace }
pub struct Card { suit: Suit, rank: Rank }
```

### Bidding Types

```rust
pub enum Call {
    Pass,
    Bid { level: u8, strain: Strain },
    Double,
    Redouble,
    Continue,  // For teaching materials (displayed as "?")
}

pub struct Auction {
    pub dealer: Direction,
    pub calls: Vec<AnnotatedCall>,
    pub notes: HashMap<u8, String>,
}
```

### Scoring Types

```rust
pub enum Strain { Clubs, Diamonds, Hearts, Spades, NoTrump }
pub enum Doubled { None, Doubled, Redoubled }

pub struct Contract {
    pub level: u8,
    pub strain: Strain,
    pub doubled: Doubled,
    pub declarer: char,
}
```

## PBN Support

This crate supports parsing and generating [Portable Bridge Notation (PBN)](https://www.tistis.nl/pbn/) format for hands, deals, calls, and contracts.

A copy of the PBN v2.1 specification is included in [docs/pbn_v21.txt](docs/pbn_v21.txt).

Reference: https://www.tistis.nl/pbn/pbn_v21.txt

## Design Philosophy

- **Zero dependencies**: Pure Rust with no external crates
- **Correct by construction**: Types prevent invalid states where possible
- **PBN compatible**: Follows standard bridge notation conventions
- **Performance**: Efficient representations suitable for solver integration

## Related Crates

- [`bridge-encodings`](https://github.com/Rick-Wilson/bridge-encodings) - PBN, LIN, BWS file format parsers
- [`bridge-solver`](https://github.com/Rick-Wilson/bridge-solver) - Double-dummy analysis

## License

This project is in the public domain (Unlicense).
