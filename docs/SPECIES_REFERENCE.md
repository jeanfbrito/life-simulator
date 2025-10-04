# Species Reference Guide

This document summarises the behaviour, reproduction numbers, and viewer metadata for every playable species currently wired into the Life Simulator. Treat it as a field manual when introducing new creatures—match the patterns below to keep the simulation balanced.

All timings below assume the current simulation rate of **10 ticks per second**.

---

## Rabbits 🐇

**Identity & Viewer**
- Emoji / label: `🐇`
- Movement speed: **20** ticks-per-tile (fastest herbivore in the sim)
- Default wander radius: **15** tiles
- Juvenile name prefix: **Bunny**
- Viewer scale & colour: **0.5×**, `#8b4513`

**Reproduction** (`src/entities/types/rabbit.rs`)
- Adult at **3 600 ticks** (~6 min)
- Gestation: **1 200 ticks** (~2 min)
- Male cooldown: **600 ticks** (~1 min); Female postpartum: **1 800 ticks** (~3 min)
- Litter size: **2–6 kits**
- Mating search radius: **50** tiles; re-evaluated every **50** ticks (~5 s)
- Mating duration: **30 ticks** (~3 s)
- Eligibility gates:
  - Energy ≥ **0.50**
  - Health ≥ **0.60**
  - Well-fed streak ≥ **300 ticks** while hunger/thirst ≤ **0.35** normalised

**Behaviour thresholds**
- Drink when ≥ **75 %** thirsty
- Eat when ≥ **50 %** hungry
- Rest when energy ≤ **30 %**
- Graze range: **3–8** tiles (prefers short hops)
- Search radius for food/water: **100** tiles each

**Stats & Needs**
- Hunger pool: max **70**; drains **0.08**/tick; eats **28** on a meal
- Thirst pool: max **90**; drains **0.03**/tick; drinks **70** per visit
- Energy pool: max **100**; drains **0.07**/tick; Health regen **+0.01**/tick

**AI Planner Hooks**
- Mate action: utility **0.45**, priority **350**, tolerance margin **±0.05** for hunger/thirst, energy slack **+0.05**
- Follow mother: stop within **2** tiles, chase up to **20** tiles

**Demo spawn defaults**
- Config spawns **5** adults around `(0,0)` with reusable names.
- `sex_sequence = None` → sexes are randomised by the spawn helper.

---

## Deer 🦌

**Identity & Viewer**
- Emoji / label: `🦌`
- Movement speed: **10** ticks-per-tile
- Wander radius: **40** tiles
- Juvenile name prefix: **Fawn**
- Viewer scale & colour: **0.9×**, `#a0522d`

**Reproduction**
- Adult at **12 000 ticks** (~20 min)
- Gestation: **6 000 ticks** (~10 min)
- Male cooldown: **2 000 ticks** (~3.3 min); Female postpartum: **9 000 ticks** (~15 min)
- Litter size: **1–2 fawns**
- Mating search radius: **60** tiles; matcher runs every **300** ticks (~30 s)
- Mating duration: **50 ticks** (~5 s)
- Eligibility gates:
  - Energy ≥ **0.35**
  - Health ≥ **0.40**
  - Well-fed streak ≥ **600 ticks** while hunger/thirst ≤ **0.55**

**Behaviour thresholds**
- Drink when ≥ **65 %** thirsty
- Eat when ≥ **45 %** hungry
- Rest when energy ≤ **30 %**
- Graze range: **5–15** tiles
- Search radius for food & water: **150** tiles

**Stats & Needs**
- Hunger pool: max **300**; drains **0.05**/tick; eats **60** per graze
- Thirst pool: max **300**; drains **0.02**/tick; drinks **150** each trip
- Energy drain **0.04**/tick; Health regen **+0.01**/tick

**AI Planner Hooks**
- Mate action: utility **0.45**, priority **350**, same slack as rabbits
- Follow mother: hold within **2** tiles, pursue up to **25** tiles (keeps fawns close)

**Demo spawn defaults**
- Config spawns a **two-deer pair** (Stag/Doe) with explicit `sex_sequence: [Male, Female]`.
- Messages announce successful pair placement for quick QA.

---

## Raccoons 🦝

**Identity & Viewer**
- Emoji / label: `🦝`
- Movement speed: **16** ticks-per-tile
- Wander radius: **25** tiles
- Juvenile name prefix: **Kit**
- Viewer scale & colour: **0.65×**, `#696969`

**Reproduction**
- Adult at **6 000 ticks** (~10 min)
- Gestation: **3 600 ticks** (~6 min)
- Male cooldown: **1 800 ticks** (~3 min); Female postpartum: **5 400 ticks** (~9 min)
- Litter size: **2–4 kits**
- Mating search radius: **50** tiles; matcher runs every **240** ticks (~24 s)
- Mating duration: **40 ticks** (~4 s)
- Eligibility gates:
  - Energy ≥ **0.40**
  - Health ≥ **0.40**
  - Well-fed streak ≥ **480 ticks** while hunger/thirst ≤ **0.50**

**Behaviour thresholds**
- Drink when ≥ **55 %** thirsty
- Eat when ≥ **45 %** hungry
- Rest when energy ≤ **30 %**
- Forage range: **4–12** tiles
- Search radius for food & water: **120** tiles

**Stats & Needs**
- Hunger pool: max **180**; drains **0.06**/tick; eats **45** each meal
- Thirst pool: max **150**; drains **0.04**/tick; drinks **90** per visit
- Energy drain **0.05**/tick; Health regen **+0.01**/tick

**AI Planner Hooks**
- Mate action: utility **0.42**, priority **320**, same hunger/thirst/energy slack as other herbivores
- Follow mother: hold within **2** tiles, pursue up to **18** tiles (tight family range)

**Demo spawn defaults**
- Config spawns a **pair** (Bandit/Maple) with `sex_sequence: [Male, Female]` near `(10,10)`.
- Optional logging describes placements when `verbose_logging` is enabled.

---

### Using this guide when adding a new species

1. **Clone a template** – copy one of the behaviour modules (`src/entities/types/`) and adjust the reproduction, behaviour, and stat sections. Use the tables above to decide on ranges.
2. **Register metadata** – add your species descriptor in `src/entities/registry.rs` (emoji, viewer colour, speed, spawn fn).
3. **Planner wiring** – create `plan_<species>_actions` mirroring the mate/follow parameters that suit your animal.
4. **Spawn config** – update `config/spawn_config.ron` (or its default in `SpawnConfig::default()`) with counts, names, and optional sex patterns.
5. **Viewer vibes** – ensure `/api/species` reflects the emoji/scale so the browser client renders juveniles correctly.

Match the numbers above when you want comparable behaviour, or push them deliberately to explore new ecosystems.
