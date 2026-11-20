//! The local top-ten High Score Table.

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

pub const TABLE_SIZE: usize = 10;
pub const INITIALS_LEN: usize = 3;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub initials: String,
    pub score: u64,
}

/// Always sorted from highest to lowest score, at most [`TABLE_SIZE`] long.
#[derive(Resource, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighScoreTable {
    pub entries: Vec<Entry>,
}

impl Default for HighScoreTable {
    /// A seeded table, as arcade cabinets shipped with.
    fn default() -> Self {
        let seeds = [
            "ACE", "BOB", "CAT", "DAN", "EVE", "FOX", "GUS", "HAL", "IVY", "JAY",
        ];
        Self {
            entries: seeds
                .iter()
                .enumerate()
                .map(|(i, initials)| Entry {
                    initials: (*initials).to_string(),
                    score: 50_000 - 5_000 * i as u64,
                })
                .collect(),
        }
    }
}

impl HighScoreTable {
    /// Whether `score` would earn a place on the table.
    pub fn qualifies(&self, score: u64) -> bool {
        score > 0
            && (self.entries.len() < TABLE_SIZE
                || self.entries.last().is_some_and(|last| score > last.score))
    }

    /// Inserts a qualifying score and returns its 0-based rank.
    /// Ties rank below existing entries with the same score.
    pub fn insert(&mut self, initials: &str, score: u64) -> Option<usize> {
        if !self.qualifies(score) {
            return None;
        }
        let rank = self.entries.partition_point(|e| e.score >= score);
        self.entries.insert(
            rank,
            Entry {
                initials: initials.to_string(),
                score,
            },
        );
        self.entries.truncate(TABLE_SIZE);
        Some(rank)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_table_is_full_and_sorted() {
        let t = HighScoreTable::default();
        assert_eq!(t.entries.len(), TABLE_SIZE);
        assert!(t.entries.windows(2).all(|w| w[0].score >= w[1].score));
    }

    #[test]
    fn low_score_does_not_qualify_on_full_table() {
        let mut t = HighScoreTable::default();
        let lowest = t.entries.last().unwrap().score;
        assert!(!t.qualifies(lowest));
        assert_eq!(t.insert("ZZZ", lowest), None);
    }

    #[test]
    fn insert_places_score_in_order_and_drops_the_tenth() {
        let mut t = HighScoreTable::default();
        let rank = t.insert("NEW", 42_000).unwrap();
        assert_eq!(rank, 2);
        assert_eq!(t.entries[2].initials, "NEW");
        assert_eq!(t.entries.len(), TABLE_SIZE);
    }

    #[test]
    fn zero_never_qualifies() {
        let t = HighScoreTable { entries: vec![] };
        assert!(!t.qualifies(0));
    }
}
