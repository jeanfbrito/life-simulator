/// Domain type newtypes for Phase 9: Newtype Pattern for Type Safety
///
/// This module provides newtype wrappers around primitives to:
/// - Add compile-time type safety (can't mix Biomass with Distance)
/// - Self-document code (Utility vs f32 is much clearer)
/// - Prevent unit confusion (is this f32 grams? kilograms? percentage?)
/// - Enable domain-specific methods
///
/// # Pattern Benefits
///
/// Instead of:
/// ```rust,ignore
/// fn calculate_utility(hunger: f32, distance: f32) -> f32 {
///     hunger / distance  // Units unclear!
/// }
/// ```
///
/// We now have:
/// ```rust,ignore
/// fn calculate_utility(hunger: Hunger, distance: Distance) -> Utility {
///     Utility::new(hunger.0 / distance.0)  // Clear what each is!
/// }
/// ```

pub mod newtypes;

pub use newtypes::{Biomass, Capacity, Distance, Duration, Utility};
