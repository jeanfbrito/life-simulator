//! Integration tests for the new species architecture
//!
//! These tests validate that the modular species system works correctly
//! and that new species can be added without breaking existing functionality.

use life_simulator::entities::{
    SpawnConfig, SpawnGroup, SpawnSex, SPECIES_REGISTRY, SPECIES_SYSTEMS_REGISTRY,
};

#[test]
fn test_species_registry_basic_functionality() {
    // Test that the species registry is properly initialized
    let descriptors = SPECIES_REGISTRY.get_descriptors();
    assert!(
        !descriptors.is_empty(),
        "Species registry should contain species"
    );

    // Test that expected species are present
    let species_names: Vec<&str> = descriptors.iter().map(|d| d.species).collect();
    assert!(
        species_names.contains(&"Rabbit"),
        "Rabbit should be in registry"
    );
    assert!(
        species_names.contains(&"Deer"),
        "Deer should be in registry"
    );
    assert!(
        species_names.contains(&"Raccoon"),
        "Raccoon should be in registry"
    );

    // Test that we can access specific species
    let rabbit = SPECIES_REGISTRY.rabbit();
    assert_eq!(rabbit.species, "Rabbit");
    assert_eq!(rabbit.emoji, "🐇");
    assert_eq!(rabbit.movement_speed, 20);

    let deer = SPECIES_REGISTRY.deer();
    assert_eq!(deer.species, "Deer");
    assert_eq!(deer.emoji, "🦌");
    assert_eq!(deer.movement_speed, 10);
}

#[test]
fn test_species_systems_registry() {
    // Test that the systems registry is properly initialized
    let descriptors = SPECIES_SYSTEMS_REGISTRY.get_descriptors();
    assert!(
        !descriptors.is_empty(),
        "Systems registry should contain species"
    );

    // Test that species have expected systems
    assert!(
        SPECIES_SYSTEMS_REGISTRY.species_has_planner_system("Rabbit"),
        "Rabbit should have planner system"
    );
    assert!(
        SPECIES_SYSTEMS_REGISTRY.species_has_mate_matching("Rabbit"),
        "Rabbit should have mate matching system"
    );
    assert!(
        SPECIES_SYSTEMS_REGISTRY.species_has_birth_system("Rabbit"),
        "Rabbit should have birth system"
    );

    // Test deer systems
    assert!(
        SPECIES_SYSTEMS_REGISTRY.species_has_planner_system("Deer"),
        "Deer should have planner system"
    );
    assert!(
        SPECIES_SYSTEMS_REGISTRY.species_has_mate_matching("Deer"),
        "Deer should have mate matching system"
    );
    assert!(
        SPECIES_SYSTEMS_REGISTRY.species_has_birth_system("Deer"),
        "Deer should have birth system"
    );
}

#[test]
fn test_spawn_config_default() {
    // Test that default spawn configuration loads correctly
    let config = SpawnConfig::default();

    assert!(
        !config.spawn_groups.is_empty(),
        "Default config should have spawn groups"
    );

    // Test rabbit spawn group
    let rabbit_group = config
        .spawn_groups
        .iter()
        .find(|g| g.species == "Rabbit")
        .expect("Should have rabbit spawn group");

    assert_eq!(rabbit_group.count, 5);
    assert_eq!(rabbit_group.names.len(), 5);
    assert_eq!(rabbit_group.names[0], "Bugs");
    assert_eq!(rabbit_group.spawn_area.center, (0, 0));
    assert_eq!(rabbit_group.spawn_area.search_radius, 15);
    assert!(rabbit_group.sex_sequence.is_none());

    // Test deer spawn group
    let deer_group = config
        .spawn_groups
        .iter()
        .find(|g| g.species == "Deer")
        .expect("Should have deer spawn group");

    assert_eq!(deer_group.count, 2);
    assert_eq!(deer_group.names[0], "Stag");
    assert_eq!(deer_group.names[1], "Doe");
    let deer_sexes = deer_group
        .sex_sequence
        .as_ref()
        .expect("Deer should define sex pattern");
    assert_eq!(deer_sexes.as_slice(), &[SpawnSex::Male, SpawnSex::Female]);

    // Test raccoon spawn group
    let raccoon_group = config
        .spawn_groups
        .iter()
        .find(|g| g.species == "Raccoon")
        .expect("Should have raccoon spawn group");

    assert_eq!(raccoon_group.count, 2);
    assert_eq!(raccoon_group.names[0], "Bandit");
    assert_eq!(raccoon_group.names[1], "Maple");
    let raccoon_sexes = raccoon_group
        .sex_sequence
        .as_ref()
        .expect("Raccoons should define sex pattern");
    assert_eq!(
        raccoon_sexes.as_slice(),
        &[SpawnSex::Male, SpawnSex::Female]
    );
}

#[test]
fn test_spawn_config_name_generation() {
    let config = SpawnConfig::default();

    // Test normal name generation
    let name = config.get_name_for_group(0, 0); // Rabbit group, first entity
    assert_eq!(name, "Bugs");

    let name = config.get_name_for_group(0, 1); // Rabbit group, second entity
    assert_eq!(name, "Roger");

    // Test name cycling (more entities than names)
    let name = config.get_name_for_group(0, 5); // Rabbit group, 6th entity (cycles back)
    assert_eq!(name, "Bugs");

    // Test unknown group (should generate default name)
    let name = config.get_name_for_group(999, 0);
    assert_eq!(name, "Entity_1");
}

#[test]
fn test_spawn_config_serialization() {
    let config = SpawnConfig::default();

    // Test that we can serialize to RON
    use ron::ser::to_string_pretty;
    let ron_str = to_string_pretty(&config).expect("Should serialize config");
    assert!(
        !ron_str.is_empty(),
        "RON serialization should produce output"
    );

    // Test that we can deserialize back
    let deserialized: SpawnConfig =
        ron::from_str(&ron_str).expect("Should be able to deserialize config");

    // Verify key properties match
    assert_eq!(deserialized.spawn_groups.len(), config.spawn_groups.len());
    assert_eq!(
        deserialized.settings.enable_spawning,
        config.settings.enable_spawning
    );
}

#[test]
fn test_species_viewer_metadata() {
    // This test simulates the viewer metadata generation
    let descriptors = SPECIES_REGISTRY.get_descriptors();

    // Test that all species have required viewer metadata
    for descriptor in descriptors {
        assert!(
            !descriptor.emoji.is_empty(),
            "Species {} should have emoji",
            descriptor.species
        );
        assert!(
            descriptor.viewer_scale > 0.0,
            "Species {} should have positive scale",
            descriptor.species
        );
        assert!(
            !descriptor.viewer_color.is_empty(),
            "Species {} should have color",
            descriptor.species
        );
        assert!(
            descriptor.viewer_order > 0,
            "Species {} should have ordering",
            descriptor.species
        );
    }

    // Test that we have the expected emoji values
    let rabbit = SPECIES_REGISTRY.rabbit();
    assert_eq!(rabbit.emoji, "🐇");

    let deer = SPECIES_REGISTRY.deer();
    assert_eq!(deer.emoji, "🦌");

    let raccoon = SPECIES_REGISTRY.raccoon();
    assert_eq!(raccoon.emoji, "🦝");
}

// Test that would require a full Bevy app - we'll simulate the key components
#[test]
fn test_spawn_using_registry_simulation() {
    // This simulates what would happen in a real Bevy app
    // We can't actually spawn entities without a full Bevy World, but we can test the lookup logic

    // Test that spawn functions exist for all registered species
    let descriptors = SPECIES_REGISTRY.get_descriptors();

    for descriptor in descriptors {
        // In a real test, we would call the spawn function
        // For now, we just verify that the registry has the expected structure
        assert!(!descriptor.species.is_empty(), "Species should have a name");
        assert!(
            !descriptor.name_prefix.is_empty(),
            "Species should have a name prefix"
        );
        assert!(!descriptor.emoji.is_empty(), "Species should have an emoji");

        // Verify movement speed is reasonable
        assert!(
            descriptor.movement_speed > 0,
            "Species should have positive movement speed"
        );
        assert!(
            descriptor.movement_speed <= 100,
            "Species movement speed should be reasonable"
        );
    }
}

#[test]
fn test_juvenile_naming_consistency() {
    // Test that species with juvenile naming have consistent prefixes
    let descriptors = SPECIES_REGISTRY.get_descriptors();

    for descriptor in descriptors {
        if let Some(juvenile_prefix) = descriptor.juvenile_name_prefix {
            // Should have different juvenile and adult names
            assert_ne!(
                descriptor.name_prefix, juvenile_prefix,
                "Species {} should have different adult and juvenile names",
                descriptor.species
            );

            // Should not be empty
            assert!(
                !juvenile_prefix.is_empty(),
                "Juvenile prefix should not be empty"
            );
        }
    }

    // Test specific known juvenile names
    let rabbit = SPECIES_REGISTRY.rabbit();
    assert_eq!(rabbit.juvenile_name_prefix, Some("Bunny"));

    let deer = SPECIES_REGISTRY.deer();
    assert_eq!(deer.juvenile_name_prefix, Some("Fawn"));

    let raccoon = SPECIES_REGISTRY.raccoon();
    assert_eq!(raccoon.juvenile_name_prefix, Some("Kit"));
}

#[test]
fn test_spawn_area_validation() {
    let config = SpawnConfig::default();

    // Validate that all spawn areas have reasonable values
    for group in &config.spawn_groups {
        assert!(
            group.spawn_area.search_radius > 0,
            "Search radius should be positive"
        );
        assert!(
            group.spawn_area.max_attempts > 0,
            "Max attempts should be positive"
        );
        assert!(group.count > 0, "Spawn count should be positive");

        // Test that coordinates are reasonable (not too far from origin for demo)
        let (center_x, center_y) = group.spawn_area.center;
        assert!(
            center_x.abs() <= 100,
            "Demo spawn X coordinate should be reasonable"
        );
        assert!(
            center_y.abs() <= 100,
            "Demo spawn Y coordinate should be reasonable"
        );
    }
}

// This would be an integration test that runs with a full Bevy app
// For now, we'll structure it as a unit test that shows the intended behavior
#[test]
fn test_demo_spawn_configuration_workflow() {
    // This test validates the complete workflow from config to spawn

    // 1. Load spawn configuration
    let config = SpawnConfig::load_or_default();
    assert!(
        config.settings.enable_spawning,
        "Demo spawning should be enabled by default"
    );

    // 2. Process spawn groups
    let mut total_entities = 0;
    for group in &config.spawn_groups {
        total_entities += group.count;

        assert!(
            !group.species.is_empty(),
            "Species name should not be empty"
        );
        assert!(!group.names.is_empty(), "Should have names for entities");

        match group.species.as_str() {
            "Rabbit" => {
                assert_eq!(group.count, 5);
                assert!(group.sex_sequence.is_none());
            }
            "Deer" | "Raccoon" => {
                assert_eq!(group.count, 2);
                let sexes = group
                    .sex_sequence
                    .as_ref()
                    .expect("Pairs should define sexes");
                assert_eq!(sexes.len(), 2);
            }
            other => panic!("Unexpected species in default config: {}", other),
        }

        assert!(
            group.spawn_area.search_radius > 0,
            "Should have positive search radius"
        );
        assert!(
            group.spawn_area.max_attempts > 0,
            "Should have positive max attempts"
        );
    }

    // 3. Verify expected total count (5 rabbits + 2 deer + 2 raccoons = 9)
    assert_eq!(
        total_entities, 9,
        "Default config should spawn 9 entities total"
    );

    // 4. Verify message templates are present
    let rabbit_group = config
        .spawn_groups
        .iter()
        .find(|g| g.species == "Rabbit")
        .expect("Should have rabbit group");

    if let Some(messages) = &rabbit_group.messages {
        assert!(
            !messages.start_message.is_empty(),
            "Should have start message"
        );
        assert!(
            !messages.success_template.is_empty(),
            "Should have success template"
        );
        assert!(
            messages.success_template.contains("{name}"),
            "Template should have name placeholder"
        );
        assert!(
            messages.success_template.contains("{pos}"),
            "Template should have position placeholder"
        );
    }

    // Verify post spawn messages exist
    assert!(
        !config.settings.post_spawn_messages.is_empty(),
        "Default config should include post-spawn messages"
    );
}

#[cfg(test)]
mod regression_tests {
    use super::*;

    #[test]
    fn test_backwards_compatibility() {
        // Ensure that old spawn function names still work through the registry

        // These would be the old direct spawn functions
        // Now they should route through the registry

        // Test that we can find expected species in registry
        assert!(SPECIES_REGISTRY.find_by_species("Rabbit").is_some());
        assert!(SPECIES_REGISTRY.find_by_species("Deer").is_some());
        assert!(SPECIES_REGISTRY.find_by_species("Raccoon").is_some());

        // Test that unknown species return None
        assert!(SPECIES_REGISTRY.find_by_species("UnknownSpecies").is_none());
    }

    #[test]
    fn test_system_registration_completeness() {
        // Ensure all species that should have systems do have them

        let expected_systems = vec!["Rabbit", "Deer", "Raccoon"];

        for species in expected_systems {
            assert!(
                SPECIES_SYSTEMS_REGISTRY.species_has_planner_system(species),
                "{} should have planner system",
                species
            );
            assert!(
                SPECIES_SYSTEMS_REGISTRY.species_has_mate_matching(species),
                "{} should have mate matching system",
                species
            );
            assert!(
                SPECIES_SYSTEMS_REGISTRY.species_has_birth_system(species),
                "{} should have birth system",
                species
            );
        }
    }
}
