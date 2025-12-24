# CoordinateConverter Quick Reference

## Import

```javascript
import { CoordinateConverter } from './utils/coordinates.js';
```

## Common Conversions

### Convert Mouse Position to World Coordinates
```javascript
// In mouse move handler
const world = CoordinateConverter.screenToWorld(screenX, screenY, CONFIG.VIEW_SIZE_X, CONFIG.VIEW_SIZE_Y);
const chunk = CoordinateConverter.worldToChunk(world.x, world.y);
const chunkKey = CoordinateConverter.chunkKey(chunk.chunkX, chunk.chunkY);

// Now look up terrain/resources
const terrain = worldData.chunks[chunkKey][chunk.localY][chunk.localX];
```

### Convert Entity Position for Rendering
```javascript
// Get screen pixel coordinates from world position
const screenPixels = CoordinateConverter.worldToScreenPixels(
    entity.position.x,
    entity.position.y,
    cameraOffsetX,
    cameraOffsetY
);

// Render at screenPixels.screenPixelX, screenPixels.screenPixelY
canvas.drawImage(image, screenPixels.screenPixelX, screenPixels.screenPixelY);
```

### Load Chunks Around Camera Position
```javascript
// Get visible world bounds
const viewCenterWorldX = Math.floor(-dragOffset.x / CONFIG.TILE_SIZE) + Math.floor(CONFIG.VIEW_SIZE_X / 2);
const viewCenterWorldY = Math.floor(-dragOffset.y / CONFIG.TILE_SIZE) + Math.floor(CONFIG.VIEW_SIZE_Y / 2);

// Convert to chunk coordinates
const centerChunk = CoordinateConverter.worldToChunk(viewCenterWorldX, viewCenterWorldY);

// Now load chunks around centerChunk.chunkX, centerChunk.chunkY
```

### Check if Coordinates are in Bounds
```javascript
if (CoordinateConverter.isWithinViewBounds(screenX, screenY)) {
    // Position is visible in viewport
}
```

## API Reference

### Screen ↔ World Conversions

**`screenToWorld(screenX, screenY, viewSizeX, viewSizeY)`**
- Input: Position in viewport (0 to viewSize)
- Output: Position in world space
- Use: Converting mouse clicks to world coordinates

**`worldToScreen(worldX, worldY, viewSizeX, viewSizeY)`**
- Input: Position in world space
- Output: Position in viewport
- Use: Checking if world position is visible

### Chunk Conversions

**`worldToChunk(worldX, worldY, chunkSize = 16)`**
- Input: World coordinates
- Output: `{chunkX, chunkY, localX, localY}`
- Use: Breaking world space into chunks for data lookup
- Note: localX/localY are always 0-15 (tile position within chunk)

**`chunkKey(chunkX, chunkY)`**
- Input: Chunk coordinates
- Output: String key "x,y"
- Use: Creating object keys for chunk storage

**`parseChunkKey(key)`**
- Input: String key "x,y"
- Output: `{chunkX, chunkY}`
- Use: Parsing chunk keys back to coordinates

### Rendering Conversions

**`worldToScreenPixels(worldX, worldY, cameraOffsetX, cameraOffsetY)`**
- Input: World position, camera offset in tiles
- Output: `{screenPixelX, screenPixelY, screenTileY}`
- Use: Converting world entities to pixel coordinates for drawing
- Note: screenTileY is used for Y-sorted rendering

**`canvasToWorld(canvasX, canvasY, dragOffset)`**
- Input: Canvas pixel coordinates, camera drag offset
- Output: `{worldX, worldY, screenX, screenY}`
- Use: Converting mouse position to world/screen coordinates

**`getVisibleBounds(dragOffset)`**
- Input: Camera drag offset
- Output: `{startX, startY, endX, endY}` in world space
- Use: Determining which world tiles are visible

**`isWithinViewBounds(x, y)`**
- Input: Screen coordinates
- Output: `true` if in bounds, `false` otherwise
- Use: Checking if screen position is visible

## Coordinate System Reference

```
SCREEN SPACE (Viewport)
- X: 0 to CONFIG.VIEW_SIZE_X
- Y: 0 to CONFIG.VIEW_SIZE_Y
- Centered on camera position

WORLD SPACE (Infinite)
- X: -∞ to +∞
- Y: -∞ to +∞
- Independent of camera

CHUNK SPACE (Partitioned)
- Each chunk is 16x16 tiles (by default)
- Chunks indexed by (chunkX, chunkY)
- Local position (localX, localY) is 0-15

CANVAS PIXELS (Rendered)
- X: 0 to canvas.width
- Y: 0 to canvas.height
- Affected by zoom and camera offset
- CONFIG.TILE_SIZE = pixels per tile

SCREEN PIXELS (Rendered)
- Position on rendered canvas
- Accounting for camera offset
- Used for entity/resource drawing
```

## Examples by Use Case

### Tooltip on Mouse Hover
```javascript
// 1. Get mouse canvas position (from event)
const canvasX = e.clientX - rect.left - dragOffset.x;
const canvasY = e.clientY - rect.top - dragOffset.y;

// 2. Convert to screen coordinates
const screenX = Math.floor(canvasX / CONFIG.TILE_SIZE);
const screenY = Math.floor(canvasY / CONFIG.TILE_SIZE);

// 3. Convert to world coordinates
const world = CoordinateConverter.screenToWorld(screenX, screenY, CONFIG.VIEW_SIZE_X, CONFIG.VIEW_SIZE_Y);

// 4. Convert to chunk for data lookup
const chunk = CoordinateConverter.worldToChunk(world.x, world.y);
const terrain = worldData.chunks[CoordinateConverter.chunkKey(chunk.chunkX, chunk.chunkY)][chunk.localY][chunk.localX];

// 5. Display tooltip with world coordinates
tooltip.textContent = `World: (${world.x}, ${world.y}), Terrain: ${terrain}`;
```

### Rendering Entity at World Position
```javascript
// 1. Get entity world position
const entityWorldX = entity.position.x;
const entityWorldY = entity.position.y;

// 2. Convert to screen pixels
const pixelCoords = CoordinateConverter.worldToScreenPixels(
    entityWorldX,
    entityWorldY,
    cameraOffsetX,
    cameraOffsetY
);

// 3. Render at calculated position
ctx.drawImage(sprite, pixelCoords.screenPixelX, pixelCoords.screenPixelY);

// 4. Sort by screenTileY for proper depth ordering
renderQueue.sort((a, b) => a.screenTileY - b.screenTileY);
```

### Loading Chunks Around Camera
```javascript
// 1. Calculate visible area center in world space
const centerWorldX = Math.floor(-dragOffset.x / CONFIG.TILE_SIZE) + Math.floor(CONFIG.VIEW_SIZE_X / 2);
const centerWorldY = Math.floor(-dragOffset.y / CONFIG.TILE_SIZE) + Math.floor(CONFIG.VIEW_SIZE_Y / 2);

// 2. Convert to chunk coordinates
const centerChunk = CoordinateConverter.worldToChunk(centerWorldX, centerWorldY);

// 3. Calculate bounds
const viewEndWorldX = centerWorldX + CONFIG.VIEW_SIZE_X;
const viewEndWorldY = centerWorldY + CONFIG.VIEW_SIZE_Y;
const endChunk = CoordinateConverter.worldToChunk(viewEndWorldX, viewEndWorldY);

// 4. Load chunk radius around center
const radiusX = Math.abs(centerChunk.chunkX - endChunk.chunkX);
const radiusY = Math.abs(centerChunk.chunkY - endChunk.chunkY);
loadChunksInArea(centerChunk.chunkX, centerChunk.chunkY, Math.max(radiusX, radiusY));
```

## Performance Tips

1. **Cache Results**: If converting the same coordinate multiple times, store the result
2. **Batch Operations**: Process multiple coordinates together when possible
3. **Avoid Redundant Conversions**: Think about what you really need (world vs chunk vs pixels)
4. **Use Appropriate Level**: Don't convert to pixels if you only need chunk coordinates

## Debugging

### Verify Coordinate System
```javascript
// Round-trip test
const world = {x: 10, y: 20};
const screen = CoordinateConverter.worldToScreen(world.x, world.y, 100, 100);
const worldAgain = CoordinateConverter.screenToWorld(screen.x, screen.y, 100, 100);
console.assert(world.x === worldAgain.x, 'X mismatch!');
console.assert(world.y === worldAgain.y, 'Y mismatch!');
```

### Validate Chunk Boundaries
```javascript
const chunk = CoordinateConverter.worldToChunk(100, 100);
console.assert(chunk.localX >= 0 && chunk.localX < 16, 'Invalid local X!');
console.assert(chunk.localY >= 0 && chunk.localY < 16, 'Invalid local Y!');
```

### Check Visibility
```javascript
const inBounds = CoordinateConverter.isWithinViewBounds(screenX, screenY);
console.log(`Point ${screenX},${screenY} is ${inBounds ? 'visible' : 'off-screen'}`);
```

## Testing

Run the test suite to verify implementation:
```bash
node web-viewer/tests/coordinates.test.js
```

All 8 tests should pass:
- Screen to World conversion
- World to Screen conversion
- World to Chunk conversion
- Chunk key operations
- Canvas to World conversion
- World to Screen Pixels conversion
- Is within view bounds
- Consistency across conversions

## Further Reading

- Full API: `web-viewer/js/utils/COORDINATES.md`
- Refactoring details: `web-viewer/COORDINATE_REFACTORING.md`
- Test examples: `web-viewer/tests/coordinates.test.js`
- Source code: `web-viewer/js/utils/coordinates.js`
