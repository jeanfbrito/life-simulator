# 🌲 Tree Texture Extraction with Palette Application

## Overview

This document describes the complete process for extracting tree textures from stone-kingdoms with proper color palette application.

## Problem Background

Stone-kingdoms trees use a **2D color lookup system** where:
- Tree textures store **red+green channel pairs** as coordinates (not simple palette indices)
- ColorTable PNG files (320×80) serve as 2D lookup tables
- Shader algorithm: `x = (red / 8) * 10`, `y = (green / 8) * 10`, then fetch ColorTable[x, y]

This is **NOT** a simple palette index system, so standard ImageMagick CLUT doesn't work correctly.

## Solution

### Custom Python Script

Created `scripts/apply_tree_palette.py` that implements the exact stone-kingdoms shader algorithm:

```python
def red_green_to_position(red, green):
    """Convert red/green values (0-255) to ColorTable position."""
    red_index = int(red)
    x = (red_index // 8) * 10

    green_index = int(green)
    y = (green_index // 8) * 10

    return (x, y)
```

**Algorithm:**
1. Extract tree from atlas with `magick -define png:preserve-colormap` (keeps red/green coordinate data)
2. For each pixel in tree texture:
   - Read (R, G) values
   - Calculate ColorTable coordinates: `x = (R // 8) * 10`, `y = (G // 8) * 10`
   - Look up ColorTable[x, y] to get final RGB color
3. Write final colored texture

### Batch Extraction Script

Created `scripts/extract_tree_textures_with_palette.sh` that:
- Extracts all 47 tree variants (25 pine + 22 birch)
- Applies **ColorTable1** (spring green palette) to all trees
- Saves to `godot-viewer/assets/tiles/trees/`

## Prerequisites

1. **Python 3.x** with **Pillow** library
2. **ImageMagick** for atlas extraction
3. **Stone-kingdoms repository** at `/Users/jean/Github/stone-kingdoms`

### Setup Python Environment

```bash
# Create virtual environment
python3 -m venv /tmp/palette_env

# Activate environment
source /tmp/palette_env/bin/activate

# Install Pillow
pip install Pillow
```

## Usage

### Extract All Trees with Green Palette

```bash
# Run the complete extraction script
./scripts/extract_tree_textures_with_palette.sh
```

**Output:**
- 25 pine tree textures (`tree_pine_large_01.png` through `tree_pine_large_25.png`)
- 22 birch tree textures (`tree_birch_large_01.png` through `tree_birch_large_22.png`)
- All with spring green ColorTable1 applied
- Location: `godot-viewer/assets/tiles/trees/`

### Extract Single Tree with Custom Palette

```bash
# Activate Python environment
source /tmp/palette_env/bin/activate

# Apply ColorTable to a tree
python3 scripts/apply_tree_palette.py \
    godot-viewer/assets/tiles/trees/tree_pine_large_01_indexed.png \
    /Users/jean/Github/stone-kingdoms/colortables/PineTree/ColorTable5.png \
    tree_pine_palette5.png
```

## ColorTable Selection

Stone-kingdoms provides **10 ColorTables per tree type** (PineTree, BirchTree, ChestnutTree).

### Current Selection

**ColorTable1** - Spring Green
- Bright green foliage
- Natural tree appearance
- Selected for general use in life-simulator

### Other Available Palettes

To view all palettes for a tree type:

```bash
# Extract one indexed tree
magick /Users/jean/Github/stone-kingdoms/assets/tiles/stronghold_assets_packed_v12-hd.png \
    -crop 75x157+2981+12264 +repage \
    -define png:preserve-colormap \
    tree_pine_indexed.png

# Apply all 10 ColorTables
for i in {1..10}; do
    source /tmp/palette_env/bin/activate
    python3 scripts/apply_tree_palette.py \
        tree_pine_indexed.png \
        /Users/jean/Github/stone-kingdoms/colortables/PineTree/ColorTable${i}.png \
        tree_palette${i}.png
done

# View results
open tree_palette*.png
```

**Expected variations:**
- ColorTable1: Spring green (currently used)
- ColorTable2-4: Various green shades
- ColorTable5-7: Autumn colors (reds, yellows)
- ColorTable8-10: Winter/dead (browns, sparse)

### Changing Palette

To use a different palette (e.g., autumn colors):

1. Edit `scripts/extract_tree_textures_with_palette.sh`
2. Change:
   ```bash
   COLORTABLE_PINE="/Users/jean/Github/stone-kingdoms/colortables/PineTree/ColorTable1.png"
   ```
   To:
   ```bash
   COLORTABLE_PINE="/Users/jean/Github/stone-kingdoms/colortables/PineTree/ColorTable5.png"  # Autumn
   ```
3. Re-run extraction script

## Integration with Godot Viewer

### TreeTextureManager.gd

Loads all extracted tree textures:

```gdscript
class_name TreeTextureManager
var pine_tree_textures: Array[Texture2D] = []
var birch_tree_textures: Array[Texture2D] = []

func load_tree_textures():
    # Load 25 pine variants
    for i in range(1, 26):
        var texture = load_texture("tree_pine_large_%02d.png" % i)
        pine_tree_textures.append(texture)

    # Load 22 birch variants
    for i in range(1, 23):
        var texture = load_texture("tree_birch_large_%02d.png" % i)
        birch_tree_textures.append(texture)

func get_random_tree_texture(tree_type: String) -> Texture2D:
    # Returns random variant for specified tree type
    # Supports: "Wood", "Pine", "Birch", "TreeOak", "TreePine", "TreeBirch"
```

### ResourceManager.gd Integration

Renders trees with actual textures instead of emojis:

```gdscript
func paint_resources(chunk_key: String, resource_data: Array):
    # For each resource in chunk:
    if _is_tree_resource(resource_type):
        # Use Sprite2D with tree texture
        var sprite = Sprite2D.new()
        sprite.texture = tree_texture_manager.get_random_tree_texture(resource_type)
        sprite.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
        sprite.offset.y = -texture_height / 2.0  # Align base with tile
    else:
        # Use emoji for rocks, bushes, flowers
        var label = _create_emoji_label(resource_type, config)
```

**Tree Detection:**
```gdscript
func _is_tree_resource(resource_type: String) -> bool:
    var type_lower = resource_type.to_lower()
    return (type_lower.contains("tree") or
            type_lower.contains("wood") or
            type_lower.contains("pine") or
            type_lower.contains("birch") or
            type_lower.contains("oak"))
```

## File Structure

```
life-simulator/
├── scripts/
│   ├── apply_tree_palette.py                  # Python palette applicator
│   ├── extract_tree_textures_with_palette.sh  # Batch extraction
│   └── extract_tree_textures.sh               # Old script (no palette)
├── godot-viewer/
│   ├── assets/tiles/trees/
│   │   ├── tree_pine_large_01.png   (green, 75×157)
│   │   ├── tree_pine_large_02.png   (green, 73×161)
│   │   ├── ...
│   │   ├── tree_birch_large_01.png  (green, 63×118)
│   │   └── ...
│   └── scripts/
│       ├── TreeTextureManager.gd      # Tree texture loader
│       └── ResourceManager.gd         # Resource renderer (uses trees)
└── docs/
    ├── TREE_PALETTE_IMPLEMENTATION_GUIDE.md  # Full system design
    ├── TREE_PALETTE_FINDINGS.md               # Initial discovery
    ├── TREE_PALETTE_SYSTEM.md                 # Technical analysis
    └── TREE_EXTRACTION_WITH_PALETTE.md        # This file
```

## Technical Details

### ColorTable Structure

**Format:** PNG, 320×80 pixels, 221 unique colors

**Coordinate Mapping:**
- X-axis: 32 possible values (0-310 in steps of 10) from red channel
- Y-axis: 32 possible values (0-310 in steps of 10) from green channel
- Total: 32×32 = 1,024 possible coordinate combinations

**Why 320×80 instead of 32×32?**
- Each "cell" is 10×10 pixels to allow better palette storage
- Provides smoother gradients and more colors per palette
- 10 palettes stacked? (Needs further investigation - current implementation uses single palette per file)

### Red+Green Coordinate System

Tree textures don't use direct RGB colors. Instead:
- **Red channel** = X coordinate into ColorTable (0-255 → 0-310)
- **Green channel** = Y coordinate into ColorTable (0-255 → 310)
- **Blue channel** = Unused (typically 0 or matching green)
- **Alpha channel** = Transparency (0-255)

**Example:**
```
Pixel at (10, 20) in tree texture has RGB(144, 96, 0):
  - Red 144 → X = (144 // 8) * 10 = 180
  - Green 96 → Y = (96 // 8) * 10 = 120
  - Lookup ColorTable[180, 120] → Returns RGB(45, 98, 32) = dark green
```

### Why Not Use ImageMagick -clut?

ImageMagick's `-clut` (Color Look-Up Table) assumes:
- Source image has grayscale or simple palette indices
- ColorTable is 1D (single row/column)
- Direct index → color mapping

Stone-kingdoms uses:
- Source has red+green coordinate pairs
- ColorTable is 2D grid
- Requires custom coordinate calculation

**Result of -clut:** Produces magenta/blue garbage because it treats red/green as regular colors, not coordinates.

## Testing

### Verify Extraction

```bash
# Check extracted files
ls -lh godot-viewer/assets/tiles/trees/

# Expected: 47 files (25 pine + 22 birch)
# Each 6-7 KB, green foliage visible

# View a tree
open godot-viewer/assets/tiles/trees/tree_pine_large_01.png
```

### Test in Godot Viewer

```bash
# Start backend
cargo run --bin life-simulator

# Launch Godot viewer
cd godot-viewer
/Applications/Godot.app/Contents/MacOS/Godot --path .
# Press F5 to run
```

**Expected result:**
- Trees render with green foliage textures
- Rocks/bushes/flowers still use emojis
- Tree sprites Y-sorted correctly
- Nearest-neighbor filtering (pixel-perfect)

### Debug Output

Look for in Godot console:
```
🌲 Loading tree textures from extracted stone-kingdoms tiles...
  Loading pine trees...
  Loading birch trees...
✅ Loaded 47 tree textures (Pine: 25, Birch: 22)
🌳 Rendered 15 resources for chunk 0,0
```

## Future Enhancements

### Multiple Color Palettes

To add seasonal color variation:

1. Extract trees with different ColorTables:
   ```bash
   # Spring (green)
   ./scripts/extract_tree_textures_with_palette.sh  # ColorTable1

   # Autumn (red/yellow) - Modify script to use ColorTable5
   # Save to: godot-viewer/assets/tiles/trees_autumn/
   ```

2. Update TreeTextureManager to load multiple seasons:
   ```gdscript
   var pine_spring_textures: Array[Texture2D] = []
   var pine_autumn_textures: Array[Texture2D] = []

   func get_tree_texture(tree_type: String, season: String) -> Texture2D:
       # Return appropriate texture based on season
   ```

3. Add season parameter to world data or time system

### Runtime Palette Shader (Advanced)

For true dynamic palette swapping like stone-kingdoms:
- Keep indexed tree PNGs (red+green coordinates)
- Load ColorTable textures as shader uniforms
- Implement canvas_item shader with 2D lookup
- Pass palette index per tree instance

See `TREE_PALETTE_IMPLEMENTATION_GUIDE.md` for full shader implementation details.

## Troubleshooting

### Trees appear with wrong colors
**Check:** Virtual environment activated before running script
```bash
source /tmp/palette_env/bin/activate
python3 scripts/apply_tree_palette.py ...
```

### "No module named 'PIL'" error
**Fix:** Install Pillow in virtual environment
```bash
source /tmp/palette_env/bin/activate
pip install Pillow
```

### Trees not loading in Godot
**Check:**
1. Files exist in `godot-viewer/assets/tiles/trees/`
2. No `.gdignore` file in trees directory
3. TreeTextureManager properly loaded in ResourceManager

### Trees positioned incorrectly
**Solution:** Match emoji positioning pattern
```gdscript
sprite.centered = false  # Use top-left as origin
sprite.position = Vector2(-texture_width / 2.0, -texture_height)
# Centers horizontally, aligns bottom with tile center
```

**Check:**
- Config offset_y is negative (raises tree above ground)
- Using `map_to_local()` to convert tile → pixel coordinates
- sprite.position matches emoji label.position pattern

## Summary

**Current Implementation:** Pre-rendered trees with spring green palette (ColorTable1)

**Advantages:**
- Simple integration (no shader complexity)
- Works immediately in Godot
- 47 texture variants provide good visual variety
- Pixel-perfect rendering with nearest-neighbor filtering

**Trade-offs:**
- No runtime color variation (would need re-extraction for different seasons)
- Not technically authentic to stone-kingdoms (they use runtime shaders)

**Future Path:**
- Phase 1 (Current): Pre-rendered green trees ✅
- Phase 2 (Optional): Add autumn/winter pre-rendered variants
- Phase 3 (Advanced): Implement full shader-based palette system

## References

- `TREE_PALETTE_IMPLEMENTATION_GUIDE.md` - Complete system design and shader implementation
- `TREE_PALETTE_FINDINGS.md` - Initial palette system discovery
- `TREE_PALETTE_SYSTEM.md` - Technical deep dive
- `/Users/jean/Github/stone-kingdoms/shaders/main.glsl` - Original shader code
- `/Users/jean/Github/stone-kingdoms/colortables/` - All ColorTable PNG files
