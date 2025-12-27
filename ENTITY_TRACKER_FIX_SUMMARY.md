# Entity Tracker Synchronization Fix

## Problem

The `/api/entities` endpoint was returning only 1 entity when 400 were spawned. The entity tracker was not properly synchronizing spawned entities.

## Root Cause

The `sync_entities_to_tracker` system was using a **clear-and-rebuild** approach that caused race conditions:

```rust
// OLD CODE - BROKEN
tracker.entities.clear();  // ❌ Clears ALL entities
for entity in query.iter() {
    tracker.update(entity_id, data);
}
```

This approach had a critical flaw: if the sync ran during or immediately after spawning, it would clear the tracker before entities were fully queryable, resulting in lost entities.

## Solution

Changed to an **incremental update** approach that:
1. Tracks which entities are seen in the current sync
2. Updates existing entities without clearing
3. Only removes entities that no longer exist in the ECS world

```rust
// NEW CODE - FIXED
let mut seen_entities = HashSet::new();

// Update existing and add new entities
for entity in query.iter() {
    let entity_id = entity.index();
    seen_entities.insert(entity_id);
    tracker.update(entity_id, data);
}

// Remove only entities that no longer exist
let to_remove: Vec<u32> = tracker
    .entities
    .keys()
    .filter(|id| !seen_entities.contains(id))
    .copied()
    .collect();

for entity_id in &to_remove {
    tracker.remove(*entity_id);
}
```

## Benefits

1. **No race conditions**: Entities are added incrementally, not cleared in bulk
2. **Accurate tracking**: All spawned entities are properly tracked
3. **Efficient cleanup**: Only removes entities that actually despawned
4. **Real-time updates**: Tracker stays in sync as entities spawn/despawn

## Verification

Tested with 400 entity load test:
- Before fix: API showed 1 entity
- After fix: API correctly shows 400 entities
- Sync runs every frame without data loss
- Entities properly added/removed as they spawn/despawn

## Files Modified

- `src/entities/entity_tracker.rs`: Changed sync algorithm from clear-rebuild to incremental update

## Debug Logging

Added optional debug logging (enabled with `DEBUG_ENTITY_TRACKER=1` env var):
```bash
DEBUG_ENTITY_TRACKER=1 cargo run --bin life-simulator
```

This logs sync operations to help diagnose issues without flooding normal logs.
