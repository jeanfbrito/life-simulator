/// Event-Driven Communication System
///
/// Replaces polling-based systems with reactive event architecture.
/// Benefits:
/// - Systems only run when events occur (no polling every tick)
/// - Decoupled producers and consumers
/// - Efficient change detection
/// - Better debugging visibility
///
/// Event Types:
/// - EntityDied: Emitted when entity health reaches zero
/// - ActionCompleted: Emitted when actions finish
/// - PathCompleted: Emitted when movement finishes
/// - StatCritical: Emitted when stats reach critical thresholds
use bevy::prelude::*;

use crate::ai::ActionType;
use crate::entities::{Carcass, Creature, Health, Hunger, Thirst, TilePosition};

// ============================================================================
// EVENT TYPE DEFINITIONS
// ============================================================================

/// Death cause enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityDied {
    Starvation,
    Dehydration,
    PredatorAttack,
    Environmental,
    Unknown,
}

/// Event: Entity died
#[derive(Event, Debug, Clone)]
pub struct EntityDiedEvent {
    pub entity: Entity,
    pub cause: EntityDied,
}

/// Event: Action completed
#[derive(Event, Debug, Clone)]
pub struct ActionCompletedEvent {
    pub entity: Entity,
    pub action_type: ActionType,
    pub success: bool,
}

/// Event: Path completed
#[derive(Event, Debug, Clone)]
pub struct PathCompletedEvent {
    pub entity: Entity,
    pub destination: IVec2,
    pub success: bool,
}

/// Stat type enumeration for critical events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatCritical {
    Hunger,
    Thirst,
    Health,
    Energy,
}

/// Event: Stat reached critical threshold
#[derive(Event, Debug, Clone)]
pub struct StatCriticalEvent {
    pub entity: Entity,
    pub stat_type: StatCritical,
    pub value: f32, // Normalized value (0.0-1.0)
}

// ============================================================================
// MARKER COMPONENTS (for tracking state transitions)
// ============================================================================

/// Marker component: Action just completed (insert to trigger event)
#[derive(Component, Debug, Clone)]
pub struct ActionJustCompleted {
    pub action_type: ActionType,
    pub success: bool,
}

/// Marker component: Path just completed (insert to trigger event)
#[derive(Component, Debug, Clone)]
pub struct PathJustCompleted {
    pub destination: IVec2,
    pub success: bool,
}

// ============================================================================
// PRODUCER SYSTEMS (detect changes and emit events)
// ============================================================================

/// Producer: Detect entity death and emit EntityDied event
///
/// Uses Changed<Health> query to only check entities whose health changed.
/// This is more efficient than polling all entities every tick.
pub fn detect_entity_death(
    query: Query<(Entity, &Health, Option<&Hunger>, Option<&Thirst>), Changed<Health>>,
    mut events: EventWriter<EntityDiedEvent>,
) {
    for (entity, health, hunger, thirst) in query.iter() {
        if health.0.current <= 0.0 {
            // Determine death cause based on stats
            let cause = if let Some(hunger) = hunger {
                if hunger.0.normalized() >= 0.9 {
                    EntityDied::Starvation
                } else if let Some(thirst) = thirst {
                    if thirst.0.normalized() >= 0.9 {
                        EntityDied::Dehydration
                    } else {
                        EntityDied::Unknown
                    }
                } else {
                    EntityDied::Unknown
                }
            } else {
                EntityDied::Unknown
            };

            events.send(EntityDiedEvent { entity, cause });

            debug!(
                "💀 EntityDied event: entity={:?}, cause={:?}",
                entity, cause
            );
        }
    }
}

/// Producer: Detect action completion and emit ActionCompleted event
///
/// Entities with ActionJustCompleted component trigger event emission.
/// The marker component is removed after event is sent.
pub fn detect_action_completion(
    query: Query<(Entity, &ActionJustCompleted)>,
    mut events: EventWriter<ActionCompletedEvent>,
    mut commands: Commands,
) {
    for (entity, completed) in query.iter() {
        events.send(ActionCompletedEvent {
            entity,
            action_type: completed.action_type.clone(),
            success: completed.success,
        });

        debug!(
            "✅ ActionCompleted event: entity={:?}, action={:?}, success={}",
            entity, completed.action_type, completed.success
        );

        // Remove marker component after processing
        commands.entity(entity).remove::<ActionJustCompleted>();
    }
}

/// Producer: Detect path completion and emit PathCompleted event
///
/// Entities with PathJustCompleted component trigger event emission.
pub fn detect_path_completion(
    query: Query<(Entity, &PathJustCompleted)>,
    mut events: EventWriter<PathCompletedEvent>,
    mut commands: Commands,
) {
    for (entity, completed) in query.iter() {
        events.send(PathCompletedEvent {
            entity,
            destination: completed.destination,
            success: completed.success,
        });

        debug!(
            "🎯 PathCompleted event: entity={:?}, destination={:?}, success={}",
            entity, completed.destination, completed.success
        );

        // Remove marker component after processing
        commands.entity(entity).remove::<PathJustCompleted>();
    }
}

/// Producer: Detect critical stats and emit StatCritical event
///
/// Checks hunger, thirst, health, and energy for critical levels.
/// Critical threshold: >= 90% for needs, <= 10% for resources.
/// Uses Changed queries to avoid polling every tick.
pub fn detect_stat_critical(
    hunger_query: Query<(Entity, &Hunger), Changed<Hunger>>,
    thirst_query: Query<(Entity, &Thirst), Changed<Thirst>>,
    mut events: EventWriter<StatCriticalEvent>,
) {
    // Check hunger
    for (entity, hunger) in hunger_query.iter() {
        let hunger_norm = hunger.0.normalized();
        if hunger_norm >= 0.9 {
            events.send(StatCriticalEvent {
                entity,
                stat_type: StatCritical::Hunger,
                value: hunger_norm,
            });

            debug!(
                "🚨 StatCritical event: entity={:?}, stat=Hunger, value={:.2}",
                entity, hunger_norm
            );
        }
    }

    // Check thirst
    for (entity, thirst) in thirst_query.iter() {
        let thirst_norm = thirst.0.normalized();
        if thirst_norm >= 0.9 {
            events.send(StatCriticalEvent {
                entity,
                stat_type: StatCritical::Thirst,
                value: thirst_norm,
            });

            debug!(
                "🚨 StatCritical event: entity={:?}, stat=Thirst, value={:.2}",
                entity, thirst_norm
            );
        }
    }
}

// ============================================================================
// CONSUMER SYSTEMS (react to events)
// ============================================================================

/// Consumer: Handle entity death events
///
/// Spawns carcass and despawns entity when death event is received.
/// This replaces the polling-based death_system.
pub fn handle_entity_death(
    mut events: EventReader<EntityDiedEvent>,
    mut commands: Commands,
    query: Query<(Option<&TilePosition>, Option<&Creature>)>,
) {
    for event in events.read() {
        debug!(
            "💀 Handling death: entity={:?}, cause={:?}",
            event.entity, event.cause
        );

        // Get entity data before despawning
        if let Ok((tile_pos, creature)) = query.get(event.entity) {
            // Spawn carcass at entity's position
            if let Some(pos) = tile_pos {
                let (species_name, carcass_nutrition) = if let Some(creature) = creature {
                    // Base nutrition on creature type
                    let nutrition = match creature.species.as_str() {
                        "Rabbit" => 40.0,
                        "Deer" => 80.0,
                        "Raccoon" => 50.0,
                        "Bear" => 120.0,
                        "Fox" => 60.0,
                        "Wolf" => 100.0,
                        _ => 30.0,
                    };
                    (creature.species.clone(), nutrition)
                } else {
                    ("Unknown".to_string(), 30.0)
                };

                // Carcass::new(species, nutrition, decay_ticks)
                commands.spawn((
                    Carcass::new(species_name.clone(), carcass_nutrition, 6000),
                    TilePosition::new(pos.tile.x, pos.tile.y),
                ));

                debug!(
                    "🥩 Spawned {} carcass at {:?} with {} nutrition",
                    species_name, pos.tile, carcass_nutrition
                );
            }
        }

        // Despawn the entity
        commands.entity(event.entity).despawn();
    }
}

// ============================================================================
// PLUGIN
// ============================================================================

/// EventSystemPlugin: Registers all events and systems
pub struct EventSystemPlugin;

impl Plugin for EventSystemPlugin {
    fn build(&self, app: &mut App) {
        // Register events
        app.add_event::<EntityDiedEvent>();
        app.add_event::<ActionCompletedEvent>();
        app.add_event::<PathCompletedEvent>();
        app.add_event::<StatCriticalEvent>();

        // Add producer systems (detect changes and emit events)
        app.add_systems(
            Update,
            (
                detect_entity_death,
                detect_action_completion,
                detect_path_completion,
                detect_stat_critical,
            ),
        );

        // Add consumer systems (react to events)
        app.add_systems(Update, handle_entity_death);
    }
}
