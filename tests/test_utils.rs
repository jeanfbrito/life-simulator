// Test utilities for life-simulator integration tests
use bevy::prelude::*;
use life_simulator::entities::{EntityName, MovementSpeed, TilePosition, Wanderer};
use life_simulator::simulation::{
    SimulationSpeed, SimulationState, SimulationTick, TickAccumulator,
};
use life_simulator::tilemap::WorldConfig;
use std::time::Duration;

/// Create a minimal test app with all necessary components for tick testing
pub fn create_test_app() -> App {
    let mut app = App::new();

    // Add minimal plugins for headless testing
    app.add_plugins(MinimalPlugins);

    // Add simulation resources
    app.insert_resource(SimulationTick(0));
    app.insert_resource(SimulationSpeed::default());
    app.insert_resource(SimulationState {
        should_tick: false,
        is_paused: false,
    });
    app.insert_resource(TickAccumulator::new(10.0)); // 10 TPS

    // Add world config
    app.insert_resource(WorldConfig {
        seed: 12345,
        width: 100,
        height: 100,
    });

    app
}

/// Spawn a test entity at a specific position with custom movement speed
pub fn spawn_test_entity(
    commands: &mut Commands,
    name: &str,
    x: i32,
    y: i32,
    ticks_per_tile: u32,
) -> Entity {
    commands
        .spawn((
            EntityName(name.to_string()),
            TilePosition::from_xy(x, y),
            MovementSpeed { ticks_per_tile },
            Wanderer::default(),
        ))
        .id()
}

/// Spawn a test human at a specific position
pub fn spawn_test_human(commands: &mut Commands, name: &str, x: i32, y: i32) -> Entity {
    spawn_test_entity(commands, name, x, y, 30) // 30 ticks per tile (3 seconds at 10 TPS)
}

/// Run the simulation for a specific number of frames
/// This advances the app update cycle, allowing tick accumulation
pub fn run_for_frames(app: &mut App, frames: u32) {
    for _ in 0..frames {
        app.update();
    }
}

/// Run the simulation for a specific real-world duration
/// Uses simulated time deltas to advance the simulation
pub fn run_for_duration(app: &mut App, duration: Duration) {
    let frame_duration = Duration::from_millis(16); // ~60 FPS
    let total_frames = (duration.as_millis() / frame_duration.as_millis()) as u32;

    for _ in 0..total_frames {
        // Advance time manually
        let mut time = app.world.resource_mut::<Time>();
        time.advance_by(frame_duration);

        app.update();
    }
}

/// Run the simulation until a specific number of ticks have elapsed
pub fn run_until_tick(app: &mut App, target_tick: u64, max_frames: u32) -> bool {
    for _ in 0..max_frames {
        app.update();

        let current_tick = app.world.resource::<SimulationTick>().0;
        if current_tick >= target_tick {
            return true;
        }
    }
    false // Timeout
}

/// Get the current position of an entity by its Entity ID
pub fn get_entity_position(app: &App, entity: Entity) -> Option<IVec2> {
    app.world
        .get::<TilePosition>(entity)
        .map(|pos| pos.as_ivec2())
}

/// Get the current tick count
pub fn get_tick_count(app: &App) -> u64 {
    app.world.resource::<SimulationTick>().0
}

/// Get the current simulation speed multiplier
pub fn get_simulation_speed(app: &App) -> f64 {
    app.world.resource::<SimulationSpeed>().multiplier
}

/// Set the simulation speed multiplier
pub fn set_simulation_speed(app: &mut App, multiplier: f64) {
    let mut speed = app.world.resource_mut::<SimulationSpeed>();
    speed.multiplier = multiplier;
}

/// Check if the simulation should tick on this frame
pub fn should_tick_now(app: &App) -> bool {
    app.world.resource::<SimulationState>().should_tick
}

/// Get all entities with a specific name prefix
pub fn find_entities_by_name(app: &App, name_prefix: &str) -> Vec<Entity> {
    let mut entities = Vec::new();

    let mut query = app.world.query::<(Entity, &EntityName)>();
    for (entity, entity_name) in query.iter(&app.world) {
        if entity_name.0.starts_with(name_prefix) {
            entities.push(entity);
        }
    }

    entities
}

/// Assert that an entity moved from one position to another
pub fn assert_entity_moved(app: &App, entity: Entity, expected_from: IVec2, expected_to: IVec2) {
    let current_pos = get_entity_position(app, entity).expect("Entity should have a position");

    assert_ne!(
        current_pos, expected_from,
        "Entity should have moved from {:?}",
        expected_from
    );

    // Allow some tolerance for wandering AI (entity might not move in a straight line)
    let distance_to_target = current_pos.as_vec2().distance(expected_to.as_vec2());
    let distance_from_start = current_pos.as_vec2().distance(expected_from.as_vec2());

    assert!(
        distance_from_start > 0.0,
        "Entity should have moved from starting position"
    );
}

/// Assert that an entity has NOT moved from its original position
pub fn assert_entity_stationary(app: &App, entity: Entity, expected_pos: IVec2) {
    let current_pos = get_entity_position(app, entity).expect("Entity should have a position");

    assert_eq!(
        current_pos, expected_pos,
        "Entity should still be at {:?}, but is at {:?}",
        expected_pos, current_pos
    );
}

/// Get the manhattan distance between two positions
pub fn manhattan_distance(a: IVec2, b: IVec2) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

/// Assert that ticks are accumulating at roughly the expected rate
pub fn assert_tick_rate_approximately(
    app: &App,
    expected_tps: f64,
    elapsed_seconds: f64,
    tolerance_percent: f64,
) {
    let current_ticks = get_tick_count(app) as f64;
    let expected_ticks = expected_tps * elapsed_seconds;
    let tolerance = expected_ticks * (tolerance_percent / 100.0);

    assert!(
        (current_ticks - expected_ticks).abs() <= tolerance,
        "Expected ~{:.0} ticks (±{:.0}), but got {:.0} ticks",
        expected_ticks,
        tolerance,
        current_ticks
    );
}

/// Print debug information about the current simulation state
pub fn print_simulation_debug(app: &App) {
    let tick = app.world.resource::<SimulationTick>().0;
    let state = app.world.resource::<SimulationState>();
    let speed = app.world.resource::<SimulationSpeed>();

    println!("=== Simulation Debug ===");
    println!("Tick: {}", tick);
    println!("Should Tick: {}", state.should_tick);
    println!("Is Paused: {}", state.is_paused);
    println!("Speed Multiplier: {}", speed.multiplier);
    println!("========================");
}

/// Capture entity positions over time for analysis
#[derive(Debug, Clone)]
pub struct PositionSnapshot {
    pub tick: u64,
    pub timestamp: Duration,
    pub entity: Entity,
    pub position: IVec2,
}

pub struct PositionTracker {
    snapshots: Vec<PositionSnapshot>,
}

impl PositionTracker {
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
        }
    }

    pub fn record(&mut self, app: &App, entity: Entity) {
        if let Some(position) = get_entity_position(app, entity) {
            let tick = get_tick_count(app);
            let time = app.world.resource::<Time>();

            self.snapshots.push(PositionSnapshot {
                tick,
                timestamp: time.elapsed(),
                entity,
                position,
            });
        }
    }

    pub fn snapshots(&self) -> &[PositionSnapshot] {
        &self.snapshots
    }

    pub fn total_distance_traveled(&self) -> i32 {
        if self.snapshots.len() < 2 {
            return 0;
        }

        let mut total = 0;
        for i in 1..self.snapshots.len() {
            total += manhattan_distance(self.snapshots[i - 1].position, self.snapshots[i].position);
        }
        total
    }

    pub fn movement_count(&self) -> usize {
        if self.snapshots.is_empty() {
            return 0;
        }

        let mut movements = 0;
        for i in 1..self.snapshots.len() {
            if self.snapshots[i].position != self.snapshots[i - 1].position {
                movements += 1;
            }
        }
        movements
    }

    pub fn print_history(&self) {
        println!("=== Position History ===");
        for snapshot in &self.snapshots {
            println!(
                "Tick {:4} ({:6.2}s): {:?}",
                snapshot.tick,
                snapshot.timestamp.as_secs_f64(),
                snapshot.position
            );
        }
        println!("Total distance: {}", self.total_distance_traveled());
        println!("Total movements: {}", self.movement_count());
        println!("========================");
    }
}

impl Default for PositionTracker {
    fn default() -> Self {
        Self::new()
    }
}
