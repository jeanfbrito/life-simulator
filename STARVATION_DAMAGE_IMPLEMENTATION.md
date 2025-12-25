# Starvation/Dehydration Damage System Implementation

## Overview
Implemented progressive health damage system for entities based on hunger and thirst levels. Entities now take damage when their needs reach critical levels, leading to natural population regulation through starvation deaths.

## Implementation Details

### Files Modified

#### 1. `src/entities/stats.rs`
**Added Methods to `Health` impl:**
- `apply_need_damage(&mut self, hunger: &Hunger, thirst: &Thirst)` - Main damage application logic
  - Calculates normalized hunger/thirst levels (0.0-1.0)
  - Applies the WORSE of hunger/thirst damage (they don't stack)
  - Only damages if either need >= 90%

- `calculate_need_damage(need_normalized: f32) -> f32` - Progressive damage calculation
  - 90-95% hungry/thirsty: 0.05 health/tick (warning - can survive indefinitely)
  - 95-98%: 0.2 health/tick (danger - death in ~500 ticks = 50 seconds)
  - 98%+: 0.5 health/tick (critical - death in ~200 ticks = 20 seconds)

**Added System:**
- `need_damage_system()` - Runs every tick in FixedUpdate schedule
  - Queries all entities with Health, Hunger, and Thirst
  - Calls `apply_need_damage()` for each entity
  - Logs damage events every 10 ticks to avoid spam
  - Positioned AFTER `tick_stats_system` and BEFORE `death_system`

#### 2. `src/entities/mod.rs`
**System Registration:**
- Exported `need_damage_system` in module public API
- Added system to entity plugin tick systems chain
- Order: `tick_stats_system` -> `need_damage_system` -> `death_system`

### Progressive Damage Thresholds

| Need Level | Damage/Tick | Time to Death | Status |
|-----------|-------------|---------------|---------|
| 0-90% | 0.0 | Infinite | Healthy |
| 90-95% | 0.05 | ~2000 ticks | Warning |
| 95-98% | 0.2 | ~500 ticks | Danger |
| 98-100% | 0.5 | ~200 ticks | Critical |

*At 10 TPS: 200 ticks = 20 seconds, 500 ticks = 50 seconds*

## Test Coverage

### Unit Tests (`src/entities/stats.rs`)
1. `test_starvation_damage_warning` - Verifies 0.05 damage at 92% hunger
2. `test_starvation_damage_critical` - Verifies 0.5 damage at 99% hunger
3. `test_thirst_overrides_hunger` - Verifies worse damage is used (not stacked)
4. `test_no_damage_below_threshold` - Verifies no damage below 90%

### Integration Tests (`tests/starvation_damage_test.rs`)
1. `test_starvation_damage_integration` - Full system integration with one tick
2. `test_progressive_damage_levels` - Validates warning level damage
3. `test_thirst_overrides_hunger` - Integration test for damage priority
4. `test_death_from_starvation` - Verifies entity health reaches zero over 210 ticks

**Test Results:** All 132 tests passing (128 lib + 4 integration)

## Behavioral Changes

### Before Implementation
- Entities could survive indefinitely at 100% hunger/thirst
- Health only increased (+0.01/tick regen), never decreased (except combat)
- Population growth unchecked by food availability

### After Implementation
- Entities take progressive damage based on starvation/dehydration
- Natural population regulation - starving populations decline
- Well-fed populations still thrive
- Entities die in ~20 seconds at critical starvation (98%+)
- Warning zone (90-95%) allows survival if food found soon

## Usage

The system runs automatically every tick. No manual intervention needed.

**To observe in action:**
1. Spawn entities with no food sources
2. Wait for hunger to reach 90%+
3. Watch damage logs: `WARN Entity taking need damage: health X.X (hunger Y%, thirst Z%)`
4. Entities die when health reaches 0

**Example log output:**
```
WARN Entity Entity(123v1) taking need damage: health 92.5 (hunger 98.5%, thirst 45.0%)
WARN Entity Entity(123v1) taking need damage: health 87.0 (hunger 99.0%, thirst 47.0%)
...
INFO Entity Entity(123v1) has died!
```

## Performance Impact
- Minimal: Single query per tick over entities with (Health, Hunger, Thirst)
- No new allocations
- Simple arithmetic calculations
- Logging throttled to every 10 ticks

## Future Enhancements
- Configurable damage thresholds per species
- Different damage rates for herbivores vs carnivores
- Age-based vulnerability (juveniles/elderly take more damage)
- Disease/injury modifiers to damage rates
