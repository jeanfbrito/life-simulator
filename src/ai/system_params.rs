//! System Parameter Bundles for AI Planning Systems
//!
//! This module provides reusable `SystemParam` bundles that reduce boilerplate
//! and improve code organization in planning systems.
//!
//! # Overview
//!
//! Planning systems traditionally require many individual parameters, making function
//! signatures hard to read and test. This module bundles commonly-used resources
//! into logical groups that can be passed as single parameters.
//!
//! # Usage Example
//!
//! ```rust,no_run
//! use crate::ai::system_params::PlanningResources;
//!
//! fn plan_herbivore_actions(
//!     mut commands: Commands,
//!     mut queue: ResMut<ActionQueue>,
//!     query: Query<(Entity, &TilePosition, &Hunger, &Thirst, &Energy)>,
//!     resources: PlanningResources,  // ✅ Bundled instead of 3+ individual params
//! ) {
//!     let world_loader = resources.world_loader;
//!     let vegetation_grid = resources.vegetation_grid;
//!     // Use resources...
//! }
//! ```

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use crate::vegetation::resource_grid::ResourceGrid;
use crate::world_loader::WorldLoader;
use crate::simulation::SimulationTick;

/// Bundle of world context resources used in AI planning systems.
///
/// This `SystemParam` groups resources that are commonly needed together
/// when planning entity actions. It provides read-only access to world
/// configuration and state.
///
/// # Contained Resources
/// - `world_loader`: Terrain and map data
/// - `vegetation_grid`: Vegetation biomass and resource locations
/// - `tick`: Current simulation tick (for logging, scheduling)
///
/// # Benefits
/// - Reduces function parameter count from 8+ to 2 (query + this bundle)
/// - Easier to test with mock resources
/// - Clear documentation of what data planning needs
/// - Future-proof: add new resources without changing function signatures
#[derive(SystemParam)]
pub struct PlanningResources<'w> {
    pub world_loader: Res<'w, WorldLoader>,
    pub vegetation_grid: Res<'w, ResourceGrid>,
    pub tick: Res<'w, SimulationTick>,
}

impl<'w> PlanningResources<'w> {
    /// Get the current simulation tick number
    #[inline]
    pub fn current_tick(&self) -> u64 {
        self.tick.0
    }

    /// Check if we should log diagnostics (every N ticks to avoid spam)
    #[inline]
    pub fn should_log_diagnostics(&self, interval: u64) -> bool {
        self.tick.0 % interval == 0
    }
}
