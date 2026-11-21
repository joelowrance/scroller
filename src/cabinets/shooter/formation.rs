//! Formations: enemies that enter together. Destroying every member drops
//! the Power-up the Stage file named, or pays the Formation bonus if it
//! named none. If any member leaves alive, the Formation can't be cleared.
//! Shared by both ship Cabinets; each checks the letters it knows.

use bevy::prelude::*;

/// What clearing a Formation earns.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Reward {
    /// Drop this Power-up where the last member died.
    PowerUp(char, Vec2),
    /// Points: the bonus per enemy times the Formation's size.
    Bonus { members: u32 },
}

/// Tracks one Formation's members. Lives on its own entity.
#[derive(Component, Clone, Debug, PartialEq)]
pub struct Formation {
    /// Members not yet destroyed or lost.
    pending: Vec<Entity>,
    size: u32,
    broken: bool,
    last_at: Vec2,
    pub drop: Option<char>,
}

impl Formation {
    pub fn new(members: Vec<Entity>, drop: Option<char>) -> Self {
        Self {
            size: members.len() as u32,
            pending: members,
            broken: false,
            last_at: Vec2::ZERO,
            drop,
        }
    }

    pub fn has(&self, member: Entity) -> bool {
        self.pending.contains(&member)
    }

    pub fn pending(&self) -> &[Entity] {
        &self.pending
    }

    /// A member was shot down at `at`.
    pub fn killed(&mut self, member: Entity, at: Vec2) {
        if let Some(i) = self.pending.iter().position(|&m| m == member) {
            self.pending.swap_remove(i);
            self.last_at = at;
        }
    }

    /// A member went away without being shot down (off screen, dissolved).
    pub fn lost(&mut self, member: Entity) {
        if let Some(i) = self.pending.iter().position(|&m| m == member) {
            self.pending.swap_remove(i);
            self.broken = true;
        }
    }

    /// True once every member is accounted for.
    pub fn is_done(&self) -> bool {
        self.pending.is_empty()
    }

    /// The reward, once done, if every member was shot down.
    pub fn reward(&self) -> Option<Reward> {
        if !self.is_done() || self.broken || self.size == 0 {
            return None;
        }
        Some(match self.drop {
            Some(letter) => Reward::PowerUp(letter, self.last_at),
            None => Reward::Bonus { members: self.size },
        })
    }
}

/// Marks an enemy that belongs to a Formation.
#[derive(Component, Clone, Copy, Debug)]
pub struct Member;

#[cfg(test)]
mod tests {
    use super::*;

    fn e(n: u32) -> Entity {
        Entity::from_raw_u32(n).unwrap()
    }

    #[test]
    fn clearing_every_member_drops_the_power_up_at_the_last_kill() {
        let mut f = Formation::new(vec![e(1), e(2), e(3)], Some('G'));
        f.killed(e(2), Vec2::new(1.0, 1.0));
        f.killed(e(1), Vec2::new(2.0, 2.0));
        assert_eq!(f.reward(), None);
        f.killed(e(3), Vec2::new(3.0, 3.0));
        assert_eq!(f.reward(), Some(Reward::PowerUp('G', Vec2::new(3.0, 3.0))));
    }

    #[test]
    fn no_power_up_means_a_bonus_for_the_whole_formation() {
        let mut f = Formation::new(vec![e(1), e(2)], None);
        f.killed(e(1), Vec2::ZERO);
        f.killed(e(2), Vec2::ZERO);
        assert_eq!(f.reward(), Some(Reward::Bonus { members: 2 }));
    }

    #[test]
    fn a_member_that_escapes_spoils_the_formation() {
        let mut f = Formation::new(vec![e(1), e(2)], Some('D'));
        f.killed(e(1), Vec2::ZERO);
        f.lost(e(2));
        assert!(f.is_done());
        assert_eq!(f.reward(), None);
    }

    #[test]
    fn strangers_and_repeats_are_ignored() {
        let mut f = Formation::new(vec![e(1)], None);
        f.killed(e(9), Vec2::ZERO);
        f.lost(e(9));
        assert!(!f.is_done());
        f.killed(e(1), Vec2::ZERO);
        f.lost(e(1));
        assert_eq!(f.reward(), Some(Reward::Bonus { members: 1 }));
    }
}
