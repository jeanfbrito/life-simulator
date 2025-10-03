# Species Architecture Roadmap

The current codebase is functionally correct, but several systems still hard-code rabbits, deer, and raccoons. The tasks below describe how to finish the modularization work so new species can plug in with minimal engine changes. Each item includes success criteria and regression checks so functionality stays intact while refactoring.

## 1. Species Spawn & Stat Registration
**Problem:** `src/entities/entity_types.rs` manually stitches reproduction stats, behavior configs, and bundles per species.

**Task:**
- Move the spawn logic (stats bundle, needs, behavior config, reproduction config, sex roll) into per-species modules or a `SpeciesRegistry` that exposes a `SpeciesDescriptor` (spawn function, emoji, juvenile naming, etc.).
- Refactor `spawn_rabbit/deer/raccoon` to query the registry so the core file only loops over descriptors.

**Verification:**
- `cargo check` and `cargo fmt`.
- Run the sim, spawn all three species, and confirm adults/juveniles keep their stats and reproduction configs (viewer + logs).

## 2. Plugin Wiring via Registry
**Problem:** `src/entities/mod.rs` and `src/ai/mod.rs` still list each species’ birth/mate/plan systems manually.

**Task:**
- Add a central `SpeciesSystems` descriptor (mate system, birth system, planner system) populated by each species module.
- Update the entity and AI plugins to iterate the descriptors when registering systems.

**Verification:**
- `cargo check` to ensure schedules register dynamically.
- Run the sim to confirm mate matching plus birth still fire for all species (watch logs for 💞/🍼 messages).

## 3. Follow Behaviour Generalisation
**Problem:** `src/ai/behaviors/follow.rs` and deer planning still target rabbits explicitly.

**Task:**
- Convert follow helper to accept a slice of candidate entities supplied by the species planner (e.g., mothers, herds).
- Update deer planner to pass its candidate list; ensure rabbits/raccoons opt in only when needed.

**Verification:**
- Unit-like smoke test: log output confirming deer still follow their mother after birth.
- Observe runtime: fawns should keep up with mothers just like before.

## 4. Viewer Metadata From Data
**Problem:** Viewer legend, ordering, juvenile scaling, and emojis in `web-viewer/js/config.js`, `entity-stats.js`, `renderer.js` are hand-coded per species.

**Task:**
- Serve species metadata from the backend (e.g., extend API or emit JSON file) containing emoji, color, scale, ordering, stat sections, juvenile flag.
- Update viewer JS to consume the data instead of switch statements.

**Verification:**
- Load viewer in browser, ensure legend/tooltips still show correct info for rabbits, deer, raccoons.
- Verify juvenile scaling and ordering remain unchanged.

## 5. Demo Bootstrap Configuration
**Problem:** `src/main.rs` demo script hardcodes rabbit setup messages and spawn counts.

**Task:**
- Move demo spawns to a config file or registry-driven loop so new species can be included without editing `main.rs`.
- Keep current behaviour (still spawn rabbits/deer/raccoons) but source data from config.

**Verification:**
- Run binary; console output should match current wording, and entities should spawn as before.

## 6. Documentation & Tests
**Problem:** Architectural expectations aren’t recorded; regressions rely on manual QA.

**Task:**
- Document the registry pattern and planner hooks in `CLAUDE.md` or a new design doc.
- Add at least one integration test that spawns each species and steps reproduction cycles to guard future refactors.

**Verification:**
- `cargo test` for new coverage.
- Review doc updates for clarity.

---
Please tackle tasks incrementally, verifying behaviour after each to ensure parity with current functionality.
