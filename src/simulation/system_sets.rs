/// System Sets for Simulation Execution
///
/// Organizes simulation systems into logical execution phases with clear ordering.
/// Systems within the same set can run in parallel, while sets execute sequentially.
///
/// # Execution Order
/// 1. Planning - AI decision making (parallel across species)
/// 2. ActionExecution - Execute queued actions (single-threaded, World access)
/// 3. Movement - Execute movement orders (parallel)
/// 4. Stats - Update stats, auto-eat (parallel)
/// 5. Reproduction - Mate matching, births (parallel)
/// 6. Cleanup - Death, carcass decay (must run last)

use bevy::prelude::*;

/// System sets that define execution order for simulation systems
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimulationSet {
    /// Planning phase: AI systems decide actions
    /// - All species planning systems (rabbits, deer, foxes, etc.)
    /// - Can run in parallel (each species independent)
    Planning,

    /// Action execution phase: Execute queued actions
    /// - execute_queued_actions (single-threaded, needs World access)
    /// - Must run after Planning, before Movement
    ActionExecution,

    /// Movement phase: Execute movement orders
    /// - execute_movement_component (parallel)
    /// - tick_movement_system (legacy, parallel)
    /// - Must run after ActionExecution
    Movement,

    /// Stats phase: Update entity statistics
    /// - tick_stats_system (hunger, thirst, energy decay)
    /// - Can run in parallel
    Stats,

    /// Reproduction phase: Mate matching and births
    /// - All mate_matching systems (parallel)
    /// - All birth systems (parallel)
    /// - update_age_and_wellfed_system
    /// - tick_reproduction_timers_system
    /// - Can run in parallel with Stats
    Reproduction,

    /// Cleanup phase: Entity cleanup and world maintenance
    /// - death_system (remove dead entities)
    /// - tick_carcasses (decay carcasses)
    /// - Must run last to ensure all other systems see alive entities
    Cleanup,
}
