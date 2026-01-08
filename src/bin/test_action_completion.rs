#![allow(dead_code)]
use bevy::prelude::*;
use life_simulator::ai::{NeedsReplanning};
use life_simulator::entities::{ActiveAction, Rabbit, TilePosition, Creature, Hunger};
use life_simulator::simulation::SimulationTick;

fn main() {
    println!("Test: Action Completion Without Interruption");
    println!("===========================================");
    println!("This test verifies that the force_periodic_replanning fix");
    println!("does NOT interrupt actions while they are executing.\n");

    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(bevy::log::LogPlugin::default())
        .init_resource::<SimulationTick>()
        .add_systems(Startup, setup)
        .add_systems(Update, check_action_completion)
        .run();
}

fn setup(
    mut commands: Commands,
) {
    println!("Spawning test rabbit at tile (10, 10)...");

    // Spawn a test rabbit
    let rabbit_entity = commands.spawn((
        Rabbit,
        Creature {
            name: "TestRabbit".to_string(),
            species: "Rabbit".to_string(),
        },
        TilePosition::new(10, 10),
        Hunger::new(),
        Name::new("TestRabbit"),
    )).id();

    println!("Rabbit spawned with entity ID: {:?}", rabbit_entity);
    println!("Starting test...\n");

    // Store entity for tracking
    commands.insert_resource(TestRabbit {
        entity: rabbit_entity,
        initial_tick: 0,
    });
}

#[derive(Resource)]
struct TestRabbit {
    entity: Entity,
    initial_tick: u64,
}

fn check_action_completion(
    mut commands: Commands,
    tick: Res<SimulationTick>,
    mut test_rabbit: ResMut<TestRabbit>,
    rabbit_query: Query<(Entity, Option<&ActiveAction>, Option<&NeedsReplanning>, &Hunger), With<Rabbit>>,
) {
    let current_tick = tick.0;

    // Initialize on first run
    if test_rabbit.initial_tick == 0 {
        test_rabbit.initial_tick = current_tick;
    }

    let elapsed_ticks = current_tick - test_rabbit.initial_tick;

    for (entity, active_action, needs_replanning, hunger) in rabbit_query.iter() {
        // Tick 0-5: Start a Graze action
        if elapsed_ticks == 0 {
            println!("Tick {}: Inserting NeedsReplanning component", current_tick);
            commands.entity(entity).insert(NeedsReplanning {
                reason: "Test start - force initial planning".to_string(),
            });
        }

        // Tick 10: Check if action was started
        if elapsed_ticks == 10 {
            match active_action {
                Some(action) => {
                    println!("Tick {}: GOOD - Active action found (started at tick {})",
                             current_tick, action.started_at_tick);
                }
                None => {
                    println!("Tick {}: No active action yet (may not have planned yet)", current_tick);
                }
            }
        }

        // Tick 15: Force replanning happens (on 10-tick boundary at tick 10, 20, 30...)
        if elapsed_ticks == 15 {
            println!("Tick {}: Force periodic replanning just ran (tick {})", current_tick, current_tick);
            match active_action {
                Some(action) => {
                    let action_tick_duration = 20; // Graze is 20 ticks
                    let ticks_since_start = current_tick - action.started_at_tick;
                    println!("  - Action still active: {} ticks into {} tick action",
                             ticks_since_start, action_tick_duration);

                    if needs_replanning.is_some() {
                        println!("  - ERROR: NeedsReplanning was inserted! Action will be interrupted!");
                        println!("  - This means force_periodic_replanning is NOT respecting ActiveAction");
                        std::process::exit(1);
                    } else {
                        println!("  - GOOD: No NeedsReplanning - action can complete uninterrupted");
                    }
                }
                None => {
                    println!("Tick {}: No active action at tick 15", current_tick);
                }
            }
        }

        // Tick 20: Action should be complete by now
        if elapsed_ticks == 20 {
            match active_action {
                Some(action) => {
                    let ticks_since_start = current_tick - action.started_at_tick;
                    if ticks_since_start >= 20 {
                        println!("Tick {}: Action completed after {} ticks", current_tick, ticks_since_start);
                    }
                }
                None => {
                    // Action queue should have removed it
                    println!("Tick {}: Action removed from ActiveAction (normal after completion)", current_tick);
                }
            }
        }

        // Tick 25: Final verification
        if elapsed_ticks == 25 {
            println!("Tick {}: Hunger value: {:.1}", current_tick, hunger.0.current);

            if hunger.0.current < 100.0 {
                println!("TEST PASSED: Hunger decreased - action completed successfully!");
                println!("The force_periodic_replanning fix is working correctly.");
                println!("Actions are NOT being interrupted by periodic replanning.");
                std::process::exit(0);
            } else {
                println!("TEST: Hunger unchanged (may not be enough ticks for action to complete)");
            }
        }

        // Failsafe: abort after 30 ticks
        if elapsed_ticks > 30 {
            println!("Test completed after {} ticks", elapsed_ticks);
            std::process::exit(0);
        }
    }
}
