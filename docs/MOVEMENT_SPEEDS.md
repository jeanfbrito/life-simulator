# Entity Movement Speeds Reference

This document provides a quick reference for movement speeds of all entity types in the life-simulator.

## Current Configuration

| Entity Type | Ticks per Tile | Time per Tile (at 10 TPS) | Tiles per Second | Speed Relative to Human | Status |
|-------------|----------------|---------------------------|------------------|------------------------|--------|
| **Rabbit** 🐇 | 20 | 2.0 seconds | 0.50 tiles/s | 1.5x faster ⚡ | Active |
| **Human** 🧍‍♂️ | 30 | 3.0 seconds | 0.33 tiles/s | 1.0x (baseline) | Active |
| **Deer** 🦌 | 10 | 1.0 seconds | 1.00 tiles/s | 3.0x faster ⚡⚡ | Future |
| **Wolf** 🐺 | 6 | 0.6 seconds | 1.67 tiles/s | 5.0x faster ⚡⚡⚡ | Future |

## Detailed Breakdown

### 🐇 Rabbits
- **Movement Speed**: 20 ticks per tile
- **Real-time**: 2 seconds per tile
- **Characteristics**: Quick and nimble
- **Wander Radius**: 15 tiles (smaller territory)
- **Behavior**: Standard wandering AI

**Why this speed?**
- Rabbits are naturally faster than humans
- Quick, darting movements fit their character
- Allows them to cover ground efficiently while wandering

### 🧍‍♂️ Humans
- **Movement Speed**: 30 ticks per tile
- **Real-time**: 3 seconds per tile
- **Characteristics**: Comfortable walking pace (Dwarf Fortress-inspired)
- **Wander Radius**: 30 tiles (moderate territory)
- **Behavior**: Standard wandering AI

**Why this speed?**
- Feels natural for a walking human
- Not too fast (unrealistic) or too slow (boring)
- Good baseline for comparing other entities

### 🦌 Deer (Future)
- **Movement Speed**: 10 ticks per tile
- **Real-time**: 1 second per tile
- **Characteristics**: Quick escape, moderate roaming
- **Wander Radius**: 40 tiles (large territory)
- **Behavior**: Standard wandering AI, future flee behavior

### 🐺 Wolves (Future)
- **Movement Speed**: 6 ticks per tile
- **Real-time**: 0.6 seconds per tile
- **Characteristics**: Fast predator
- **Wander Radius**: 50 tiles (roams widely)
- **Behavior**: Hunting behavior (future implementation)

## Calculation Examples

### At 10 TPS (Base Speed)

```
Rabbit:  20 ticks/tile = 2.0 seconds/tile = 0.50 tiles/second
Human:   30 ticks/tile = 3.0 seconds/tile = 0.33 tiles/second
Deer:    10 ticks/tile = 1.0 seconds/tile = 1.00 tiles/second
Wolf:     6 ticks/tile = 0.6 seconds/tile = 1.67 tiles/second
```

### At Different Simulation Speeds

#### 2x Speed (Speed multiplier = 2.0)

```
Rabbit:  20 ticks/tile = 1.0 seconds/tile = 1.00 tiles/second
Human:   30 ticks/tile = 1.5 seconds/tile = 0.67 tiles/second
Deer:    10 ticks/tile = 0.5 seconds/tile = 2.00 tiles/second
Wolf:     6 ticks/tile = 0.3 seconds/tile = 3.33 tiles/second
```

#### 0.5x Speed (Speed multiplier = 0.5)

```
Rabbit:  20 ticks/tile = 4.0 seconds/tile = 0.25 tiles/second
Human:   30 ticks/tile = 6.0 seconds/tile = 0.17 tiles/second
Deer:    10 ticks/tile = 2.0 seconds/tile = 0.50 tiles/second
Wolf:     6 ticks/tile = 1.2 seconds/tile = 0.83 tiles/second
```

## Testing Movement Speeds

### Visual Comparison Test

```bash
# Start simulation
cargo run --bin life-simulator

# In another terminal, track both:
watch -n 1 'curl -s http://127.0.0.1:54321/api/entities | \
  jq ".entities[] | select(.name | startswith(\"Human\") or startswith(\"Rabbit\")) | \
  {name, position}"'
```

### Automated Speed Test

```bash
# Run the test script
./scripts/test_movement.sh

# Or compare specific entities:
for i in {1..10}; do
  echo "$(date +%S)s:"
  curl -s http://127.0.0.1:54321/api/entities | \
    jq -r '.entities[] | select(.name == "Human_0" or .name == "Rabbit_0") | 
    "\(.name): (\(.position.x),\(.position.y))"'
  sleep 2
done
```

## Observational Notes

From testing on 2025-10-02:

### Rabbit Movement (20 ticks/tile)
- Moves approximately every 2 seconds
- Noticeably faster than humans
- Quick direction changes
- Covers more ground in same time period

### Human Movement (30 ticks/tile)
- Moves approximately every 3 seconds
- Comfortable, realistic pace
- Steady progression through world
- Good baseline for gameplay feel

## Adjusting Movement Speeds

To change an entity's movement speed, edit `src/entities/entity_types.rs`:

```rust
pub const RABBIT: EntityTemplate = EntityTemplate {
    name_prefix: "Rabbit",
    species: "Rabbit",
    movement_speed: 20,  // <-- Change this value
    wander_radius: 15,
    emoji: "🐇",
};
```

**Guidelines:**
- Lower value = faster movement
- Higher value = slower movement
- Minimum practical: ~5 ticks (0.5 seconds)
- Maximum practical: ~100 ticks (10 seconds)

## Performance Considerations

Movement speed affects:
- **AI decision frequency**: Faster entities make more decisions
- **Pathfinding load**: More movement = more pathfinding
- **Network traffic**: Position updates in API
- **Visual smoothness**: Very fast entities might appear jumpy

**Recommendations:**
- Keep most entities in the 10-50 tick range
- Reserve very fast speeds (< 10 ticks) for special entities
- Test with multiple entities to ensure stable TPS

## Related Files

- **Configuration**: `src/entities/entity_types.rs`
- **Movement System**: `src/entities/movement.rs`
- **Wandering AI**: `src/entities/wandering.rs`
- **Testing**: `scripts/test_movement.sh`

## Change Log

### 2025-10-02
- **Rabbits**: Updated from 60 ticks to 20 ticks per tile (3x faster)
  - Was: 6 seconds per tile (actually slower than humans)
  - Now: 2 seconds per tile (faster than humans, as intended)
- **Humans**: Remain at 30 ticks per tile (3 seconds)
- **Reasoning**: Rabbits should be noticeably faster than humans

### Initial Configuration
- **Humans**: 30 ticks per tile (comfortable walking)
- **Rabbits**: 60 ticks per tile (originally intended faster, but was actually slower)

---

**Last Updated**: 2025-10-02
