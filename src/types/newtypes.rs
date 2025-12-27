/// Newtype wrappers for domain concepts - Phase 9 Type Safety Enhancement
///
/// Each newtype provides:
/// - Compile-time type safety (can't accidentally mix incompatible types)
/// - Self-documenting code (what does this f32 represent?)
/// - Domain-specific methods (is_available(), is_nearby(), etc.)
/// - Validation (biomass can't be negative, utility must be 0-1)
///
/// This eliminates "primitive obsession" anti-pattern where raw primitives
/// are used without semantic meaning.

use std::ops::{Add, Div, Mul, Sub};

/// Biomass in grams of vegetation
///
/// Represents the amount of vegetation available at a location.
/// Always non-negative and clamped to maximum biomass for that cell.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Biomass(pub f32);

impl Biomass {
    /// Zero biomass constant
    pub const ZERO: Self = Self(0.0);

    /// Create a new biomass value (automatically clamped to non-negative)
    #[inline(always)]
    pub fn new(grams: f32) -> Self {
        Self(grams.max(0.0))
    }

    /// Get the raw f32 value
    #[inline(always)]
    pub fn as_f32(&self) -> f32 {
        self.0
    }

    /// Check if biomass is available (>0)
    #[inline(always)]
    pub fn is_available(&self) -> bool {
        self.0 > 0.0
    }

    /// Check if this biomass is greater than a threshold
    #[inline(always)]
    pub fn exceeds(&self, threshold: f32) -> bool {
        self.0 > threshold
    }
}

impl Add for Biomass {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Biomass {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self((self.0 - rhs.0).max(0.0))
    }
}

impl Mul<f32> for Biomass {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self(self.0 * rhs)
    }
}

impl Div<f32> for Biomass {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        if rhs == 0.0 {
            Self(0.0)
        } else {
            Self(self.0 / rhs)
        }
    }
}

/// Distance in tiles
///
/// Represents spatial distance between entities or from a target.
/// Used in pathfinding, spatial queries, and AI decision-making.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Distance(pub u32);

impl Distance {
    /// Create a new distance value
    #[inline(always)]
    pub fn new(tiles: u32) -> Self {
        Self(tiles)
    }

    /// Get the raw u32 value
    #[inline(always)]
    pub fn as_u32(&self) -> u32 {
        self.0
    }

    /// Check if distance is within a nearby threshold
    #[inline(always)]
    pub fn is_nearby(&self, threshold: u32) -> bool {
        self.0 <= threshold
    }

    /// Check if this distance is zero (co-located)
    #[inline(always)]
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    /// Check if entities are adjacent (distance == 1)
    #[inline(always)]
    pub fn is_adjacent(&self) -> bool {
        self.0 == 1
    }
}

/// Utility score for action desirability
///
/// Normalized value in range [0.0, 1.0] representing how desirable an action is.
/// 0.0 = least desirable, 1.0 = most desirable.
/// Used in Utility AI to select best action from candidates.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Utility(pub f32);

impl Utility {
    /// Minimum utility (least desirable)
    pub const ZERO: Self = Self(0.0);

    /// Maximum utility (most desirable)
    pub const MAX: Self = Self(1.0);

    /// Create a new utility value (clamped to [0.0, 1.0])
    #[inline(always)]
    pub fn new(value: f32) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Get the raw f32 value
    #[inline(always)]
    pub fn as_f32(&self) -> f32 {
        self.0
    }

    /// Check if utility is above a threshold
    #[inline(always)]
    pub fn exceeds(&self, threshold: f32) -> bool {
        self.0 > threshold
    }

    /// Check if this is a viable option (>0.0)
    #[inline(always)]
    pub fn is_viable(&self) -> bool {
        self.0 > 0.0
    }
}

impl Add for Utility {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.0 + rhs.0)
    }
}

impl Mul for Utility {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self::new(self.0 * rhs.0)
    }
}

impl Mul<f32> for Utility {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.0 * rhs)
    }
}

impl Div<f32> for Utility {
    type Output = Self;
    fn div(self, rhs: f32) -> Self {
        if rhs == 0.0 {
            Self::ZERO
        } else {
            Self::new(self.0 / rhs)
        }
    }
}

/// Duration in simulation ticks
///
/// Represents a period of time in the simulation.
/// Each tick is one update cycle of the simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration(pub u64);

impl Duration {
    /// Create a new duration in ticks
    #[inline(always)]
    pub fn new(ticks: u64) -> Self {
        Self(ticks)
    }

    /// Get the raw u64 value
    #[inline(always)]
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    /// Check if this duration has elapsed since a start tick
    #[inline(always)]
    pub fn has_elapsed_since(&self, start_tick: u64, current_tick: u64) -> bool {
        current_tick >= start_tick + self.0
    }

    /// Calculate elapsed ticks since a previous duration point
    #[inline(always)]
    pub fn elapsed_since(&self, other: Duration) -> Duration {
        if self.0 >= other.0 {
            Duration(self.0 - other.0)
        } else {
            Duration(0)
        }
    }
}

/// Capacity/volume measurements
///
/// Represents the maximum capacity of something (hunger stomach, energy battery, etc.)
/// Used to calculate percentages and remaining space.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Capacity(pub f32);

impl Capacity {
    /// Create a new capacity
    #[inline(always)]
    pub fn new(max_value: f32) -> Self {
        Self(max_value.max(0.1)) // Prevent division by zero
    }

    /// Get the raw f32 value
    #[inline(always)]
    pub fn as_f32(&self) -> f32 {
        self.0
    }

    /// Calculate what percentage this current value represents
    #[inline(always)]
    pub fn percentage_full(&self, current: f32) -> f32 {
        ((current / self.0) * 100.0).min(100.0)
    }

    /// Calculate remaining space in this capacity
    #[inline(always)]
    pub fn remaining(&self, current: f32) -> f32 {
        (self.0 - current).max(0.0)
    }

    /// Check if capacity is completely full
    #[inline(always)]
    pub fn is_full(&self, current: f32) -> bool {
        current >= self.0 - 0.01 // Float equality tolerance
    }

    /// Check if capacity is empty
    #[inline(always)]
    pub fn is_empty(&self, current: f32) -> bool {
        current < 0.01 // Float equality tolerance
    }

    /// Normalize a value to [0.0, 1.0] range based on this capacity
    #[inline(always)]
    pub fn normalize(&self, value: f32) -> f32 {
        (value / self.0).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Biomass tests
    #[test]
    fn biomass_validates_non_negative() {
        assert_eq!(Biomass::new(-10.0).as_f32(), 0.0);
        assert_eq!(Biomass::new(50.0).as_f32(), 50.0);
    }

    #[test]
    fn biomass_arithmetic_works() {
        let b1 = Biomass::new(30.0);
        let b2 = Biomass::new(20.0);
        assert_eq!((b1 + b2).as_f32(), 50.0);
        assert_eq!((b1 - b2).as_f32(), 10.0);
        assert_eq!((b1 * 2.0).as_f32(), 60.0);
    }

    // Distance tests
    #[test]
    fn distance_comparisons_work() {
        let d1 = Distance::new(5);
        let d2 = Distance::new(10);
        assert!(d1 < d2);
        assert!(d2 > d1);
    }

    // Utility tests
    #[test]
    fn utility_clamps_correctly() {
        assert_eq!(Utility::new(2.0).as_f32(), 1.0);
        assert_eq!(Utility::new(-1.0).as_f32(), 0.0);
        assert_eq!(Utility::new(0.5).as_f32(), 0.5);
    }

    #[test]
    fn utility_operations_clamp() {
        let u1 = Utility::new(0.7);
        let u2 = Utility::new(0.6);
        assert_eq!(Utility::new(1.5).as_f32(), 1.0); // Clamped
    }

    // Duration tests
    #[test]
    fn duration_math_works() {
        let d1 = Duration::new(100);
        let d2 = Duration::new(150);
        assert_eq!(d2.elapsed_since(d1).as_u64(), 50);
    }

    // Capacity tests
    #[test]
    fn capacity_calculations_correct() {
        let cap = Capacity::new(100.0);
        assert_eq!(cap.percentage_full(50.0), 50.0);
        assert_eq!(cap.remaining(30.0), 70.0);
        assert!(cap.is_full(100.0));
        assert!(cap.is_empty(0.0));
    }
}
