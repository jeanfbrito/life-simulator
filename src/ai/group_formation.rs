//! Generic group formation system - works for ANY species with GroupFormationConfig
//!
//! This system uses spatial clustering to find nearby entities and form groups
//! based on species-specific configuration.

use bevy::prelude::*;
use crate::entities::{TilePosition, GroupFormationConfig, PackLeader, PackMember, GroupType};
use crate::simulation::SimulationTick;
use std::collections::{HashMap, HashSet};

/// Distance helper function
fn distance(a: IVec2, b: IVec2) -> f32 {
    let diff = a - b;
    ((diff.x * diff.x + diff.y * diff.y) as f32).sqrt()
}

/// Generic group formation system - works for ANY species with GroupFormationConfig
pub fn generic_group_formation_system(
    mut commands: Commands,
    tick: Res<SimulationTick>,
    // Any entity with config, position, not already in a group
    candidates: Query<
        (Entity, &TilePosition, &GroupFormationConfig),
        (Without<PackLeader>, Without<PackMember>)
    >,
) {
    // Group candidates by species (same group_type = same species for grouping)
    let mut species_groups: HashMap<GroupType, Vec<(Entity, IVec2, GroupFormationConfig)>> = HashMap::new();

    for (entity, pos, config) in candidates.iter() {
        if !config.enabled {
            continue;
        }

        // Only check at configured intervals
        if tick.0 % config.check_interval_ticks != 0 {
            continue;
        }

        species_groups
            .entry(config.group_type)
            .or_default()
            .push((entity, pos.tile, config.clone()));
    }

    // Form groups for each species type
    for (_group_type, entities) in species_groups {
        if entities.is_empty() {
            continue;
        }

        let config = &entities[0].2; // All same species = same config

        // Find clusters of nearby entities
        let clusters = find_proximity_clusters(&entities, config.formation_radius);

        // Form groups from clusters that meet min size
        for cluster in clusters {
            if cluster.len() >= config.min_group_size {
                form_group_from_cluster(
                    &mut commands,
                    cluster,
                    config,
                    tick.0,
                );
            }
        }
    }
}

/// Find clusters of entities within formation radius
fn find_proximity_clusters(
    entities: &[(Entity, IVec2, GroupFormationConfig)],
    radius: f32,
) -> Vec<Vec<Entity>> {
    let mut clusters = Vec::new();
    let mut assigned = HashSet::new();

    for (i, (entity1, pos1, config)) in entities.iter().enumerate() {
        if assigned.contains(&i) {
            continue;
        }

        let mut cluster = vec![*entity1];
        assigned.insert(i);

        for (j, (entity2, pos2, _)) in entities.iter().enumerate() {
            if i == j || assigned.contains(&j) {
                continue;
            }

            if distance(*pos1, *pos2) <= radius {
                cluster.push(*entity2);
                assigned.insert(j);

                // Stop if we hit max group size
                if cluster.len() >= config.max_group_size {
                    break;
                }
            }
        }

        if cluster.len() >= config.min_group_size {
            clusters.push(cluster);
        }
    }

    clusters
}

/// Form a group from a cluster of entities
fn form_group_from_cluster(
    commands: &mut Commands,
    cluster: Vec<Entity>,
    config: &GroupFormationConfig,
    tick: u64,
) {
    if cluster.is_empty() {
        return;
    }

    // First entity becomes leader
    let leader = cluster[0];
    let members = &cluster[1..];

    // Establish leadership
    commands.entity(leader).insert(PackLeader {
        members: members.to_vec(),
        formed_tick: tick,
        group_type: config.group_type,
    });

    // Add members
    for &member in members {
        commands.entity(member).insert(PackMember {
            leader,
            joined_tick: tick,
            group_type: config.group_type,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RED: Test distance function works correctly
    #[test]
    fn test_distance_calculation() {
        let a = IVec2::new(0, 0);
        let b = IVec2::new(3, 4);

        let dist = distance(a, b);
        assert_eq!(dist, 5.0); // 3-4-5 triangle
    }

    /// RED: Test distance between same point is zero
    #[test]
    fn test_distance_same_point() {
        let a = IVec2::new(10, 20);
        let dist = distance(a, a);
        assert_eq!(dist, 0.0);
    }

    /// RED: Test find_proximity_clusters finds nearby entities
    #[test]
    fn test_find_proximity_clusters_basic() {
        let config = GroupFormationConfig::wolf_pack();

        let entities = vec![
            (Entity::from_raw(1), IVec2::new(0, 0), config.clone()),
            (Entity::from_raw(2), IVec2::new(10, 0), config.clone()),
            (Entity::from_raw(3), IVec2::new(20, 0), config.clone()),
        ];

        // Formation radius 50 - all should cluster together
        let clusters = find_proximity_clusters(&entities, 50.0);

        assert_eq!(clusters.len(), 1, "Should form one cluster");
        assert_eq!(clusters[0].len(), 3, "Cluster should have all 3 entities");
    }

    /// RED: Test find_proximity_clusters separates distant entities
    #[test]
    fn test_find_proximity_clusters_separated() {
        let config = GroupFormationConfig::wolf_pack();

        let entities = vec![
            (Entity::from_raw(1), IVec2::new(0, 0), config.clone()),
            (Entity::from_raw(2), IVec2::new(10, 0), config.clone()),
            (Entity::from_raw(3), IVec2::new(200, 0), config.clone()), // Far away
        ];

        // Formation radius 50 - third entity too far
        let clusters = find_proximity_clusters(&entities, 50.0);

        // Should only form cluster with min_group_size (3 for wolves)
        // Since we only have 2 nearby entities, no valid clusters
        assert_eq!(clusters.len(), 0, "Should not form cluster (only 2 nearby)");
    }

    /// RED: Test find_proximity_clusters respects min_group_size
    #[test]
    fn test_find_proximity_clusters_min_size() {
        let config = GroupFormationConfig::wolf_pack(); // min_group_size = 3

        let entities = vec![
            (Entity::from_raw(1), IVec2::new(0, 0), config.clone()),
            (Entity::from_raw(2), IVec2::new(10, 0), config.clone()),
        ];

        let clusters = find_proximity_clusters(&entities, 50.0);

        assert_eq!(clusters.len(), 0, "Should not form cluster (below min size)");
    }

    /// RED: Test find_proximity_clusters respects max_group_size
    #[test]
    fn test_find_proximity_clusters_max_size() {
        let config = GroupFormationConfig::wolf_pack(); // max_group_size = 8

        // Create 10 entities all close together
        let entities: Vec<_> = (0..10)
            .map(|i| (Entity::from_raw(i as u32), IVec2::new(i * 5, 0), config.clone()))
            .collect();

        let clusters = find_proximity_clusters(&entities, 50.0);

        assert!(clusters.len() > 0, "Should form at least one cluster");
        for cluster in &clusters {
            assert!(cluster.len() <= config.max_group_size, "Cluster should not exceed max size");
        }
    }

    /// RED: Test form_group_from_cluster creates leader and members
    #[test]
    fn test_form_group_creates_components() {
        let mut app = App::new();
        let config = GroupFormationConfig::wolf_pack();

        let e1 = app.world_mut().spawn_empty().id();
        let e2 = app.world_mut().spawn_empty().id();
        let e3 = app.world_mut().spawn_empty().id();

        let cluster = vec![e1, e2, e3];

        // Use commands queue to avoid borrow checker issues
        app.world_mut().commands().queue(move |world: &mut World| {
            let mut commands = world.commands();
            form_group_from_cluster(
                &mut commands,
                cluster,
                &config,
                100,
            );
        });

        app.update();

        // Leader should have PackLeader component
        let leader = app.world().get::<PackLeader>(e1);
        assert!(leader.is_some(), "First entity should be leader");

        let leader = leader.unwrap();
        assert_eq!(leader.members.len(), 2, "Leader should have 2 members");
        assert_eq!(leader.group_type, GroupType::Pack);
        assert_eq!(leader.formed_tick, 100);

        // Members should have PackMember component
        let member2 = app.world().get::<PackMember>(e2);
        assert!(member2.is_some(), "Second entity should be member");
        assert_eq!(member2.unwrap().leader, e1);
        assert_eq!(member2.unwrap().group_type, GroupType::Pack);

        let member3 = app.world().get::<PackMember>(e3);
        assert!(member3.is_some(), "Third entity should be member");
        assert_eq!(member3.unwrap().leader, e1);
    }

    /// RED: Test empty cluster doesn't crash
    #[test]
    fn test_form_group_empty_cluster() {
        let mut app = App::new();
        let config = GroupFormationConfig::wolf_pack();

        let cluster = vec![];

        // Use commands queue to avoid borrow checker issues
        app.world_mut().commands().queue(move |world: &mut World| {
            let mut commands = world.commands();
            form_group_from_cluster(
                &mut commands,
                cluster,
                &config,
                100,
            );
        });

        app.update();
        // Should not crash, just do nothing
    }
}
