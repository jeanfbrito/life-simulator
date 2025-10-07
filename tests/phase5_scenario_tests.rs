//! Phase 5 Scenario Tests - Updated for spawn/hunt APIs
use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use life_simulator::ai::action::{create_action, ActionResult, ActionType};
use life_simulator::entities::{
    spawn_using_registry, Carcass, Creature, Hunger, SpeciesNeeds, Stat, TilePosition,
};

#[test]
fn spawn_using_registry_creates_creature_with_position() {
    let mut world = World::new();
    let mut queue = CommandQueue::default();
    let spawn_pos = IVec2::new(5, -3);

    let entity = {
        let mut commands = Commands::new(&mut queue, &world);
        spawn_using_registry(
            &mut commands,
            "Rabbit",
            "Test Hopper".to_string(),
            spawn_pos,
        )
    };

    queue.apply(&mut world);

    let creature = world
        .get::<Creature>(entity)
        .expect("spawned creature should have Creature component");
    assert_eq!(creature.name, "Test Hopper");
    assert_eq!(creature.species, "Rabbit");

    let tile = world
        .get::<TilePosition>(entity)
        .expect("spawned creature should have TilePosition");
    assert_eq!(tile.tile, spawn_pos);
}

#[test]
fn hunt_action_consumes_prey_and_spawns_carcass() {
    let mut world = World::new();

    let predator_pos = IVec2::new(0, 1);
    let prey_pos = IVec2::new(0, 0);

    let predator = world
        .spawn((
            Creature {
                name: "Hunter".to_string(),
                species: "Wolf".to_string(),
            },
            TilePosition::from_tile(predator_pos),
            SpeciesNeeds {
                hunger_max: 120.0,
                thirst_max: 80.0,
                eat_amount: 40.0,
                drink_amount: 20.0,
            },
            Hunger(Stat::new(80.0, 0.0, 120.0, 0.0)),
        ))
        .id();

    let prey = world
        .spawn((
            Creature {
                name: "Target".to_string(),
                species: "Deer".to_string(),
            },
            TilePosition::from_tile(prey_pos),
            SpeciesNeeds {
                hunger_max: 90.0,
                thirst_max: 70.0,
                eat_amount: 30.0,
                drink_amount: 15.0,
            },
        ))
        .id();

    let mut action = create_action(ActionType::Hunt { prey });
    assert_eq!(action.name(), "Hunt");
    assert!(action.can_execute(&world, predator, 0));

    let result = action.execute(&mut world, predator, 0);
    assert_eq!(result, ActionResult::Success);

    let hunger_after = world
        .get::<Hunger>(predator)
        .expect("predator should keep Hunger component");
    assert!(
        hunger_after.0.current < 80.0,
        "predator should consume nutrition"
    );

    assert!(
        world.get_entity(prey).is_err(),
        "prey entity should be despawned after a successful hunt"
    );

    let mut carcass_query = world.query::<(&Carcass, &TilePosition)>();
    let carcasses: Vec<_> = carcass_query.iter(&world).collect();
    assert_eq!(carcasses.len(), 1, "hunt should leave a carcass behind");

    let (carcass, tile) = carcasses[0];
    assert_eq!(tile.tile, prey_pos, "carcass should spawn at prey location");
    assert!(
        (carcass.nutrition - 50.0).abs() < 0.01,
        "expected leftover nutrition"
    );
}
