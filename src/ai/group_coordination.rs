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
    leader_query: &Query<&PackLeader>,
    member_query: &Query<&PackMember>,
) {
    // Check if entity is in a group
    let group_info = get_group_info(entity, leader_query, member_query);

    if let Some((group_type, leader, members)) = group_info {
        // Apply species-specific bonuses based on group type
        match group_type {
            GroupType::Pack => apply_pack_hunting_bonus(entity, actions, leader, members),
            GroupType::Herd => apply_herd_safety_bonus(entity, actions, leader, members),
            GroupType::Warren => apply_warren_defense_bonus(entity, actions, leader, members),
            GroupType::Flock => apply_flock_coordination_bonus(entity, actions, leader, members),
            _ => {} // Other types not yet implemented
        }
    }
}

/// Get group info for an entity (if in a group)
fn get_group_info(
    entity: Entity,
    leader_query: &Query<&PackLeader>,
    member_query: &Query<&PackMember>,
) -> Option<(GroupType, Entity, Vec<Entity>)> {
    // Check if leader
    if let Ok(leader) = leader_query.get(entity) {
        return Some((leader.group_type, entity, leader.members.clone()));
    }

    // Check if member
    if let Ok(member) = member_query.get(entity) {
        if let Ok(leader_comp) = leader_query.get(member.leader) {
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
) {
    // TODO: Implement in behaviors/flock_coordination.rs
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use bevy::prelude::*;

    /// Test helper: get group info from world directly (bypasses Query types)
    fn get_group_info_from_world(
        world: &World,
        entity: Entity,
    ) -> Option<(GroupType, Entity, Vec<Entity>)> {
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

    /// Test get_group_info_from_world returns None for non-grouped entity
    #[test]
    fn test_get_group_info_no_group() {
        let mut app = App::new();
        let entity = app.world_mut().spawn_empty().id();

        let info = get_group_info_from_world(app.world(), entity);
        assert!(info.is_none(), "Entity not in group should return None");
    }

    /// Test get_group_info_from_world returns info for leader
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

        let info = get_group_info_from_world(app.world(), leader);
        assert!(info.is_some(), "Leader should return group info");

        let (group_type, leader_entity, members) = info.unwrap();
        assert_eq!(group_type, GroupType::Pack);
        assert_eq!(leader_entity, leader);
        assert_eq!(members.len(), 2);
        assert!(members.contains(&member1));
        assert!(members.contains(&member2));
    }

    /// Test get_group_info_from_world returns info for member
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

        let info = get_group_info_from_world(app.world(), member);
        assert!(info.is_some(), "Member should return group info");

        let (group_type, leader_entity, _members) = info.unwrap();
        assert_eq!(group_type, GroupType::Pack);
        assert_eq!(leader_entity, leader);
    }

    /// Test get_group_info_from_world handles orphaned member (no leader)
    #[test]
    fn test_get_group_info_orphaned_member() {
        let mut app = App::new();

        let member = app.world_mut().spawn(PackMember {
            leader: Entity::from_raw(999), // Non-existent leader
            joined_tick: 100,
            group_type: GroupType::Pack,
        }).id();

        let info = get_group_info_from_world(app.world(), member);
        assert!(info.is_none(), "Orphaned member should return None");
    }

    /// Test that entity spawning with group components works
    #[test]
    fn test_group_components_spawn() {
        let mut app = App::new();

        // Test Pack leader spawns correctly
        let pack_leader = app.world_mut().spawn(PackLeader {
            members: vec![],
            formed_tick: 100,
            group_type: GroupType::Pack,
        }).id();

        assert!(app.world().get::<PackLeader>(pack_leader).is_some());
        assert_eq!(app.world().get::<PackLeader>(pack_leader).unwrap().group_type, GroupType::Pack);
    }

    /// Test different group types via component inspection
    #[test]
    fn test_different_group_types() {
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

        // Verify all spawn correctly
        let pack_info = get_group_info_from_world(app.world(), pack_leader);
        let herd_info = get_group_info_from_world(app.world(), herd_leader);
        let warren_info = get_group_info_from_world(app.world(), warren_leader);

        assert!(pack_info.is_some());
        assert!(herd_info.is_some());
        assert!(warren_info.is_some());

        assert_eq!(pack_info.unwrap().0, GroupType::Pack);
        assert_eq!(herd_info.unwrap().0, GroupType::Herd);
        assert_eq!(warren_info.unwrap().0, GroupType::Warren);
    }
}
