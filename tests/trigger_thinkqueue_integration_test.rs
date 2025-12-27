/// Phase 2 UltraThink Integration Tests
/// Test that trigger systems properly populate ThinkQueue
use bevy::prelude::*;
use life_simulator::ai::replan_queue::ReplanQueue;
use life_simulator::ai::trigger_emitters::{IdleTracker, StatThresholdTracker};
use life_simulator::ai::ultrathink::ThinkQueue;
use life_simulator::entities::stats::{Energy, Hunger, Thirst, Stat};
use life_simulator::entities::{BehaviorConfig, FearState};
use life_simulator::simulation::{SimulationTick, TickProfiler};

#[test]
fn test_fear_trigger_schedules_urgent() {
    // Test that fear triggers schedule urgent ThinkQueue requests
    let mut app = App::new();
    app.init_resource::<ThinkQueue>();
    app.init_resource::<SimulationTick>();
    app.init_resource::<ReplanQueue>();
    app.init_resource::<TickProfiler>();

    // Spawn entity with high fear
    let _entity = app
        .world_mut()
        .spawn((
            FearState {
                fear_level: 0.5,
                nearby_predators: 2,
                ticks_since_danger: 0,
                peak_fear: 0.5,
                last_logged_fear: 0.0,
            },
            IdleTracker::new(1),
        ))
        .id();

    // Add fear trigger system
    app.add_systems(Update, life_simulator::ai::trigger_emitters::fear_trigger_system);

    // Run one update
    app.update();

    // Check queue
    let queue = app.world().resource::<ThinkQueue>();
    let (urgent, _, _) = queue.queue_sizes();
    assert!(urgent > 0, "Fear should schedule urgent request");
}

#[test]
fn test_critical_hunger_schedules_urgent() {
    // Test that critical hunger schedules urgent ThinkQueue requests
    let mut app = App::new();
    app.init_resource::<ThinkQueue>();
    app.init_resource::<SimulationTick>();
    app.init_resource::<ReplanQueue>();
    app.init_resource::<TickProfiler>();

    // Spawn entity with critical hunger (85%)
    let mut hunger = Hunger::new();
    hunger.0 = Stat::new(85.0, 0.0, 100.0, 0.1); // 85% hunger = critical

    let _entity = app
        .world_mut()
        .spawn((
            hunger,
            Thirst::new(),
            Energy::new(),
            BehaviorConfig {
                hunger_threshold: 0.5,
                thirst_threshold: 0.5,
                ..Default::default()
            },
            StatThresholdTracker::default(),
        ))
        .id();

    // Add stat threshold system
    app.add_systems(Update, life_simulator::ai::trigger_emitters::stat_threshold_system);

    // Run first update to initialize tracker
    app.update();

    // Run second update to actually trigger
    app.update();

    // Check queue
    let queue = app.world().resource::<ThinkQueue>();
    let (urgent, _, _) = queue.queue_sizes();
    assert!(urgent > 0, "Critical hunger should schedule urgent request");
}

#[test]
fn test_moderate_hunger_schedules_normal() {
    // Test that moderate hunger schedules normal ThinkQueue requests
    let mut app = App::new();
    app.init_resource::<ThinkQueue>();
    app.init_resource::<SimulationTick>();
    app.init_resource::<ReplanQueue>();
    app.init_resource::<TickProfiler>();

    // Spawn entity with moderate hunger (60%)
    let mut hunger = Hunger::new();
    hunger.0 = Stat::new(60.0, 0.0, 100.0, 0.1); // 60% hunger = moderate (50-79%)

    let _entity = app
        .world_mut()
        .spawn((
            hunger,
            Thirst::new(),
            Energy::new(),
            BehaviorConfig {
                hunger_threshold: 0.5,
                thirst_threshold: 0.5,
                ..Default::default()
            },
            StatThresholdTracker::default(),
        ))
        .id();

    // Add stat threshold system
    app.add_systems(Update, life_simulator::ai::trigger_emitters::stat_threshold_system);

    // Run first update to initialize tracker
    app.update();

    // Run second update to actually trigger
    app.update();

    // Check queue
    let queue = app.world().resource::<ThinkQueue>();
    let (_, normal, _) = queue.queue_sizes();
    assert!(normal > 0, "Moderate hunger should schedule normal request");
}

#[test]
fn test_idle_schedules_low_priority() {
    // Test that idle entities schedule low priority requests
    let mut app = App::new();
    app.init_resource::<ThinkQueue>();
    app.init_resource::<ReplanQueue>();
    app.init_resource::<TickProfiler>();
    let tick = SimulationTick(60); // Start at tick 60 (must be divisible by 20 and > 50)
    app.insert_resource(tick.clone());

    // Spawn idle entity
    let entity = app
        .world_mut()
        .spawn((
            BehaviorConfig::default(),
            IdleTracker::new(0), // Been idle since tick 0 (60 ticks idle)
        ))
        .id();

    // Manually update idle tracker
    {
        let mut query = app.world_mut().query::<&mut IdleTracker>();
        if let Ok(mut tracker) = query.get_mut(app.world_mut(), entity) {
            tracker.update_idle_time(tick.0);
        }
    }

    // Add long idle system
    app.add_systems(Update, life_simulator::ai::trigger_emitters::long_idle_system);

    // Run one update (system checks tick % 20 == 0, so tick 60 will run)
    app.update();

    // Check queue
    let queue = app.world().resource::<ThinkQueue>();
    let (_, _, low) = queue.queue_sizes();
    assert!(low > 0, "Long idle should schedule low priority request");
}
