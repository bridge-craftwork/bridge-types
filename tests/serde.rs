//! JSON round trips with the `serde` feature.
#![cfg(feature = "serde")]

use bridge_types::{Auction, Call, Direction, Hand, Strain};

#[test]
fn auction_round_trips_through_json() {
    let mut auction = Auction::new(Direction::South);
    auction.add_call(Call::bid(1, Strain::NoTrump));
    auction.add_annotated_call(Call::Pass, None);
    auction.add_annotated_call(Call::bid(2, Strain::Diamonds), Some("=1=".into()));
    auction.add_note(1, "Transfer");

    let json = serde_json::to_string(&auction).unwrap();
    let back: Auction = serde_json::from_str(&json).unwrap();
    assert_eq!(back.dealer, Direction::South);
    assert_eq!(back.calls.len(), 3);
    assert_eq!(back.calls[2].call, Call::bid(2, Strain::Diamonds));
    assert_eq!(back.calls[2].annotation.as_deref(), Some("=1="));
    assert_eq!(back.get_note(1), Some("Transfer"));
}

#[test]
fn hand_round_trips_through_json() {
    let hand = Hand::from_pbn("AKQ5.KQ7.A95.K87").unwrap();
    let json = serde_json::to_string(&hand).unwrap();
    let back: Hand = serde_json::from_str(&json).unwrap();
    assert_eq!(back, hand);
}
