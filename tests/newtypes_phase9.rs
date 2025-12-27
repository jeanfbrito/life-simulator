/// Phase 9: Newtype Pattern Tests
/// Tests for domain type newtypes to validate type safety and self-documenting behavior
use life_simulator::types::newtypes::{Biomass, Distance, Utility, Duration, Capacity};

#[test]
fn test_biomass_creation() {
    let bio = Biomass::new(50.0);
    assert_eq!(bio.as_f32(), 50.0);
}

#[test]
fn test_biomass_never_negative() {
    let bio = Biomass::new(-10.0);
    assert_eq!(bio.as_f32(), 0.0);
}

#[test]
fn test_biomass_zero_constant() {
    assert_eq!(Biomass::ZERO.as_f32(), 0.0);
    assert!(!Biomass::ZERO.is_available());
}

#[test]
fn test_biomass_is_available() {
    assert!(Biomass::new(0.1).is_available());
    assert!(!Biomass::new(0.0).is_available());
}

#[test]
fn test_biomass_addition() {
    let bio1 = Biomass::new(30.0);
    let bio2 = Biomass::new(20.0);
    let result = bio1 + bio2;
    assert_eq!(result.as_f32(), 50.0);
}

#[test]
fn test_biomass_subtraction() {
    let bio1 = Biomass::new(50.0);
    let bio2 = Biomass::new(20.0);
    let result = bio1 - bio2;
    assert_eq!(result.as_f32(), 30.0);
}

#[test]
fn test_biomass_subtraction_clamps_to_zero() {
    let bio1 = Biomass::new(10.0);
    let bio2 = Biomass::new(20.0);
    let result = bio1 - bio2;
    assert_eq!(result.as_f32(), 0.0);
}

#[test]
fn test_biomass_multiplication() {
    let bio = Biomass::new(50.0);
    let result = bio * 2.0;
    assert_eq!(result.as_f32(), 100.0);
}

#[test]
fn test_distance_creation() {
    let dist = Distance::new(5);
    assert_eq!(dist.as_u32(), 5);
}

#[test]
fn test_distance_comparison() {
    let dist1 = Distance::new(5);
    let dist2 = Distance::new(10);
    assert!(dist1 < dist2);
    assert!(dist2 > dist1);
}

#[test]
fn test_distance_is_nearby() {
    let dist = Distance::new(3);
    assert!(dist.is_nearby(5));
    assert!(!dist.is_nearby(2));
}

#[test]
fn test_utility_creation() {
    let util = Utility::new(0.5);
    assert_eq!(util.as_f32(), 0.5);
}

#[test]
fn test_utility_clamps_to_valid_range() {
    let util_too_high = Utility::new(2.0);
    assert_eq!(util_too_high.as_f32(), 1.0);

    let util_negative = Utility::new(-0.5);
    assert_eq!(util_negative.as_f32(), 0.0);
}

#[test]
fn test_utility_zero_constant() {
    assert_eq!(Utility::ZERO.as_f32(), 0.0);
}

#[test]
fn test_utility_max_constant() {
    assert_eq!(Utility::MAX.as_f32(), 1.0);
}

#[test]
fn test_utility_multiplication() {
    let util1 = Utility::new(0.8);
    let util2 = Utility::new(0.5);
    let result = util1 * util2;
    assert_eq!(result.as_f32(), 0.4);
}

#[test]
fn test_utility_addition() {
    let util1 = Utility::new(0.3);
    let util2 = Utility::new(0.5);
    let result = util1 + util2;
    assert_eq!(result.as_f32(), 0.8);
}

#[test]
fn test_utility_addition_clamps() {
    let util1 = Utility::new(0.7);
    let util2 = Utility::new(0.6);
    let result = util1 + util2;
    assert_eq!(result.as_f32(), 1.0); // Clamped
}

#[test]
fn test_duration_creation() {
    let dur = Duration::new(100);
    assert_eq!(dur.as_u64(), 100);
}

#[test]
fn test_duration_elapsed_since() {
    let dur1 = Duration::new(100);
    let dur2 = Duration::new(150);
    // elapsed_since calculates dur2 - dur1
    assert_eq!(dur2.elapsed_since(dur1).as_u64(), 50);
}

#[test]
fn test_duration_has_elapsed_since() {
    let duration = Duration::new(10);
    let start_tick = 100u64;
    assert!(duration.has_elapsed_since(start_tick, 110));
    assert!(!duration.has_elapsed_since(start_tick, 105));
}

#[test]
fn test_capacity_creation() {
    let cap = Capacity::new(100.0);
    assert_eq!(cap.as_f32(), 100.0);
}

#[test]
fn test_capacity_percentage() {
    let cap = Capacity::new(100.0);
    let current = 30.0;
    // Allow for floating point precision
    assert!((cap.percentage_full(current) - 30.0).abs() < 0.01);
}

#[test]
fn test_capacity_remaining() {
    let cap = Capacity::new(100.0);
    let current = 30.0;
    assert_eq!(cap.remaining(current), 70.0);
}

#[test]
fn test_capacity_is_full() {
    let cap = Capacity::new(100.0);
    assert!(cap.is_full(100.0));
    assert!(!cap.is_full(50.0));
}

#[test]
fn test_capacity_is_empty() {
    let cap = Capacity::new(100.0);
    assert!(cap.is_empty(0.0));
    assert!(!cap.is_empty(0.1));
}
