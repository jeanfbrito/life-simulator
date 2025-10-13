# Testing Grass Textures in Godot Viewer

## What Was Integrated

I've integrated the stone-kingdoms grass textures into your Godot viewer. Here's what was done:

### Files Modified

1. **`godot-viewer/scripts/TerrainTileMap.gd`**
   - Added `GrassTextureManager` integration
   - Modified `paint_terrain_tile()` to use grass textures for Grass and Forest terrain
   - Added fallback to colored diamonds if textures don't load
   - All non-grass terrain still uses colored diamonds

2. **`godot-viewer/scripts/GrassTextureManager.gd`** (NEW)
   - Loads 8 grass texture variants from extracted files
   - Provides random grass texture selection
   - Simple, lightweight implementation

### How It Works

```
When painting a tile:
1. Check if terrain type is "Grass" or "Forest"
2. If yes AND grass textures loaded:
   → Use random stone-kingdoms grass texture
3. If no OR textures failed to load:
   → Use colored diamond (original behavior)
```

**This means:**
- ✅ Grass and Forest tiles will show beautiful stone-kingdoms grass
- ✅ All other terrain (water, sand, mountains, etc.) still use colored diamonds
- ✅ Random variety: each grass tile picks from 8 different texture variants
- ✅ Fallback: if textures don't load, colored diamonds still work

## How to Test

### Step 1: Verify Extracted Textures

```bash
# Check that grass textures were extracted successfully
ls godot-viewer/assets/tiles/grass/

# Should see:
# abundant_grass_1x1_01.png through abundant_grass_1x1_08.png
# Plus light variants and macro tiles (24 files total)
```

### Step 2: Start the Backend

```bash
# Terminal 1: Start the life simulator backend
cargo run --bin life-simulator

# Wait for: "🌐 Web server started on http://127.0.0.1:54321"
```

### Step 3: Launch Godot Viewer

```bash
# Terminal 2: Launch Godot viewer
cd godot-viewer
/Applications/Godot.app/Contents/MacOS/Godot --path .

# In Godot: Press F5 or click the Play button
```

### Step 4: What to Look For

**In the Godot console output, look for:**

```
🗺️ TerrainTileMap initialized
🌿 GrassTextureManager initialized
🌿 Loading grass textures from extracted stone-kingdoms tiles...
✅ Loaded 8 grass texture variants
```

**If textures loaded successfully:**
```
🌿 Painted grass tile at world (-48, -48) (pixel: ...) as Grass
🌿 Painted grass tile at world (-47, -48) (pixel: ...) as Grass
```

**If textures failed to load:**
```
🌿 Grass texture not found: res://assets/tiles/grass/abundant_grass_1x1_01.png
❌ Failed to load any grass textures from res://assets/tiles/grass/
🎨 Painted terrain tile at world (-48, -48) ... as Grass  (falls back to colored)
```

### Step 5: Visual Check

**On the screen, you should see:**

- **Grass areas**: Textured grass from stone-kingdoms (subtle variations between tiles)
- **Water**: Blue colored diamond (unchanged)
- **Sand**: Tan colored diamond (unchanged)
- **Mountains**: Gray colored diamond (unchanged)

**The grass should look noticeably different** - more detailed and natural compared to the solid colored diamonds.

## Troubleshooting

### Problem: "Grass texture not found" errors

**Cause:** Textures not in the correct location

**Solution:**
```bash
# Re-extract textures
./scripts/extract_grass_textures.sh

# Verify they're in the right place
ls -lh godot-viewer/assets/tiles/grass/*.png
```

### Problem: Grass still looks like colored diamonds

**Cause:** Textures may not be auto-imported by Godot

**Solution:**
1. In Godot Editor, go to FileSystem panel
2. Navigate to `res://assets/tiles/grass/`
3. Right-click → "Reimport"
4. Select all grass PNG files
5. In Import tab, set:
   - **Filter**: Nearest (for pixel art look)
   - **Mipmaps**: Off
6. Click "Reimport"
7. Run the scene again (F5)

### Problem: Textures look blurry or scaled wrong

**Cause:** Texture import settings or size mismatch

**Solution 1 - Import Settings:**
1. Select a grass texture in FileSystem
2. Import tab → Filter = Nearest
3. Reimport all textures

**Solution 2 - Scale Textures:**
```bash
# Scale textures to match your 128×64 tile size
./scripts/scale_grass_textures.sh

# This creates grass_scaled/ directory
# Then update GrassTextureManager.gd to use:
# "res://assets/tiles/grass_scaled/..." instead
```

### Problem: Performance drops

**Cause:** Creating many texture sources

**Current behavior:** Each unique grass texture gets one TileSet source, so max 8 sources for all grass tiles. This should be fine for performance.

**If you need better performance later:** Implement the macro tile system (2×2, 3×3, 4×4 tiles) from the documentation.

## Expected Results

### Console Output (Success)
```
🗺️ TerrainTileMap initialized
🌿 GrassTextureManager initialized
🌿 Loading grass textures from extracted stone-kingdoms tiles...
✅ Loaded 8 grass texture variants
🔧 Created new texture source for res://assets/tiles/grass/abundant_grass_1x1_01.png with ID 13
🌿 Painted grass tile at world (-48, -48) (pixel: ...) as Grass
🔧 Created new texture source for res://assets/tiles/grass/abundant_grass_1x1_03.png with ID 14
🌿 Painted grass tile at world (-47, -48) (pixel: ...) as Grass
...
🎨 Painted 2448 terrain tiles for chunk -3,-3
📊 Total rendered chunks: 49 / 49 visible
```

### Visual Result

**Before (colored diamonds):**
- Grass: Solid green diamonds
- No texture detail
- All grass tiles look identical

**After (stone-kingdoms textures):**
- Grass: Detailed grass texture with subtle color variations
- Each tile slightly different (8 random variants)
- More natural "field" appearance
- Matches stone-kingdoms visual quality

## Next Steps (Optional)

### 1. Scale Textures (Recommended)

The extracted textures are 30×18 pixels but your tiles are 128×64. Godot will scale them automatically, but pre-scaling gives better control:

```bash
./scripts/scale_grass_textures.sh
```

Then update `GrassTextureManager.gd` line 18:
```gdscript
var path = "res://assets/tiles/grass_scaled/abundant_grass_1x1_%02d.png" % i
```

### 2. Add Macro Tile System (Advanced)

For 4-6× better performance, implement the full macro tile system:
- Read: `scripts/GRASS_MACRO_TILE_GUIDE.md`
- Use: `godot-viewer/scripts/GrassMacroTileRenderer.gd`
- Result: 2×2, 3×3, 4×4 tiles covering multiple cells with one texture

### 3. Add More Terrain Textures

You could extract other terrain types from stone-kingdoms:
- Dirt textures
- Sand/beach textures
- Mountain/stone textures
- Forest (darker grass) textures

Same extraction process, just different quad coordinates from `object_quads.lua`.

## Verification Checklist

- [ ] Backend running (port 54321)
- [ ] Godot viewer launches without errors
- [ ] Console shows "✅ Loaded 8 grass texture variants"
- [ ] Console shows "🌿 Painted grass tile" messages
- [ ] Grass areas look textured (not solid green)
- [ ] Water/sand/other terrain still show colored diamonds
- [ ] No performance issues
- [ ] Random variety visible (grass tiles look slightly different)

## Summary

**What you have:**
- ✅ 8 grass texture variants extracted from stone-kingdoms
- ✅ GrassTextureManager loading and caching textures
- ✅ TerrainTileMap using grass textures for Grass/Forest terrain
- ✅ Fallback to colored diamonds if textures don't load
- ✅ Random variation for natural appearance

**Test it now:**
```bash
# Terminal 1
cargo run --bin life-simulator

# Terminal 2
cd godot-viewer && /Applications/Godot.app/Contents/MacOS/Godot --path .
# Press F5
```

Look for grass textures in the rendered world!
