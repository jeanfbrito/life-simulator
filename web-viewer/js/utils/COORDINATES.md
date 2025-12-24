# Coordinate Conversion Utilities

## Overview

The `CoordinateConverter` utility module provides a centralized, reusable system for converting between different coordinate systems used in the Life Simulator web viewer:

- **Screen Coordinates**: Position within the viewport (0 to VIEW_SIZE_X/Y)
- **World Coordinates**: Position in the infinite world space (unbounded)
- **Chunk Coordinates**: Partitioned chunks of the world (16x16 tiles per chunk by default)
- **Canvas Pixels**: Actual pixel positions on the HTML canvas
- **Screen Pixels**: Rendered pixel positions accounting for camera offset

## Key Benefits

1. **Code Deduplication**: Single source of truth for all coordinate conversions
2. **Consistency**: Identical logic across controls.js, renderer.js, and chunk-manager.js
3. **Maintainability**: Changes to conversion logic only need to be made in one place
4. **Testability**: Comprehensive test suite ensures correctness
5. **Performance**: Optimized conversion functions avoid redundant calculations

## Core Methods

### `screenToWorld(screenX, screenY, viewSizeX, viewSizeY)`

Convert screen coordinates (viewport position) to world coordinates (world space position).

**Parameters:**
- `screenX` (number): X coordinate in screen space (0 to viewSizeX)
- `screenY` (number): Y coordinate in screen space (0 to viewSizeY)
- `viewSizeX` (number): Total viewport width in tiles
- `viewSizeY` (number): Total viewport height in tiles

**Returns:** `{x: number, y: number}` - World coordinates

**Example:**
```javascript
const world = CoordinateConverter.screenToWorld(50, 60, 100, 100);
// {x: 0, y: 10}
```

### `worldToScreen(worldX, worldY, viewSizeX, viewSizeY)`

Convert world coordinates to screen coordinates.

**Parameters:**
- `worldX` (number): X coordinate in world space
- `worldY` (number): Y coordinate in world space
- `viewSizeX` (number): Total viewport width in tiles
- `viewSizeY` (number): Total viewport height in tiles

**Returns:** `{x: number, y: number}` - Screen coordinates

**Example:**
```javascript
const screen = CoordinateConverter.worldToScreen(0, 10, 100, 100);
// {x: 50, y: 60}
```

### `worldToChunk(worldX, worldY, chunkSize = 16)`

Convert world coordinates to chunk coordinates with local tile position.

**Parameters:**
- `worldX` (number): X coordinate in world space
- `worldY` (number): Y coordinate in world space
- `chunkSize` (number, optional): Size of each chunk, default 16

**Returns:** `{chunkX: number, chunkY: number, localX: number, localY: number}`

**Example:**
```javascript
const chunk = CoordinateConverter.worldToChunk(20, 20, 16);
// {chunkX: 1, chunkY: 1, localX: 4, localY: 4}
```

### `chunkKey(chunkX, chunkY)`

Create a string key from chunk coordinates for use as object keys.

**Parameters:**
- `chunkX` (number): Chunk X coordinate
- `chunkY` (number): Chunk Y coordinate

**Returns:** `string` - Key in format "x,y"

**Example:**
```javascript
const key = CoordinateConverter.chunkKey(5, -3);
// "5,-3"
```

### `parseChunkKey(key)`

Parse a chunk key string back into coordinates.

**Parameters:**
- `key` (string): Chunk key in format "x,y"

**Returns:** `{chunkX: number, chunkY: number}`

**Example:**
```javascript
const coords = CoordinateConverter.parseChunkKey("5,-3");
// {chunkX: 5, chunkY: -3}
```

### `canvasToWorld(canvasX, canvasY, dragOffset)`

Convert canvas pixel coordinates (accounting for drag offset) to world coordinates.

**Parameters:**
- `canvasX` (number): X coordinate in canvas pixels
- `canvasY` (number): Y coordinate in canvas pixels
- `dragOffset` (object): Camera drag offset `{x: number, y: number}`

**Returns:** `{worldX: number, worldY: number, screenX: number, screenY: number}`

**Example:**
```javascript
const result = CoordinateConverter.canvasToWorld(400, 480, {x: 0, y: 0});
// {worldX: 0, worldY: 10, screenX: 50, screenY: 60}
```

### `worldToScreenPixels(worldX, worldY, cameraOffsetX, cameraOffsetY)`

Convert world coordinates to screen pixel coordinates for rendering.

**Parameters:**
- `worldX` (number): X coordinate in world space
- `worldY` (number): Y coordinate in world space
- `cameraOffsetX` (number): Camera X offset in tiles
- `cameraOffsetY` (number): Camera Y offset in tiles

**Returns:** `{screenPixelX: number, screenPixelY: number, screenTileY: number}`

**Example:**
```javascript
const pixels = CoordinateConverter.worldToScreenPixels(0, 0, 0, 0);
// {screenPixelX: 404, screenPixelY: 404, screenTileY: 50}
```

### `getVisibleBounds(dragOffset)`

Get the visible world bounds based on camera position.

**Parameters:**
- `dragOffset` (number): Current camera drag offset in pixels

**Returns:** `{startX: number, startY: number, endX: number, endY: number}`

**Example:**
```javascript
const bounds = CoordinateConverter.getVisibleBounds(0);
// {startX: 0, startY: 0, endX: 100, endY: 100}
```

### `isWithinViewBounds(x, y)`

Check if screen coordinates are within valid viewport bounds.

**Parameters:**
- `x` (number): X coordinate in screen space
- `y` (number): Y coordinate in screen space

**Returns:** `boolean` - True if within bounds, false otherwise

**Example:**
```javascript
const inBounds = CoordinateConverter.isWithinViewBounds(50, 50);
// true
```

## Usage in Components

### controls.js - Hover Tooltip

**Before:**
```javascript
const worldX = x - Math.floor(CONFIG.VIEW_SIZE_X / 2);
const worldY = y - Math.floor(CONFIG.VIEW_SIZE_Y / 2);
const chunkX = Math.floor(worldX / 16);
const chunkY = Math.floor(worldY / 16);
const localX = ((worldX % 16) + 16) % 16;
const localY = ((worldY % 16) + 16) % 16;
const chunkKey = `${chunkX},${chunkY}`;
```

**After:**
```javascript
const world = CoordinateConverter.screenToWorld(x, y, CONFIG.VIEW_SIZE_X, CONFIG.VIEW_SIZE_Y);
const chunk = CoordinateConverter.worldToChunk(world.x, world.y);
const chunkKey = CoordinateConverter.chunkKey(chunk.chunkX, chunk.chunkY);
```

### renderer.js - Terrain Rendering

**Before:**
```javascript
const worldX = x + cameraOffsetX - Math.floor(CONFIG.VIEW_SIZE_X / 2);
const worldY = y + cameraOffsetY - Math.floor(CONFIG.VIEW_SIZE_Y / 2);
const chunkX = Math.floor(worldX / 16);
const chunkY = Math.floor(worldY / 16);
const localX = ((worldX % 16) + 16) % 16;
const localY = ((worldY % 16) + 16) % 16;
const chunkKey = `${chunkX},${chunkY}`;
```

**After:**
```javascript
const worldX = x + cameraOffsetX - Math.floor(CONFIG.VIEW_SIZE_X / 2);
const worldY = y + cameraOffsetY - Math.floor(CONFIG.VIEW_SIZE_Y / 2);
const chunk = CoordinateConverter.worldToChunk(worldX, worldY);
const chunkKey = CoordinateConverter.chunkKey(chunk.chunkX, chunk.chunkY);
const chunkX = chunk.chunkX;
const chunkY = chunk.chunkY;
const localX = chunk.localX;
const localY = chunk.localY;
```

### renderer.js - Entity Rendering

**Before:**
```javascript
const screenX = (entityWorldX - cameraOffsetX + Math.floor(CONFIG.VIEW_SIZE_X / 2)) * CONFIG.TILE_SIZE + CONFIG.TILE_SIZE / 2;
const screenY = (entityWorldY - cameraOffsetY + Math.floor(CONFIG.VIEW_SIZE_Y / 2)) * CONFIG.TILE_SIZE + CONFIG.TILE_SIZE / 2;
const screenTileY = entityWorldY - cameraOffsetY + Math.floor(CONFIG.VIEW_SIZE_Y / 2);
```

**After:**
```javascript
const screenCoords = CoordinateConverter.worldToScreenPixels(entityWorldX, entityWorldY, cameraOffsetX, cameraOffsetY);
const screenX = screenCoords.screenPixelX;
const screenY = screenCoords.screenPixelY;
const screenTileY = screenCoords.screenTileY;
```

### chunk-manager.js - Chunk Loading

**Before:**
```javascript
const chunkX = Math.floor(viewCenterWorldX / 16);
const chunkY = Math.floor(viewCenterWorldY / 16);
```

**After:**
```javascript
const centerChunk = CoordinateConverter.worldToChunk(viewCenterWorldX, viewCenterWorldY);
const chunkX = centerChunk.chunkX;
const chunkY = centerChunk.chunkY;
```

## Testing

Comprehensive tests are available in `web-viewer/tests/coordinates.test.js`. Run with:

```bash
node web-viewer/tests/coordinates.test.js
```

Test coverage includes:
- Screen to world conversions
- World to screen conversions
- World to chunk conversions
- Chunk key operations (create and parse)
- Canvas to world conversions
- World to screen pixel conversions
- Bounds checking
- Round-trip consistency

## Performance Considerations

1. **Caching**: For repeated conversions of the same coordinates, consider caching results
2. **Batch Operations**: Use these utilities for batch coordinate operations
3. **Camera Offset**: Store camera offset state to minimize recalculations

## Migration Guide

To migrate existing code to use CoordinateConverter:

1. Add import at top of file:
   ```javascript
   import { CoordinateConverter } from './utils/coordinates.js';
   ```

2. Replace inline conversion logic with appropriate converter methods

3. Test thoroughly - use the test suite as reference for correct behavior

4. Verify rendering and interaction still work correctly

## Future Enhancements

Potential improvements to consider:
- Isometric coordinate support for alternative rendering
- Z-coordinate support for elevation/layers
- Perspective projection support for 3D rendering
- GPU-accelerated batch conversions
- Caching layer for frequently accessed conversions
