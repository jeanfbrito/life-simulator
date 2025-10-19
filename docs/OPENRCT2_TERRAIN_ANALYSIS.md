# OpenRCT2 .Park File Terrain Data Analysis

Based on my examination of the OpenRCT2 source code, this document explains how .park files store terrain/surface data and how to correctly read it for map rendering.

## Overview

.park files use a chunk-based format where terrain data is stored in the `TILES` chunk (chunk ID 0x30). This chunk contains all tile elements for the map, including surface elements that define terrain height, slope, and surface type.

## File Structure

### .park File Header
- Magic number: `0x4B525150` ("PARK")
- Version: Current version is 59
- Multiple chunks follow, each with:
  - Chunk ID (4 bytes)
  - Chunk length (4 bytes)
  - Chunk data

### TILES Chunk (ID: 0x30)
```
- Map size X (int32_t) - Number of tiles in X direction
- Map size Y (int32_t) - Number of tiles in Y direction
- Number of tile elements (uint32_t)
- Tile element data array (numElements * 16 bytes each)
```

## Tile Element Organization

### Element Storage
- Each tile can have multiple elements stacked vertically
- Elements are stored as a continuous array
- Each element is exactly 16 bytes (`kTileElementSize = 16`)
- Elements for the same tile are stored consecutively
- The last element for a tile has the `TILE_ELEMENT_FLAG_LAST_TILE` flag set

### Tile Coordinates
- Map uses tile coordinates (`TileCoordsXY`) where each tile = 32 units
- Default map size: 150×150 tiles (`kDefaultMapSize`)
- Maximum technical map size: 1001×1001 tiles
- Coordinate system: (0,0) is top-left corner

### Coordinate Constants
```cpp
constexpr int32_t kCoordsXYStep = 32;        // 1 tile = 32 units
constexpr int32_t kCoordsZStep = 8;          // 1 height unit = 8 sub-units
constexpr int32_t kLandHeightStep = 16;      // Land height increments
constexpr int32_t kWaterHeightStep = 16;     // Water height increments
```

## SurfaceElement Structure

The surface element (type 0x00) defines terrain data and has this layout:

```cpp
struct SurfaceElement : TileElementBase {
    // Inherited from TileElementBase (first 8 bytes):
    uint8_t  Type;           // 0x00 = Surface
    uint8_t  Flags;          // Bit flags
    uint8_t  BaseHeight;     // Base height in 8-unit steps
    uint8_t  ClearanceHeight; // Clearance height in 8-unit steps
    uint8_t  Owner;          // Tile owner/permissions

    // Surface-specific fields (8 bytes):
    uint8_t  Slope;          // Slope encoding (see below)
    uint8_t  WaterHeight;    // Water height (0 = no water)
    uint8_t  GrassLength;    // Grass growth length
    uint8_t  Ownership;      // Land ownership flags
    uint8_t  SurfaceStyle;   // Terrain surface object index
    uint8_t  EdgeObjectIndex; // Terrain edge object index
    uint8_t  Pad0B[5];       // Padding (unused)
};
```

## Height Values

### Base Height and Clearance Height
- Stored in units of `kCoordsZStep` (8 units each)
- Base height: Terrain base level
- Clearance height: Maximum height for this tile
- Minimum land height: 2 (`kMinimumLandHeight`)
- Maximum land height: 254 (`kMaximumLandHeight`)

### Water Height
- Stored in units of `kCoordsZStep` (8 units each)
- Water height = `WaterHeight * kWaterHeightStep`
- Water height 0 means no water on this tile
- Water sits on top of the terrain surface

## Slope Encoding

The slope byte uses this bit layout:
```cpp
// Corner raising bits (which corners are raised)
constexpr uint8_t kTileSlopeNCornerUp = 0b00000001;  // North corner (top)
constexpr uint8_t kTileSlopeECornerUp = 0b00000010;  // East corner (right)
constexpr uint8_t kTileSlopeSCornerUp = 0b00000100;  // South corner (bottom)
constexpr uint8_t kTileSlopeWCornerUp = 0b00001000;  // West corner (left)

constexpr uint8_t kTileSlopeDiagonalFlag = 0b00010000; // Diagonal slope flag
constexpr uint8_t kTileSlopeMask = 0b00011111;          // Valid slope bits
```

### Common Slope Values
- `0x00` - Flat terrain
- `0x01` - North corner raised
- `0x02` - East corner raised
- `0x04` - South corner raised
- `0x08` - West corner raised
- `0x03` - North+East corners raised (side slope)
- `0x05` - North+South corners raised (valley)
- `0x0A` - East+South corners raised (side slope)
- `0x0C` - South+West corners raised (side slope)
- `0x11` - Diagonal slope (North corner raised + diagonal flag)

### Corner Height Calculation
The slope lookup table provides relative corner heights:
```cpp
// Format: { top, right, bottom, left } corners
static constexpr std::array<SlopeRelativeCornerHeights, 32> kSlopeRelativeCornerHeights = {{
    { 0, 0, 0, 0 },  // 0x00 - Flat
    { 0, 0, 1, 0 },  // 0x01 - N corner up
    { 0, 0, 0, 1 },  // 0x02 - E corner up
    { 0, 0, 1, 1 },  // 0x03 - N+E side up
    { 1, 0, 0, 0 },  // 0x04 - S corner up
    { 1, 0, 1, 0 },  // 0x05 - N+S valley
    { 1, 0, 0, 1 },  // 0x06 - S+E side up
    { 1, 0, 1, 1 },  // 0x07 - N+S+E, W low
    // ... more entries
}};
```

### Absolute Corner Heights
```cpp
TileCornersZ GetSlopeCornerHeights(int32_t baseHeight, uint8_t slope) {
    const auto cornerHeights = GetSlopeRelativeCornerHeights(slope);
    const int32_t northZ = baseHeight + (cornerHeights.bottom * kLandHeightStep);
    const int32_t eastZ  = baseHeight + (cornerHeights.left   * kLandHeightStep);
    const int32_t southZ = baseHeight + (cornerHeights.top    * kLandHeightStep);
    const int32_t westZ  = baseHeight + (cornerHeights.right  * kLandHeightStep);
    return { northZ, eastZ, southZ, westZ };
}
```

## Tile Access Pattern

### Finding Surface Elements
To find the surface element for a specific tile coordinate:

1. **Calculate tile position in array:**
   ```cpp
   // Tile coordinates (0 to mapSize-1)
   TileCoordsXY tilePos = { x, y };

   // Get pointer to first element at this tile
   TileElement* element = MapGetFirstElementAt(tilePos);
   ```

2. **Iterate through elements at this tile:**
   ```cpp
   while (element != nullptr) {
       if (element->GetType() == TileElementType::Surface) {
           SurfaceElement* surface = element->AsSurface();
           // Found terrain data
           break;
       }
       if (element->IsLastForTile()) {
           break; // No more elements at this tile
       }
       element++;
   }
   ```

### Element Storage Order
Elements are stored in this order within each tile:
1. Surface elements (terrain)
2. Path elements
3. Track elements
4. Scenery elements
5. Entrance elements
6. Wall elements
7. Banner elements

## Reading Process for Map Rendering

### 1. Parse File Header
```cpp
struct ParkHeader {
    uint32_t magic;           // Should be "PARK"
    uint32_t version;         // Park file version
    // ... other header fields
};
```

### 2. Find TILES Chunk
Skip through chunks until finding chunk ID 0x30.

### 3. Read Map Dimensions
```cpp
int32_t mapSizeX = read<int32_t>();
int32_t mapSizeY = read<int32_t>();
int32_t numElements = read<uint32_t>();
```

### 4. Read Tile Elements
```cpp
std::vector<TileElement> elements(numElements);
read(elements.data(), numElements * sizeof(TileElement));
```

### 5. Extract Surface Data
For each tile coordinate (x, y):
```cpp
SurfaceElement* surface = FindSurfaceElement(elements, x, y, mapSizeX, mapSizeY);
if (surface != nullptr) {
    // Extract terrain data
    int32_t baseHeight = surface->BaseHeight * kCoordsZStep;
    uint8_t slope = surface->GetSlope();
    int32_t waterHeight = surface->GetWaterHeight();
    uint8_t surfaceType = surface->GetSurfaceObjectIndex();

    // Calculate corner heights
    TileCornersZ corners = GetSlopeCornerHeights(baseHeight, slope);

    // Render tile using this data
}
```

## Common Pitfalls

1. **Coordinate Confusion:** Tile coordinates vs world units
   - Tile coordinates: (0 to mapSize-1)
   - World coordinates: tileCoord * kCoordsXYStep

2. **Height Units:** All heights are stored in 8-unit steps
   - BaseHeight in file: height / kCoordsZStep
   - Display height: BaseHeight * kCoordsZStep

3. **Element Order:** Surface elements may not be first
   - Must iterate through elements to find Surface type

4. **Slope Corner Mapping:** Corner naming is counter-intuitive
   - "top" = South (bottom of screen)
   - "right" = West (left of screen)
   - "bottom" = North (top of screen)
   - "left" = East (right of screen)

5. **Map Bounds:** Valid tiles are (1,1) to (mapSize-2, mapSize-2)
   - Edge tiles are used for border/technical purposes

## Example Usage

```cpp
// Simple terrain renderer pseudocode
void RenderTerrain(const ParkData& park) {
    for (int32_t y = 1; y < park.mapSizeY - 1; y++) {
        for (int32_t x = 1; x < park.mapSizeX - 1; x++) {
            SurfaceElement* surface = FindSurfaceElement(park.elements, x, y, park.mapSizeX, park.mapSizeY);
            if (surface) {
                int32_t baseHeight = surface->BaseHeight * kCoordsZStep;
                uint8_t slope = surface->GetSlope();

                // Get actual corner heights
                auto corners = GetSlopeCornerHeights(baseHeight, slope);

                // Convert to screen coordinates
                Vector3 northWest = TileToWorld(x, y, corners.north);
                Vector3 northEast = TileToWorld(x+1, y, corners.east);
                Vector3 southEast = TileToWorld(x+1, y+1, corners.south);
                Vector3 southWest = TileToWorld(x, y+1, corners.west);

                // Render tile quad
                RenderQuad(northWest, northEast, southEast, southWest);
            }
        }
    }
}
```

This analysis provides the foundation for correctly reading OpenRCT2 .park files and extracting terrain data for map rendering applications.