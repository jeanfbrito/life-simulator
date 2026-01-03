# Map Generator 2.0 - Configuration Reference

This document provides comprehensive documentation of all configuration parameters for the enhanced terrain generation system introduced in Map Generator 2.0.

## Table of Contents
- [Overview](#overview)
- [MapGen2Config](#mapgen2config)
- [SpotNoiseConfig](#spotnoiseconfig)
- [OpenRCT2TerrainConfig](#openrct2terrainconfig)
- [Configuration Usage](#configuration-usage)
- [Parameter Tuning Guide](#parameter-tuning-guide)
- [Examples](#examples)

---

## Overview

Map Generator 2.0 introduces three main configuration structures:

1. **MapGen2Config** - Boundary enforcement and terrain distribution targets
2. **SpotNoiseConfig** - Factorio-inspired water spot placement
3. **OpenRCT2TerrainConfig** - Height-to-terrain thresholds (enhanced from original)

All parameters are numeric and fully configurable for easy iteration and map customization.

---

## MapGen2Config

**Location:** `src/tilemap/openrct2/settings.rs`

Controls boundary enforcement, water placement, and terrain distribution targets for Map Generator 2.0.

### Perimeter Boundary Parameters

| Parameter | Type | Default | Range | Description |
|-----------|------|---------|-------|-------------|
| `perimeter_deep_water_width` | `u32` | `1` | 1-10 | Width of outermost deep water layer at map edges (in tiles) |
| `perimeter_shallow_water_width` | `u32` | `2` | 1-10 | Width of shallow water layer between deep water and beach (in tiles) |
| `perimeter_sand_min_width` | `u32` | `1` | 1-10 | Minimum width of sand/beach layer before land begins (in tiles) |

**Boundary Layer Structure:**
```
Map Edge → [Deep Water] → [Shallow Water] → [Sand] → Land Interior
           ↑ width=1      ↑ width=2          ↑ width=1
```

**Effect on Map:**
- **Larger widths** create wider coastal zones, reducing usable land area
- **Smaller widths** maximize land but may feel less natural
- Total boundary width = deep + shallow + sand (default: 4 tiles from edge)

### Internal Water Transition Parameters

| Parameter | Type | Default | Range | Description |
|-----------|------|---------|-------|-------------|
| `internal_water_transition_width` | `u32` | `1` | 1-5 | Width of shallow water buffer between internal deep water and land (in tiles) |

**Internal Water Structure:**
```
Land → [Shallow Water] → [Deep Water] → [Shallow Water] → Land
       ↑ buffer=1                        ↑ buffer=1
```

**Effect on Map:**
- Prevents abrupt deep water → land transitions
- Sand is NOT required at internal water bodies (only at perimeter)
- Larger values create gentler lake/pond edges

### Terrain Distribution Targets

| Parameter | Type | Default | Range | Description |
|-----------|------|---------|-------|-------------|
| `land_coverage_target` | `f32` | `0.60` | 0.0-1.0 | Minimum percentage of map that must be walkable land (validation threshold) |
| `grass_forest_ratio` | `f32` | `0.50` | 0.0-1.0 | Minimum percentage of land tiles that should be grass or forest (validation threshold) |

**Usage:**
- Used by `MapValidation::validate()` to determine if generated map is playable
- Maps failing these targets will be rejected and regenerated with modified seed
- `0.60` = 60% land coverage minimum
- `0.50` = 50% of land must be green (grass/forest)

### Water Spot Placement Parameters

| Parameter | Type | Default | Range | Description |
|-----------|------|---------|-------|-------------|
| `water_spot_count` | `u32` | `8` | 0-50 | Target number of internal water bodies to generate |
| `water_spot_radius_min` | `f32` | `3.0` | 1.0-20.0 | Minimum radius of water spots (in tiles) |
| `water_spot_radius_max` | `f32` | `8.0` | 5.0-50.0 | Maximum radius of water spots (in tiles) |

**Effect on Map:**
- Higher `water_spot_count` creates more drinking water sources for animals
- Larger radius values create bigger lakes/ponds
- Actual spot placement controlled by `SpotNoiseConfig` (see below)

### Default Configuration

```rust
MapGen2Config {
    perimeter_deep_water_width: 1,
    perimeter_shallow_water_width: 2,
    perimeter_sand_min_width: 1,
    internal_water_transition_width: 1,
    land_coverage_target: 0.60,
    grass_forest_ratio: 0.50,
    water_spot_count: 8,
    water_spot_radius_min: 3.0,
    water_spot_radius_max: 8.0,
}
```

---

## SpotNoiseConfig

**Location:** `src/tilemap/openrct2/settings.rs`

Factorio-inspired noise-based water spot placement for natural distribution of internal water bodies.

### Parameters

| Parameter | Type | Default | Range | Description |
|-----------|------|---------|-------|-------------|
| `frequency` | `f64` | `0.02` | 0.005-0.1 | Base frequency of spot noise (lower = larger regions) |
| `spot_threshold` | `f64` | `0.6` | 0.0-1.0 | Noise value threshold for spot placement (higher = fewer spots) |
| `spot_radius_scale` | `f32` | `1.0` | 0.1-5.0 | Multiplier for water spot size |
| `jitter_amount` | `f32` | `0.3` | 0.0-1.0 | Random position offset for natural variation |

### How Spot Noise Works

1. **Multi-octave noise generation** at two scales:
   - Primary layer (scale = `frequency`): Large-scale distribution
   - Detail layer (scale = `frequency * 4`): Small-scale variation

2. **Threshold filtering**:
   - Noise values above `spot_threshold` become water spot centers
   - Higher threshold = fewer, more spaced-out spots
   - Lower threshold = more, potentially overlapping spots

3. **Radius calculation**:
   ```
   spot_radius = water_spot_radius_min + (noise_strength * range * spot_radius_scale)
   where range = water_spot_radius_max - water_spot_radius_min
   ```

4. **Radial falloff**:
   - Water spots have smooth edges with distance-based interpolation
   - Height interpolates from current terrain to water level over spot radius

### Parameter Tuning Guide

**For more water spots:**
- Decrease `spot_threshold` (e.g., 0.6 → 0.4)
- Decrease `frequency` (e.g., 0.02 → 0.01) for larger regions

**For fewer, larger water bodies:**
- Increase `spot_threshold` (e.g., 0.6 → 0.75)
- Increase `spot_radius_scale` (e.g., 1.0 → 2.0)

**For more natural/random placement:**
- Increase `jitter_amount` (e.g., 0.3 → 0.7)
- Decrease `frequency` for less regular patterns

**For more uniform distribution:**
- Decrease `jitter_amount` (e.g., 0.3 → 0.1)
- Increase `frequency` for smaller, more numerous regions

### Default Configuration

```rust
SpotNoiseConfig {
    frequency: 0.02,
    spot_threshold: 0.6,
    spot_radius_scale: 1.0,
    jitter_amount: 0.3,
}
```

---

## OpenRCT2TerrainConfig

**Location:** `src/tilemap/world_generator.rs`

Height-to-terrain threshold mapping (enhanced in Map Generator 2.0 for maximized grass/forest coverage).

### Water Level Parameters

| Parameter | Type | Default (Old) | Default (New) | Range | Description |
|-----------|------|---------------|---------------|-------|-------------|
| `deep_water_max` | `u8` | `35` | `25` | 0-255 | Heights ≤ this value become DeepWater |
| `shallow_water_max` | `u8` | `60` | `45` | 0-255 | Heights ≤ this value become ShallowWater |
| `beach_max` | `u8` | `65` | `52` | 0-255 | Heights ≤ this value become Sand (beach) |

**Changes in Map Generator 2.0:**
- Reduced water thresholds to minimize water coverage (~25% reduction)
- More land area available for vegetation and animal habitats

### Land Elevation Parameters

| Parameter | Type | Default (Old) | Default (New) | Range | Description |
|-----------|------|---------------|---------------|-------|-------------|
| `plains_max` | `u8` | `120` | `180` | 0-255 | Heights ≤ this value eligible for Grass/Forest/Dirt |
| `hills_max` | `u8` | `160` | `200` | 0-255 | Heights ≤ this value become Stone (rocky terrain) |
| `mountain_min` | `u8` | `160` | `200` | 0-255 | Heights ≥ this value become Mountain |

**Changes in Map Generator 2.0:**
- **+60 increase to plains_max** (120 → 180): Maximizes grass/forest zones
- **+40 increase to hills/mountain** (160 → 200): Reduces rocky terrain
- Result: Grassland/forest zone spans heights 52-180 instead of 65-120 (~60% larger)

### Terrain Variety Parameters

| Parameter | Type | Default | Range | Description |
|-----------|------|---------|-------|-------------|
| `forest_frequency` | `f64` | `0.05` | 0.01-0.2 | Perlin noise frequency for forest vs grass determination |
| `forest_threshold` | `f64` | `0.0` | -1.0-1.0 | Noise threshold for forest placement (0.0 = 50% forest) |
| `desert_frequency` | `f64` | `0.03` | 0.01-0.2 | Frequency for desert zones (unused in current biome system) |
| `desert_threshold` | `f64` | `0.5` | -1.0-1.0 | Threshold for desert placement (unused in current biome system) |
| `snow_altitude` | `u8` | `255` | 0-255 | Height above which snow appears (255 = disabled) |

**Note:** In Map Generator 2.0, terrain determination now uses `BiomeGenerator` with moisture/temperature/elevation factors instead of simple noise thresholds. These parameters are preserved for backwards compatibility but may not be actively used.

### Height Threshold Cascade

```
   0 ─────────────────────── DeepWater
  25 ┐ deep_water_max
     │
  45 ┘ shallow_water_max ──── ShallowWater
     │
  52 ┘ beach_max ──────────── Sand (Beach)
     │
 180 ┘ plains_max ──────────┐
                             │ Grass / Forest / Dirt
                             │ (Biome-determined)
 200 ┘ hills_max ────────────┘
     │
 200 ┐ mountain_min ───────── Stone / Mountain
     │
 255 ─────────────────────────
```

### Default Configuration (Map Generator 2.0)

```rust
OpenRCT2TerrainConfig {
    deep_water_max: 25,        // ↓ Reduced from 35
    shallow_water_max: 45,     // ↓ Reduced from 60
    beach_max: 52,             // ↓ Reduced from 65
    plains_max: 180,           // ↑ Increased from 120 (+60)
    hills_max: 200,            // ↑ Increased from 160 (+40)
    mountain_min: 200,         // ↑ Increased from 160 (+40)
    forest_frequency: 0.05,
    forest_threshold: 0.0,     // 50% forest coverage
    desert_frequency: 0.03,
    desert_threshold: 0.5,
    snow_altitude: 255,        // Snow disabled
}
```

---

## Configuration Usage

### In Code

```rust
use crate::tilemap::openrct2::{MapGen2Config, SpotNoiseConfig};
use crate::tilemap::world_generator::{OpenRCT2TerrainConfig, WorldGenerator};

// Create custom configuration
let mapgen2_config = MapGen2Config {
    perimeter_deep_water_width: 2,      // Wider deep water boundary
    land_coverage_target: 0.70,         // More stringent land requirement
    water_spot_count: 12,               // More water sources
    ..Default::default()
};

let spot_noise_config = SpotNoiseConfig {
    spot_threshold: 0.7,                // Fewer, more selective spots
    spot_radius_scale: 1.5,             // Larger water bodies
    ..Default::default()
};

let terrain_config = OpenRCT2TerrainConfig {
    plains_max: 200,                    // Even more grassland
    ..Default::default()
};

// Apply to WorldGenerator
let mut generator = WorldGenerator::new(world_seed, world_size_chunks);
generator = generator
    .with_mapgen2_config(mapgen2_config)
    .with_spot_noise_config(spot_noise_config)
    .with_terrain_config(terrain_config);
```

### Via CLI (Future Enhancement)

The `map_generator` CLI currently accepts `--water-density` and `--forest-density` flags but does not yet wire them to these config structs. This is planned for a future update.

```bash
# Current CLI (parameters reserved for future use)
cargo run --bin map_generator generate my_map "My World" 12345 \
  --water-density 0.3 \
  --forest-density 0.6

# Future CLI (when parameter mapping is implemented)
# --water-density will control SpotNoiseConfig::spot_threshold
# --forest-density will control terrain biome parameters
```

---

## Parameter Tuning Guide

### Scenario: "Island Paradise" (More Water)

```rust
MapGen2Config {
    perimeter_deep_water_width: 3,      // Wider ocean boundary
    perimeter_shallow_water_width: 4,   // Gentle beach slope
    land_coverage_target: 0.50,         // Accept 50% land (more water)
    water_spot_count: 15,               // Many internal lakes
    water_spot_radius_min: 5.0,         // Larger lakes
    water_spot_radius_max: 12.0,
    ..Default::default()
}

SpotNoiseConfig {
    spot_threshold: 0.5,                // Lower threshold = more water spots
    spot_radius_scale: 1.5,             // 50% larger spots
    ..Default::default()
}
```

### Scenario: "Grassy Plains" (Maximize Land)

```rust
MapGen2Config {
    perimeter_deep_water_width: 1,      // Minimal ocean
    perimeter_shallow_water_width: 1,
    perimeter_sand_min_width: 1,
    land_coverage_target: 0.75,         // Require 75% land
    grass_forest_ratio: 0.70,           // 70% grass/forest
    water_spot_count: 4,                // Minimal internal water
    water_spot_radius_min: 2.0,         // Small ponds
    water_spot_radius_max: 5.0,
    ..Default::default()
}

SpotNoiseConfig {
    spot_threshold: 0.75,               // High threshold = fewer spots
    spot_radius_scale: 0.7,             // Smaller spots
    ..Default::default()
}

OpenRCT2TerrainConfig {
    deep_water_max: 20,                 // Even less deep water
    shallow_water_max: 35,
    plains_max: 200,                    // Maximum grassland
    ..Default::default()
}
```

### Scenario: "Balanced Ecosystem" (Default)

```rust
// Use all defaults for balanced terrain suitable for animal simulation
let mapgen2_config = MapGen2Config::default();
let spot_noise_config = SpotNoiseConfig::default();
let terrain_config = OpenRCT2TerrainConfig::default();

// Results in:
// - 60%+ land coverage
// - 50%+ grass/forest
// - ~8 water spots distributed across map
// - Natural boundaries with deep→shallow→sand transitions
```

### Scenario: "Dense Forest" (Maximum Vegetation)

```rust
MapGen2Config {
    grass_forest_ratio: 0.80,           // 80% must be green
    land_coverage_target: 0.70,         // More land for trees
    water_spot_count: 6,                // Fewer water bodies
    ..Default::default()
}

OpenRCT2TerrainConfig {
    plains_max: 220,                    // Expand forest-eligible zone
    forest_threshold: -0.3,             // 65% forest (lower = more forest)
    ..Default::default()
}
```

---

## Examples

### Example 1: Small Map with High Water Coverage

```rust
let world_config = WorldConfig {
    seed: 99999,
    world_size_chunks: 50,              // Smaller map (50x50 chunks)
    ..Default::default()
};

let mapgen2_config = MapGen2Config {
    land_coverage_target: 0.45,         // Accept 45% land (55% water)
    water_spot_count: 20,               // Many lakes for small map
    water_spot_radius_min: 2.0,         // Small lakes
    water_spot_radius_max: 6.0,
    ..Default::default()
};
```

### Example 2: Large Map with Minimal Water

```rust
let world_config = WorldConfig {
    seed: 42424,
    world_size_chunks: 150,             // Large map (150x150 chunks)
    ..Default::default()
};

let mapgen2_config = MapGen2Config {
    land_coverage_target: 0.80,         // Require 80% land
    water_spot_count: 8,                // Standard water spots (spread over larger area)
    perimeter_deep_water_width: 1,      // Minimal ocean
    ..Default::default()
};

let terrain_config = OpenRCT2TerrainConfig {
    deep_water_max: 18,                 // Very little deep water
    shallow_water_max: 30,
    plains_max: 210,                    // Maximum grassland
    ..Default::default()
};
```

### Example 3: Testing Extreme Boundaries

```rust
let mapgen2_config = MapGen2Config {
    perimeter_deep_water_width: 5,      // Very wide ocean (5 tiles)
    perimeter_shallow_water_width: 8,   // Very wide shallow zone (8 tiles)
    perimeter_sand_min_width: 3,        // Wide beach (3 tiles)
    // Total boundary width: 16 tiles from edge
    land_coverage_target: 0.40,         // Accept reduced land due to wide boundaries
    ..Default::default()
};
```

---

## Validation and Quality Assurance

Map Generator 2.0 includes automatic validation via `MapValidation::validate()`:

### Validation Checks

1. **Land Coverage**: `land_tiles / total_tiles >= land_coverage_target`
2. **Green Coverage**: `(grass + forest) / land_tiles >= grass_forest_ratio`
3. **Spawn Point**: At least one walkable land tile exists
4. **Water Accessibility**: Both water and land tiles present

### Map Rejection and Retry

If validation fails, the generator:
1. Modifies the seed using large prime increment (982451653)
2. Regenerates the map with new seed
3. Retries up to 5 times (configurable via `max_attempts`)
4. Returns validation results with detailed error messages

### Verification in CLI

```bash
# Generate map with verbose output showing validation
cargo run --bin map_generator generate test_map "Test" 12345 --verbose

# Output includes validation results:
# ✓ Land coverage: 62.3% (target: 60%)
# ✓ Green coverage: 54.1% (target: 50%)
# ✓ Spawn point: Found
# ✓ Water accessible: Yes
# Map validation: PASSED
```

---

## File Locations

| Configuration Struct | File Path |
|---------------------|-----------|
| `MapGen2Config` | `src/tilemap/openrct2/settings.rs` |
| `SpotNoiseConfig` | `src/tilemap/openrct2/settings.rs` |
| `OpenRCT2TerrainConfig` | `src/tilemap/world_generator.rs` |
| `WorldConfig` | `src/tilemap/world_generator.rs` |

---

## See Also

- **E2E_VERIFICATION_GUIDE.md** - End-to-end testing procedures
- **ANIMAL_SPAWN_VERIFICATION.md** - Entity spawning verification
- **STABILITY_VERIFICATION_GUIDE.md** - System stability testing
- **docs/OPENRCT2_TERRAIN_EXTRACTION.md** - Original terrain generation algorithm
- **docs/PLANT_SYSTEM_PARAMS.md** - Vegetation and resource configuration

---

## Summary

Map Generator 2.0 provides comprehensive configurability for terrain generation:

✅ **Boundary Control**: Precise control over map edge layers (deep water → shallow → sand)
✅ **Water Placement**: Factorio-inspired spot noise for natural water distribution
✅ **Terrain Distribution**: Adjustable targets for land coverage and vegetation density
✅ **Quality Assurance**: Automatic validation with retry on failure
✅ **Backward Compatible**: All original parameters preserved with sensible new defaults

All parameters are numeric, making experimentation and iteration straightforward. The default configuration is tuned for balanced animal-friendly habitats with 60%+ land, 50%+ vegetation, and strategic water placement.
