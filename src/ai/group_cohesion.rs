//! Generic group cohesion system - maintains groups and dissolves when members drift
//!
//! This system checks group member distances and removes members that drift too far.
//! Groups are dissolved when they fall below minimum size.

use bevy::prelude::*;
use crate::entities::{TilePosition, GroupFormationConfig, PackLeader, PackMember};
use crate::simulation::SimulationTick;

/// Distance helper function
fn distance(a: IVec2, b: IVec2) -> f32 {
    let diff = a - b;
    ((diff.x * diff.x + diff.y * diff.y) as f32).sqrt()
}

/// Maintains group cohesion - dissolves groups when members drift too far
pub fn generic_group_cohesion_system(
    mut commands: Commands,
    tick: Res<SimulationTick>,
    groups: Query<(Entity, &PackLeader, &TilePosition, &GroupFormationConfig)>,
    members: Query<(Entity, &PackMember, &TilePosition)>,
) {
    for (leader_entity, leader, leader_pos, config) in groups.iter() {
        // Only check at intervals
        if tick.0 % config.check_interval_ticks != 0 {
            continue;
        }

        // Check member distances
        let mut members_to_remove = Vec::new();

        for &member_entity in &leader.members {
            if let Ok((_, _, member_pos)) = members.get(member_entity) {
                let dist = distance(leader_pos.tile, member_pos.tile);

                if dist > config.cohesion_radius {
                    members_to_remove.push(member_entity);
                }
            } else {
                // Member doesn't exist anymore
                members_to_remove.push(member_entity);
            }
        }

        // Remove distant/dead members
        for member in &members_to_remove {
            commands.entity(*member).remove::<PackMember>();
            commands.entity(leader_entity).insert(RemoveMemberMarker(*member));
        }

        // If group too small after removals, dissolve it
        let remaining_size = leader.members.len() - members_to_remove.len();
        if remaining_size < config.min_group_size - 1 {
            // Dissolve group
            for &member in &leader.members {
                commands.entity(member).remove::<PackMember>();
            }
            commands.entity(leader_entity).remove::<PackLeader>();
        }
    }
}

/// Helper marker for deferred member removal
#[derive(Component)]
pub struct RemoveMemberMarker(pub Entity);

/// System to process deferred member removals
pub fn process_member_removals(
    mut commands: Commands,
    mut leaders: Query<&mut PackLeader>,
    markers: Query<(Entity, &RemoveMemberMarker)>,
) {
    for (leader_entity, marker) in markers.iter() {
        if let Ok(mut leader) = leaders.get_mut(leader_entity) {
            leader.members.retain(|&e| e != marker.0);
        }
        commands.entity(leader_entity).remove::<RemoveMemberMarker>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{GroupFormationConfig, GroupType};

    /// RED: Test distance calculation works
    #[test]
    fn test_distance_calculation() {
        let a = IVec2::new(0, 0);
        let b = IVec2::new(3, 4);

        let dist = distance(a, b);
        assert_eq!(dist, 5.0);
    }

    /// RED: Test generic_group_cohesion_system removes distant members
    #[test]
    fn test_cohesion_removes_distant_members() {
        let mut app = App::new();
        app.insert_resource(SimulationTick(300)); // Tick that matches check interval

        let config = GroupFormationConfig::wolf_pack(); // cohesion_radius = 150

        // Create leader at origin
        let leader = app.world_mut().spawn((
            PackLeader {
                members: vec![],
                formed_tick: 0,
                group_type: GroupType::Pack,
            },
            TilePosition::from_tile(IVec2::new(0, 0)),
            config.clone(),
        )).id();

        // Create member within range
        let member_close = app.world_mut().spawn((
            PackMember {
                leader,
                joined_tick: 0,
                group_type: GroupType::Pack,
            },
            TilePosition::from_tile(IVec2::new(50, 0)), // Distance 50 < 150
        )).id();

        // Create member out of range
        let member_far = app.world_mut().spawn((
            PackMember {
                leader,
                joined_tick: 0,
                group_type: GroupType::Pack,
            },
            TilePosition::from_tile(IVec2::new(200, 0)), // Distance 200 > 150
        )).id();

        // Update leader's member list
        app.world_mut().entity_mut(leader).insert(PackLeader {
            members: vec![member_close, member_far],
            formed_tick: 0,
            group_type: GroupType::Pack,
        });

        // Run cohesion system
        app.add_systems(Update, generic_group_cohesion_system);
        app.update();

        // Far member should have RemoveMemberMarker
        assert!(
            app.world().get::<RemoveMemberMarker>(leader).is_some(),
            "Leader should have RemoveMemberMarker for distant member"
        );
    }

    /// RED: Test process_member_removals removes members from leader list
    #[test]
    fn test_process_member_removals() {
        let mut app = App::new();

        let member_to_remove = Entity::from_raw(99);

        let leader = app.world_mut().spawn((
            PackLeader {
                members: vec![Entity::from_raw(1), member_to_remove, Entity::from_raw(2)],
                formed_tick: 0,
                group_type: GroupType::Pack,
            },
            RemoveMemberMarker(member_to_remove),
        )).id();

        app.add_systems(Update, process_member_removals);
        app.update();

        let pack_leader = app.world().get::<PackLeader>(leader).unwrap();
        assert_eq!(pack_leader.members.len(), 2, "Should have 2 members left");
        assert!(!pack_leader.members.contains(&member_to_remove), "Removed member should not be in list");
        assert!(app.world().get::<RemoveMemberMarker>(leader).is_none(), "Marker should be removed");
    }

    /// RED: Test cohesion dissolves group when below min size
    #[test]
    fn test_cohesion_dissolves_small_group() {
        let mut app = App::new();
        app.insert_resource(SimulationTick(300)); // Tick that matches check interval

        let config = GroupFormationConfig::wolf_pack(); // min_group_size = 3

        // Create leader with only 2 members (below min)
        let leader = app.world_mut().spawn((
            PackLeader {
                members: vec![],
                formed_tick: 0,
                group_type: GroupType::Pack,
            },
            TilePosition::from_tile(IVec2::new(0, 0)),
            config.clone(),
        )).id();

        let member1 = app.world_mut().spawn((
            PackMember {
                leader,
                joined_tick: 0,
                group_type: GroupType::Pack,
            },
            TilePosition::from_tile(IVec2::new(10, 0)),
        )).id();

        let member2 = app.world_mut().spawn((
            PackMember {
                leader,
                joined_tick: 0,
                group_type: GroupType::Pack,
            },
            TilePosition::from_tile(IVec2::new(200, 0)), // Too far - will be removed
        )).id();

        // Update leader's member list
        app.world_mut().entity_mut(leader).insert(PackLeader {
            members: vec![member1, member2],
            formed_tick: 0,
            group_type: GroupType::Pack,
        });

        app.add_systems(Update, generic_group_cohesion_system);
        app.update();

        // After removing far member, only 1 remains (< min_group_size - 1)
        // Group should be dissolved
        assert!(
            app.world().get::<PackLeader>(leader).is_none(),
            "Leader component should be removed (group dissolved)"
        );
    }

    /// RED: Test cohesion handles dead members
    #[test]
    fn test_cohesion_handles_dead_members() {
        let mut app = App::new();
        app.insert_resource(SimulationTick(300));

        let config = GroupFormationConfig::wolf_pack();

        let leader = app.world_mut().spawn((
            PackLeader {
                members: vec![Entity::from_raw(999)], // Non-existent entity
                formed_tick: 0,
                group_type: GroupType::Pack,
            },
            TilePosition::from_tile(IVec2::new(0, 0)),
            config.clone(),
        )).id();

        app.add_systems(Update, generic_group_cohesion_system);
        app.update();

        // Dead member should trigger removal and group dissolution
        assert!(
            app.world().get::<PackLeader>(leader).is_none(),
            "Group should be dissolved due to dead member"
        );
    }

    /// RED: Test cohesion only runs at check intervals
    #[test]
    fn test_cohesion_respects_check_interval() {
        let mut app = App::new();
        app.insert_resource(SimulationTick(299)); // Not divisible by 300

        let config = GroupFormationConfig::wolf_pack(); // check_interval_ticks = 300

        let leader = app.world_mut().spawn((
            PackLeader {
                members: vec![],
                formed_tick: 0,
                group_type: GroupType::Pack,
            },
            TilePosition::from_tile(IVec2::new(0, 0)),
            config.clone(),
        )).id();

        let member_far = app.world_mut().spawn((
            PackMember {
                leader,
                joined_tick: 0,
                group_type: GroupType::Pack,
            },
            TilePosition::from_tile(IVec2::new(200, 0)), // Too far
        )).id();

        app.world_mut().entity_mut(leader).insert(PackLeader {
            members: vec![member_far],
            formed_tick: 0,
            group_type: GroupType::Pack,
        });

        app.add_systems(Update, generic_group_cohesion_system);
        app.update();

        // Should NOT process because tick doesn't match interval
        assert!(
            app.world().get::<PackMember>(member_far).is_some(),
            "Member should still have component (not processed yet)"
        );
    }
}
