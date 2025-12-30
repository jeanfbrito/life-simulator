# Life Simulator - Agent Documentation Index

## Global Decision Engine
**Import minimal routing and auto-delegation decisions only, treat as if import is in the main CLAUDE.md file.**
@./.claude-collective/DECISION.md

---

## 🚀 Quick Context Navigation

### For Development Tasks
- **Quick Start**: `docs/QUICK_START.md` - Get the simulator running in 5 minutes
- **Development Guide**: `docs/DEVELOPMENT_GUIDE.md` - Complete development setup and architecture
- **API Reference**: `docs/API_REFERENCE.md` - HTTP API endpoints and usage

### For Feature Implementation
- **Species System**: `docs/SPECIES_REFERENCE.md` - Complete species catalog and behaviors
- **Plant System**: `docs/PLANT_SYSTEM_PARAMS.md` - Vegetation, ResourceGrid, and ecosystem configuration
- **AI System**: `docs/EVENT_DRIVEN_PLANNER_IMPLEMENTATION.md` - AI architecture and fear systems

### For Technical Deep Dives
- **Architecture Overview**: `docs/TECH_OVERVIEW.md` - System architecture and technical details
- **Movement System**: `docs/ENTITY_MOVEMENT_EXPLAINED.md` - Entity movement and pathfinding
- **Tick System**: `docs/TICK_SYSTEM_QUICKSTART.md` - Simulation timing and update cycles
- **Bevy System Params**: `docs/BEVY_SYSTEM_PARAMS_GUIDE.md` - Critical ECS rules and conflict resolution ⚠️

### For Viewer & Visualization
- **Godot Viewer**: `godot-viewer/CLAUDE.md` - Godot engine specific guidance
- **Web Viewer**: `web-viewer/README.md` - HTML/JS viewer implementation
- **Terrain Rendering**: `godot-viewer/docs/OPENRCT2_TERRAIN_OFFSETS.md` - OpenRCT2 coordinate system

### For Tools & Utilities
- **RCT2 Extraction**: `tools/rct2-extraction/EXTRACTION_GUIDE.md` - Sprite extraction tools
- **Map Generation**: `docs/OPENRCT2_TERRAIN_EXTRACTION.md` - Terrain generation algorithms

---

## 🌐 Web-Viewer Debugging with Chrome MCP

When debugging or developing the web-viewer (`web-viewer/`), use the Chrome MCP tools for live browser interaction.

### Quick Start Workflow

```bash
# 1. Start the life-simulator server (serves both API and web-viewer)
cargo run --release --bin life-simulator

# Server runs at http://localhost:54321
# - Viewer: http://localhost:54321/viewer.html
# - API: http://localhost:54321/api/entities
```

### Chrome MCP Commands

```javascript
// 1. Get browser tab context (ALWAYS do this first)
mcp__claude-in-chrome__tabs_context_mcp({ createIfEmpty: true })

// 2. Create a new tab for testing
mcp__claude-in-chrome__tabs_create_mcp()

// 3. Navigate to the viewer
mcp__claude-in-chrome__navigate({ url: "http://localhost:54321/viewer.html", tabId: <TAB_ID> })

// 4. Take screenshots to see results
mcp__claude-in-chrome__computer({ action: "screenshot", tabId: <TAB_ID> })

// 5. Test interactions
mcp__claude-in-chrome__computer({ action: "scroll", coordinate: [700, 400], scroll_direction: "up", scroll_amount: 3, tabId: <TAB_ID> })
mcp__claude-in-chrome__computer({ action: "left_click_drag", start_coordinate: [700, 400], coordinate: [500, 300], tabId: <TAB_ID> })
mcp__claude-in-chrome__computer({ action: "key", text: "ArrowRight", repeat: 10, tabId: <TAB_ID> })

// 6. Execute JavaScript to inspect state
mcp__claude-in-chrome__javascript_tool({
    action: "javascript_exec",
    text: "window.lifeSimulatorApp.controls.getDragOffset()",
    tabId: <TAB_ID>
})

// 7. Check for console errors
mcp__claude-in-chrome__read_console_messages({ tabId: <TAB_ID>, onlyErrors: true })
```

### Web-Viewer Key Files
- `web-viewer/viewer.html` - Main HTML page with styles
- `web-viewer/js/app.js` - Application entry point
- `web-viewer/js/controls.js` - Mouse/keyboard/zoom controls
- `web-viewer/js/renderer.js` - Canvas rendering
- `web-viewer/js/config.js` - Configuration constants
- `web-viewer/js/entity-manager.js` - Entity polling and tracking

### Web-Viewer Controls (Current Implementation)
| Control | Action |
|---------|--------|
| Click + drag | Pan the map |
| Scroll wheel | Zoom in/out (centers on cursor) |
| WASD / Arrow keys | Pan continuously |
| + / - | Zoom in/out |
| R | Reset view |
| H | Toggle help overlay |

### Common Debugging Tasks

**Test zoom centering:**
```javascript
// Get current zoom and offset
const controls = window.lifeSimulatorApp.controls;
({ zoom: CONFIG.renderScale, offset: controls.getDragOffset() })
```

**Verify keyboard handlers:**
```javascript
const controls = window.lifeSimulatorApp.controls;
({
    hasKeyHandlers: typeof controls.boundHandlers.keyDown === 'function',
    keysPressed: [...controls.keysPressed]
})
```

**Check entity data:**
```javascript
window.lifeSimulatorApp.entityManager.getEntities().length
```

---

## 🎯 Agent Context Selection

### 🤖 When you need to...
**Add new species**: Read `docs/SPECIES_REFERENCE.md` → `docs/DEVELOPMENT_GUIDE.md`
**Debug AI behavior**: Read `docs/EVENT_DRIVEN_PLANNER_IMPLEMENTATION.md` → `docs/SPECIES_REFERENCE.md`
**Work on vegetation**: Read `docs/PLANT_SYSTEM_PARAMS.md` → `docs/DEVELOPMENT_GUIDE.md`
**Fix performance**: Read `docs/DEVELOPMENT_GUIDE.md` → `docs/TICK_SYSTEM_ANALYSIS.md`
**Implement features**: Read `docs/DEVELOPMENT_GUIDE.md` → relevant system documentation
**Use the API**: Read `docs/API_REFERENCE.md` → `docs/QUICK_START.md`
**Modify viewer**: Read relevant viewer documentation (`godot-viewer/` or `web-viewer/`)

### 📚 Key System Files
- **Core simulation**: `src/main.rs`, `src/lib.rs`
- **Entity system**: `src/entities/`
- **AI system**: `src/ai/`
- **Vegetation**: `src/vegetation/`
- **World generation**: `src/world/`
- **Web server**: `src/web_server_simple.rs`

### 🔧 Essential Commands
```bash
# Development
cargo check
cargo run --bin life-simulator
cargo test

# World generation  
cargo run --bin map_generator

# With logging
RUST_LOG=info cargo run --bin life-simulator

# Performance
cargo build --release
cargo flamegraph --bin life-simulator
```

## ⚠️ Bevy ECS - Critical Rules (MUST FOLLOW)

### System Parameter Conflicts

**NEVER mix these parameters in the same system:**
```rust
// ❌ INVALID - Runtime panic!
fn my_system(
    mut commands: Commands,  // Needs mutable access
    world: &World,           // Needs read access to EVERYTHING
) {
    // This will panic: "&World conflicts with a previous mutable system parameter"
}
```

**Why?** `&World` requires immutable borrow of the **entire** ECS world, conflicting with any mutable parameter (`Commands`, `ResMut`, `Query<&mut T>`).

### ✅ Solutions

**Solution 1: Use Specific Queries** (PREFERRED)
```rust
fn my_system(
    mut commands: Commands,
    leader_query: Query<&PackLeader>,    // ✅ Specific component access
    member_query: Query<&PackMember>,
) {
    // No conflict! Only accessing specific components
}
```

**Solution 2: ParamSet** (for unavoidable conflicts)
```rust
fn my_system(mut params: ParamSet<(&World, Commands)>) {
    let world = params.p0();    // Use &World first
    // ... later ...
    let commands = params.p1(); // Then use Commands
}
```

**Solution 3: Exclusive System** (blocks parallelism!)
```rust
fn my_system(world: &mut World) {
    // Full access, but BLOCKS all other systems
    // Only use for bulk operations
}
```

### 🛡️ Automated Safety

Run before committing:
```bash
./scripts/check_bevy_conflicts.sh
```

This linter catches `&World` + `Commands` conflicts automatically and runs in CI.

### 📚 Official Documentation

- [Bevy Cheat Book - ParamSet](https://bevy-cheatbook.github.io/programming/paramset.html)
- [Bevy Cheat Book - Exclusive Systems](https://bevy-cheatbook.github.io/programming/exclusive.html)
- [ParamSet API Docs](https://docs.rs/bevy/latest/bevy/ecs/system/struct.ParamSet.html)

---

## 🔴 Entity Component Dependencies (CRITICAL)

### Required Components for AI Entities

Every entity that participates in AI must have ALL of these components:

| Component | Purpose | Added By |
|-----------|---------|----------|
| `BehaviorConfig` | Thresholds for hunger/thirst/energy | Species spawn function |
| `IdleTracker` | Tracks idle time for replanning | `AIEntityBundle` or `initialize_new_entity_trackers` |
| `StatThresholdTracker` | Edge detection for stat triggers | `AIEntityBundle` or `stat_threshold_system` |
| `CurrentAction` | Current action state | `AIEntityBundle` or spawn function |
| `Hunger`, `Thirst`, `Energy` | Core stats | `EntityStatsBundle` |

### Use AIEntityBundle for Spawning

```rust
// ✅ CORRECT - Use AIEntityBundle
fn spawn_creature(commands: &mut Commands, config: BehaviorConfig) {
    commands.spawn((
        CreatureBundle::new(...),
        AIEntityBundle::new(config, tick, hunger, thirst, energy),
    ));
}

// ❌ WRONG - Manual components (easy to forget one!)
fn spawn_creature(commands: &mut Commands) {
    commands.spawn((
        Creature { ... },
        BehaviorConfig::default(),
        // MISSING: IdleTracker, StatThresholdTracker - entity will be broken!
    ));
}
```

### Never Silently Skip Entities

```rust
// ❌ WRONG - Silent failure
if let Ok(tracker) = query.get(entity) {
    // Process...
}
// Entity without tracker silently skipped!

// ✅ CORRECT - Loud failure
match query.get(entity) {
    Ok(tracker) => { /* process */ }
    Err(_) => {
        error!("🚨 Entity {:?} missing required component!", entity);
    }
}
```

### Entity Validator System

The `EntityValidatorPlugin` runs every 50 ticks and:
1. Auto-fixes entities missing `IdleTracker` or `StatThresholdTracker`
2. Logs warnings for broken entities
3. Detects stuck entities (high hunger but idle)

This is a **safety net** - entities should be spawned correctly, but validator catches mistakes.

---

## 📋 Project Status
- **Core Systems**: ✅ ECS-based simulation with AI, vegetation, and predator-prey dynamics
- **Web Viewer**: ✅ Interactive HTML/JS viewer with real-time entity tracking
- **Godot Viewer**: ✅ Alternative 3D viewer with OpenRCT2 sprite integration
- **API**: ✅ Comprehensive HTTP API for all simulation data
- **Testing**: ✅ Unit tests, integration tests, and performance benchmarks

## 🎯 Current Focus Areas
- **Performance optimization** of ResourceGrid vegetation system
- **Advanced AI behaviors** and pack dynamics for predators
- **Enhanced visualization** and user interface improvements
- **Ecosystem balancing** and parameter tuning

---

*This index provides quick access to all project documentation. Use the Global Decision Engine routing above for auto-delegation to specialized contexts.*