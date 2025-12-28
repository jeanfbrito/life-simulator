//! Wolf pack hierarchy relationships using type-safe components.
//!
//! This module implements pack relationships for coordinated wolf behavior.
//! A pack relationship is established when a wolf becomes a leader or joins an existing pack.
//!
//! # Pattern
//! ```text
//! PackLeader (wolf entity)
//!   ├─ member1
//!   ├─ member2
//!   └─ member3
//!
//! PackMember (each member wolf)
//!   └─ leader (reference back to pack leader)
//! ```

use bevy::prelude::*;

/// Type of group that entities can belong to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroupType {
    /// Wolf pack - hunting group of wolves
    Pack,
    /// Deer/cattle herd - safety in numbers, grazing
    Herd,
    /// Bird flock - coordinated flight, foraging
    Flock,
    /// Rabbit warren - shared burrow, defense
    Warren,
    /// Ant/bee colony - shared nest, resource gathering
    Colony,
    /// Fish school - coordinated swimming, predator avoidance
    School,
}

impl GroupType {
    pub fn name(&self) -> &str {
        match self {
            GroupType::Pack => "pack",
            GroupType::Herd => "herd",
            GroupType::Flock => "flock",
            GroupType::Warren => "warren",
            GroupType::Colony => "colony",
            GroupType::School => "school",
        }
    }
}

/// Marker component indicating that this wolf is a pack leader.
/// Applied to the wolf entity when it forms or leads a pack.
#[derive(Component, Debug, Clone)]
pub struct PackLeader {
    /// Pack members following this leader
    pub members: Vec<Entity>,
    /// Simulation tick when pack was formed
    pub formed_tick: u64,
    /// Type of group this leader is managing
    pub group_type: GroupType,
}

impl PackLeader {
    /// Create a new pack leader
    pub fn new(formed_tick: u64, group_type: GroupType) -> Self {
        Self {
            members: Vec::new(),
            formed_tick,
            group_type,
        }
    }

    /// Add a member to the pack
    pub fn add_member(&mut self, member: Entity) {
        if !self.members.contains(&member) {
            self.members.push(member);
        }
    }

    /// Remove a member from the pack
    pub fn remove_member(&mut self, member: Entity) {
        self.members.retain(|&m| m != member);
    }

    /// Get number of pack members
    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    /// Check if entity is a member of this pack
    pub fn has_member(&self, member: Entity) -> bool {
        self.members.contains(&member)
    }
}

/// Marker component indicating that this wolf is a pack member.
/// Applied to the wolf entity when it joins a pack.
#[derive(Component, Debug, Clone, Copy)]
pub struct PackMember {
    /// Which pack leader to follow
    pub leader: Entity,
    /// Simulation tick when joined the pack
    pub joined_tick: u64,
    /// Type of group this member belongs to
    pub group_type: GroupType,
}

impl PackMember {
    /// Create a new pack member
    pub fn new(leader: Entity, joined_tick: u64, group_type: GroupType) -> Self {
        Self { leader, joined_tick, group_type }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RED: Test that PackLeader can be created with tick info
    #[test]
    fn test_pack_leader_creation() {
        let leader = PackLeader::new(100, GroupType::Pack);

        assert_eq!(leader.formed_tick, 100);
        assert_eq!(leader.group_type, GroupType::Pack);
        assert_eq!(leader.member_count(), 0);
        assert!(leader.members.is_empty());
    }

    /// RED: Test that PackMember can be created with leader and tick info
    #[test]
    fn test_pack_member_creation() {
        let leader_entity = Entity::from_raw(1);
        let member = PackMember::new(leader_entity, 100, GroupType::Pack);

        assert_eq!(member.leader, leader_entity);
        assert_eq!(member.joined_tick, 100);
        assert_eq!(member.group_type, GroupType::Pack);
    }

    /// Test: PackLeader can add members
    #[test]
    fn test_pack_leader_add_member() {
        let mut leader = PackLeader::new(100, GroupType::Pack);
        let member1 = Entity::from_raw(1);
        let member2 = Entity::from_raw(2);

        leader.add_member(member1);
        assert_eq!(leader.member_count(), 1);
        assert!(leader.has_member(member1));

        leader.add_member(member2);
        assert_eq!(leader.member_count(), 2);
        assert!(leader.has_member(member2));
    }

    /// Test: PackLeader won't add duplicate members
    #[test]
    fn test_pack_leader_no_duplicate_members() {
        let mut leader = PackLeader::new(100, GroupType::Pack);
        let member = Entity::from_raw(1);

        leader.add_member(member);
        leader.add_member(member); // Try to add again

        assert_eq!(leader.member_count(), 1);
    }

    /// Test: PackLeader can remove members
    #[test]
    fn test_pack_leader_remove_member() {
        let mut leader = PackLeader::new(100, GroupType::Pack);
        let member1 = Entity::from_raw(1);
        let member2 = Entity::from_raw(2);

        leader.add_member(member1);
        leader.add_member(member2);
        assert_eq!(leader.member_count(), 2);

        leader.remove_member(member1);
        assert_eq!(leader.member_count(), 1);
        assert!(!leader.has_member(member1));
        assert!(leader.has_member(member2));
    }

    /// Test: PackLeader returns members list
    #[test]
    fn test_pack_leader_members_list() {
        let mut leader = PackLeader::new(100, GroupType::Pack);
        let member1 = Entity::from_raw(1);
        let member2 = Entity::from_raw(2);
        let member3 = Entity::from_raw(3);

        leader.add_member(member1);
        leader.add_member(member2);
        leader.add_member(member3);

        assert_eq!(leader.members.len(), 3);
        assert!(leader.members.contains(&member1));
        assert!(leader.members.contains(&member2));
        assert!(leader.members.contains(&member3));
    }

    /// Test: PackMember is Copy
    #[test]
    fn test_pack_member_is_copy() {
        let leader = Entity::from_raw(1);
        let member1 = PackMember::new(leader, 100, GroupType::Pack);
        let member2 = member1; // Should copy without issue

        assert_eq!(member1.leader, member2.leader);
        assert_eq!(member1.joined_tick, member2.joined_tick);
        assert_eq!(member1.group_type, member2.group_type);
    }

    /// Test: Pack duration calculation
    #[test]
    fn test_pack_duration_calculation() {
        let leader = PackLeader::new(100, GroupType::Pack);
        let current_tick = 150;
        let pack_duration = current_tick - leader.formed_tick;

        assert_eq!(pack_duration, 50);
    }

    /// Test: Pack member join duration calculation
    #[test]
    fn test_pack_member_join_duration() {
        let leader = Entity::from_raw(1);
        let member = PackMember::new(leader, 120, GroupType::Pack);
        let current_tick = 170;
        let member_duration = current_tick - member.joined_tick;

        assert_eq!(member_duration, 50);
    }

    /// Test: Multiple packs with different leaders
    #[test]
    fn test_multiple_pack_leaders() {
        let leader1 = PackLeader::new(100, GroupType::Pack);
        let leader2 = PackLeader::new(100, GroupType::Pack);
        let member1 = PackMember::new(Entity::from_raw(1), 100, GroupType::Pack);
        let member2 = PackMember::new(Entity::from_raw(2), 100, GroupType::Pack);

        assert_ne!(member1.leader, member2.leader);
        assert_eq!(leader1.formed_tick, leader2.formed_tick);
    }
}
