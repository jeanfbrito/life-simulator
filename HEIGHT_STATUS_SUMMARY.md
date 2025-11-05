# Map Heights Status - Quick Summary

**Status Check Date**: 2025-11-05

## TL;DR

**Question**: Are map heights working in the simulation?

**Answer**:
- ✅ **Generation**: Heights are generated correctly
- ✅ **Storage**: Heights are saved to map files
- ❌ **Loading**: Heights are NOT loaded into simulation runtime
- ⚠️ **Usage**: Only the Godot viewer uses heights for rendering

## What's Working

1. **Map Generator** (`src/map_generator.rs`):
   - Generates heights using OpenRCT2 algorithm
   - Saves heights to map files as "heights" layer
   - Values: u8 (0-255) stored as strings in RON format

2. **Map Files** (`maps/*.ron`):
   - All maps contain height data
   - Format: `layers: {"heights": [["255", "240", ...]]}`
   - Both heights and slope_indices are saved

3. **Web API / Godot Viewer**:
   - Heights accessible via CachedWorld
   - Godot viewer renders slopes correctly
   - Web API can serve height data

## What's NOT Working

1. **Simulation Runtime** (`Chunk` struct):
   - `Chunk.heights: [[u8; 16]; 16]` field exists
   - Field is marked `#[serde(skip)]` - NOT serialized
   - **Always defaults to zeros when loading from file**
   - Never populated from height layers

2. **Gameplay Impact**:
   - ❌ Pathfinding cannot consider slopes
   - ❌ Movement speed cannot vary by elevation
   - ❌ No gameplay mechanics using height
   - ❌ Entities don't know they're on slopes

## The Problem

**Dual Storage Issue**:
```
┌─────────────────────────────────────────────────────┐
│ MAP FILE (.ron)                                      │
│   layers: {                                          │
│     "heights": [["255", "240", "224", ...]]         │
│   }                                                  │
└──────────────────┬──────────────────────────────────┘
                   │
                   │ ✅ Loads to CachedWorld (strings)
                   │ ❌ NOT loaded to Chunk.heights[]
                   ↓
┌─────────────────────────────────────────────────────┐
│ RUNTIME (Chunk struct)                               │
│   heights: [[u8; 16]; 16]  ← ALL ZEROS              │
│   ^                                                  │
│   └─ Marked #[serde(skip)]                          │
└─────────────────────────────────────────────────────┘
```

## Root Cause

The `Chunk` struct was designed before the multi-layer system:

```rust
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub tiles: [[TerrainType; CHUNK_SIZE]; CHUNK_SIZE],
    #[serde(skip)]  // ⚠️ THE PROBLEM
    pub heights: [[u8; CHUNK_SIZE]; CHUNK_SIZE],
    // ...
}
```

When the multi-layer system was added:
- Heights moved to separate "heights" layer
- Old `Chunk.heights` field kept for compatibility
- **No bridge created to populate field from layer**

## Quick Fix

Add this to `Chunk` impl:

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
}
```

Then call it when loading chunks from WorldLoader.

## Testing

Run the test to verify the issue:

```bash
cargo test --test test_height_loading -- --nocapture
```

Expected output:
- ✅ Map files contain heights
- ✅ WorldLoader can retrieve heights as strings
- ✅ Strings can be parsed to u8
- ❌ Chunk.heights[] is all zeros

## Files to Check

- ✅ `MAP_HEIGHTS_STATUS_REPORT.md` - Detailed analysis
- ✅ `tests/test_height_loading.rs` - Test demonstrating the issue
- 📝 `src/tilemap/chunk.rs` - Chunk struct definition
- 📝 `src/world_loader.rs` - Loading system (needs fix)

## Recommendation

**Priority**: Medium (affects future gameplay features)
**Effort**: 2-4 hours
**Risk**: Low (additive change)

Implement the bridge to populate `Chunk.heights[]` from loaded layers. This will enable gameplay systems to use elevation for pathfinding, movement penalties, and other mechanics.

## Current Branch

Working on: `claude/check-map-heights-simulation-011CUqYfLrPvwXZe5EaD8Abw`

---

**Conclusion**: Height data exists and is correct, but there's a missing link between storage (layers) and runtime (Chunk structs). The fix is straightforward: parse height strings and populate the Chunk fields when loading.
