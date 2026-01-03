# Specification: Map Generator 2.0 - Enhanced Terrain Generation

## Overview

This feature improves the map generation system to create animal-friendly habitats with proper water boundaries, maximized green terrain coverage, and strategic resource placement. The primary goals are: (1) enforce consistent boundary rules with deep water perimeter → shallow water → sand coastline, (2) implement proper internal water body transitions, (3) maximize grass/forest coverage while providing adequate drinking water spots, and (4) make all generation parameters fully configurable for easy iteration.

## Workflow Type

**Type**: feature

**Rationale**: This is a significant feature implementation that enhances an existing system (OpenRCT2-style terrain generation) with new boundary rules, improved biome distribution, and resource placement. It requires coordinated changes across multiple files while maintaining backward compatibility with existing simulation systems.

## Task Scope

### Services Involved
- **main** (primary) - Rust simulation engine containing all map generation code

### This Task Will:
- [ ] **Create snapshot** of current working state (git tag or backup) before any changes
- [ ] Implement perimeter boundary rules: 1 tile deep water → shallow water layer → minimum 1 tile sand
- [ ] Implement internal water body transition rules: deep water → shallow water (no sand required)
- [ ] Add configurable parameters for all terrain thresholds and ratios
- [ ] Maximize grass/forest terrain coverage as primary land type
- [ ] Implement strategic water spot placement using spot noise algorithm (Factorio-inspired)
- [ ] Extend BiomeGenerator usage for improved biome determination (moisture/temperature layers already exist)
- [ ] Re-implement foraging bushes and fruit resources in the resource layer
- [ ] Optimize map size parameters for animal population density
- [ ] Add map validation/rejection system to ensure quality constraints are met

### Out of Scope:
- Rock terrain types (explicitly excluded per requirements)
- New animal species or AI behavior changes
- Viewer modifications (Godot/Web)
- Performance optimizations beyond current system

## Service Context

### Main Service

**Tech Stack:**
- Language: Rust
- Framework: Bevy ECS
- Key directories: `src/tilemap/`, `src/vegetation/`, `src/bin/`

**Entry Point:** `src/main.rs`

**How to Run:**
```bash
cargo run --release --bin life-simulator
# Or for map generation only:
cargo run --bin map_generator generate my_world "World Name" 12345
```

**Port:** 54321 (HTTP API)

## Files to Modify

| File | Service | What to Change |
|------|---------|---------------|
| `src/tilemap/world_generator.rs` | main | Add boundary enforcement, spot noise water placement, integrate existing BiomeGenerator for multi-factor terrain |
| `src/tilemap/terrain.rs` | main | Ensure terrain types support new generation needs (already complete) |
| `src/tilemap/openrct2/settings.rs` | main | Add new configurable parameters for boundaries, water ratio, vegetation density |
| `src/tilemap/biome.rs` | main | Extend BiomeGenerator to support world_generator integration (moisture/temperature layers already implemented) |
| `src/map_generator.rs` | main | Primary CLI tool - add new configurable parameters, verbose output options, map validation |
| `src/bin/map_generator.rs` | main | Simple CLI wrapper - update to pass new parameters to generator |
| `src/resources/mod.rs` | main | Add/verify foraging bush and fruit resource types |

## Files to Reference

These files show patterns to follow:

| File | Pattern to Copy |
|------|----------------|
| `src/tilemap/world_generator.rs` | OpenRCT2-style noise generation, height thresholds, terrain mapping |
| `src/tilemap/biome.rs` | Multi-octave noise generation, biome determination from climate factors |
| `src/vegetation/resource_grid.rs` | Resource cell creation, spatial organization, event-driven updates |
| `src/tilemap/openrct2/settings.rs` | Configuration struct pattern with sensible defaults |

## Patterns to Follow

### OpenRCT2 Terrain Configuration Pattern

From `src/tilemap/world_generator.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRCT2TerrainConfig {
    // Water levels
    pub deep_water_max: u8,      // Below this = DeepWater (default: 35)
    pub shallow_water_max: u8,   // Below this = ShallowWater (default: 60)
    pub beach_max: u8,           // Below this = Sand (beach) (default: 65)

    // Land elevations
    pub plains_max: u8,          // Below this = Grass/Dirt (default: 120)
    pub hills_max: u8,           // Below this = Stone (default: 160)
    pub mountain_min: u8,        // Above this = Mountain (default: 160)

    // Terrain variety parameters
    pub forest_frequency: f64,   // Perlin noise frequency for forests
    pub forest_threshold: f64,   // Noise threshold for forest placement
}
```

**Key Points:**
- All parameters are numeric and configurable
- Thresholds cascade from low to high
- Frequency and threshold pairs control feature density

### Multi-Octave Noise Generation Pattern

From `src/tilemap/biome.rs` (already implemented - reference for world_generator integration):

```rust
pub fn get_moisture(&self, x: i32, y: i32) -> f32 {
    let scale = 0.02;
    let nx = x as f64 * scale;
    let ny = y as f64 * scale;

    // Primary layer (large scale)
    let primary = self.moisture_noise.get([nx, ny]) as f32;

    // Secondary detail layer (smaller scale)
    let detail_scale = 0.08;
    let detail_x = x as f64 * detail_scale;
    let detail_y = y as f64 * detail_scale;
    let detail = self.moisture_noise.get([detail_x, detail_y]) as f32 * 0.3;

    // Combine and normalize to 0..1
    let combined = primary * 0.7 + detail;
    ((combined + 1.0) * 0.5).clamp(0.0, 1.0)
}
```

**Key Points:**
- Use multiple noise layers at different frequencies
- Combine layers with weighted averaging
- Normalize output to 0.0-1.0 range

### Spot Noise Algorithm Pattern (Factorio-inspired)

```rust
/// Spot noise for controlled feature placement
/// 1. Divide map into regions (chunks)
/// 2. Generate random points per region
/// 3. Calculate density, quantity, radius for each spot
/// 4. Sort by favorability, choose until quota met
/// 5. Output falls to zero at radius distance
pub struct SpotNoiseConfig {
    pub region_size: i32,        // Size of each region in tiles
    pub spots_per_region: i32,   // Max spots per region
    pub min_spot_radius: f32,    // Minimum spot radius
    pub max_spot_radius: f32,    // Maximum spot radius
    pub spot_density: f32,       // Overall spot density (0.0-1.0)
}
```

**Key Points:**
- Ensures even distribution across map
- Prevents clustering
- Allows tuning via density parameter

## Requirements

### Functional Requirements

1. **Perimeter Boundary Enforcement**
   - Description: Map edges must follow strict layer pattern: outermost tile = deep water, then shallow water layer, then minimum 1 tile sand before any land terrain
   - Acceptance: All tiles within 1 unit of map edge are DeepWater; shallow water ring exists between deep water and land; sand exists between shallow water and grass/forest

2. **Internal Water Body Transitions**
   - Description: Internal water bodies (lakes, ponds) must have shallow water buffer between deep water and land terrain; sand is NOT required at internal water edges
   - Acceptance: No deep water tile directly adjacent to land (grass/forest/dirt); shallow water buffer of at least 1 tile exists

3. **Maximized Green Coverage**
   - Description: Primary terrain should be grass and forest; land area should be at least 60% of total map
   - Acceptance: Terrain distribution statistics show grass+forest >= 50% of land tiles; land area >= 60% of total tiles

4. **Strategic Water Placement**
   - Description: Water bodies distributed using spot noise algorithm for even coverage; water spots sized appropriately for animal drinking
   - Acceptance: Water spots appear in all quadrants of map; minimum 4 water sources on standard map size

5. **Foraging Resource Re-implementation**
   - Description: Bush and fruit resources must spawn in appropriate terrain (forest, grass) using existing ResourceGenerator
   - Acceptance: Resource layer contains berry bushes, fruit trees; resources spawn only on walkable land tiles

6. **Configurable Parameters**
   - Description: All generation parameters exposed as numeric values in configuration structs
   - Acceptance: Can regenerate map with different water ratio, forest density, boundary widths via config changes

7. **Map Validation System**
   - Description: Generated maps must pass quality checks before use; invalid maps rejected and regenerated
   - Acceptance: Validation checks land percentage, water accessibility, spawn point availability

### Edge Cases

1. **Small Map Sizes** - Boundary layers may consume significant area; validate minimum map size supports all layers
2. **Seed Variation** - Some seeds may produce invalid maps; implement retry with different seed modifications
3. **Extreme Parameters** - Clamp parameters to valid ranges to prevent degenerate maps
4. **Chunk Boundaries** - Ensure boundary rules apply across chunk edges; use whole-map height generation

## Implementation Notes

### BEFORE STARTING
- **Create a git tag** to snapshot the current working state: `git tag -a pre-mapgen2.0 -m "Snapshot before Map Generator 2.0 implementation"`
- Verify the current system works: `cargo test && cargo run --bin map_generator -n test_snapshot -s 12345`
- This ensures we can restore to a known-good state if needed

### DO
- Follow the pattern in `world_generator.rs` for height-to-terrain mapping
- Reuse `BiomeGenerator` for moisture/temperature noise layers (already has `get_moisture`, `get_temperature`, `get_elevation`)
- Use `OpenRCT2TerrainConfig` pattern for new configuration parameters
- Apply boundary rules in `finalize_chunk_from_whole_map` phase (Phase 3)
- Generate whole-map heights first, then apply boundary overrides
- Use existing `ResourceGenerator::create_resources_for_chunk` for vegetation resources
- Add foraging resources to `RESOURCE_DEFINITIONS` in `src/resources/mod.rs`

### DON'T
- Create separate terrain types for boundary tiles (use existing types)
- Modify terrain walkability rules (ShallowWater is already non-walkable)
- Add rock terrain (explicitly out of scope)
- Change chunk size (keep 16x16)
- Break existing API endpoints

## Development Environment

### Start Services

```bash
# Run the simulation (includes HTTP server)
cargo run --release --bin life-simulator

# Generate a test map
cargo run --bin map_generator generate test_map "Test Map" 12345

# Run with logging
RUST_LOG=info cargo run --bin life-simulator

# Run tests
cargo test --lib
```

### Service URLs
- Simulation API: http://localhost:54321
- Viewer: http://localhost:54321/viewer.html

### Required Environment Variables
- None required for map generation (optional API keys in `.env.example` for other features)

## Success Criteria

The task is complete when:

0. [ ] **Snapshot created**: Git tag `pre-mapgen2.0` exists before any code changes
1. [ ] Map edges have 1-tile deep water perimeter
2. [ ] Shallow water layer exists between deep water and land at all boundaries
3. [ ] Minimum 1 tile sand at coastlines (map edges)
4. [ ] Internal water bodies have shallow water buffer (no direct deep→land)
5. [ ] Land coverage >= 60% of map area
6. [ ] Grass + Forest >= 50% of land tiles
7. [ ] Water spots distributed across all map quadrants
8. [ ] Foraging bushes and fruits spawn in resource layer
9. [ ] All parameters configurable via `OpenRCT2TerrainConfig` and new config struct
10. [ ] Map validation prevents invalid maps from being used
11. [ ] No console errors
12. [ ] Existing tests still pass
13. [ ] New functionality verified via map_generator CLI and viewer

## QA Acceptance Criteria

**CRITICAL**: These criteria must be verified by the QA Agent before sign-off.

### Unit Tests
| Test | File | What to Verify |
|------|------|----------------|
| test_boundary_enforcement | `src/tilemap/world_generator.rs` | Perimeter tiles are correct types (deep→shallow→sand) |
| test_internal_water_transitions | `src/tilemap/world_generator.rs` | No deep water adjacent to land in interior |
| test_terrain_distribution | `src/tilemap/world_generator.rs` | Land >= 60%, grass+forest >= 50% of land |
| test_spot_noise_distribution | `src/tilemap/world_generator.rs` | Water spots in all quadrants |
| test_resource_spawning | `src/resources/mod.rs` | Foraging resources spawn on valid terrain |
| test_config_parameters | `src/tilemap/world_generator.rs` | All parameters affect generation output |

### Integration Tests
| Test | Services | What to Verify |
|------|----------|----------------|
| test_map_generation_full | world_generator ↔ chunk_manager | Full map generates without errors |
| test_resource_layer_integration | world_generator ↔ ResourceGenerator | Resources placed correctly on generated terrain |
| test_spawn_point_validity | world_generator ↔ simulation | Animals can spawn on generated map |

### End-to-End Tests
| Flow | Steps | Expected Outcome |
|------|-------|------------------|
| Map Generation | 1. Run map_generator CLI 2. Load saved map 3. Verify in viewer | Map displays correctly with proper boundaries |
| Simulation Start | 1. Generate map 2. Start simulation 3. Spawn animals | Animals spawn and move on valid terrain |
| Resource Foraging | 1. Start simulation 2. Observe herbivores 3. Check foraging behavior | Animals find and consume vegetation resources |

### Browser Verification (Web Viewer)
| Page/Component | URL | Checks |
|----------------|-----|--------|
| Viewer | `http://localhost:54321/viewer.html` | Map renders, boundaries visible, water/land distribution correct |
| Entity Rendering | `http://localhost:54321/viewer.html` | Animals spawn and move on land tiles |

### CLI Verification
| Command | Expected Output |
|---------|----------------|
| `cargo run --bin map_generator -- -n test -s 12345` | Map generated using clap CLI, statistics show proper distribution |
| `cargo run --bin map_generator -- -n test -s 12345 -v` | Verbose output showing boundary rules and validation |
| Simple CLI: `cargo run --bin map_generator generate test TestMap 12345` | Map generated (simple wrapper in src/bin/) |
| Simple CLI: `cargo run --bin map_generator list` | Lists saved maps with stats |

**Note**: Two map_generator binaries exist:
- `src/map_generator.rs` - Full-featured CLI with clap (radius, terrain-mode, verbose flags)
- `src/bin/map_generator.rs` - Simple wrapper with basic commands

### QA Sign-off Requirements
- [ ] Pre-implementation snapshot exists (git tag `pre-mapgen2.0`)
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] All E2E tests pass
- [ ] Browser verification complete
- [ ] CLI generates valid maps
- [ ] No regressions in existing functionality
- [ ] Code follows established patterns
- [ ] No security vulnerabilities introduced
- [ ] Map boundary rules enforced correctly
- [ ] Terrain distribution meets targets (60% land, 50% green)
- [ ] System restored to working state comparable to pre-implementation
