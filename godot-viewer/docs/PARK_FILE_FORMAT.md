# OpenRCT2 Park File Format (.park)

## Overview

OpenRCT2 `.park` files use a binary chunk-based format called "Orca". Each chunk contains specific game data.

## File Structure

```
Park File (.park)
├── Header (magic, version info)
├── Chunk: AUTHORING (0x01)
├── Chunk: OBJECTS (0x02)
├── Chunk: SCENARIO (0x03)
├── Chunk: GENERAL (0x04)
├── Chunk: CLIMATE (0x05)
├── Chunk: PARK (0x06)
├── Chunk: RESEARCH (0x08)
├── Chunk: NOTIFICATIONS (0x09)
├── Chunk: INTERFACE (0x20)
├── Chunk: TILES (0x30) ⬅️ **Map data is here**
├── Chunk: ENTITIES (0x31)
├── Chunk: RIDES (0x32)
├── Chunk: BANNERS (0x33)
├── Chunk: CHEATS (0x36)
├── Chunk: RESTRICTED_OBJECTS (0x37)
└── Chunk: PLUGIN_STORAGE (0x38)
```

## TILES Chunk (0x30)

The TILES chunk contains all terrain and object data:

```cpp
void ReadWriteTilesChunk(GameState_t& gameState, OrcaStream& os)
{
    cs.readWrite(gameState.mapSize.x);      // Map width
    cs.readWrite(gameState.mapSize.y);      // Map height
    
    auto numElements = cs.read<uint32_t>(); // Number of tile elements
    
    // Read array of TileElement structures
    std::vector<TileElement> tileElements;
    tileElements.resize(numElements);
    cs.read(tileElements.data(), tileElements.size() * sizeof(TileElement));
}
```

### TileElement Structure

Each tile position can have multiple stacked elements:
- **Surface** (terrain) - base land tile
- **Path** - footpaths
- **Track** - ride tracks
- **SmallScenery** - trees, benches, etc.
- **LargeScenery** - buildings, structures
- **Wall** - fences, edges
- **Entrance** - park entrances
- **Banner** - signs

## Loading Park Files in Godot

### Current Status

❌ **No direct parser** - The park format is complex and OpenRCT2-specific
❌ **Binary format** - Not human-readable like RON/JSON
❌ **Requires** - Understanding of OpenRCT2's TileElement structure

### Approaches to Load Park Files

#### Option 1: Use OpenRCT2 as Converter (Recommended)

OpenRCT2 can export data in various formats:

```bash
# Option A: Load park in OpenRCT2 and screenshot for reference
open -a OpenRCT2 good-generated-map.park

# Option B: Use OpenRCT2 scripting API to export JSON
# (Requires custom plugin to extract map data)

# Option C: Convert to .sv6 format
# (Still binary, but potentially simpler to parse)
```

#### Option 2: Create Park File Parser in Rust

Add a park file parser to the Rust backend:

1. **Parse binary format:**
   - Implement Orca stream reader
   - Parse chunk headers
   - Extract TILES chunk

2. **Convert TileElement to internal format:**
   - Map OpenRCT2 terrain types to our terrain types
   - Extract height and slope from TileElement
   - Convert to RON format

3. **Benefits:**
   - Direct integration with existing backend
   - Can load any OpenRCT2 park file
   - One-time conversion to RON

#### Option 3: Manual Map Recreation

For testing purposes:

1. Open `good-generated-map.park` in OpenRCT2
2. Take screenshots and note dimensions
3. Create equivalent RON file manually with similar terrain features
4. Use for visual comparison testing

#### Option 4: Use Existing Map Data

The Godot viewer is **already working** with generated maps! 

Current capabilities:
- ✅ Loading backend-generated terrain
- ✅ Rendering with proper height offsets
- ✅ Drawing edge faces (cliff walls)
- ✅ All terrain edge types (rock, wood, ice)
- ✅ Proper isometric projection

**You can test the current implementation without needing the park file!**

## Recommended Next Steps

### For Testing Edge Rendering

1. **Use current backend maps** - Already generating varied terrain
2. **Visual comparison** - Open park file in OpenRCT2 side-by-side with Godot
3. **Identify discrepancies** - Note any visual differences
4. **Iterate on rendering** - Fix edge cases as needed

### For Loading Park Files (Future)

1. **Document park file requirements** - What data do we need?
2. **Rust parser implementation** - Add park file support to backend
3. **Conversion utility** - `park-to-ron` command-line tool
4. **Integration** - Add to ChunkManager's file detection

## References

- OpenRCT2 source: `/Users/jean/Github/OpenRCT2/src/openrct2/park/ParkFile.cpp`
- TileElement definition: `OpenRCT2/src/openrct2/world/tile_element/TileElement.h`
- Orca stream format: `OpenRCT2/src/openrct2/core/OrcaStream.hpp`

## Current Workaround

**The Godot viewer is fully functional without park files!**

Test the edge rendering with existing maps:
```bash
cd godot-viewer
/Applications/Godot.app/Contents/MacOS/Godot --path . scenes/World.tscn
```

The terrain and edge faces are rendering successfully based on the terminal output:
- ✅ "🏔️ Painted edge faces for 256 tiles in chunk"
- ✅ Multiple chunks loaded and rendered
- ✅ Edge textures loading from sprites

**For visual parity testing:** Simply open OpenRCT2 with your park file and compare side-by-side with the Godot viewer's current output.

