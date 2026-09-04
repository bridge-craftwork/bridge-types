//! The double-dummy result of a deal: how many tricks each declarer takes in
//! each strain, with best play by both sides.

use crate::{Direction, Strain};
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
    /// A table with every cell zero.
    ///
    /// Zero is a legitimate result — a declarer can take no tricks — so an
    /// all-zero table is not distinguishable from an unsolved one. Callers that
    /// need "not analysed" should hold an `Option<DdTable>`.
    pub fn new() -> Self {
        Self::default()
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
    }
}
