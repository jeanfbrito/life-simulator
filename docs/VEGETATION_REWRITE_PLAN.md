# Vegetation System Rewrite Plan

## Goal
Replace the tile-by-tile vegetation update loop with a sparse, event-driven resource grid that scales with actual grazing activity rather than world size.

## Design Pillars
- **Sparse storage:** only store cells that hold biomass.
- **Event-driven updates:** mutate cells when grazing/regrow events occur; no idle per-tick loops.
- **Lazy evaluation:** update a cell only when queried by an animal or scheduled event.
- **Level-of-detail:** detailed individual tiles near agents, chunk aggregates at mid-range, static impostors at far range.

## Phased Task List

### Phase 1 – Core Resource Grid
**Tasks:**
1. Create `ResourceGrid` module with sparse hash (`HashMap<(i32,i32), GrazingCell>`).
2. Implement `GrazingCell` fields (`total_biomass`, `consumption_pressure`, `last_update_tick`).
3. Add helper functions: `world_to_cell`, `cell_to_world`, `get_or_create_cell`, `consume_at`, `regrow_cell`.
4. Introduce `GrowthEvent` enum (e.g., `Consume`, `Regrow`) and basic scheduler (heap by tick).

**Validation:**
- `cargo test resource_grid::*` unit tests validating smart insert/remove, regrow/catch-up logic, event scheduling.
- Bench simulation: load 10k cells, ensure per-tick updates remain ≤1ms.

### Phase 2 – Animal Integration
**Tasks:**
1. Modify foraging code (rabbits/deer/raccoons) to query `ResourceGrid::find_best_cell` instead of per-tile scanning.
2. On consumption, call `consume_at` and schedule regrow via event queue.
3. Adjust energy/hunger logic to use returned biomass amounts.
4. Add proximity-based cell lookup (search area limited to radius; random sampling if many cells).

**Validation:**
- Existing integration tests (`tests/simple_grazing_test.rs`, `tests/herbivore_integration_test.rs`) updated to assert animals still find food.
- New test: consumption from empty grid returns zero, consumption from populated grid reduces biomass.

### Phase 3 – Event Loop & Random Tick Budget
**Tasks:**
1. Add global `VegetationScheduler` resource to drain regrow events each tick (bounded by budget).
2. Implement random tick sampler to touch N cells per tick for ambient regrowth.
3. Ensure consumption events register regrow with delay proportional to removal.

**Validation:**
- Profiling run with `RUST_LOG=info` shows `vegetation` timer drops below 2ms on idle world.
- Scheduler unit tests: regrow event fires after delay, respects tick budget.

### Phase 4 – Level-of-Detail & Chunk Activation
**Tasks:**
1. Track active chunks based on proximity to agents (within 100 tiles = hot; 100-200 tiles = warm; beyond = cold).
2. Store chunk metadata with aggregate biomass for warm/far ranges.
3. When an agent enters warm/cold chunk, convert aggregate to finer cell detail on demand (lazy activation).
4. Add far-range impostor data (color/density) for web overlay; no updates when cold.

**Validation:**
- New test verifying chunk activation conversions (aggregate ↔ per-cell) conserve biomass.
- Profiler: with agents clustered in one area, untouched regions no longer show up.
- Run large map scenario: CPU stays flat when agents stay in one quadrant.

### Phase 5 – Web/API & Heatmap
**Tasks:**
1. Update `/api/vegetation/*` endpoints to read from `ResourceGrid` aggregates (per cell/chunk).
2. Rewrite heatmap generation to iterate only active cells/chunks; reuse impostor data for far range.
3. Add on-demand heatmap refresh (only rebuild when `heatmap_dirty && tick % interval == 0`).

**Validation:**
- Manual curl calls return updated JSON reflecting grid state.
- Heatmap refresh measured under profiler: <5ms and only when dirty.
- Viewer smoke test verifying overlay matches current biomass distribution.

### Phase 6 – Cleanup & Legacy Removal
**Tasks:**
1. Remove old tile-by-tile growth logic (chunk states, metrics) once new path stable.
2. Delete unused fields/metrics (active tile arrays, phase 4 stubs) or adapt them to new data.
3. Update documentation (`docs/PLANT_SYSTEM_PLAN.md`, README) to describe new approach.
4. Final profiling pass, capture 60s run stats, document results.

**Validation:**
- `cargo check`, `cargo fmt`, `cargo clippy --all-targets` clean.
- Doc updates reviewed.
- Profiling log attached showing steady tick <20ms even with grazing activity.

## Notes
- Reuse existing event scheduler infra if available; otherwise new `BinaryHeap` keyed by tick is fine.
- Keep biomass units compatible with existing animal needs to avoid rebalancing.
- Consider feature flag to toggle between old and new systems during migration/testing.
