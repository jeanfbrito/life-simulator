# Godot Isometric Rendering Plan

Target: replicate the existing web-viewer map rendering inside a Godot client while switching to an isometric presentation. The plan keeps data parity with the current REST endpoints so the Godot view shows the same world state as the browser viewer.

## Phase 0 – Foundations

**Task 0.1 – Audit Web Rendering Assumptions** ✅ COMPLETED
- Capture tile size, chunk dimensions, terrain/resource identifiers, and zoom limits from `web-viewer`.
- Document any implicit rules (e.g., walkability, resource overlays) that must carry over.

### Web Rendering Analysis Summary

**Core Configuration Values (from `web-viewer/js/config.js`):**
- **Tile Size**: Dynamic base of 8px, scaled by `renderScale` factor (4px minimum)
- **Chunk Dimensions**: 16×16 tiles per chunk (from `src/tilemap/chunk.rs`)
- **View Size**: Dynamic, calculated to fill container viewport
- **API Port**: 54321
- **Default Terrain**: DeepWater

**Terrain Types (12 total):**
- Grass, Forest, Sand, Water, Dirt, Snow, Mountain, Stone, Swamp, Desert, DeepWater, ShallowWater
- Each has distinct hex colors in `TERRAIN_COLORS`

**Resource Types (6 total):**
- TreeOak, TreePine, TreeBirch, Rock, Bush, Flower
- Each has emoji symbols and rendering configurations (size, offset)

**Zoom Configuration:**
- **Min Zoom**: 0.25x (4px tile minimum)
- **Max Zoom**: 4.0x
- **Zoom Factor**: 1.25x per step
- **View Scaling**: `CONFIG.TILE_SIZE = Math.max(4, Math.floor(8 * CONFIG.renderScale))`

**Critical Rendering Rules:**
1. **Entity Y Offset**: All entities render with -0.2 tile Y offset to keep feet in grid
2. **Y-Sorted Rendering**: Resources and entities sorted by Y coordinate for proper depth
3. **Chunk Coordinates**: String format `"x,y"` used as HashMap keys
4. **World-to-Local Conversion**: `((worldX % 16) + 16) % 16` for handling negative coordinates
5. **Resource Blocking**: All resources currently block movement (impassable tiles)

**API Endpoints Used:**
- `/api/world/current` - Current world info
- `/api/world_info` - Additional world metadata
- `/api/chunks?coords=x,y&layers=true` - Multi-layer chunk data
- `/api/entities` - Entity positions and states
- `/api/species` - Species configuration for rendering
- `/api/vegetation/biomass` - Optional grass density overlay

**Performance Settings:**
- **Chunk Load Radius**: 5 chunks (11×11 grid initially)
- **Chunk Batch Size**: 10 chunks per request (URL length management)
- **Chunk Load Debounce**: 100ms delay
- **Entity Polling**: 200ms intervals

**Verification**
- ✅ Internal note summarising values and rules documented above
- ✅ All rendering assumptions captured from web-viewer source code
- ✅ API endpoints and data flow documented

**Task 0.2 – Prepare Godot Workspace** ✅ COMPLETED
- Install/verify Godot 4.x at agreed patch version.
- Create `godot-viewer/` project in repo (or chosen folder) with Git ignore/settings aligned to household conventions.
- Enable Web API access (Project Settings → `network/threads/thread_safe` etc. if needed) and add placeholder scene.

### Workspace Setup Summary

**Godot Version**: 4.5.stable.official.876b29033 ✅
**Project Structure Created**:
- `godot-viewer/scenes/` - Scene files
- `godot-viewer/scripts/` - GDScript files
- `godot-viewer/resources/` - Resource files
- `godot-viewer/addons/` - Godot addons
- `godot-viewer/project.godot` - Project configuration
- `godot-viewer/.gitignore` - Git ignore file
- `godot-viewer/README.md` - Project documentation

**Placeholder Scene Created**:
- Main scene (`scenes/Main.tscn`) with basic UI
- Main script (`scripts/Main.gd`) for initialization
- Confirms workspace loads without errors

**Verification**
- ✅ Godot project opens without warnings
- ✅ Placeholder scene runs from CLI and exits cleanly: `godot --headless --path godot-viewer --quit-after 1`
- ✅ Main scene loads successfully: "Life Simulator Viewer - Main scene loaded"

## Phase 1 – Data Pipeline

**Task 1.1 – Port Configuration Constants** ✅ COMPLETED
- Reproduce CONFIG values that impact rendering (tile size base, default terrain type, chunk radius defaults).
- Expose them in a `Config.gd` singleton for easy tweaks.

### Configuration Constants Implementation Summary

**Config.gd Singleton Created** ✅:
- **Tile Size**: Dynamic base of 8px, matches web-viewer
- **Chunk Size**: 16×16 tiles per chunk
- **API Base URL**: http://localhost:54321
- **Zoom Settings**: Min 0.25x, Max 4.0x, Factor 1.25x
- **Performance Settings**: Target 60 FPS, chunk loading radius 5, batch size 10
- **Terrain Colors**: All 12 terrain types with exact hex colors from web-viewer
- **Resource Symbols**: All 6 resource types with emoji symbols
- **Entity Y Offset**: -0.2 tile offset to keep feet in grid (critical rendering rule)

**Key Features**:
- **Singleton AutoLoad**: Registered as global `Config` object
- **Helper Functions**: `get_terrain_color()`, `get_resource_symbol()`, `get_entity_config()`
- **API Integration**: `load_species_config()` method for dynamic entity configuration
- **Dynamic Updates**: `update_tile_size()` for zoom functionality

**Verification Results** ✅:
```
=== Config Constants Verification ===

Tile Size: 8 ✓ PASS
API Base URL: http://localhost:54321 ✓ PASS
Zoom Settings: Min 0.25, Max 4.0, Factor 1.25 ✓ PASS
Terrain Types: 12 types loaded ✓ PASS
Resource Types: 6 types loaded ✓ PASS
Grass Color: #3a7f47 ✓ PASS (exact match)
Tree Oak Symbol: 🌳 ✓ PASS
Chunk Size: 16 ✓ PASS
Performance Settings: All match expected values ✓ PASS
Default Values: DeepWater terrain, -0.2 entity offset ✓ PASS

=== Verification Complete ===
✅ All critical web-viewer constants successfully ported to Godot
```

**Files Created**:
- `scripts/Config.gd` - Main singleton with all configuration
- `scenes/ConfigTest.tscn` - Verification test scene
- `scripts/ConfigTest.gd` - Comprehensive test script
- Updated `project.godot` - AutoLoad singleton registration

**Verification**
- ✅ Unit test prints config constants and matches web values exactly
- ✅ Code review confirms all required constants are present
- ✅ All critical web-viewer rendering rules preserved (entity Y offset, chunk size, etc.)

**Task 1.2 – HTTP Client + ChunkManager** ✅ COMPLETED
- Implement `ChunkManager.gd` autoload mirroring JS logic: track loaded/loading chunk sets, debounce visible chunk fetches, merge results.
- Hit `/api/world/current` and `/api/chunks?coords=…&layers=true` via Godot `HTTPRequest` or `AwaitableHTTP`.
- Ensure error handling toggles connection status signal for UI.

### HTTP Client + ChunkManager Implementation Summary

**ChunkManager.gd Autoload Created** ✅:
- **HTTP Request System**: Uses Godot's `HTTPRequest` for API communication
- **Chunk Tracking**: `loaded_chunks` and `loading_chunks` dictionaries for state management
- **Batch Processing**: Splits large requests into batches of 10 chunks (configurable)
- **Debounced Loading**: Timer-based debouncing for visible chunk requests
- **Connection Status**: Automatic connection status tracking with signals
- **Error Handling**: Comprehensive error handling for HTTP failures and JSON parsing

**Key Features Implemented**:
- **World Info Loading**: `/api/world/current` and `/api/world_info` endpoints
- **Chunk Loading**: `/api/chunks?coords=x,y&layers=true` with multi-layer support
- **Batch Processing**: Prevents URL length issues with configurable batch sizes
- **State Management**: Tracks which chunks are loaded vs. loading
- **API Endpoint Matching**: Identical endpoints to web-viewer
- **Connection Signals**: `chunks_loaded`, `world_info_loaded`, `connection_status_changed`

**Core Functionality Verified** ✅:
```
=== HTTP Connection Test ===
Testing basic HTTP request...
Request started, waiting for response...
Request completed with result: 0 code: 200
✅ Successfully parsed JSON
World name: final_validation
=== HTTP Test Complete ===
```

**API Endpoints Tested** ✅:
- ✅ `/api/world/current` - Successfully loads world metadata
- ✅ `/api/world_info` - Successfully loads additional world info
- ✅ `/api/chunks?coords=x,y&layers=true` - Successfully loads chunk data
- ✅ Connection status tracking working
- ✅ JSON parsing working correctly

**Architecture Matches JavaScript Version**:
- ✅ Debounced loading with configurable timer (100ms)
- ✅ Batch requests (10 chunks per request)
- ✅ Chunk coordinate format: `"x,y"` string keys
- ✅ Multi-layer data structure (terrain + resources)
- ✅ Loading state tracking prevents duplicate requests
- ✅ Connection status signals for UI updates

**Files Created**:
- `scripts/ChunkManager.gd` - Main chunk manager autoload singleton
- `scenes/ChunkManagerTest.tscn` - Comprehensive test scene
- `scripts/ChunkManagerTest.gd` - Full functionality test
- `scenes/ChunkManagerSimpleTest.tscn` - Simple test scene
- `scripts/ChunkManagerSimpleTest.gd` - Basic functionality test
- `scenes/HTTPTest.tscn` - HTTP connection test
- `scripts/HTTPTest.gd` - Basic HTTP request test
- Updated `project.godot` - ChunkManager autoload registration

**Verification**
- ✅ Manual test: backend running, HTTP requests working, logs show proper API calls
- ✅ Error handling implemented with connection status changes
- ✅ Batch processing identical to browser implementation
- ✅ All core HTTP client functionality operational

**Task 1.3 – World Data Cache** ✅ COMPLETED
- Store fetched terrain/resource layers in dictionaries keyed by chunk key (`"x,y"`).
- Provide helper to translate world tile coordinate → terrain/resource lookups.

### World Data Cache Implementation Summary

**WorldDataCache.gd Autoload Created** ✅:
- **Separate Caches**: Independent `terrain_cache` and `resource_cache` dictionaries
- **Chunk Key Format**: Uses `"x,y"` string format matching web-viewer
- **Coordinate Translation**: World coordinates ↔ chunk coordinates + local coordinates
- **Efficient Lookups**: Direct coordinate-based terrain/resource queries
- **Cache Management**: Statistics, clearing, memory usage tracking
- **Integration Ready**: Signals for cache updates and clearing

**Core Functionality**:
- **Storage**: `store_terrain_chunk()`, `store_resource_chunk()`, `store_chunk_data()`
- **Retrieval**: `get_terrain_at()`, `get_resource_at()` with world coordinates
- **Coordinate Conversion**: `get_chunk_key()`, `get_local_coords()` for negative coordinates
- **Bulk Queries**: `get_terrain_in_area()`, `get_resources_in_area()` for rectangular regions
- **Cache Management**: `clear_cache()`, `clear_chunk()`, cache statistics
- **Integration**: `merge_chunk_data()` for seamless ChunkManager integration

**Coordinate System Verified** ✅:
```
World (0, 0) -> Chunk 0,0 Local (0, 0)
World (16, 16) -> Chunk 1,1 Local (0, 0)
World (-1, -1) -> Chunk -1,-1 Local (15, 15)
World (31, 31) -> Chunk 1,1 Local (15, 15)
World (-17, -17) -> Chunk -2,-2 Local (15, 15)
```

**Test Results** ✅:
```
=== World Data Cache Test Results ===
✅ Coordinate conversion: Perfect negative coordinate handling
✅ Data storage/retrieval: 16x16 chunks stored and accessed correctly
✅ Out-of-bounds handling: Returns DeepWater default terrain and empty resources
✅ Area queries: 2x2 terrain and resource areas working
✅ Cache statistics: Tracks chunks, tiles, and memory usage
✅ Cache clearing: Individual and bulk clearing functional
✅ Real data integration: Successfully loads 16x16 chunks from backend API
✅ Real terrain lookup: (0,0) returns "Grass" from actual world data
```

**Performance Features**:
- **Memory Efficient**: Chunk-based storage with statistics tracking
- **No Duplicates**: Chunk key system prevents redundant storage
- **Fast Lookups**: Direct coordinate-to-data access without iteration
- **Bulk Operations**: Area queries for rendering optimization
- **Cache Signals**: `cache_updated`, `cache_cleared` for UI integration

**Integration with Existing Systems**:
- ✅ **ChunkManager Integration**: `merge_chunk_data()` processes ChunkManager responses
- ✅ **Config Integration**: Uses `Config.DEFAULT_TERRAIN_TYPE` for out-of-bounds
- ✅ **Real API Data**: Successfully caches 16×16 chunks from backend
- ✅ **Coordinate Consistency**: Matches web-viewer coordinate system exactly

**Files Created**:
- `scripts/WorldDataCache.gd` - Main cache management autoload singleton
- `scenes/WorldDataCacheTest.tscn` - Comprehensive test scene
- `scripts/WorldDataCacheTest.gd` - Full functionality test with real API integration
- Updated `project.godot` - WorldDataCache autoload registration

**Verification**
- ✅ Unit script with mock API returns correct terrain/resource lookups
- ✅ Edge cases tested: negative coordinates, boundaries, out-of-bounds access
- ✅ Real API integration successful: loads actual 16×16 world chunks
- ✅ No duplicate fetches: chunk key system prevents redundant storage
- ✅ Memory tracking: Statistics and memory usage monitoring working

## Phase 2 – Isometric Terrain Rendering

**Task 2.1 – TileSet Authoring**
- Decide initial art approach (colored quads vs. hand-painted). Implement baseline: single white diamond sprite tinted per terrain type via `CanvasItemMaterial`.
- Build `TileSet` with 12 terrain entries, set `tile_shape = isometric`, `tile_layout = stacked`, `tile_size = 128x64` (example; confirm ratio).
**Verification**
- In editor, painting a single chunk’s worth of tiles displays correct diamond grid without gaps.
- Texture import warnings resolved; re-opening project keeps TileSet intact.

**Task 2.2 – Terrain TileMap Node**
- Create `TerrainTileMap` scene/node handling painting. Accept chunk data, convert each tile to map coords (`chunk_origin + local_offset`), call `set_cell`.
- Ensure coordinate math aligns with negative chunks (wraps like JS modulo logic).
**Verification**
- Load known seed (same as web). Compare screenshot of a 3×3 chunk area with browser viewer; tile types align after a manual 1:1 check.
- Logging proves chunk boundaries meet seamlessly (no gaps or offsets).

**Task 2.3 – Grass Density Overlay (Optional Toggle)**
- Port biomass overlay logic: when enabled, adjust tile material color/alpha based on biomass data from `/api/vegetation/biomass`.
- Provide UI toggle mirroring web.
**Verification**
- Toggle on/off in runtime; observe color change only on Grass/Forest/Dirt tiles.
- Compare biomass values for random coordinates with backend JSON to ensure mapping accuracy.

## Phase 3 – Dynamic Chunk Streaming

**Task 3.1 – Camera & Controls**
- Implement `Camera2D` with drag-to-pan (mouse/WASD) and discrete zoom steps.
- Maintain drag offset to determine visible world center; feed into `ChunkManager` to trigger loads.
**Verification**
- Manual test: pan across multiple chunk boundaries; logs show new chunk batches when crossing thresholds.
- Zoom preserves isometric proportions (no stretching); ensures tile selection still hits correct coords.

**Task 3.2 – Chunk Lifecycle Management**
- Add buffer radius identical to web viewer (at least +1 chunk). Unload far-away chunks or keep in cache depending on memory.
- On chunk removal, clear corresponding TileMap cells.
**Verification**
- Move camera far from origin: memory profiler shows stable usage; no stale tiles remain in unloaded regions.
- Revisit earlier area → no re-fetch if cache enabled; otherwise re-fetch triggered as expected.

## Phase 4 – Resources and Entities

**Task 4.1 – Resource Sprites**
- Create `ResourceManager.gd` that spawns `Sprite2D` or `Label` nodes (emoji/textures) for resource overlay.
- Use `TileMap.map_to_local()` to convert tile coords to pixel positions; parent under `YSort` for depth.
**Verification**
- Sample chunk with resources matches web viewer positions and counts.
- Toggle overlay on/off to confirm update path works.

**Task 4.2 – Entity Rendering**
- Implement entity fetch/sync (reuse existing API). Represent each entity as a child under `YSort`. Handle juvenile scaling (emoji scale factor).
- Draw action labels when zoom level ≥ threshold.
**Verification**
- Spawn simulation with known entity positions; verify order front/back matches browser (entity behind trees renders behind).
- Changing action text in backend reflects within one update tick.

## Phase 5 – UI & Statistics

**Task 5.1 – HUD Parity**
- Build `Control` canvas replicating stats panel (walkable %, water %, forest %, resources).
- Update values from terrain render pass (reuse counts already computed).
**Verification**
- Same world/viewport: numbers match web viewer within rounding tolerance.
- Disconnect backend: HUD shows offline indicator (from Task 1.2 signal).

**Task 5.2 – Controls Panel**
- Implement buttons for reset view, new world seed, zoom +/- matching keyboard shortcuts.
- Ensure calling “New World” hits backend and refreshes caches.
**Verification**
- Click controls while monitoring backend logs: requests align with browser behaviour.
- Reset view returns camera to default chunk without drift.

## Phase 6 – Validation & Polish

**Task 6.1 – Side-by-Side Regression Checks**
- Capture synchronized screenshots (web + Godot) for at least three seeds/locations.
- Create comparison montage and attach to issue/PR for archival.
**Verification**
- Review shows tile colors, resources, entities, and statistics identical aside from isometric perspective.
- Sign-off from stakeholder recorded (comment or meeting notes).

**Task 6.2 – Packaging & Docs**
- Document build/run instructions in `docs/GODOT_ISOMETRIC_RENDERING_PLAN.md` (update this file) and include any caveats.
- Set up CI smoke test (headless launch to ensure main scene loads).
**Verification**
- CI job passes; pipeline artifact includes log snippet confirming scene ready.
- README/Docs merged with internal review approval.

## Done Definition

- Camera displays same terrain content as browser viewer for identical seeds and view bounds.
- Resources/entities align with web data; overlays match toggles.
- Team has verification evidence (screenshots + metrics) and automated smoke test coverage.
- Remaining differences documented as follow-up issues (if any).
