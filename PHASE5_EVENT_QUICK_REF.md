# Phase 5: Event-Driven Communication - Quick Reference

**Status**: COMPLETE ✅
**TPS**: 10.0 maintained ✅
**Tests**: 281/281 passing (274 lib + 7 event) ✅

---

## Event Types

### EntityDiedEvent
```rust
EntityDiedEvent {
    entity: Entity,
    cause: EntityDied, // Starvation | Dehydration | Unknown
}
```
**Trigger**: Health <= 0 (with Changed<Health> detection)

### ActionCompletedEvent
```rust
ActionCompletedEvent {
    entity: Entity,
    action_type: ActionType,
    success: bool,
}
```
**Trigger**: ActionJustCompleted marker component

### PathCompletedEvent
```rust
PathCompletedEvent {
    entity: Entity,
    destination: IVec2,
    success: bool,
}
```
**Trigger**: PathJustCompleted marker component

### StatCriticalEvent
```rust
StatCriticalEvent {
    entity: Entity,
    stat_type: StatCritical, // Hunger | Thirst | Health | Energy
    value: f32,
}
```
**Trigger**: Stat >= 90% (needs) or <= 10% (resources)

---

## Usage Patterns

### Emitting Events

```rust
// Producer system with change detection
fn detect_entity_death(
    query: Query<(Entity, &Health), Changed<Health>>,
    mut events: EventWriter<EntityDiedEvent>,
) {
    for (entity, health) in query.iter() {
        if health.current <= 0.0 {
            events.send(EntityDiedEvent {
                entity,
                cause: EntityDied::Starvation,
            });
        }
    }
}
```

### Consuming Events

```rust
// Consumer system reacts to events
fn handle_entity_death(
    mut events: EventReader<EntityDiedEvent>,
    mut commands: Commands,
) {
    for event in events.read() {
        // React to death
        commands.entity(event.entity).despawn();
    }
}
```

### Using EventSystemPlugin

```rust
// In main.rs or app composition
app.add_plugins(EventSystemPlugin);
```

---

## Files

**Implementation**: `src/events/mod.rs` (316 lines)
**Tests**: `tests/event_system_test.rs` (283 lines)
**Documentation**: `PHASE5_EVENT_DRIVEN_DELIVERY.md`

---

## Performance

**TPS**: 10.0 (no regression)
**CPU**: Reduced via change detection
**Memory**: Minimal event queue overhead

---

## Benefits

1. **Reactive**: Systems only run when events occur
2. **Decoupled**: Producers don't know consumers
3. **Efficient**: Change detection avoids polling
4. **Debuggable**: Event stream visibility
5. **Extensible**: Easy to add new consumers

---

## Next Phase

**Recommended**: Phase 2 (PathResult as Component)
- Leverages PathCompletedEvent
- Similar TDD methodology
- 3-4 hour effort
