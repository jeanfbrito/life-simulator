/// EMERGENCY FIX: Force periodic replanning
///
/// The trigger system is broken - entities lack the NeedsReplanning component.
/// This system forces all entities to replan every 10 ticks, bypassing
/// the broken trigger/ultrathink system entirely.
///
/// Without this fix:
/// - Entities stay Idle with 100% hunger
/// - plan_rabbit_actions, plan_deer_actions, etc. run but do nothing (0.0ms)
/// - Entities never evaluate utilities or select actions
///
/// With this fix:
/// - All entities replan every 10 ticks (once per second at 10 TPS)
/// - Utilities are evaluated and actions selected (Graze, DrinkWater, Wander)
/// - Simulation remains functional while proper trigger system is debugged

use bevy::prelude::*;
use crate::ai::NeedsReplanning;
use crate::entities::{ActiveAction, entity_types::{Rabbit, Deer, Raccoon, Bear, Fox, Wolf}};
use crate::simulation::SimulationTick;

/// CRITICAL FIX: Force periodic replanning for IDLE entities only
///
/// Previously, this system was interrupting active actions by forcing replanning
/// every 10 ticks regardless of action state. Now it only forces replanning for
/// entities that are truly idle (have no ActiveAction component).
///
/// ## The Problem (Fixed)
/// - Rabbit starts Graze action (20-tick duration)
/// - Force replanning runs every 10 ticks
/// - Force replanning queues NEW Graze action, canceling the current one
/// - Graze action NEVER completes, hunger never decreases
///
/// ## The Solution
/// - Only force replanning for entities WITHOUT ActiveAction
/// - This ensures active actions complete uninterrupted
/// - Idle entities still replan every 10 ticks
///
/// ## When This Runs
/// - Every 10 ticks (once per second at 10 TPS)
/// - Runs BEFORE event_driven_planner_system in the FixedUpdate schedule
///
/// ## What It Does
/// - Queries all entity types (Rabbit, Deer, Raccoon, Bear, Fox, Wolf)
/// - ONLY processes entities WITHOUT ActiveAction (truly idle)
/// - Inserts NeedsReplanning component for idle entities
/// - Skips entities with active actions to let them complete
///
/// ## Expected Result
/// - Graze completes after 20 ticks
/// - Hunger decreases
/// - Rabbit becomes idle
/// - Force replanning triggers for idle rabbit
/// - Cycle continues properly
pub fn force_periodic_replanning(
    mut commands: Commands,
    tick: Res<SimulationTick>,
    // Query all entity types separately, but ONLY those WITHOUT active actions
    rabbits: Query<Entity, (With<Rabbit>, Without<ActiveAction>)>,
    deer: Query<Entity, (With<Deer>, Without<ActiveAction>)>,
    raccoons: Query<Entity, (With<Raccoon>, Without<ActiveAction>)>,
    bears: Query<Entity, (With<Bear>, Without<ActiveAction>)>,
    foxes: Query<Entity, (With<Fox>, Without<ActiveAction>)>,
    wolves: Query<Entity, (With<Wolf>, Without<ActiveAction>)>,
) {
    // Force replanning every 10 ticks (once per second at 10 TPS)
    if tick.0 % 10 != 0 {
        return;
    }

    let mut forced_count = 0;

    // Force all IDLE rabbits to replan (exclude those with active actions)
    for entity in rabbits.iter() {
        commands.entity(entity).insert(NeedsReplanning {
            reason: format!("Forced periodic replan for idle entity (tick {})", tick.0),
        });
        forced_count += 1;
    }

    // Force all IDLE deer to replan
    for entity in deer.iter() {
        commands.entity(entity).insert(NeedsReplanning {
            reason: format!("Forced periodic replan for idle entity (tick {})", tick.0),
        });
        forced_count += 1;
    }

    // Force all IDLE raccoons to replan
    for entity in raccoons.iter() {
        commands.entity(entity).insert(NeedsReplanning {
            reason: format!("Forced periodic replan for idle entity (tick {})", tick.0),
        });
        forced_count += 1;
    }

    // Force all IDLE bears to replan
    for entity in bears.iter() {
        commands.entity(entity).insert(NeedsReplanning {
            reason: format!("Forced periodic replan for idle entity (tick {})", tick.0),
        });
        forced_count += 1;
    }

    // Force all IDLE foxes to replan
    for entity in foxes.iter() {
        commands.entity(entity).insert(NeedsReplanning {
            reason: format!("Forced periodic replan for idle entity (tick {})", tick.0),
        });
        forced_count += 1;
    }

    // Force all IDLE wolves to replan
    for entity in wolves.iter() {
        commands.entity(entity).insert(NeedsReplanning {
            reason: format!("Forced periodic replan for idle entity (tick {})", tick.0),
        });
        forced_count += 1;
    }

    if forced_count > 0 {
        info!("Forced {} idle entities to replan at tick {}", forced_count, tick.0);
    }
}
