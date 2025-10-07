# Plant System Parameters Documentation ⚠️ **SUPERSEDED**

**⚠️ This document has been superseded by the completed vegetation system rewrite.**
**See: [Vegetation System Rewrite Plan](VEGETATION_REWRITE_PLAN.md) ✅ **FULLY IMPLEMENTED & VALIDATED****

## Overview

**HISTORICAL NOTE:** This document described parameters for the original dense tile-by-tile vegetation system. The current implementation uses a completely different sparse, event-driven ResourceGrid architecture with significantly different parameters and performance characteristics.

**🔧 Current System:**
- **Sparse Event-Driven ResourceGrid**: Only stores cells with active biomass
- **Level-of-Detail Management**: Proximity-based optimization
- **On-Demand Heatmaps**: Refresh only when data changes
- **Real-World Performance**: 10.0 TPS achieved with 0.0ms processing time

For the current system parameters, see the constants and implementation in the source code and [Vegetation System Rewrite Plan](VEGETATION_REWRITE_PLAN.md).

---

## Legacy Documentation (For Historical Reference)

The following describes the original dense tile-by-tile approach that has been **replaced**:

The plant system implemented a logistic growth model for vegetation biomass with herbivore consumption dynamics, designed to be ecologically realistic and performant.
- **Configurable**: All parameters centralized for easy tuning
- **Extensible**: Modular design supports multiple species and terrain types

## Core Growth Parameters

### Logistic Growth Model

The system uses the classic logistic growth equation:

```
B(t+1) = B(t) + r * B(t) * (1 - B(t)/Bmax)
```

Where:
- `B(t)` = Current biomass at time t
- `r` = Growth rate coefficient
- `Bmax` = Maximum sustainable biomass (carrying capacity)

#### Growth Rate (`r`)

```rust
pub const GROWTH_RATE: f32 = 0.05; // 5% per tick at optimal conditions
```

**Rationale**:
- 5% per growth tick provides reasonable regrowth speed
- Higher values risk instability (oscillations)
- Lower values make recovery from grazing too slow
- Calibrated for 1-second growth intervals (10 ticks at 10 TPS)

#### Maximum Biomass (`Bmax`)

```rust
pub const MAX_BIOMASS: f32 = 100.0; // Arbitrary biomass units
```

**Rationale**:
- Provides sufficient resolution for consumption calculations
- Scales well with fraction-based consumption (30% rule = 30 units)
- Compatible with species-specific daily needs (rabbit: 25 units/day)

#### Growth Frequency

```rust
pub const GROWTH_INTERVAL_TICKS: u64 = 10; // Every 1 second at 10 TPS
```

**Rationale**:
- Balances responsiveness with performance
- 1Hz updates match typical plant growth observation scales
- Aligns with game's discrete tick system (10 TPS base)

## Consumption Parameters

### 30% Rule (Herbivore Foraging Limit)

```rust
pub const MAX_MEAL_FRACTION: f32 = 0.3; // 30% of available biomass
pub const MAX_MEAL_ABSOLUTE: f32 = 30.0; // 30% of Bmax = 30 units
```

**Rationale**:
- Prevents unrealistic consumption (eating entire patch in one meal)
- Based on optimal foraging theory - herbivores rarely deplete patches completely
- Leaves residual biomass for regrowth
- Creates natural giving-up points for patch switching

### Species-Specific Consumption

#### Rabbit Profile

```rust
// Daily requirements
pub const DAILY_BIOMASS_NEED: f32 = 25.0; // biomass units per day

// Meal behavior
pub const MEAL_SIZE_FRACTION: f32 = 0.15; // 15% of daily needs per meal
// = 3.75 units per meal, well below 30-unit maximum

// Foraging preferences
pub const PREFERRED_BIOMASS_MIN: f32 = 30.0; // Avoid depleted patches
pub const PREFERRED_BIOMASS_MAX: f32 = 80.0; // Avoid overly dense patches
```

**Rationale**:
- Rabbits are small grazers with high metabolic rates
- Small, frequent meals match natural behavior
- Preference range balances travel time with foraging quality

#### Deer Profile (Future)

```rust
// Daily requirements (larger animal)
pub const DAILY_BIOMASS_NEED: f32 = 80.0; // 3.2x rabbit needs

// Meal behavior
pub const MEAL_SIZE_FRACTION: f32 = 0.25; // Larger meals, less frequent
// = 20 units per meal

// Foraging range
pub const SEARCH_RADIUS: i32 = 25; // Wider range than rabbits
```

## Terrain Modifiers

### Maximum Biomass by Terrain

```rust
pub fn max_biomass_multiplier(terrain: &str) -> f32 {
    match terrain {
        "Grass" => 1.0,      // Baseline: 100 units
        "Forest" => 1.2,     // Understory: 120 units
        "Dirt" => 0.7,       // Poor soil: 70 units
        "Swamp" => 0.8,      // Waterlogged: 80 units
        "Desert" => 0.2,     // Sparse: 20 units
        "Mountain" => 0.1,   // Lichens only: 10 units
        _ => 0.0,            // No vegetation
    }
}
```

**Rationale**:
- Grasslands represent optimal grazing conditions
- Forests provide understory vegetation (higher biomass)
- Desert and mountain terrain severely limit vegetation
- Water terrain supports no vegetation

### Growth Rate Modifiers

```rust
pub fn growth_rate_modifier(terrain: &str) -> f32 {
    match terrain {
        "Grass" => 1.0,      // Baseline growth
        "Forest" => 1.1,     // Protected environment
        "Swamp" => 1.2,      // High moisture availability
        "Desert" => 0.4,     // Water-limited growth
        "Mountain" => 0.3,   // Harsh conditions
        _ => 0.0,            // No growth
    }
}
```

**Rationale**:
- Moisture availability is primary growth driver
- Temperature and nutrients affect growth rates
- Forest canopy provides protection but reduces light

## Performance Parameters

### Active Tile Tracking

```rust
pub const ACTIVE_TILE_THRESHOLD: f32 = 95.0; // 95% of Bmax
pub const DEPLETED_TILE_COOLDOWN: u64 = 50; // 5 seconds at 10 TPS
```

**Rationale**:
- Only tiles below 95% capacity need frequent updates
- Recently grazed tiles stay active for cooldown period
- Reduces computation from O(n²) to O(active_tiles)

### Update Limits

```rust
pub const MAX_ACTIVE_TILES_PER_UPDATE: usize = 1000;
pub const INACTIVE_SAMPLE_SIZE: usize = 100;
```

**Rationale**:
- Limits CPU usage per growth cycle
- Sampling ensures all tiles eventually updated
- Provides consistent performance regardless of map size

## Behavioral Parameters

### Giving-Up Thresholds

```rust
pub const GIVING_UP_THRESHOLD: f32 = 20.0; // biomass units
pub const FORAGE_MIN_BIOMASS: f32 = 10.0; // minimum for foraging
```

**Rationale**:
- Animals leave patches when biomass becomes uneconomical
- Minimum threshold prevents wasted travel to depleted areas
- Creates natural grazing patterns and patch rotation

### Predator Fear Effects

```rust
pub const FEAR_FEEDING_REDUCTION: f32 = 0.3; // 30% shorter feeding
pub const FEAR_RADIUS: i32 = 40; // tiles
pub const FEAR_SPEED_BOOST: f32 = 1.5; // 1.5x movement speed
```

**Rationale**:
- Predation risk affects foraging behavior
- Trade-off between feeding and vigilance
- Larger radius for early warning system

## Integration Points

### Current System Integration

1. **AI Planner (`src/ai/planner.rs`)**:
   - Actions evaluated through `herbivore_toolkit::evaluate_core_actions()`
   - Grazing actions use `ActionType::Graze { target_tile }`
   - Utility scoring combines hunger (80%) and distance (20%)

2. **Auto-Eat System (`src/entities/auto_eat.rs`)**:
   - Triggers when entity stands on grass with hunger > 15%
   - Uses `SpeciesNeeds.eat_amount` for consumption
   - **Integration Point**: Replace with vegetation-based consumption

3. **Behavior Configuration (`src/entities/types/`)**:
   - `graze_range: (3, 8)` for rabbits (3-8 tiles)
   - `food_search_radius: 100` tiles for wide area search
   - **Integration Point**: Add vegetation-aware search

### Future Integration Points

1. **Grazing Behavior (`src/ai/behaviors/grazing.rs`)**:
   - Currently checks `TerrainType::Grass` only
   - **Integration Point**: Query vegetation biomass instead

2. **Eating Behavior (`src/ai/behaviors/eating.rs`)**:
   - Currently finds nearest grass tile
   - **Integration Point**: Rank by biomass availability

3. **Species Configurations**:
   - Add vegetation-specific parameters to behavior configs
   - **Integration Point**: Extend `BehaviorConfig` structure

## Memory and Performance Estimates

### Storage Requirements

**Sparse HashMap Approach**:
```
Per tile: 24 bytes (IVec2) + 16 bytes (TileVegetation) = 40 bytes
Typical world (100x100 = 10,000 tiles):
- 10% vegetation coverage = 1,000 tiles = 40 KB
- 50% vegetation coverage = 5,000 tiles = 200 KB
- 100% vegetation coverage = 10,000 tiles = 400 KB
```

**Dense Grid Alternative**:
```
Per tile: 4 bytes (f32 biomass) = 4 bytes
100x100 world = 10,000 tiles = 40 KB (fixed)
```

**Decision**: Sparse HashMap chosen for:
- Lower memory usage on sparsely vegetated maps
- No overhead for water/deep water tiles
- Better cache locality for active tiles
- Scales well with large map sizes

### CPU Performance

**Growth System**:
- Active tiles: 1,000 max per update = ~1ms
- Inactive sampling: 100 tiles = ~0.1ms
- Total: <2ms per growth cycle (1 second)

**Foraging Queries**:
- Radius search: O(r²) where r = search radius (15-25 tiles)
- 400-625 tiles checked per query
- ~0.1ms per entity with spatial optimization

## Validation Metrics

### Ecological Validation

1. **Equilibrium Biomass**: Should stabilize at 60-80% of Bmax under moderate grazing
2. **Patch Rotation**: Animals should naturally rotate between patches
3. **Recovery Time**: Depleted patches should recover in 2-5 minutes
4. **Carrying Capacity**: Population should self-regulate based on vegetation

### Performance Validation

1. **Memory Usage**: <1MB for 100x100 worlds
2. **CPU Usage**: <5% of total simulation time
3. **Frame Rate**: No impact on 10 TPS simulation
4. **Scalability**: Linear performance with map size

## Tuning Guidelines

### If Vegetation Recovers Too Slowly

1. Increase `GROWTH_RATE` (max 0.1 for stability)
2. Decrease `GROWTH_INTERVAL_TICKS` (min 5 for 0.5s updates)
3. Increase terrain-specific growth modifiers
4. Decrease `MAX_MEAL_FRACTION` to 0.2

### If Vegetation Recovers Too Quickly

1. Decrease `GROWTH_RATE` (min 0.02 for visible change)
2. Increase `GROWTH_INTERVAL_TICKS` (max 20 for 2s updates)
3. Increase `MAX_MEAL_FRACTION` to 0.4
4. Lower terrain biomass multipliers

### If Animals Overgraze

1. Decrease species `DAILY_BIOMASS_NEED`
2. Increase `GIVING_UP_THRESHOLD`
3. Add predator fear effects
4. Increase `DEPLETED_TILE_COOLDOWN`

### If Animals Undergraze

1. Increase species `DAILY_BIOMASS_NEED`
2. Decrease `GIVING_UP_THRESHOLD`
3. Reduce predator fear effects
4. Improve foraging search efficiency

## Testing Scenarios

### Unit Tests

1. **Logistic Growth**: Verify B(t) approaches Bmax asymptotically
2. **Consumption Limits**: Verify 30% rule enforcement
3. **Terrain Modifiers**: Verify biomass and growth rate adjustments
4. **Active Tile Tracking**: Verify sparse optimization

### Integration Tests

1. **Rabbit Foraging**: Single rabbit depletes patch, moves to next
2. **Population Dynamics**: Multiple rabbits reach equilibrium
3. **Terrain Variation**: Animals prefer high-biomass terrain
4. **Performance**: Large worlds maintain 10 TPS

### Regression Tests

1. **Baseline**: Current grass-only behavior
2. **Biomass Integration**: Verify consumption uses biomass
3. **Growth System**: Verify vegetation regrows after grazing
4. **Multi-species**: Verify different species coexist

## References

1. **Optimal Foraging Theory**: Charnov (1976) - Giving-up time and patch selection
2. **Logistic Growth**: Verhulst (1838) - Classic population growth model
3. **Herbivore Grazing**: McNaughton (1984) - Grassland grazing dynamics
4. **Spatial Heterogeneity**: Tilman (1994) - Resource distribution and competition

## Version History

- **v0.1.0** (2025-01-04): Initial parameter specification based on ecological research
- **Future**: Parameter tuning based on simulation testing and feedback

---

*This document serves as the definitive reference for plant system parameters. All changes should be documented here with rationale and impact analysis.*
