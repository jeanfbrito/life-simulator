//! Generic group behavior coordination system
//!
//! This system applies species-specific behavior bonuses based on group membership.
//! It delegates to species-specific behavior modules based on GroupType.

use bevy::prelude::*;
use crate::entities::{PackLeader, PackMember, GroupType};
use crate::ai::UtilityScore;
use crate::ai::behaviors::{
    apply_pack_hunting_bonus, apply_herd_safety_bonus, apply_warren_defense_bonus,
};

/// Apply group-specific behavior bonuses to actions
pub fn apply_group_behavior_bonuses(
    entity: Entity,
    actions: &mut Vec<UtilityScore>,
    world: &World,
) {
    // Check if entity is in a group
    let group_info = get_group_info(entity, world);

    if let Some((group_type, leader, members)) = group_info {
        // Apply species-specific bonuses based on group type
        match group_type {
            GroupType::Pack => apply_pack_hunting_bonus(entity, actions, leader, members, world),
            GroupType::Herd => apply_herd_safety_bonus(entity, actions, leader, members, world),
            GroupType::Warren => apply_warren_defense_bonus(entity, actions, leader, members, world),
            GroupType::Flock => apply_flock_coordination_bonus(entity, actions, leader, members, world),
            _ => {} // Other types not yet implemented
        }
    }
}

/// Get group info for an entity (if in a group)
fn get_group_info(entity: Entity, world: &World) -> Option<(GroupType, Entity, Vec<Entity>)> {
    // Check if leader
    if let Some(leader) = world.get::<PackLeader>(entity) {
        return Some((leader.group_type, entity, leader.members.clone()));
    }

    // Check if member
    if let Some(member) = world.get::<PackMember>(entity) {
        if let Some(leader_comp) = world.get::<PackLeader>(member.leader) {
            return Some((member.group_type, member.leader, leader_comp.members.clone()));
        }
    }

    None
}


/// Apply flock coordination bonus for birds
fn apply_flock_coordination_bonus(
    _entity: Entity,
    _actions: &mut Vec<UtilityScore>,
    _leader: Entity,
    _members: Vec<Entity>,
    _world: &World,
) {
    // TODO: Implement in behaviors/flock_coordination.rs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::GroupFormationConfig;

    /// RED: Test get_group_info returns None for non-grouped entity
    #[test]
    fn test_get_group_info_no_group() {
        let mut app = App::new();

        let entity = app.world_mut().spawn_empty().id();

        let info = get_group_info(entity, app.world());
        assert!(info.is_none(), "Entity not in group should return None");
    }

    /// RED: Test get_group_info returns info for leader
    #[test]
    fn test_get_group_info_leader() {
        let mut app = App::new();

        let member1 = Entity::from_raw(10);
        let member2 = Entity::from_raw(11);

        let leader = app.world_mut().spawn(PackLeader {
            members: vec![member1, member2],
            formed_tick: 100,
            group_type: GroupType::Pack,
        }).id();

        let info = get_group_info(leader, app.world());
        assert!(info.is_some(), "Leader should return group info");

        let (group_type, leader_entity, members) = info.unwrap();
        assert_eq!(group_type, GroupType::Pack);
        assert_eq!(leader_entity, leader);
        assert_eq!(members.len(), 2);
        assert!(members.contains(&member1));
        assert!(members.contains(&member2));
    }

    /// RED: Test get_group_info returns info for member
    #[test]
    fn test_get_group_info_member() {
        let mut app = App::new();

        let leader = app.world_mut().spawn(PackLeader {
            members: vec![],
            formed_tick: 100,
            group_type: GroupType::Pack,
        }).id();

        let member = app.world_mut().spawn(PackMember {
            leader,
            joined_tick: 100,
            group_type: GroupType::Pack,
        }).id();

        let info = get_group_info(member, app.world());
        assert!(info.is_some(), "Member should return group info");

        let (group_type, leader_entity, _members) = info.unwrap();
        assert_eq!(group_type, GroupType::Pack);
        assert_eq!(leader_entity, leader);
    }

    /// RED: Test get_group_info handles orphaned member (no leader)
    #[test]
    fn test_get_group_info_orphaned_member() {
        let mut app = App::new();

        let member = app.world_mut().spawn(PackMember {
            leader: Entity::from_raw(999), // Non-existent leader
            joined_tick: 100,
            group_type: GroupType::Pack,
        }).id();

        let info = get_group_info(member, app.world());
        assert!(info.is_none(), "Orphaned member should return None");
    }

    /// RED: Test apply_group_behavior_bonuses doesn't crash for non-grouped entity
    #[test]
    fn test_apply_bonuses_no_group() {
        let mut app = App::new();

        let entity = app.world_mut().spawn_empty().id();
        let mut actions = vec![];

        // Should not crash
        apply_group_behavior_bonuses(entity, &mut actions, app.world());
    }

    /// RED: Test apply_group_behavior_bonuses calls correct handler
    #[test]
    fn test_apply_bonuses_pack_type() {
        let mut app = App::new();

        let leader = app.world_mut().spawn(PackLeader {
            members: vec![],
            formed_tick: 100,
            group_type: GroupType::Pack,
        }).id();

        let mut actions = vec![];

        // Should call pack hunting bonus (currently stubbed)
        apply_group_behavior_bonuses(leader, &mut actions, app.world());
        // No assertion yet - just testing it doesn't crash
    }

    /// RED: Test apply_group_behavior_bonuses for different group types
    #[test]
    fn test_apply_bonuses_different_types() {
        let mut app = App::new();

        // Test Pack
        let pack_leader = app.world_mut().spawn(PackLeader {
            members: vec![],
            formed_tick: 100,
            group_type: GroupType::Pack,
        }).id();

        // Test Herd
        let herd_leader = app.world_mut().spawn(PackLeader {
            members: vec![],
            formed_tick: 100,
            group_type: GroupType::Herd,
        }).id();

        // Test Warren
        let warren_leader = app.world_mut().spawn(PackLeader {
            members: vec![],
            formed_tick: 100,
            group_type: GroupType::Warren,
        }).id();

        let mut actions = vec![];

        // All should run without crashing
        apply_group_behavior_bonuses(pack_leader, &mut actions, app.world());
        apply_group_behavior_bonuses(herd_leader, &mut actions, app.world());
        apply_group_behavior_bonuses(warren_leader, &mut actions, app.world());
    }
}
