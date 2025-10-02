# Viewport/Camera Fix Plan

## Problem Analysis

Currently the system works like this:
```
1. Render tiles at world coords: (-VIEW_SIZE_X/2 to +VIEW_SIZE_X/2, -VIEW_SIZE_Y/2 to +VIEW_SIZE_Y/2)
2. Apply canvas translate by dragOffset
3. Load chunks based on dragOffset position
```

**Issue**: Step 1 always renders the same world coordinates! The dragOffset only moves the canvas, not the rendered content.

## Solution

We need to **offset the world coordinates being rendered** based on dragOffset:

```
1. Calculate camera/viewport position in world coordinates from dragOffset
2. Render tiles at: (cameraX to cameraX + VIEW_SIZE_X, cameraY to cameraY + VIEW_SIZE_Y)
3. NO canvas translate needed - we're rendering the correct world area
4. Load chunks based on the camera position
```

## Implementation Steps

### Step 1: Add camera position calculation
In `renderer.js`, convert dragOffset (pixels) to world tile offset:
```javascript
const cameraOffsetX = Math.floor(-dragOffset.x / CONFIG.TILE_SIZE);
const cameraOffsetY = Math.floor(-dragOffset.y / CONFIG.TILE_SIZE);
```

### Step 2: Update renderTerrain to use camera offset
Instead of:
```javascript
const worldX = x - Math.floor(CONFIG.VIEW_SIZE_X / 2);
const worldY = y - Math.floor(CONFIG.VIEW_SIZE_Y / 2);
```

Do:
```javascript
const worldX = x + cameraOffsetX - Math.floor(CONFIG.VIEW_SIZE_X / 2);
const worldY = y + cameraOffsetY - Math.floor(CONFIG.VIEW_SIZE_Y / 2);
```

### Step 3: Remove canvas translate
We no longer need `ctx.translate(dragOffset.x, dragOffset.y)` because we're rendering the correct world tiles directly.

### Step 4: Handle sub-pixel smoothness
For smooth panning, we need to handle the remainder:
```javascript
const pixelOffsetX = -dragOffset.x % CONFIG.TILE_SIZE;
const pixelOffsetY = -dragOffset.y % CONFIG.TILE_SIZE;
ctx.translate(pixelOffsetX, pixelOffsetY);
```

This gives us smooth pixel-perfect panning while loading the correct chunks.

## Benefits

✅ Renders correct world coordinates based on camera position
✅ Loads chunks for visible area
✅ Smooth sub-pixel panning
✅ No empty blue space - always shows actual terrain
✅ Efficient - only renders what's visible

## Testing

After implementation:
1. Pan around - should see continuous terrain
2. Zoom in and pan - should load appropriate chunks
3. Check console - chunk loading should match visible area
4. Verify tooltips show correct world coordinates
