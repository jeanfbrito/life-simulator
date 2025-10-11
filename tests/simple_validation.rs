//! Simple validation tests for Map Upgrade Plan core functionality

use life_simulator::resources::{ResourceType, ResourceConfig};

/// Test basic resource type functionality
#[test]
fn test_basic_resource_functionality() {
    println!("🧪 Testing basic resource functionality...");

    // Test new resource types exist and convert correctly
    let berry_bush = ResourceType::from_str("BerryBush").unwrap();
    let hazel_shrub = ResourceType::from_str("HazelShrub").unwrap();
    let mushroom_patch = ResourceType::from_str("MushroomPatch").unwrap();
    let wild_root = ResourceType::from_str("WildRoot").unwrap();

    assert_eq!(berry_bush.as_str(), "BerryBush");
    assert_eq!(hazel_shrub.as_str(), "HazelShrub");
    assert_eq!(mushroom_patch.as_str(), "MushroomPatch");
    assert_eq!(wild_root.as_str(), "WildRoot");

    // Test metadata exists
    assert!(berry_bush.get_profile().is_some());
    assert!(hazel_shrub.get_profile().is_some());
    assert!(mushroom_patch.get_profile().is_some());
    assert!(wild_root.get_profile().is_some());

    println!("✅ Basic resource functionality works");
}

/// Test default configuration
#[test]
fn test_default_configuration() {
    println!("🧪 Testing default configuration...");

    let config = ResourceConfig::default();

    // Verify densities are reasonable
    assert!(config.tree_density >= 0.0 && config.tree_density <= 0.2);
    assert!(config.berry_bush_density >= 0.0 && config.berry_bush_density <= 0.1);
    assert!(config.hazel_shrub_density >= 0.0 && config.hazel_shrub_density <= 0.1);
    assert!(config.mushroom_patch_density >= 0.0 && config.mushroom_patch_density <= 0.05);
    assert!(config.wild_root_density >= 0.0 && config.wild_root_density <= 0.05);

    // Calculate total density
    let total_density = config.tree_density + config.berry_bush_density +
                       config.hazel_shrub_density + config.mushroom_patch_density +
                       config.wild_root_density + config.rock_density +
                       config.bush_density + config.flower_density;

    println!("📊 Total resource density: {:.3}%", total_density * 100.0);
    assert!(total_density >= 0.10 && total_density <= 0.25, "Total density should be 10-25%");

    println!("✅ Default configuration is balanced");
}

/// Test resource categories and properties
#[test]
fn test_resource_categories() {
    println!("🧪 Testing resource categories...");

    // Test shrub category
    let berry_bush = ResourceType::from_str("BerryBush").unwrap();
    assert!(berry_bush.get_category().is_some());
    assert!(berry_bush.is_herbivore_edible());
    assert!(!berry_bush.is_gatherable());

    // Test collectable category
    let mushroom_patch = ResourceType::from_str("MushroomPatch").unwrap();
    assert!(mushroom_patch.get_category().is_some());
    assert!(!mushroom_patch.is_herbivore_edible());
    assert!(mushroom_patch.is_gatherable());

    // Test tree category
    let oak_tree = ResourceType::from_str("TreeOak").unwrap();
    assert!(oak_tree.get_category().is_some());
    assert!(!oak_tree.is_herbivore_edible());
    assert!(!oak_tree.is_gatherable());

    println!("✅ Resource categories are correctly assigned");
}

/// Test herbivore diet system
#[test]
fn test_herbivore_diet_system() {
    println!("🧪 Testing herbivore diet system...");

    // Test rabbit diet - rabbits prefer grass over shrubs
    let rabbit_diet = life_simulator::ai::behaviors::eating::HerbivoreDiet::rabbit();
    assert!(rabbit_diet.grass_preference > rabbit_diet.shrub_preference);
    assert_eq!(rabbit_diet.min_biomass_threshold, 8.0); // Lower threshold for small animals

    // Test deer diet - deer prefer shrubs over grass
    let deer_diet = life_simulator::ai::behaviors::eating::HerbivoreDiet::deer();
    assert!(deer_diet.shrub_preference > deer_diet.grass_preference);
    assert_eq!(deer_diet.min_biomass_threshold, 15.0); // Higher threshold for larger animals

    println!("✅ Herbivore diet system works correctly");
}

/// Test collectables API
#[test]
fn test_collectables_api() {
    println!("🧪 Testing collectables API...");

    use life_simulator::ai::collectables::is_collectable;

    // Test collectable identification
    assert!(is_collectable(&ResourceType::MushroomPatch));
    assert!(is_collectable(&ResourceType::WildRoot));
    assert!(!is_collectable(&ResourceType::BerryBush));
    assert!(!is_collectable(&ResourceType::TreeOak));

    println!("✅ Collectables API works correctly");
}

/// Main test runner
#[test]
fn run_all_simple_validations() {
    println!("🚀 Running simple validation tests...\n");

    test_basic_resource_functionality();
    test_default_configuration();
    test_resource_categories();
    test_herbivore_diet_system();
    test_collectables_api();

    println!("\n🎉 All simple validation tests passed!");
    println!("📋 Core Map Upgrade functionality is working correctly");
}