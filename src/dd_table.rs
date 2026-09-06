//! The double-dummy result of a deal: how many tricks each declarer takes in
//! each strain, with best play by both sides.

use crate::{Deal, Direction, Strain};
use std::fmt;

/// Every strain, in the order [`DdTable`] stores its columns.
pub const STRAINS: [Strain; 5] = [
    Strain::Clubs,
    Strain::Diamonds,
    Strain::Hearts,
    Strain::Spades,
    Strain::NoTrump,
];

/// Every declarer, in the order [`DdTable`] stores its rows — the same order as
/// [`Direction::to_index`].
pub const DECLARERS: [Direction; 4] = [
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
];

/// The twenty double-dummy results of one deal: for each declarer, in each
/// strain, the tricks that declarer takes against best defence.
///
/// # Read and write it by seat and strain, never by index
///
/// The storage order is deliberately private, because getting it wrong is the
/// characteristic bug with this data and it has already happened more than once
/// across this family of crates. Three different row orders were in circulation
/// when this type was written — `N, E, S, W` in one solver module, `N, S, E, W`
/// in the CLI that serialises it, and `W, N, E, S` in the PBN specification's
/// own column description — and two different column orders, one starting at
/// clubs and one at notrump.
///
/// None of that is visible here. [`Self::tricks`] and [`Self::set`] take a
/// [`Direction`] and a [`Strain`], so a caller cannot silently transpose a
/// table, and every serialisation states its own order at the point it emits.
///
/// ```
/// use bridge_types::{DdTable, Direction, Strain};
///
/// let mut table = DdTable::new();
/// table.set(Direction::North, Strain::NoTrump, 9);
/// assert_eq!(table.tricks(Direction::North, Strain::NoTrump), 9);
/// assert_eq!(table.tricks(Direction::South, Strain::Hearts), 0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DdTable {
    /// Indexed `[declarer][strain]`, following [`DECLARERS`] and [`STRAINS`].
    tricks: [[u8; 5]; 4],
}

impl DdTable {
    /// The null table: every cell zero, which callers read as "not analysed".
    ///
    /// # Why an all-zero table is safe to give a meaning of its own
    ///
    /// Zero is a legitimate *cell* value — a declarer can take no tricks — but
    /// an all-zero *table* is not something a solver can produce for a complete
    /// deal.
    ///
    /// Over thirteen tricks all fifty-two cards are played, so the ace of trumps
    /// is played to some trick. A trick containing a trump is won by the highest
    /// trump played to it, and no trump outranks the ace, so the side holding
    /// the ace of trumps wins at least one trick — whoever declares, whoever
    /// leads, whatever the defence. That fixes eight of the twenty cells as
    /// non-zero: in each of the four suit strains, both cells of the partnership
    /// holding that suit's ace.
    ///
    /// # Suit strains only, and the restriction is load-bearing
    ///
    /// In notrump an ace can be discarded without ever winning a trick, so there
    /// is no notrump form of the argument. The whole notrump column can be zero
    /// on a legal deal:
    ///
    /// ```text
    /// N ♠AKQJT98765432   S ♥AKQJT98765432   E ♦AKQJT98765432   W ♣AKQJT98765432
    /// ```
    ///
    /// Whoever is on lead runs twenty-six cards while the other side can only
    /// discard, so the defending side takes all thirteen tricks in each of the
    /// four notrump cells. That deal is also the tight witness for the bound:
    /// exactly eight non-zero cells and twelve zeros.
    ///
    /// None of this is asserted in code, and no wider invariant is checked here.
    /// Promotion makes many other holdings winners too — `K Q` in one hand, `Q J
    /// T` against `A K` — with no natural stopping point, and adjudicating
    /// double-dummy numbers is `bridge-solver`'s work, not this crate's. The
    /// argument is recorded so the next reader need not re-derive it.
    pub const NULL: Self = Self {
        tricks: [[0; 5]; 4],
    };

    /// A table with every cell zero, the same value as [`Self::NULL`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether every cell is zero — whether this is [`Self::NULL`].
    ///
    /// Structural only: it reports what the numbers are, not what they mean.
    /// For "was this deal solved", which needs the deal in hand, use
    /// [`is_solved`].
    pub fn is_null(&self) -> bool {
        *self == Self::NULL
    }

    /// Build a table by solving each cell.
    ///
    /// The closure is called once per (declarer, strain) pair. Order is
    /// unspecified and callers must not depend on it.
    pub fn from_fn(mut f: impl FnMut(Direction, Strain) -> u8) -> Self {
        let mut table = Self::new();
        for declarer in DECLARERS {
            for strain in STRAINS {
                table.set(declarer, strain, f(declarer, strain));
            }
        }
        table
    }

    /// Tricks `declarer` takes in `strain`.
    pub fn tricks(&self, declarer: Direction, strain: Strain) -> u8 {
        self.tricks[declarer.to_index()][strain_index(strain)]
    }

    /// Record the tricks `declarer` takes in `strain`.
    ///
    /// Values above 13 are meaningless here but are stored as given; validation
    /// belongs to whatever produced the number.
    pub fn set(&mut self, declarer: Direction, strain: Strain, tricks: u8) {
        self.tricks[declarer.to_index()][strain_index(strain)] = tricks;
    }

    /// Every cell, as `(declarer, strain, tricks)`, declarer-major in
    /// [`DECLARERS`] then [`STRAINS`] order.
    ///
    /// For serialising to a format with a different order, iterate that
    /// format's own order and call [`Self::tricks`] instead.
    pub fn cells(&self) -> impl Iterator<Item = (Direction, Strain, u8)> + '_ {
        DECLARERS.into_iter().flat_map(move |declarer| {
            STRAINS
                .into_iter()
                .map(move |strain| (declarer, strain, self.tricks(declarer, strain)))
        })
    }

    /// The best result the declaring side can reach in `strain`, taking the
    /// better of the two partners.
    ///
    /// Which partner declares does not change the cards, but it changes the
    /// opening lead, so the two cells can differ.
    pub fn best_for_side(&self, declarer: Direction, strain: Strain) -> u8 {
        self.tricks(declarer, strain)
            .max(self.tricks(declarer.partner(), strain))
    }
}

/// Whether `table` holds a solve of `deal`: `Some(true)` solved, `Some(false)`
/// unsolved, `None` when the pair cannot answer the question.
///
/// A non-null table is a solve. A null table is the "not analysed" sentinel,
/// but only for a complete deal — see [`DdTable::NULL`] for why an all-zero
/// table is unreachable there. For an incomplete deal that argument does not
/// apply, so a null table says nothing and the answer is `None`.
///
/// `Some(true)` means "not the unsolved sentinel", not "correctly solved". It
/// says a solver wrote something here. It does not say the numbers are right,
/// that they belong to this deal, or that the axes were not transposed.
///
/// ```
/// use bridge_types::{is_solved, DdTable, Deal, Direction, Strain};
///
/// let deal = Deal::from_pbn(
///     "N:K843.T542.J6.863 AQJ7.K.Q75.AT942 962.AJ7.KT82.J75 T5.Q9863.A943.KQ",
/// )
/// .expect("a complete deal");
///
/// assert_eq!(is_solved(&deal, &DdTable::NULL), Some(false));
///
/// let mut table = DdTable::NULL;
/// table.set(Direction::East, Strain::Clubs, 9);
/// assert_eq!(is_solved(&deal, &table), Some(true));
///
/// // Nothing to go on: no deal, and a null table.
/// assert_eq!(is_solved(&Deal::new(), &DdTable::NULL), None);
/// ```
pub fn is_solved(deal: &Deal, table: &DdTable) -> Option<bool> {
    if !table.is_null() {
        Some(true)
    } else if deal.is_complete() {
        Some(false)
    } else {
        None
    }
}

/// A strain's column index, following [`STRAINS`].
fn strain_index(strain: Strain) -> usize {
    match strain {
        Strain::Clubs => 0,
        Strain::Diamonds => 1,
        Strain::Hearts => 2,
        Strain::Spades => 3,
        Strain::NoTrump => 4,
    }
}

/// Rows in `DECLARERS` order, columns in `STRAINS` order, for debugging. This
/// is not a PBN encoding — those live in `bridge-encodings`.
impl fmt::Display for DdTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "    C  D  H  S NT")?;
        for declarer in DECLARERS {
            write!(f, "{}  ", declarer.to_char())?;
            for strain in STRAINS {
                write!(f, "{:2} ", self.tricks(declarer, strain))?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cells_are_addressed_by_seat_and_strain_not_position() {
        let mut table = DdTable::new();
        table.set(Direction::West, Strain::Clubs, 11);
        table.set(Direction::North, Strain::NoTrump, 3);

        assert_eq!(table.tricks(Direction::West, Strain::Clubs), 11);
        assert_eq!(table.tricks(Direction::North, Strain::NoTrump), 3);
        // The two must not have collided, which is what a transposed index does.
        assert_eq!(table.tricks(Direction::West, Strain::NoTrump), 0);
        assert_eq!(table.tricks(Direction::North, Strain::Clubs), 0);
    }

    #[test]
    fn from_fn_visits_every_cell_exactly_once() {
        let mut seen = Vec::new();
        let table = DdTable::from_fn(|declarer, strain| {
            seen.push((declarer, strain));
            (declarer.to_index() * 5 + strain_index(strain)) as u8
        });

        assert_eq!(seen.len(), 20);
        let mut unique = seen.clone();
        unique.sort_by_key(|(d, s)| (d.to_index(), strain_index(*s)));
        unique.dedup();
        assert_eq!(unique.len(), 20);

        for declarer in DECLARERS {
            for strain in STRAINS {
                let expected = (declarer.to_index() * 5 + strain_index(strain)) as u8;
                assert_eq!(table.tricks(declarer, strain), expected);
            }
        }
    }

    #[test]
    fn cells_yields_all_twenty_in_declarer_major_order() {
        let table = DdTable::from_fn(|d, s| (d.to_index() * 5 + strain_index(s)) as u8);
        let cells: Vec<_> = table.cells().collect();

        assert_eq!(cells.len(), 20);
        assert_eq!(cells[0], (Direction::North, Strain::Clubs, 0));
        assert_eq!(cells[4], (Direction::North, Strain::NoTrump, 4));
        assert_eq!(cells[5], (Direction::East, Strain::Clubs, 5));
        assert_eq!(cells[19], (Direction::West, Strain::NoTrump, 19));
    }

    /// The two partners can differ, because the opening lead differs.
    #[test]
    fn best_for_side_takes_the_better_partner() {
        let mut table = DdTable::new();
        table.set(Direction::North, Strain::Hearts, 8);
        table.set(Direction::South, Strain::Hearts, 10);

        assert_eq!(table.best_for_side(Direction::North, Strain::Hearts), 10);
        assert_eq!(table.best_for_side(Direction::South, Strain::Hearts), 10);
        assert_eq!(table.best_for_side(Direction::East, Strain::Hearts), 0);
    }

    #[test]
    fn a_new_table_is_all_zero() {
        let table = DdTable::new();
        assert!(table.cells().all(|(_, _, tricks)| tricks == 0));
        assert_eq!(table, DdTable::NULL);
        assert!(table.is_null());
    }

    /// Any one cell is enough to leave the sentinel behind, wherever it sits.
    #[test]
    fn one_non_zero_cell_anywhere_makes_a_table_non_null() {
        for declarer in DECLARERS {
            for strain in STRAINS {
                let mut table = DdTable::NULL;
                table.set(declarer, strain, 1);
                assert!(!table.is_null(), "{declarer:?} {strain:?} left it null");
            }
        }
    }

    const COMPLETE_DEAL: &str =
        "N:K843.T542.J6.863 AQJ7.K.Q75.AT942 962.AJ7.KT82.J75 T5.Q9863.A943.KQ";

    fn complete_deal() -> Deal {
        Deal::from_pbn(COMPLETE_DEAL).expect("a complete deal")
    }

    #[test]
    fn a_null_table_on_a_complete_deal_is_unsolved() {
        assert_eq!(is_solved(&complete_deal(), &DdTable::NULL), Some(false));
    }

    #[test]
    fn any_non_null_table_is_solved_whatever_the_deal() {
        let mut table = DdTable::NULL;
        table.set(Direction::South, Strain::Spades, 10);

        assert_eq!(is_solved(&complete_deal(), &table), Some(true));
        // The deal is never consulted once the table is non-null.
        assert_eq!(is_solved(&Deal::new(), &table), Some(true));
    }

    /// Without all fifty-two cards the trump-ace argument does not apply, so a
    /// null table carries no meaning to read back.
    #[test]
    fn a_null_table_on_an_incomplete_deal_is_indeterminate() {
        let mut deal = complete_deal();
        deal.set_hand(Direction::West, crate::Hand::new());

        assert!(!deal.is_complete());
        assert_eq!(is_solved(&deal, &DdTable::NULL), None);
        assert_eq!(is_solved(&Deal::new(), &DdTable::NULL), None);
    }
}
