# OpenRCT2 Map Generation Port

_Last updated: 2025-01-06_

This document tracks the work required to port OpenRCT2's map generation pipeline into the life-simulator backend so our exported worlds match the original game's behaviour exactly. It breaks the effort into manageable steps, records decisions, and gives each sub-task a place to live.

---

## Current Snapshot

- **Scaffolding ready**: `src/tilemap/openrct2/` now mirrors the foundational pieces from OpenRCT2 (settings, dense height map, simplex/fBm noise, smoothing).
- **Chunk heights updated**: `WorldGenerator::generate_height_chunk_openrct2()` now uses the dense pipeline, exports shoreline-adjusted heights, and matches OpenRCT2’s 16-unit quantisation.
- **Legacy terrain still active**: `WorldGenerator::generate_chunk` keeps using the heuristic terrain/resource logic; water tiles and surface data still pending.
- **Slope metadata available**: Height generation now emits both corner masks and 0–18 slope indices (`slope_indices` layer in serialized chunks).
- **Simulation default map**: Headless runs now target `maps/slopes_demo.ron` by default (override with `WORLD_MAP_NAME`).
- **Viewer expectations**: Godot viewer still infers slopes and terrain from exported chunk JSON; it must eventually consume canonical data generated here.

---

## Porting Roadmap

### 1. Height Map Pipeline (DONE)

- [x] Replace the current `generate_height_chunk_openrct2()` with the new dense height map flow:
  - build a `HeightMap` at density 2;
  - run simplex noise + smoothing (OpenRCT2 parameters);
  - downsample into tile heights with `BaseHeight = max(2, average * 2)`.
- [x] Store quantised heights (multiples of 16) in chunk metadata rather than regenerating them client-side.

**Notes**
- Keep shoreline adjustment (`if height in [4, water_level] -> subtract 2`).
- Ensure exported RON/JSON includes the height grid for every chunk.

### 2. Slope Flags & Tile Metadata (IN PROGRESS)

- [x] Port `setMapHeight` slope-bit logic so each tile records OpenRCT2 corner flags (stored as 4-bit masks).
- [x] Export canonical slope indices (0–18) via the new `slope_indices` layer so downstream rendering can pick the correct sprite variant.
- [ ] Remove slope-guessing from Godot (`SlopeCalculator.gd`) once authoritative data flows through.

### 3. Water Levels & Beaches

- [ ] Implement `setWaterLevel` to populate water tiles/heights.
- [ ] Mirror `addBeaches` behaviour (surface switch for tiles near water).
- [ ] Export water metadata so the viewer can render water sprites correctly.

### 4. Surface / Edge Selection

- [ ] Port `SurfaceSelection` heuristics or author equivalent rust logic.
- [ ] Introduce surface/edge IDs in chunk data (needed for texture atlas parity with OpenRCT2).

### 5. Terrain Type Mapping

- [ ] Replace ad-hoc terrain noise with OpenRCT2’s thresholds driven by surface IDs and height.
- [ ] Ensure terrain types align with viewer assets (grass, sand, snow, etc.).

### 6. Optional Enhancements

- [ ] Tree placement (`TreePlacement.cpp`) parity.
- [ ] `smoothMap()` / `smoothTileStrong` cleanup for cliff reduction.
- [ ] Additional tooling (e.g., debug visualisers, comparison scripts).

---

## Open Questions

1. **Data format** – _Resolved_: we will focus solely on the new format (no backwards-compat mode). Target is a richer JSON/RON structure that mirrors OpenRCT2 metadata.
2. **Compatibility/versioning** – _Resolved_: legacy tools are out of scope; new format can evolve as needed.
3. **Performance** – Do we need caching/incremental updates once the generator mirrors OpenRCT2 complexity?

---

## Next Actions

1. Regenerate a sample world (`slopes_demo.ron`) to validate height & slope index layers end-to-end.
2. Update the Godot viewer to consume `heights` and `slope_indices`, removing the fallback `SlopeCalculator` implementation.
3. Begin plumbing water/beach metadata (Step 3) once viewer-side slope integration is stable.

---

## Reference Files

- OpenRCT2 source:
  - `src/openrct2/world/map_generator/SimplexNoise.cpp`
  - `src/openrct2/world/map_generator/MapGen.cpp`
  - `src/openrct2/world/map_generator/MapHelpers.cpp`
- Current Rust scaffolding:
  - `src/tilemap/openrct2/`
  - `src/tilemap/world_generator.rs`
  - `map_generator.rs` (CLI entry point)

---

_Add notes, findings, or design changes here as the port progresses._
