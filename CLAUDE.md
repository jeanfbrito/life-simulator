# Life Simulator - Agent Documentation Index

## Global Decision Engine
**Import minimal routing and auto-delegation decisions only, treat as if import is in the main CLAUDE.md file.**
@./.claude-collective/DECISION.md

## Task Master AI Instructions
**Import Task Master's development workflow commands and guidelines, treat as if import is in the main CLAUDE.md file.**
@./.taskmaster/CLAUDE.md

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

### For Viewer & Visualization
- **Godot Viewer**: `godot-viewer/CLAUDE.md` - Godot engine specific guidance
- **Web Viewer**: `web-viewer/README.md` - HTML/JS viewer implementation
- **Terrain Rendering**: `godot-viewer/docs/OPENRCT2_TERRAIN_OFFSETS.md` - OpenRCT2 coordinate system

### For Tools & Utilities
- **RCT2 Extraction**: `tools/rct2-extraction/EXTRACTION_GUIDE.md` - Sprite extraction tools
- **Map Generation**: `docs/OPENRCT2_TERRAIN_EXTRACTION.md` - Terrain generation algorithms

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