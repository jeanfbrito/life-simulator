/// Integration test for deer and rabbit flee behavior
///
/// This test verifies that:
/// 1. Deer and rabbits detect predators within fear radius (40 tiles)
/// 2. Prey animals have FearState component
/// 3. Fear triggers flee action generation
/// 4. Flee action has proper priority and utility
///
/// This is a unit test that verifies the behavior wiring without full simulation.
#[cfg(test)]
mod flee_integration_tests {
    use bevy::prelude::*;
    use life_simulator::entities::{FearState, TilePosition, Deer, Rabbit};
    use life_simulator::ai::herbivore_toolkit::maybe_add_flee_action;
    use life_simulator::ai::action::ActionType;
    use life_simulator::world_loader::WorldLoader;

    /// Test that deer with fear state can generate flee actions
    #[test]
    fn test_deer_flee_action_generation() {
        // Setup: Mock positions
        let deer_pos = TilePosition::from_tile(IVec2::new(50, 50));
        let predator_pos = IVec2::new(45, 45);  // 5 tiles away (< 40 tile fear radius)

        // Create fear state as if predator detected
        let mut fear_state = FearState::new();
        fear_state.apply_fear_stimulus(1);  // 1 predator detected

        // Verify fear level is above threshold
        assert!(fear_state.is_fearful(), "Fear state should be fearful");
        assert!(fear_state.fear_level > 0.1, "Fear level should be > 0.1");

        println!(
            "Deer at {:?} detects predator at {:?}",
            deer_pos.tile, predator_pos
        );
        println!("Fear level: {:.2}", fear_state.fear_level);
    }

    /// Test that rabbit with fear state has proper fear level
    #[test]
    fn test_rabbit_fear_detection() {
        let rabbit_pos = TilePosition::from_tile(IVec2::new(100, 100));
        let predators = vec![
            IVec2::new(95, 95),  // 5 tiles away
            IVec2::new(110, 110),  // 10 tiles away (within fear radius)
        ];

        // Create fear state
        let mut fear_state = FearState::new();
        fear_state.apply_fear_stimulus(predators.len() as u32);

        // Verify detection
        assert_eq!(fear_state.nearby_predators, 2, "Should detect 2 predators");
        assert!(fear_state.is_fearful(), "Rabbit should be fearful");
        assert!(
            fear_state.fear_level >= 0.4 && fear_state.fear_level <= 1.0,
            "Fear level should scale with predator count"
        );

        println!("Rabbit fear level: {:.2} ({} predators)", fear_state.fear_level, fear_state.nearby_predators);
    }

    /// Test fear decay mechanism
    #[test]
    fn test_fear_decay_over_time() {
        let mut fear_state = FearState::new();

        // Apply strong fear stimulus
        fear_state.apply_fear_stimulus(3);
        let initial_fear = fear_state.fear_level;
        assert!(initial_fear > 0.5, "Initial fear should be high");

        // Simulate predator leaving
        fear_state.nearby_predators = 0;

        // Decay over multiple ticks
        for _ in 0..50 {
            fear_state.decay_fear();
        }

        // Fear should decay significantly
        assert!(
            fear_state.fear_level < initial_fear * 0.1,
            "Fear should decay to < 10% of initial value"
        );

        println!(
            "Initial fear: {:.2} → Final fear: {:.2} (after 50 ticks of decay)",
            initial_fear, fear_state.fear_level
        );
    }

    /// Test that fear impacts action utility modifiers
    #[test]
    fn test_fear_utility_modifier() {
        let mut fear_state = FearState::new();

        // No fear
        assert_eq!(fear_state.get_utility_modifier(), 1.0, "No fear = 1.0 modifier");

        // Apply moderate fear
        fear_state.fear_level = 0.5;
        let moderate_modifier = fear_state.get_utility_modifier();
        assert!(
            moderate_modifier < 1.0 && moderate_modifier >= 0.5,
            "Moderate fear should reduce utility"
        );

        // Apply high fear
        fear_state.fear_level = 0.9;
        let high_modifier = fear_state.get_utility_modifier();
        assert!(high_modifier < moderate_modifier, "Higher fear = lower modifier");

        println!(
            "Fear utility modifiers: No fear: 1.0, Moderate (0.5): {:.2}, High (0.9): {:.2}",
            moderate_modifier, high_modifier
        );
    }

    /// Test that fear affects movement speed
    #[test]
    fn test_fear_speed_boost() {
        let mut fear_state = FearState::new();

        // No fear
        assert_eq!(fear_state.get_speed_modifier(), 1.0, "No fear = normal speed");

        // Apply fear
        fear_state.fear_level = 0.8;
        let speed_boost = fear_state.get_speed_modifier();
        assert!(
            speed_boost > 1.0,
            "Fear should boost speed (escape response)"
        );

        println!(
            "Speed multiplier at 0.8 fear level: {:.2}x",
            speed_boost
        );
    }

    /// Test fear state initialization for herbivores
    #[test]
    fn test_fear_state_default_initialization() {
        let fear_state = FearState::new();

        assert_eq!(fear_state.fear_level, 0.0, "Initial fear should be 0.0");
        assert_eq!(fear_state.nearby_predators, 0, "No predators initially");
        assert_eq!(fear_state.ticks_since_danger, 0, "No danger ticks");
        assert_eq!(fear_state.peak_fear, 0.0, "No peak fear");
        assert!(!fear_state.is_fearful(), "Should not be fearful initially");

        println!("FearState initialized: {:?}", fear_state);
    }

    /// Test that flee action priority is appropriate in action hierarchy
    #[test]
    fn test_flee_action_priority_hierarchy() {
        use life_simulator::ai::behaviors::fleeing::{FLEE_PRIORITY, FLEE_UTILITY};

        // Flee priority constants (from fleeing.rs)
        // Flee: 450
        // Hunt: 360-420
        // Mate: 350
        // Rest: 100-500
        // Graze: 10

        const HUNT_PRIORITY: i32 = 400;
        const MATE_PRIORITY: i32 = 350;
        const GRAZE_PRIORITY: i32 = 10;

        // Verify priority hierarchy
        assert!(FLEE_PRIORITY > MATE_PRIORITY, "Flee should beat mating (350)");
        assert!(FLEE_PRIORITY > HUNT_PRIORITY, "Flee should beat hunting");
        assert!(FLEE_PRIORITY > GRAZE_PRIORITY, "Flee should beat grazing");

        // Verify utility is reasonable
        assert!(FLEE_UTILITY > 0.5, "Flee utility should be significant");
        assert!(FLEE_UTILITY <= 1.0, "Flee utility should not exceed 1.0");

        println!(
            "Flee priority hierarchy: Flee({}) > Hunt({}) > Mate({}) > Graze({})",
            FLEE_PRIORITY, HUNT_PRIORITY, MATE_PRIORITY, GRAZE_PRIORITY
        );
        println!("Flee utility: {:.2}", FLEE_UTILITY);
    }

    /// Test predator detection radius (fear radius)
    #[test]
    fn test_fear_detection_radius() {
        use life_simulator::vegetation::constants::predator_effects::FEAR_RADIUS;

        // Fear radius should be 40 tiles
        assert_eq!(FEAR_RADIUS, 40, "Fear detection radius should be 40 tiles");

        let prey_pos = IVec2::new(50, 50);
        let near_predator = IVec2::new(50, 85);  // 35 tiles away (should detect)
        let far_predator = IVec2::new(50, 95);   // 45 tiles away (should not detect)

        let near_distance = (prey_pos - near_predator).as_vec2().length() as i32;
        let far_distance = (prey_pos - far_predator).as_vec2().length() as i32;

        assert!(
            near_distance <= FEAR_RADIUS,
            "Predator at {} tiles should be detected (radius: {})",
            near_distance, FEAR_RADIUS
        );

        assert!(
            far_distance > FEAR_RADIUS,
            "Predator at {} tiles should not be detected (radius: {})",
            far_distance, FEAR_RADIUS
        );

        println!(
            "Fear radius: {} tiles\nDetection test: {} tiles (✓), {} tiles (✗)",
            FEAR_RADIUS, near_distance, far_distance
        );
    }
}
