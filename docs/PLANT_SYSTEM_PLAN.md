# Plant System Implementation Plan

This plan breaks the proposed vegetation layer into incremental milestones. Each section lists the goal, concrete tasks, reference files, and verification steps so we can ship a performant plant loop without seasons or day/night cycles.

---

## Phase 0 – Research & Parameter Capture

**Objective:** Lock down constants and integration points before touching runtime code.

**Tasks**
1. **Consolidate parameters**
   - Document target growth rate `r`, `Bmax`, and per-species consumption constants in a shared module (e.g., `src/vegetation/constants.rs`).
   - Capture the 0.3·B per-meal throttle, search radii, and predator fear multipliers as configurable values.
2. **Audit existing hunger actions**
   - Review `src/ai/herbivore_toolkit.rs` and species planners to confirm where food lookups are injected.
   - Identify where to insert biomass queries (likely alongside current auto-eat/pasture logic).
3. **Decide storage layout**
   - Choose between `HashMap<TileIndex, TileVegetation>` vs. dense grid aligned with the map loader.
   - Outline memory expectations for typical map sizes.

**Verification**
- Parameter doc committed (could be `docs/PLANT_SYSTEM_PARAMS.md`) with the chosen constants.
- ADR or short write-up in PR description explaining storage choice.

---

## Phase 1 – Core Data Model & Growth Loop

**Objective:** Introduce `TileVegetation { biomass, last_grazed_tick }` and logistic growth without consuming it yet.

**Tasks**
1. **Data structure**
   - Add a new module `src/vegetation/mod.rs` exposing:
     ```rust
     pub struct TileVegetation {
         pub biomass: f32,
         pub last_grazed_tick: u64,
     }
     pub struct VegetationGrid { /* storage + active set */ }
     ```
   - Implement helper methods: `add_biomass`, `remove_biomass`, `fraction_full`, `mark_grazed`.
2. **Resource wiring**
   - Register `VegetationGrid` as a Bevy resource during Startup in `EntitiesPlugin` or a dedicated `VegetationPlugin`.
   - Seed initial biomass to `Bmax` for all tiles (or lazy-load on first access).
3. **Growth system**
   - Add `growth_system` scheduled at 1 Hz (every 10 ticks). Use `r·B·(1 − B/Bmax)` and clamp to `[0,Bmax]`.
   - Support two strategies: update all tiles OR update `active_tiles + random sample`. Start with all tiles for correctness, keep TODO for sparse variant.
4. **Debug overlay scaffolding**
   - Expose a `fn sample_biomass(tile: IVec2) -> f32` API for eventual viewer overlay.

**Verification**
- `cargo check` and `cargo test` pass.
- Unit test covering logistic growth (e.g., empty patch reaches ~80% Bmax after expected ticks).
- Optional: temporary log every 60 seconds printing mean biomass to ensure growth is happening.

---

## Phase 2 – Herbivory Consumption Hook

**Objective:** Tie the new vegetation grid into existing eat/graze actions.

**Tasks**
1. **Consumption API**
   - Provide `VegetationGrid::consume(tile, requested_amount, max_fraction)` returning `(consumed, remaining)`.
   - Apply the `min(species_intake, 0.3 * B)` rule and update `last_grazed_tick`.
2. **Planner integration**
   - In `herbivore_toolkit::evaluate_eating_behavior` (and any grazing helpers), replace the constant “grass” availability check with vegetation queries:
     - When evaluating candidate tiles, read biomass; if below threshold, lower utility or skip.
     - On action execution, call `consume` to deduct biomass and feed the animal only the amount actually obtained.
3. **Fallback behaviour**
   - Decide what happens when biomass is insufficient (< demand): animal should either queue another forage action or mark the patch as depleted.
   - Possibly add a short cooldown before re-evaluating the same tile.
4. **Logging / instrumentation**
   - Emit debug logs when animals encounter depleted patches for tuning.

**Verification**
- Integration test that spawns a rabbit, drains a tile, and asserts biomass reduction matches rules.
- Manual run observing animals leaving low-biomass areas (confirm via logs or overlay).

---

## Phase 3 – Patch Selection & Behaviour Tweaks

**Objective:** Improve foraging realism using biomass-aware search and giving-up rules.

**Tasks**
1. **Search ranking**
   - Modify species planners to request candidate tiles from `VegetationGrid`; rank by `B / (1 + distance_penalty)` using existing search radii.
   - Option: sample K random tiles within radius for O(K) lookup.
2. **Giving-up density**
   - Track per-forage action how much biomass remains when the animal leaves. If it falls below `quitting_threshold` (e.g., 20 units), automatically queue a move to the next patch.
3. **Feeding duration**
   - Adjust action durations to reflect biomass availability: fewer ticks spent when low B, more when high.
4. **Optional predator fear**
   - Stub a utility modifier that reduces feeding duration by 20–40% when a predator entity or scent component is within R tiles. Keep it data-driven so it can be toggled later.

**Verification**
- Behavioural test: spawn a grassy plain, add one predator near deer, observe shortened feeding sessions (via logs).
- Visual confirmation (with overlay) that grazing halos form around resting spots.

---

## Phase 4 – Performance Optimisation

**Objective:** Ensure the vegetation layer stays lightweight at scale.

**Tasks**
1. **Active tile tracking**
   - Maintain a queue/set of tiles whose biomass < Bmax or were grazed in last N seconds.
   - Growth system should process only active tiles plus a small random sample of others each tick.
2. **Batch updates**
   - Profile growth system (with Bevy diagnostics or `bevy_mod_debugdump`).
   - If necessary, switch to updating a fixed batch size per tick to smooth CPU usage.
3. **Memory tuning**
   - Evaluate storing biomass as `f32` vs `u16` depending on precision needs.
   - Consider region-based grids to align with map chunks for cache-friendly access.
4. **Viewer overlay**
   - Extend `/api/` to expose biomass heatmaps for debugging (e.g., normalized values).

**Verification**
- Benchmarks showing growth system stays under budget at 10 TPS on target map sizes.
- Manual inspection of overlay confirms no “frozen” tiles when random tick strategy is active.

---

## Phase 5 – Scenario Tuning & Regression Suite

**Objective:** Validate ecological feedbacks and guard against future regressions.

**Tasks**
1. **Scenario tests**
   - Ungrazed regrowth: start empty map, no herbivores, ensure biomass approaches Bmax at target timeline.
   - Rabbit-only vs rabbit+fox: confirm vegetation rebounds with predator fear penalty toggled.
2. **Metrics dashboard**
   - Add debug counters (e.g., average biomass, number of depleted tiles) to logging output every few in-game minutes.
3. **Documentation**
   - Update `docs/SPECIES_REFERENCE.md` with biomass consumption mapping.
   - Add plant system overview (this plan) to main documentation index.

**Verification**
- All new tests pass (`cargo test plant::*` etc.).
- Documented tuning notes committed for future species additions.

---

## Stretch Goals (Post MVP)

- Introduce shrub tier for browsers (deer fallback) with separate regrowth rate.
- Add seasonal modifiers (global scalar on `r` and `Bmax`).
- Persist vegetation state in save files (extend serialization module).
- Emit scent components when predators linger to drive the fear penalty with decay over time.

---

**Legend**
- 📄 = Documentation change
- 🧪 = Test to add
- 🐛 = Verification target (manual or automated)

Keep each phase in its own PR when possible for easier review.
