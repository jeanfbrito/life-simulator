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

## Bears 🐻

**Identity & Viewer**
- Emoji / label: `🐻`
- Movement speed: **12** ticks-per-tile
- Wander radius: **80** tiles (wide roaming range)
- Juvenile name prefix: **Cub**
- Viewer scale & colour: **1.2×**, `#3b2f2f`

**Reproduction**
- Adult at **18 000 ticks** (~30 min)
- Gestation: **6 000 ticks** (~10 min)
- Male cooldown: **8 000 ticks**; Female postpartum: **12 000 ticks**
- Litter size: **1–3 cubs**
- Mating search radius: **90** tiles; matcher runs every **420** ticks (~42 s)
- Mating duration: **60 ticks** (~6 s)
- Eligibility gates:
  - Energy ≥ **0.55**
  - Health ≥ **0.55**
  - Well-fed streak ≥ **900 ticks** while hunger/thirst ≤ **0.45**

**Behaviour thresholds**
- Drink when ≥ **40 %** thirsty
- Eat when ≥ **40 %** hungry
- Rest when energy ≤ **30 %**
- Forage range: **6–18** tiles (berry patches and shrubs)
- Search radius for food & water: **150** tiles each

**Stats & Needs**
- Hunger pool: max **350**; drains **0.05**/tick; eats **120** per meal
- Thirst pool: max **250**; drains **0.03**/tick; drinks **150** per visit
- Energy drain **0.05**/tick; Health regen **+0.01**/tick

**AI Planner Hooks**
- *Scavenge*: priority **400**, utility scales with hunger and carcass proximity (150-tile scan)
- *Hunt fawn*: priority **320**, seeks deer fawns within 20 tiles when hunger ≥ 0.6
- Herbivore toolkit still provides baseline drink/rest/forage actions

**Demo spawn defaults**
- Default config spawns **1** adult ("Kodiak") near `(-25,18)`.
- Bears do not follow the auto-graze system; they rely on scavenging and omnivore foraging.

---

## Foxes 🦊

**Identity & Viewer**
- Emoji / label: `🦊`
- Movement speed: **16** ticks-per-tile
- Wander radius: **40** tiles
- Juvenile name prefix: **Kit**
- Viewer scale & colour: **0.6×**, `#c1440e`

**Reproduction**
- Adult at **10 500 ticks** (~17.5 min)
- Gestation: **4 500 ticks** (~7.5 min)
- Male cooldown: **4 000 ticks**; Female postpartum: **6 000 ticks**
- Litter size: **3–5 kits**
- Mating search radius: **120** tiles; matcher runs every **360** ticks (~36 s)
- Mating duration: **50 ticks** (~5 s)
- Eligibility gates:
  - Energy ≥ **0.50**
  - Health ≥ **0.60**
  - Well-fed streak ≥ **600 ticks** while hunger/thirst ≤ **0.50**

**Behaviour thresholds**
- Drink when ≥ **50 %** thirsty
- Hunt/forage when ≥ **50 %** hungry
- Rest when energy ≤ **30 %**
- Search radius for food: **160** tiles (targets rabbits first)

**Stats & Needs**
- Hunger pool: max **180**; drains **0.08**/tick; eats **60** per meal
- Thirst pool: max **150**; drains **0.04**/tick; drinks **90** per visit
- Energy drain **0.06**/tick; Health regen **+0.01**/tick

**AI Planner Hooks**
- *Hunt rabbit*: priority **360**, hunts closest rabbit within ~60 tiles when hungry
- *Scavenge*: priority **320**, opportunistic carrion cleanup with 150 tile scan
- Baseline planner contributions (drink/rest) sourced from herbivore toolkit without grazing

**Demo spawn defaults**
- Config spawns a fox pair ("Saffron" & "Russet") near `(5,-12)` with mixed sexes.

---

## Wolves 🐺

**Identity & Viewer**
- Emoji / label: `🐺`
- Movement speed: **12** ticks-per-tile
- Wander radius: **200** tiles (territorial patrols)
- Juvenile name prefix: **Pup**
- Viewer scale & colour: **0.9×**, `#666666`

**Reproduction**
- Adult at **14 000 ticks** (~23.3 min)
- Gestation: **4 500 ticks** (~7.5 min)
- Male cooldown: **7 000 ticks**; Female postpartum: **10 000 ticks**
- Litter size: **2–4 pups**
- Mating search radius: **160** tiles; matcher runs every **480** ticks (~48 s)
- Mating duration: **60 ticks** (~6 s)
- Eligibility gates:
  - Energy ≥ **0.60**
  - Health ≥ **0.60**
  - Well-fed streak ≥ **900 ticks** while hunger/thirst ≤ **0.55**

**Behaviour thresholds**
- Drink when ≥ **55 %** thirsty
- Hunt when ≥ **45 %** hungry
- Rest when energy ≤ **25 %**
- Food search radius: **220** tiles (pack hunts target deer)

**Stats & Needs**
- Hunger pool: max **260**; drains **0.07**/tick; eats **100** per meal
- Thirst pool: max **200**; drains **0.04**/tick; drinks **120** per visit
- Energy drain **0.06**/tick; Health regen **+0.01**/tick

**AI Planner Hooks**
- *Pack hunt deer*: priority **420**, hunts nearest adult deer within ~200 tiles
- *Scavenge*: priority **300**, claims carcasses when hunger ≥ 0.35
- Wolves share the drink/rest logic from the herbivore toolkit but skip grazing

**Demo spawn defaults**
- Default config spawns a **pack of three** (Luna, Ash, Bran) near `(-60,-40)`.
- Pack hunts create carcasses that bears/foxes can scavenge, tightening energy loops.

---

## Biomass Consumption Mapping (Phase 5 Metrics)

The following sections detail how each species interacts with the vegetation system for the Phase 5 metrics dashboard. All consumption values are per feeding action and impact the vegetation biomass tracking system.

### Rabbits 🐇
- **Biomass per graze**: **2.8 units** (28 hunger units ÷ 10 conversion ratio)
- **Graze frequency**: Every **50-80 ticks** when hungry ≥ 50%
- **Daily biomass impact**: ~**8-12 units** per rabbit (3-4 grazes per day at 10 TPS)
- **Preferred terrain**: Grass and Forest tiles (terrain multiplier: 1.0-1.2)
- **Foraging pattern**: Short hops within 3-8 tiles, creates localized grazing pressure

### Deer 🦌
- **Biomass per graze**: **6.0 units** (60 hunger units ÷ 10 conversion ratio)
- **Graze frequency**: Every **100-150 ticks** when hungry ≥ 45%
- **Daily biomass impact**: ~**24-36 units** per deer (4-6 grazes per day)
- **Preferred terrain**: Grass and Forest tiles (terrain multiplier: 1.0-1.2)
- **Foraging pattern**: Medium distance grazing within 5-15 tiles, moderate territory pressure

### Raccoons 🦝
- **Biomass per forage**: **4.5 units** (45 hunger units ÷ 10 conversion ratio)
- **Forage frequency**: Every **75-120 ticks** when hungry ≥ 45%
- **Daily biomass impact**: ~**12-18 units** per raccoon (3-4 forages per day)
- **Preferred terrain**: All vegetated tiles (generalist forager)
- **Foraging pattern**: Opportunistic within 4-12 tiles, varied grazing pressure

### Biomass Impact Metrics

The Phase 5 metrics dashboard tracks the following biomass consumption patterns:

- **Total Biomass Consumed**: Cumulative consumption across all herbivores
- **Consumption Rate**: Average biomass units consumed per tick
- **Grazing Pressure**: Number of active feeding actions per simulation tick
- **Terrain Impact**: Biomass depletion patterns by terrain type
- **Species Impact**: Per-species consumption contribution to total depletion

### Vegetation Recovery Parameters

- **Growth Rate**: 0.02 per tick on suitable tiles (logistic growth model)
- **Carrying Capacity**: Varies by terrain (Grass: 100, Forest: 120, Desert: 40)
- **Recovery Time**: 50-200 ticks to reach 80% carrying capacity after grazing
- **Active Tile Tracking**: Recently grazed tiles (< 100 ticks) get priority growth updates

### Using this guide when adding a new species

1. **Clone a template** – copy one of the behaviour modules (`src/entities/types/`) and adjust the reproduction, behaviour, and stat sections. Use the tables above to decide on ranges.
2. **Register metadata** – add your species descriptor in `src/entities/registry.rs` (emoji, viewer colour, speed, spawn fn).
3. **Planner wiring** – create `plan_<species>_actions` mirroring the mate/follow parameters that suit your animal.
4. **Spawn config** – update `config/spawn_config.ron` (or its default in `SpawnConfig::default()`) with counts, names, and optional sex patterns.
5. **Viewer vibes** – ensure `/api/species` reflects the emoji/scale so the browser client renders juveniles correctly.
6. **Biomass mapping** – calculate consumption impact using: `hunger_pool_consumed ÷ 10 = biomass_units_consumed`

Match the numbers above when you want comparable behaviour, or push them deliberately to explore new ecosystems.
