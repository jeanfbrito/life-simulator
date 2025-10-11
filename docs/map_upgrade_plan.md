# Map Upgrade Task Plan: Shrubs & Collectables Integration

## Context & Goals
- Introduce shrub and collectable vegetation types using the existing resource layer so they compete for tile occupancy with trees.
- Extend the ResourceGrid biomass system to recognize richer plant guilds (ground cover, shrubs, mushrooms) without adding a new layer.
- Keep collectable resources ready for future gameplay systems while preserving current fauna behaviour.

## Guiding Principles
- Prefer reuse of current ResourceGrid and ResourceType infrastructure over new layers to avoid data duplication.
- Keep terrain-driven spawning deterministic per chunk using the existing seed pipeline.
- Document validation steps for each task so we can verify map upgrades before enabling new interactions.

## Task Breakdown & Validation

### 1. Expand Resource Taxonomy
**Description**: Add shrub and collectable entries (e.g., `BerryBush`, `HazelShrub`, `MushroomPatch`, `WildRoot`) to `ResourceType` and ensure string conversions, serialization, and defaults cover them.
**Implementation Steps**:
- Update `ResourceType` enum, `as_str`, and `from_str` in `src/resources.rs`.
- Add default density knobs to `ResourceConfig` for new entries.
- Create a central metadata map `RESOURCE_DEFINITIONS` describing category (`shrub`, `collectable`, `tree`), nutritional value, regrowth profile, and tool requirements.
**Validation**:
- Unit test: round-trip conversions for new types (`ResourceType::from_str` ↔ `as_str`).
- Unit test: default config exposes non-negative density values for each resource class.
- Static assert: every enum variant has a metadata entry defined (catch missing cases at compile time with `lazy_static!` lookup or helper macro).

### 2. Biome-Aware Resource Generation
**Description**: Extend `ResourceGenerator::determine_resource_for_terrain` to place shrubs/collectables with biome-aware probabilities.
**Implementation Steps**:
- Accept biome context (Forest vs Taiga vs Swamp) when generating chunk resources; derive from `Chunk::biome` or pass as parameter.
- Configure per-biome density multipliers in `ResourceConfig` (e.g., berry shrubs favored in forest edges, mushroom patches in forest/swamp tiles).
- Ensure only one resource occupies a tile by ordering placement priority (trees > shrubs > collectables > flowers).
**Validation**:
- Property test: generate multiple resource layers for a fixed seed and confirm probability distributions fall within expected ranges (± tolerance).
- Snapshot test: record representative chunk outputs for each biome to catch accidental regression in spawn tables.
- Performance check: run generator across 100 chunks and assert runtime stays within current benchmarks (log execution time in doc).

### 3. Resource Metadata & ResourceGrid Sync
**Description**: Connect each `ResourceType` to a `HarvestProfile` that the ResourceGrid can read to set `max_biomass`, `growth_rate_modifier`, and harvest rules.
**Implementation Steps**:
- Define `HarvestProfile { biomass_cap, growth_rate_multiplier, harvest_yield, regrowth_delay_ticks, consumption_kind }`.
- Update `ResourceGrid::get_or_create_cell` to pull `HarvestProfile` for the tile's resource and adjust new cells accordingly.
- Expose helper `ResourceGrid::apply_profile(pos, profile)` for manual overrides (e.g., scripted events).
**Validation**:
- Unit test: creating a cell with a mushroom profile sets low biomass cap and long regrowth delay.
- Integration test: simulate consumption events on berry/shrub profiles and confirm regrowth scheduler picks up custom delays.
- Metrics validation: confirm `ResourceGridMetrics` reports new cell types by extending debug logging (ensure no panic on unknown profile).

### 4. Herbivore Interaction Updates
**Description**: Allow existing herbivores to recognize shrubs as higher-quality forage while preventing them from harvesting collectables meant for gathering.
**Implementation Steps**:
- Extend herbivore forage queries to read `HarvestProfile::consumption_kind`; only allow `HerbivoreBrowse` types.
- Adjust diet preferences so deer prioritize shrubs when ground cover is low; rabbits may sample berry shrubs but at a cap.
- Update consumption logic to deduct from ResourceGrid using the proper `max_fraction` for shrub biomass.
**Validation**:
- Behavior test: run existing herbivore simulation and ensure no panic; monitor log for new forage decisions.
- Metrics: track biomass consumption split by resource kind for 10k ticks and confirm shrubs are utilized but not depleted instantly.
- Regression: ensure mushrooms/collectables remain untouched by herbivores (add assertion in AI tests).

### 5. Collectable Harvest Pipeline
**Description**: Implement groundwork so collectable resources can be harvested via generic gameplay systems without impacting current fauna.
**Implementation Steps**:
- Add `CollectAction` scaffolding that reads `HarvestProfile` (yield into inventory, mark tile cooldown).
- Define storage payload (`HarvestItem` struct) for inventory integration; keep behind feature flag if required.
- Document API for future collection behaviours (`get_collectable_targets(radius, filters)`).
**Validation**:
- Unit test: `CollectAction` reduces ResourceGrid biomass and returns expected item quantity.
- Serialization check: save/load cycle preserves collectable state (add to existing savegame tests if any).
- Manual validation: trigger a collection event via debug command and inspect ResourceGrid metrics.

### 6. Tooling & Visualization
**Description**: Expose new resources in debug overlays and any web viewer.
**Implementation Steps**:
- Extend biomass heatmap to differentiate shrubs vs ground cover (e.g., color-coding or legend entry) without extra layers.
- Update any CLI/world-inspection commands to list new resource categories.
- Refresh documentation references (README, viewer docs) to mention shrubs + collectables.
**Validation**:
- Visual check: run web viewer overlay and confirm legend, color mapping matches new categories.
- CLI regression: ensure existing diagnostics still output success (run `phase5_verification.sh` or equivalent).
- Doc lint: ensure README snippets compile if using mdbook or doctest tooling.

### 7. Balancing & Tuning Pass
**Description**: Fine-tune densities and growth parameters so ecosystems stay stable across biomes.
**Implementation Steps**:
- Run multi-biome simulations (Plains, Forest, Swamp, Taiga) for 50k ticks capturing biomass histograms.
- Adjust density knobs to avoid overpopulation of shrubs that choke tree spawns.
- Document recommended config ranges in the new plan.
**Validation**:
- Produce balancing report with key metrics (average biomass per tile, shrub/tree ratio) and check against targets (define thresholds in doc).
- Ensure CPU metrics remain within budgets (ResourceGrid processing time < current 2ms target).
- Verify herbivore populations remain healthy (no starvation spikes) using existing monitoring scripts.

### 8. Release Checklist
**Description**: Aggregate validation into a checklist before merging the map upgrade.
**Checklist Items**:
- [ ] Unit/integration tests from tasks 1–5 green.
- [ ] Snapshot diffs approved by art/design (resource distribution).
- [ ] Simulation soak tests completed with metrics archived in `/docs/balance-reports`.
- [ ] Debug UI updated and verified.
- [ ] Documentation refreshed (this plan + README updates).

## Validation Summary Table
| Task | Key Tests | Manual Checks | Metrics Targets |
| --- | --- | --- | --- |
| Taxonomy | `resources::types_roundtrip` | None | N/A |
| Generator | `resource_generator::probability_window` | Snapshot review | ≤ current generation time |
| Metadata Sync | `resource_grid::profile_assignment` | Inspect debug logs | No "unknown profile" warnings |
| Herbivores | `herbivore::forage_preferences` | Observe grazing sim | Shrub biomass ≥ 30% after 10k ticks |
| Collectables | `collect_action::yield_flow` | Trigger debug collect | Regrowth delay honored |
| Tooling | `viewer::legend_includes_shrubs` | Web overlay | Heatmap render < 16ms |
| Balancing | `balancing::biome_metrics` | Review balance report | ResourceGrid CPU < 2ms |

## Deliverables
- Code changes implementing tasks 1–7.
- Automated tests and scripts referenced above.
- Balance report artifacts in `/docs/balance-reports` once tuning completes.
- Updated documentation (README, viewer docs).

## Open Questions for Future Iterations
- Do we need separate densities for edge tiles (transition between forest and plains)?
- Should tree regeneration compete dynamically with shrubs (e.g., shade suppression)?
- Will collectable items require rarity tiers or spoilage timers once active gathering systems arrive?
- How do weather patterns or future season mechanics influence shrub vs ground-cover balance?

