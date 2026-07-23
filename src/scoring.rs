//! Contract and scoring types for duplicate bridge.

/// A parsed contract
#[derive(Debug, Clone, PartialEq)]
pub struct Contract {
    pub level: u8,
    pub strain: Strain,
    pub doubled: Doubled,
    pub declarer: char,
}

/// The strain (denomination) of a contract
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strain {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
    NoTrump,
}

impl Strain {
    /// Whether this is a minor suit (clubs or diamonds)
    pub fn is_minor(&self) -> bool {
        matches!(self, Strain::Clubs | Strain::Diamonds)
    }

    /// Whether this is a major suit (hearts or spades)
    pub fn is_major(&self) -> bool {
        matches!(self, Strain::Hearts | Strain::Spades)
    }

    /// Whether this is a red suit (hearts or diamonds)
    pub fn is_red(&self) -> bool {
        matches!(self, Strain::Hearts | Strain::Diamonds)
    }

    /// Get the Unicode symbol for this strain
    pub fn symbol(&self) -> &'static str {
        match self {
            Strain::Clubs => "♣",
            Strain::Diamonds => "♦",
            Strain::Hearts => "♥",
            Strain::Spades => "♠",
            Strain::NoTrump => "NT",
        }
    }

    /// Points per trick for this strain
    pub fn trick_value(&self) -> i32 {
        match self {
            Strain::Clubs | Strain::Diamonds => 20,
            Strain::Hearts | Strain::Spades => 30,
            Strain::NoTrump => 30,
        }
    }

    /// Parse strain from string
    // Shadows `std::str::FromStr::from_str` by name. Kept as-is deliberately:
    // this is public API of a crate consumed by several sibling repos, so
    // renaming it (or moving to a real `FromStr` impl, whose `Err` type would
    // change callers) is a breaking change that doesn't belong in a CI PR.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Strain> {
        match s.to_uppercase().as_str() {
            "C" | "CLUBS" => Some(Strain::Clubs),
            "D" | "DIAMONDS" => Some(Strain::Diamonds),
            "H" | "HEARTS" => Some(Strain::Hearts),
            "S" | "SPADES" => Some(Strain::Spades),
            "NT" | "N" | "NOTRUMP" | "NO TRUMP" => Some(Strain::NoTrump),
            _ => None,
        }
    }

    /// Convert to character representation
    pub fn to_char(&self) -> char {
        match self {
            Strain::Clubs => 'C',
            Strain::Diamonds => 'D',
            Strain::Hearts => 'H',
            Strain::Spades => 'S',
            Strain::NoTrump => 'N',
        }
    }
}

impl std::fmt::Display for Strain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Strain::Clubs => write!(f, "♣"),
            Strain::Diamonds => write!(f, "♦"),
            Strain::Hearts => write!(f, "♥"),
            Strain::Spades => write!(f, "♠"),
            Strain::NoTrump => write!(f, "NT"),
        }
    }
}

/// Doubling state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Doubled {
    #[default]
    None,
    Doubled,
    Redoubled,
}

impl Contract {
    /// Create a new contract
    pub fn new(level: u8, strain: Strain, doubled: Doubled, declarer: char) -> Self {
        Contract {
            level,
            strain,
            doubled,
            declarer,
        }
    }

    /// Parse a contract string like "3 NT", "4 S X", "6 H XX"
    pub fn parse(s: &str) -> Option<Contract> {
        let s = s.trim().to_uppercase();
        if s.is_empty() || s == "PASS" || s == "PASSED" || s == "AP" || s == "ALL PASS" {
            return None;
        }

        let mut parts: Vec<&str> = s.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let level: u8 = parts[0].chars().next()?.to_digit(10)? as u8;
        if !(1..=7).contains(&level) {
            return None;
        }

        let first_part = parts[0];
        let strain_start = if first_part.len() > 1 && first_part.chars().next()?.is_ascii_digit() {
            &first_part[1..]
        } else if parts.len() > 1 {
            parts.remove(0);
            parts[0]
        } else {
            return None;
        };

        let strain = Strain::from_str(strain_start)?;

        let doubled = if parts.iter().any(|p| *p == "XX" || *p == "REDOUBLED") {
            Doubled::Redoubled
        } else if parts.iter().any(|p| *p == "X" || *p == "DOUBLED") {
            Doubled::Doubled
        } else {
            Doubled::None
        };

        Some(Contract {
            level,
            strain,
            doubled,
            declarer: 'N',
        })
    }

    /// Parse a result string like "+3", "-1", "="
    pub fn parse_result(s: &str) -> Option<i32> {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }

        if s == "=" || s == "0" || s == "+0" {
            return Some(0);
        }

        // A leading '+' must be stripped (i32 won't parse it); '-' is parsed
        // as-is, which is also what a bare number needs — so those two arms
        // were identical and collapse into the `else`.
        if let Some(rest) = s.strip_prefix('+') {
            rest.parse::<i32>().ok()
        } else {
            s.parse::<i32>().ok()
        }
    }

    /// Calculate the score for this contract
    /// `tricks_relative` is the number of tricks relative to the contract
    /// `vulnerable` indicates if the declaring side is vulnerable
    pub fn score(&self, tricks_relative: i32, vulnerable: bool) -> i32 {
        if tricks_relative < 0 {
            self.undertrick_penalty(-tricks_relative, vulnerable)
        } else {
            let tricks_made = self.level as i32 + 6 + tricks_relative;
            self.making_score(tricks_made, vulnerable)
        }
    }

    fn making_score(&self, tricks_made: i32, vulnerable: bool) -> i32 {
        let contracted_tricks = self.level as i32;
        let overtricks = tricks_made - (contracted_tricks + 6);

        let mut contract_value = match self.strain {
            Strain::NoTrump => 40 + (contracted_tricks - 1) * 30,
            _ => contracted_tricks * self.strain.trick_value(),
        };

        contract_value = match self.doubled {
            Doubled::None => contract_value,
            Doubled::Doubled => contract_value * 2,
            Doubled::Redoubled => contract_value * 4,
        };

        let game_bonus = if contract_value >= 100 {
            if vulnerable {
                500
            } else {
                300
            }
        } else {
            50
        };

        let slam_bonus = match self.level {
            6 => {
                if vulnerable {
                    750
                } else {
                    500
                }
            }
            7 => {
                if vulnerable {
                    1500
                } else {
                    1000
                }
            }
            _ => 0,
        };

        let overtrick_value = match self.doubled {
            Doubled::None => overtricks * self.strain.trick_value(),
            Doubled::Doubled => overtricks * if vulnerable { 200 } else { 100 },
            Doubled::Redoubled => overtricks * if vulnerable { 400 } else { 200 },
        };

        let insult = match self.doubled {
            Doubled::None => 0,
            Doubled::Doubled => 50,
            Doubled::Redoubled => 100,
        };

        contract_value + game_bonus + slam_bonus + overtrick_value + insult
    }

    fn undertrick_penalty(&self, undertricks: i32, vulnerable: bool) -> i32 {
        let penalty = match self.doubled {
            Doubled::None => {
                if vulnerable {
                    undertricks * 100
                } else {
                    undertricks * 50
                }
            }
            Doubled::Doubled => {
                if vulnerable {
                    match undertricks {
                        1 => 200,
                        n => 200 + (n - 1) * 300,
                    }
                } else {
                    match undertricks {
                        1 => 100,
                        2 => 300,
                        3 => 500,
                        n => 500 + (n - 3) * 300,
                    }
                }
            }
            Doubled::Redoubled => {
                let doubled_penalty = if vulnerable {
                    match undertricks {
                        1 => 200,
                        n => 200 + (n - 1) * 300,
                    }
                } else {
                    match undertricks {
                        1 => 100,
                        2 => 300,
                        3 => 500,
                        n => 500 + (n - 3) * 300,
                    }
                };
                doubled_penalty * 2
            }
        };

        -penalty
    }
}

impl std::fmt::Display for Contract {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.level, self.strain)?;
        match self.doubled {
            Doubled::None => {}
            Doubled::Doubled => write!(f, "X")?,
            Doubled::Redoubled => write!(f, "XX")?,
        }
        write!(f, " by {}", self.declarer)
    }
}

/// Calculate matchpoints for a set of scores on a board
/// Returns matchpoints as percentages for each score
pub fn calculate_matchpoints(scores_ns: &[i32]) -> Vec<f64> {
    let n = scores_ns.len();
    if n == 0 {
        return vec![];
    }

    let mut matchpoints = vec![0.0; n];
    let max_mp = (n - 1) as f64 * 2.0;

    for i in 0..n {
        for j in 0..n {
            if i != j {
                if scores_ns[i] > scores_ns[j] {
                    matchpoints[i] += 2.0;
                } else if scores_ns[i] == scores_ns[j] {
                    matchpoints[i] += 1.0;
                }
            }
        }
    }

    if max_mp > 0.0 {
        for mp in &mut matchpoints {
            *mp = (*mp / max_mp) * 100.0;
        }
    }

    matchpoints
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_contract() {
        let c = Contract::parse("3 NT").unwrap();
        assert_eq!(c.level, 3);
        assert_eq!(c.strain, Strain::NoTrump);
        assert_eq!(c.doubled, Doubled::None);

        let c = Contract::parse("4 S").unwrap();
        assert_eq!(c.level, 4);
        assert_eq!(c.strain, Strain::Spades);

        let c = Contract::parse("2 H X").unwrap();
        assert_eq!(c.level, 2);
        assert_eq!(c.strain, Strain::Hearts);
        assert_eq!(c.doubled, Doubled::Doubled);

        let c = Contract::parse("6 C XX").unwrap();
        assert_eq!(c.level, 6);
        assert_eq!(c.strain, Strain::Clubs);
        assert_eq!(c.doubled, Doubled::Redoubled);
    }

    #[test]
    fn test_parse_result() {
        assert_eq!(Contract::parse_result("+3"), Some(3));
        assert_eq!(Contract::parse_result("-1"), Some(-1));
        assert_eq!(Contract::parse_result("="), Some(0));
        assert_eq!(Contract::parse_result("+0"), Some(0));
    }

    #[test]
    fn test_score_1nt_making_3() {
        let c = Contract::parse("1 NT").unwrap();
        assert_eq!(c.score(3, false), 180);
    }

    #[test]
    fn test_score_3nt_making() {
        let c = Contract::parse("3 NT").unwrap();
        assert_eq!(c.score(0, false), 400);
        assert_eq!(c.score(0, true), 600);
    }

    #[test]
    fn test_score_4s_making() {
        let c = Contract::parse("4 S").unwrap();
        assert_eq!(c.score(0, false), 420);
    }

    #[test]
    fn test_score_down() {
        let c = Contract::parse("3 NT").unwrap();
        assert_eq!(c.score(-1, false), -50);
        assert_eq!(c.score(-1, true), -100);

        let c = Contract::parse("3 NT X").unwrap();
        assert_eq!(c.score(-2, false), -300);
    }

    #[test]
    fn test_matchpoints() {
        let scores = vec![420, 420, 400, -50];
        let mps = calculate_matchpoints(&scores);

        assert!((mps[0] - 83.33).abs() < 0.1);
        assert!((mps[1] - 83.33).abs() < 0.1);
        assert!((mps[2] - 33.33).abs() < 0.1);
        assert!((mps[3] - 0.0).abs() < 0.1);
    }
}
