/// System Sets Test - Phase 6: System Organization and Parallelism
///
/// Tests for proper system set organization with clear execution ordering.
/// Systems should be grouped into logical sets with explicit dependencies.

use bevy::prelude::*;
use life_simulator::simulation::SimulationSet;

#[test]
fn test_simulation_set_exists() {
    // Verify SimulationSet enum exists and has expected variants
    let _planning = SimulationSet::Planning;
    let _action_execution = SimulationSet::ActionExecution;
    let _movement = SimulationSet::Movement;
    let _stats = SimulationSet::Stats;
    let _reproduction = SimulationSet::Reproduction;
    let _cleanup = SimulationSet::Cleanup;
}

#[test]
fn test_simulation_set_derives_system_set() {
    // Verify SimulationSet implements required traits
    let set1 = SimulationSet::Planning;
    let set2 = SimulationSet::Planning;
    let set3 = SimulationSet::ActionExecution;

    // Should implement Debug
    assert_eq!(format!("{:?}", set1), "Planning");

    // Should implement Clone
    let _cloned = set1.clone();

    // Should implement PartialEq
    assert_eq!(set1, set2);
    assert_ne!(set1, set3);

    // Should implement Hash (can be used in HashSet)
    let mut set = std::collections::HashSet::new();
    set.insert(set1);
    assert!(set.contains(&set2));
}

#[test]
fn test_system_set_ordering() {
    // Create a minimal app to test system ordering
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Add a simple condition resource
    #[derive(Resource, Default)]
    struct TestCondition(bool);
    app.insert_resource(TestCondition(true));

    // Create test systems that track execution order
    #[derive(Resource, Default)]
    struct ExecutionLog(Vec<String>);
    app.insert_resource(ExecutionLog(vec![]));

    fn planning_system(mut log: ResMut<ExecutionLog>) {
        log.0.push("Planning".to_string());
    }

    fn action_system(mut log: ResMut<ExecutionLog>) {
        log.0.push("ActionExecution".to_string());
    }

    fn movement_system(mut log: ResMut<ExecutionLog>) {
        log.0.push("Movement".to_string());
    }

    fn stats_system(mut log: ResMut<ExecutionLog>) {
        log.0.push("Stats".to_string());
    }

    fn reproduction_system(mut log: ResMut<ExecutionLog>) {
        log.0.push("Reproduction".to_string());
    }

    fn cleanup_system(mut log: ResMut<ExecutionLog>) {
        log.0.push("Cleanup".to_string());
    }

    // Add systems with proper ordering
    app.add_systems(Update, planning_system.in_set(SimulationSet::Planning));
    app.add_systems(Update, action_system.in_set(SimulationSet::ActionExecution).after(SimulationSet::Planning));
    app.add_systems(Update, movement_system.in_set(SimulationSet::Movement).after(SimulationSet::ActionExecution));
    app.add_systems(Update, stats_system.in_set(SimulationSet::Stats).after(SimulationSet::Movement));
    app.add_systems(Update, reproduction_system.in_set(SimulationSet::Reproduction).after(SimulationSet::Movement));
    app.add_systems(Update, cleanup_system.in_set(SimulationSet::Cleanup).after(SimulationSet::Stats).after(SimulationSet::Reproduction));

    // Run one update cycle
    app.update();

    // Verify execution order
    let log = app.world().resource::<ExecutionLog>();

    // Planning should run first
    assert_eq!(log.0[0], "Planning");

    // ActionExecution should run after Planning
    let planning_idx = log.0.iter().position(|s| s == "Planning").unwrap();
    let action_idx = log.0.iter().position(|s| s == "ActionExecution").unwrap();
    assert!(action_idx > planning_idx, "ActionExecution should run after Planning");

    // Movement should run after ActionExecution
    let movement_idx = log.0.iter().position(|s| s == "Movement").unwrap();
    assert!(movement_idx > action_idx, "Movement should run after ActionExecution");

    // Stats and Reproduction should run after Movement
    let stats_idx = log.0.iter().position(|s| s == "Stats").unwrap();
    let repro_idx = log.0.iter().position(|s| s == "Reproduction").unwrap();
    assert!(stats_idx > movement_idx, "Stats should run after Movement");
    assert!(repro_idx > movement_idx, "Reproduction should run after Movement");

    // Cleanup should run last (after both Stats and Reproduction)
    let cleanup_idx = log.0.iter().position(|s| s == "Cleanup").unwrap();
    assert!(cleanup_idx > stats_idx, "Cleanup should run after Stats");
    assert!(cleanup_idx > repro_idx, "Cleanup should run after Reproduction");
}

#[test]
fn test_parallel_execution_within_set() {
    // Test that multiple systems in the same set can run (in parallel)
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    #[derive(Resource, Default)]
    struct Counter(i32);
    app.insert_resource(Counter(0));

    fn planning_system_1(mut counter: ResMut<Counter>) {
        counter.0 += 1;
    }

    fn planning_system_2(mut counter: ResMut<Counter>) {
        counter.0 += 1;
    }

    fn planning_system_3(mut counter: ResMut<Counter>) {
        counter.0 += 1;
    }

    // Add multiple systems to the same set
    app.add_systems(Update, (
        planning_system_1,
        planning_system_2,
        planning_system_3,
    ).in_set(SimulationSet::Planning));

    // Run one update
    app.update();

    // All three systems should have executed
    let counter = app.world().resource::<Counter>();
    assert_eq!(counter.0, 3, "All systems in Planning set should execute");
}

#[test]
fn test_system_set_with_run_condition() {
    // Test that system sets work with run conditions
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    #[derive(Resource, Default)]
    struct ShouldRun(bool);
    app.insert_resource(ShouldRun(false));

    #[derive(Resource, Default)]
    struct ExecutionCount(i32);
    app.insert_resource(ExecutionCount(0));

    fn test_system(mut count: ResMut<ExecutionCount>) {
        count.0 += 1;
    }

    fn should_run_condition(should_run: Res<ShouldRun>) -> bool {
        should_run.0
    }

    // Add system with run condition
    app.add_systems(Update, test_system.in_set(SimulationSet::Planning).run_if(should_run_condition));

    // First update - should NOT run
    app.update();
    let count = app.world().resource::<ExecutionCount>();
    assert_eq!(count.0, 0, "System should not run when condition is false");

    // Enable condition
    app.world_mut().resource_mut::<ShouldRun>().0 = true;

    // Second update - should run
    app.update();
    let count = app.world().resource::<ExecutionCount>();
    assert_eq!(count.0, 1, "System should run when condition is true");
}

#[test]
fn test_all_simulation_sets_exist() {
    // Verify we have all expected sets
    use std::collections::HashSet;

    let sets = vec![
        SimulationSet::Planning,
        SimulationSet::ActionExecution,
        SimulationSet::Movement,
        SimulationSet::Stats,
        SimulationSet::Reproduction,
        SimulationSet::Cleanup,
    ];

    // Should have 6 unique sets
    let unique_sets: HashSet<_> = sets.into_iter().collect();
    assert_eq!(unique_sets.len(), 6, "Should have 6 unique simulation sets");
}
