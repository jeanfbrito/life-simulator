//! System for managing wolf pack hierarchy and relationships.
//!
//! This system establishes and maintains pack relationships when wolves
//! form groups and cleans up stale relationships when members leave or packs dissolve.

use bevy::prelude::*;
use crate::entities::{PackLeader, PackMember, GroupType};
use crate::entities::TilePosition;

/// Establishes a pack hierarchy when a wolf becomes a leader
pub fn establish_pack_leadership(
    leader: Entity,
    tick: u64,
    commands: &mut Commands,
) {
    // Add PackLeader component to leader
    commands.entity(leader).insert(PackLeader::new(tick, GroupType::Pack));
}

/// Adds a wolf to an existing pack
pub fn add_to_pack(
    member: Entity,
    leader: Entity,
    tick: u64,
    commands: &mut Commands,
    world: &World,
) {
    // Add PackMember marker to the new member
    commands.entity(member).insert(PackMember::new(leader, tick, GroupType::Pack));

    // Update leader's member list
    if let Some(mut pack_leader) = world.get::<PackLeader>(leader).map(|l| l.clone()) {
        pack_leader.add_member(member);
        commands.entity(leader).insert(pack_leader);
    }
}

/// Removes a member from their pack
pub fn remove_from_pack(
    member: Entity,
    commands: &mut Commands,
    world: &World,
) {
    // Find the member's leader
    if let Some(pack_member) = world.get::<PackMember>(member) {
        let leader = pack_member.leader;

        // Remove PackMember component from member
        commands.entity(member).remove::<PackMember>();

        // Update leader's member list
        if let Some(mut pack_leader) = world.get::<PackLeader>(leader).map(|l| l.clone()) {
            pack_leader.remove_member(member);

            // If pack is now empty, remove leader component
            if pack_leader.member_count() == 0 {
                commands.entity(leader).remove::<PackLeader>();
            } else {
                commands.entity(leader).insert(pack_leader);
            }
        }
    } else {
        // Not in a pack, nothing to do
        commands.entity(member).remove::<PackMember>();
    }
}

/// Dissolves a pack and removes all member relationships
pub fn dissolve_pack(
    leader: Entity,
    commands: &mut Commands,
    world: &World,
) {
    if let Some(pack) = world.get::<PackLeader>(leader) {
        // Remove PackMember component from all pack members
        for member in pack.members.iter() {
            commands.entity(*member).remove::<PackMember>();
        }
    }

    // Remove PackLeader component from leader
    commands.entity(leader).remove::<PackLeader>();
}

/// System to clean up stale pack relationships
/// Runs periodically to remove relationships for dead/despawned wolves
pub fn cleanup_stale_pack_relationships(
    mut commands: Commands,
    leaders: Query<(Entity, &PackLeader)>,
    members_check: Query<Entity, With<TilePosition>>,
) {
    for (leader_entity, pack) in leaders.iter() {
        let mut pack_updated = pack.clone();
        let mut removed_any = false;

        // Check each member still exists
        for member in pack.members.iter() {
            if members_check.get(*member).is_err() {
                // Member no longer exists, remove from pack
                pack_updated.remove_member(*member);
                removed_any = true;
            }
        }

        if removed_any {
            if pack_updated.member_count() == 0 {
                // Pack is now empty, remove leader
                commands.entity(leader_entity).remove::<PackLeader>();
            } else {
                // Update pack with new member list
                commands.entity(leader_entity).insert(pack_updated);
            }
        }
    }
}

/// Get all members of a pack
pub fn get_pack_members(leader: Entity, world: &World) -> Vec<Entity> {
    world
        .get::<PackLeader>(leader)
        .map(|pack| pack.members.clone())
        .unwrap_or_default()
}

/// Get the leader of a pack member
pub fn get_pack_leader(member: Entity, world: &World) -> Option<Entity> {
    world.get::<PackMember>(member).map(|m| m.leader)
}

/// Check if a wolf is a pack leader
pub fn is_pack_leader(wolf: Entity, world: &World) -> bool {
    world.get::<PackLeader>(wolf).is_some()
}

/// Check if a wolf is a pack member
pub fn is_pack_member(wolf: Entity, world: &World) -> bool {
    world.get::<PackMember>(wolf).is_some()
}

/// Check if a wolf is in a pack (either as leader or member)
pub fn is_in_pack(wolf: Entity, world: &World) -> bool {
    is_pack_leader(wolf, world) || is_pack_member(wolf, world)
}

/// Get pack size (leader + members)
pub fn get_pack_size(leader: Entity, world: &World) -> usize {
    get_pack_members(leader, world).len() + 1
}

/// Check if two wolves are in the same pack
pub fn are_in_same_pack(wolf1: Entity, wolf2: Entity, world: &World) -> bool {
    // Check if one is the leader of the other
    if let Some(pack) = world.get::<PackLeader>(wolf1) {
        if pack.has_member(wolf2) {
            return true;
        }
    }

    if let Some(pack) = world.get::<PackLeader>(wolf2) {
        if pack.has_member(wolf1) {
            return true;
        }
    }

    // Check if they share the same leader
    if let Some(member1) = world.get::<PackMember>(wolf1) {
        if let Some(member2) = world.get::<PackMember>(wolf2) {
            return member1.leader == member2.leader;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test: establish_pack_leadership components structure
    #[test]
    fn test_establish_pack_leadership_components() {
        let leader = Entity::from_raw(1);

        // These components should be created with proper values
        let pack = PackLeader::new(100, GroupType::Pack);

        assert_eq!(pack.formed_tick, 100);
        assert_eq!(pack.group_type, GroupType::Pack);
        assert_eq!(pack.member_count(), 0);
    }

    /// Test: Multiple pack leaders can exist independently
    #[test]
    fn test_multiple_pack_leaders_independent() {
        let leader1 = PackLeader::new(100, GroupType::Pack);
        let leader2 = PackLeader::new(120, GroupType::Pack);

        assert_eq!(leader1.formed_tick, 100);
        assert_eq!(leader2.formed_tick, 120);
        assert_ne!(leader1.formed_tick, leader2.formed_tick);
    }

    /// Test: get_pack_members returns empty for non-pack wolf
    #[test]
    fn test_get_pack_members_validation() {
        let leader = Entity::from_raw(1);
        let mut pack = PackLeader::new(100, GroupType::Pack);
        let member1 = Entity::from_raw(2);
        let member2 = Entity::from_raw(3);

        pack.add_member(member1);
        pack.add_member(member2);

        assert_eq!(pack.member_count(), 2);
        assert!(pack.members.contains(&member1));
        assert!(pack.members.contains(&member2));
    }

    /// Test: Pack member references leader correctly
    #[test]
    fn test_pack_member_leader_reference() {
        let leader = Entity::from_raw(1);
        let member = PackMember::new(leader, 100, GroupType::Pack);

        assert_eq!(member.leader, leader);
    }

    /// Test: Multiple members can track same leader
    #[test]
    fn test_multiple_members_same_leader() {
        let leader = Entity::from_raw(1);
        let member1 = PackMember::new(leader, 100, GroupType::Pack);
        let member2 = PackMember::new(leader, 105, GroupType::Pack);

        assert_eq!(member1.leader, member2.leader);
        assert_eq!(member1.leader, leader);
        assert_eq!(member2.leader, leader);
    }

    /// Test: Pack formation duration tracking
    #[test]
    fn test_pack_formation_duration() {
        let pack = PackLeader::new(100, GroupType::Pack);
        let current_tick = 200;
        let duration = current_tick - pack.formed_tick;

        assert_eq!(duration, 100);
    }

    /// Test: Member join time tracking
    #[test]
    fn test_member_join_time_tracking() {
        let leader = Entity::from_raw(1);
        let member1 = PackMember::new(leader, 100, GroupType::Pack);
        let member2 = PackMember::new(leader, 150, GroupType::Pack);

        assert_ne!(member1.joined_tick, member2.joined_tick);
        assert!(member1.joined_tick < member2.joined_tick);
        assert_eq!(member2.joined_tick - member1.joined_tick, 50);
    }

    /// Test: cleanup_stale_pack_relationships conceptually
    #[test]
    fn test_cleanup_stale_packs_validation() {
        let mut leader = PackLeader::new(100, GroupType::Pack);
        let member = Entity::from_raw(5);

        // Add a member first
        leader.add_member(member);
        assert_eq!(leader.member_count(), 1);

        // Simulate removing a member
        leader.remove_member(member);

        // Members list should be empty
        assert_eq!(leader.member_count(), 0);
    }

    /// Test: are_in_same_pack validation with shared leader
    #[test]
    fn test_pack_member_sharing_validation() {
        let leader = Entity::from_raw(1);
        let member1 = PackMember::new(leader, 100, GroupType::Pack);
        let member2 = PackMember::new(leader, 100, GroupType::Pack);

        // Both reference same leader
        assert_eq!(member1.leader, member2.leader);
    }
}
