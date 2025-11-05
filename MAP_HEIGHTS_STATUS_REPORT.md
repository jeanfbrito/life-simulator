# Map Heights Status Report

**Date**: 2025-11-05
**System**: Life Simulator - Height Data Integration Analysis

## Executive Summary

✅ **Height data IS generated and saved to map files**
⚠️ **Height data IS NOT loaded into simulation runtime structures**
❌ **Simulation cannot use height data for gameplay logic**

## Detailed Findings

### 1. Map Generation (✅ Working)

**File**: `src/map_generator.rs`

The map generator successfully creates height data:

- Uses OpenRCT2 3-phase algorithm (simplex noise → smoothing → finalization)
- Generates heights as `u8` values (0-255)
- Converts heights to strings for serialization
- Saves to map files in "heights" and "slope_indices" layers

**Code Evidence** (lines 164-181):
```rust
// Convert heights (Vec<Vec<u8>>) to Vec<Vec<String>> for serialization
let height_tiles_str: Vec<Vec<String>> = height_data
    .heights
    .iter()
    .map(|row| row.iter().map(|h| h.to_string()).collect())
    .collect();

// Create multi-layer chunk
let mut chunk_layers = HashMap::new();
chunk_layers.insert("terrain".to_string(), terrain_tiles);
chunk_layers.insert("resources".to_string(), resources_tiles);
chunk_layers.insert("heights".to_string(), height_tiles_str);
chunk_layers.insert("slope_indices".to_string(), slope_tiles_str);
```

### 2. Map Files (✅ Heights Present)

**Example**: `maps/test_heights.ron`

Map files contain height data in the multi-layer system:

```ron
layers: {
    "terrain": [["Snow", "Snow", ...]],
    "resources": [["", "WildRoot", ...]],
    "heights": [["255", "240", "224", ...]],
    "slope_indices": [["0", "1", "11", ...]]
}
```

**Verified**:
- ✅ 9 chunks in test_heights.ron all have height layers
- ✅ Height values range from 192-255 (snow peaks)
- ✅ Slope indices include various slope types (0-14)

### 3. Loading System (⚠️ Partial)

**File**: `src/world_loader.rs`

The WorldLoader CAN load height data but returns it as strings:

```rust
/// Get a specific layer for a chunk
pub fn get_chunk_layer(
    &self,
    chunk_x: i32,
    chunk_y: i32,
    layer_name: &str,
) -> Option<Vec<Vec<String>>> {
    self.get_chunk_layers(chunk_x, chunk_y)
        .and_then(|layers| layers.get(layer_name).cloned())
}
```

**Status**:
- ✅ Heights can be loaded via `get_chunk_layer(x, y, "heights")`
- ⚠️ Returns `Vec<Vec<String>>`, not `Vec<Vec<u8>>`
- ❌ No method to parse strings back to u8 values
- ❌ No method to populate Chunk.heights[] from loaded data

### 4. Runtime Structures (❌ Not Populated)

**File**: `src/tilemap/chunk.rs`

The Chunk struct HAS height fields but they're never populated from loaded data:

```rust
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub coordinate: ChunkCoordinate,
    pub tiles: [[TerrainType; CHUNK_SIZE]; CHUNK_SIZE],
    #[serde(skip)]  // ⚠️ NOT SERIALIZED/DESERIALIZED
    pub heights: [[u8; CHUNK_SIZE]; CHUNK_SIZE],
    #[serde(skip)]
    pub slope_masks: [[u8; CHUNK_SIZE]; CHUNK_SIZE],
    #[serde(skip)]
    pub slope_indices: [[u8; CHUNK_SIZE]; CHUNK_SIZE],
    pub biome: BiomeType,
    pub is_dirty: bool,
    pub generation_seed: u64,
}
```

**Problem**:
- The `#[serde(skip)]` attribute means these fields are NOT serialized/deserialized
- When loading chunks from RON files, these fields default to all zeros
- Heights are stored separately in the layer system, not in the Chunk struct

### 5. The Disconnect

**Generation Flow** (✅ Working):
```
WorldGenerator.generate_height_chunk()
    → ChunkHeightData { heights: [[u8]] }
    → Convert to strings
    → Save to map file as "heights" layer
```

**Loading Flow** (❌ Broken):
```
Load map file
    → WorldLoader.get_chunk_layer("heights")
    → Returns Vec<Vec<String>>
    → ???
    → Chunk.heights[][] = all zeros (never populated)
```

**Current Usage**:
- ✅ CachedWorld stores heights as strings (for web API)
- ✅ Godot viewer reads heights from API for rendering
- ❌ Simulation Chunk structs have heights=0 (unusable)
- ❌ No gameplay logic can use height data

## Impact Assessment

### What Works:
1. ✅ Height generation (OpenRCT2 algorithm)
2. ✅ Height storage in map files
3. ✅ Height retrieval via web API
4. ✅ Godot viewer slope rendering

### What Doesn't Work:
1. ❌ Loading heights into Chunk structs
2. ❌ Simulation access to height data
3. ❌ Pathfinding based on slopes
4. ❌ Movement speed affected by elevation
5. ❌ Any gameplay mechanics requiring height

## Root Cause Analysis

### Design Issue: Dual Storage Systems

The project has TWO separate ways to store chunk data:

1. **Chunk Struct** (Runtime ECS component):
   - Contains `heights: [[u8; 16]; 16]`
   - Used by simulation systems
   - NOT serialized (`#[serde(skip)]`)
   - Always zeros when loaded from file

2. **Layer System** (Serialization format):
   - Contains `layers: {"heights": [["255", ...]]}`
   - Stored in map files as strings
   - Accessible via WorldLoader/CachedWorld
   - NOT copied to Chunk structs

### Why This Happened:

Looking at the code history:

1. Original design: Chunk had height fields for runtime use
2. Multi-layer system added later for serialization flexibility
3. Heights moved to layers but Chunk fields kept (for backward compatibility?)
4. No bridge created to populate Chunk.heights from loaded layers
5. Systems never refactored to use layer-based heights

## Recommendations

### Option 1: Populate Chunk.heights from Layers (Preferred)

**Pros**:
- Maintains current API
- Allows simulation logic to use Chunk.heights[]
- No breaking changes to existing systems

**Cons**:
- Duplicate data (in layers AND in Chunk)
- Requires parsing strings to u8

**Implementation**:
```rust
// In world_loader.rs or chunk loading system
fn populate_chunk_heights(chunk: &mut Chunk, layers: &HashMap<String, Vec<Vec<String>>>) {
    if let Some(heights_str) = layers.get("heights") {
        for (y, row) in heights_str.iter().enumerate().take(CHUNK_SIZE) {
            for (x, val_str) in row.iter().enumerate().take(CHUNK_SIZE) {
                if let Ok(height) = val_str.parse::<u8>() {
                    chunk.heights[y][x] = height;
                }
            }
        }
    }
    // Similar for slope_masks and slope_indices
}
```

### Option 2: Remove Chunk Height Fields

**Pros**:
- Single source of truth (layers only)
- No duplicate data
- Cleaner architecture

**Cons**:
- Breaking change to Chunk API
- All systems must query WorldLoader/CachedWorld for heights
- Performance impact (string parsing on every query)

### Option 3: Make Heights Part of Terrain

**Pros**:
- Heights always available with terrain
- Natural coupling (terrain + elevation)

**Cons**:
- Large refactor of TerrainType enum
- Changes serialization format (breaking change)

## Recommended Solution

**Implement Option 1**: Create a bridge system that populates Chunk.heights[] when loading from layers.

**Specific Changes**:

1. Add parsing utility in `src/tilemap/chunk.rs`:
   ```rust
   impl Chunk {
       pub fn populate_heights_from_layer(&mut self, heights_layer: &Vec<Vec<String>>) {
           for (y, row) in heights_layer.iter().enumerate().take(CHUNK_SIZE) {
               for (x, val_str) in row.iter().enumerate().take(CHUNK_SIZE) {
                   if let Ok(height) = val_str.parse::<u8>() {
                       self.heights[y][x] = height;
                   }
               }
           }
       }

       pub fn populate_slopes_from_layer(&mut self, slopes_layer: &Vec<Vec<String>>) {
           // Similar implementation
       }
   }
   ```

2. Call during chunk loading in WorldLoader or ChunkManager

3. Update chunk spawning systems to populate heights

4. Add validation/tests to ensure heights match between layers and Chunk

## Testing Checklist

- [ ] Verify heights load from map files as strings
- [ ] Parse string heights to u8 values
- [ ] Populate Chunk.heights[] arrays
- [ ] Verify values match between layers and Chunk
- [ ] Test with multiple maps (island, openrct2, etc.)
- [ ] Validate slope indices parsing
- [ ] Check performance impact of parsing
- [ ] Update documentation

## Conclusion

The simulation's height system is **partially implemented**:
- ✅ Generation works perfectly
- ✅ Storage works perfectly
- ❌ **Loading is incomplete** - heights never reach simulation Chunks
- ⚠️ Only visualization uses heights, not gameplay

**Priority**: Medium-High
**Effort**: Small (~2-4 hours)
**Risk**: Low (additive change, no breaking changes)

The fix is straightforward: add a parsing step to populate Chunk.heights[] from the "heights" layer when loading chunks. This will enable gameplay systems to use elevation data for pathfinding, movement, and other mechanics.
