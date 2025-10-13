# 🎨 Tree Palette System - Implementation Guide

## Overview

Stone-kingdoms uses a sophisticated **palette-indexed texture system** for trees that allows runtime color variation through shader-based palette lookup. This document explains how to implement this system in Godot.

## Current Status (2025-01-11)

### ✅ What We Have Now
- **Pre-rendered trees**: Trees extracted with one palette applied (autumn red)
- **Simple integration**: Trees loaded as standard PNG textures via TreeTextureManager
- **Working but limited**: No runtime color variation

### 🎯 Future Goal
- **Dynamic palette system**: Runtime shader-based palette swapping
- **Color variations**: 10 different color palettes per tree type (spring, summer, autumn, winter, etc.)
- **Memory efficient**: One base texture + 10 small palette textures instead of 250 pre-rendered variants

## How Stone-Kingdoms Palette System Works

### 1. Indexed Color Textures

Trees are stored as **8-bit palette-indexed PNG images**:

```bash
$ identify -verbose tree_pine_large_01.png | grep Type
Type: PaletteAlpha

$ file tree_pine_large_01.png
PNG image data, 75 x 157, 8-bit colormap, non-interlaced
```

**What this means:**
- Each pixel stores an **index** (0-255) into a color palette
- The actual RGB color is determined by looking up that index in a **ColorTable**
- NOT direct RGB color data

### 2. Color Lookup Tables

Each tree type has **10 ColorTable PNG files**:

```
/colortables/
├── PineTree/
│   ├── ColorTable1.png   (320×80, 1.1 KB)
│   ├── ColorTable2.png
│   ├── ...
│   └── ColorTable10.png
├── BirchTree/
│   ├── ColorTable1.png   (320×80, 1.3 KB)
│   └── ...
└── ChestnutTree/
    ├── ColorTable1.png
    └── ...
```

**ColorTable format:**
- Size: 320×80 pixels
- Contains: 221 unique colors
- Purpose: Maps palette index → RGB color

### 3. Shader-Based Rendering

Stone-kingdoms renders trees using a **mesh vertex shader**:

**Tree.lua (line 106):**
```lua
self.instancemesh:setVertex(self.vertId, x, y, self:inferZ(),
    qx, qy, qw, qh, 1, 1, 1, self.pallete)  -- Last param = palette index (0-9)
```

**Object.lua (line 141):**
```lua
self.pallete = _G.ctables[_G.colortables[classname][number]] + 1
```

**How it works:**
1. Tree entity is assigned a `palette` value (0-9) at spawn time
2. Vertex shader receives palette index as uniform
3. Shader reads pixel from indexed tree texture → gets palette index
4. Shader looks up that index in the selected ColorTable → gets final RGB color
5. Fragment shader outputs the final color

## Implementation Options

### Option 1: Simple Pre-Rendered (Current) ⭐ RECOMMENDED FOR NOW

**What we're using:**
- Extract trees with one palette pre-applied (spring green)
- Load as standard Godot textures
- No shader complexity

**Pros:**
- ✅ Works immediately
- ✅ No shader programming needed
- ✅ Simple integration

**Cons:**
- ❌ No runtime color variation
- ❌ To add more colors, must re-extract all trees

**Implementation:**
```gdscript
# TreeTextureManager.gd
var pine_tree_textures: Array[Texture2D] = []

func load_tree_textures():
    for i in range(1, 26):
        var texture = load("res://assets/tiles/trees/tree_pine_large_%02d.png" % i)
        pine_tree_textures.append(texture)

func get_random_tree() -> Texture2D:
    return pine_tree_textures[randi() % pine_tree_textures.size()]
```

### Option 2: Full Palette Shader System ⭐ FUTURE AUTHENTIC IMPLEMENTATION

**What stone-kingdoms does:**
- Keep indexed tree PNGs as-is
- Load ColorTable textures
- Implement Godot canvas_item shader for palette lookup
- Pass palette index as shader uniform per tree instance

**Pros:**
- ✅ Exactly matches stone-kingdoms
- ✅ Memory efficient (25 textures + 10 palettes = 35 files for pine)
- ✅ Runtime color variation (10 palettes available)
- ✅ Easy to add more palettes later

**Cons:**
- ❌ Requires custom shader development
- ❌ Need to understand palette mapping logic
- ❌ More complex integration

**Implementation Steps:**

#### Step 1: Extract Indexed Tree Textures

Extract trees **without** color conversion (keep palette indices):

```bash
# Extract with -define png:preserve-colormap
magick /path/to/atlas.png -crop 75x157+2981+12264 +repage \
    -define png:preserve-colormap \
    tree_pine_large_01_indexed.png
```

#### Step 2: Extract ColorTable Textures

Copy all ColorTable PNG files to Godot assets:

```bash
cp /Users/jean/Github/stone-kingdoms/colortables/PineTree/*.png \
   godot-viewer/assets/colortables/pine/
```

#### Step 3: Create Godot Palette Shader

**palette_tree.gdshader:**
```glsl
shader_type canvas_item;

// The indexed tree texture (8-bit palette indices)
uniform sampler2D texture_atlas : filter_nearest;

// The color lookup table (320×80 with 221 colors)
uniform sampler2D color_table : filter_nearest;

// Which palette to use (0-9)
uniform int palette_index : hint_range(0, 9) = 0;

void fragment() {
    // Sample the indexed texture to get palette index
    vec4 indexed_color = texture(texture_atlas, UV);
    float palette_idx = indexed_color.r;  // Red channel has the index

    // Calculate UV for ColorTable lookup
    // X = palette index (0-255 normalized), Y = which palette row (0-9)
    vec2 palette_uv = vec2(
        palette_idx,  // Index position (0.0-1.0)
        float(palette_index) / 10.0  // Palette row (0.0-0.9)
    );

    // Look up the final color in the ColorTable
    vec4 final_color = texture(color_table, palette_uv);
    final_color.a = indexed_color.a;  // Preserve alpha

    COLOR = final_color;
}
```

#### Step 4: Apply Shader to Tree Sprites

```gdscript
# ResourceManager.gd - Tree rendering with palette shader

func create_tree_sprite(tree_type: String, palette_idx: int) -> Sprite2D:
    var sprite = Sprite2D.new()

    # Load indexed tree texture
    var tree_texture = load("res://assets/tiles/trees/tree_pine_large_01_indexed.png")
    sprite.texture = tree_texture

    # Load palette shader material
    var material = ShaderMaterial.new()
    material.shader = load("res://shaders/palette_tree.gdshader")

    # Load ColorTable texture
    var color_table = load("res://assets/colortables/pine/ColorTable%d.png" % (palette_idx + 1))
    material.set_shader_parameter("color_table", color_table)
    material.set_shader_parameter("palette_index", palette_idx)
    material.set_shader_parameter("texture_atlas", tree_texture)

    sprite.material = material
    sprite.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST

    return sprite
```

#### Step 5: Spawn Trees with Random Palettes

```gdscript
func paint_tree_resource(tile_pos: Vector2i, tree_type: String):
    var palette_idx = randi() % 10  # Random palette 0-9
    var tree_sprite = create_tree_sprite(tree_type, palette_idx)

    var pixel_pos = tilemap.map_to_local(tile_pos)
    tree_sprite.position = pixel_pos
    tree_sprite.offset.y = -tree_sprite.texture.get_height() / 2.0  # Center sprite

    add_child(tree_sprite)
```

### Option 3: Selective Pre-Render (Compromise)

**Middle ground approach:**
- Select 3-5 best ColorTables (spring, summer, autumn, winter, dead)
- Pre-render trees with those palettes
- Load 75-150 pre-rendered variants (25 trees × 3-5 palettes)

**Pros:**
- ✅ Some color variety
- ✅ No shader complexity
- ✅ Manageable file count (75-150 vs 250)

**Cons:**
- ❌ Still requires pre-rendering step
- ❌ Less variety than full 10 palettes
- ❌ Not technically authentic

## Palette Selection Guide

### Identifying ColorTable Purposes

To select palettes, examine all 10 ColorTable PNG files:

```bash
open /Users/jean/Github/stone-kingdoms/colortables/PineTree/*.png
```

**Expected variations:**
- **Spring**: Bright greens, fresh leaves
- **Summer**: Dark greens, full foliage
- **Autumn**: Reds, yellows, oranges
- **Winter**: Browns, sparse foliage
- **Dead**: Gray-browns, no leaves

### Applying Palettes with ImageMagick

To apply a ColorTable to an indexed tree PNG:

```bash
# Method 1: Using CLUT (Color Look-Up Table)
magick tree_pine_large_01_indexed.png \
    ColorTable5.png -clut \
    tree_pine_large_01_green.png

# Method 2: Using palette swap
convert tree_pine_large_01_indexed.png \
    -type Palette \
    +dither -colors 256 \
    -remap ColorTable5.png \
    tree_pine_large_01_green.png
```

## Technical Details

### Palette Index Format

**How indices map to ColorTable:**
- ColorTable is 320×80 = 25,600 pixels
- Indexed tree texture has palette indices 0-255
- Shader samples ColorTable at `(index/255.0, palette_row/10.0)`

### ColorTable Structure

```
Width: 320 pixels
Height: 80 pixels
Rows: 10 (one per palette)
Each row: 320 colors representing full palette range
```

**Example ColorTable layout:**
```
Row 0 (Y=0-7):   ColorTable1 colors (palette 0)
Row 1 (Y=8-15):  ColorTable2 colors (palette 1)
...
Row 9 (Y=72-79): ColorTable10 colors (palette 9)
```

### Memory Comparison

| Approach | Files | Total Size | Runtime Memory |
|----------|-------|------------|----------------|
| **Current (Pre-rendered)** | 47 trees | ~330 KB | ~330 KB |
| **Full Palette System** | 47 + 20 palettes | ~350 KB | ~330 KB + shader overhead |
| **Full Pre-render** | 470 trees (47×10) | ~3.3 MB | ~3.3 MB |

## Migration Path

### Phase 1: Current (Working Now) ✅
- Use pre-rendered trees with spring green palette
- Simple TreeTextureManager loading
- No shader complexity

### Phase 2: Add More Pre-Rendered Variants (Optional)
- Extract additional color variants (autumn, winter)
- Expand TreeTextureManager to handle multiple color sets
- Random selection between color variants

### Phase 3: Full Palette System (Future)
- Implement shader-based palette lookup
- Extract indexed tree textures
- Copy ColorTable files
- Create palette shader
- Update ResourceManager to use shader material

## Testing Palette Application

### Test Script: Apply ColorTable to Tree

```bash
#!/bin/bash
# test_palette_application.sh

TREE_FILE="godot-viewer/assets/tiles/trees/tree_pine_large_01_indexed.png"
COLOR_TABLE="/Users/jean/Github/stone-kingdoms/colortables/PineTree/ColorTable1.png"
OUTPUT_FILE="/tmp/tree_test_palette1.png"

# Apply palette using ImageMagick CLUT
magick "$TREE_FILE" "$COLOR_TABLE" -clut "$OUTPUT_FILE"

echo "Applied ColorTable1 to tree"
echo "Output: $OUTPUT_FILE"
open "$OUTPUT_FILE"
```

### Verify Palette Variations

```bash
# Apply all 10 palettes to one tree to see variations
for i in {1..10}; do
    magick tree_pine_large_01_indexed.png \
        ColorTable${i}.png -clut \
        tree_test_palette${i}.png
done

# View all variations
open tree_test_palette*.png
```

## References

### Files to Examine

**Source (stone-kingdoms):**
- `/Users/jean/Github/stone-kingdoms/colortables/PineTree/ColorTable1-10.png`
- `/Users/jean/Github/stone-kingdoms/colortables/BirchTree/ColorTable1-10.png`
- `/Users/jean/Github/stone-kingdoms/objects/Environment/Tree.lua` (line 106)
- `/Users/jean/Github/stone-kingdoms/objects/Object.lua` (line 141)

**Extracted (current):**
- `godot-viewer/assets/tiles/trees/tree_pine_large_01.png` (47 trees total)

**Documentation:**
- `TREE_PALETTE_FINDINGS.md` - Initial discovery and analysis
- `TREE_PALETTE_SYSTEM.md` - Technical deep dive
- This file - Implementation guide

### External Resources

- **Godot Shaders**: https://docs.godotengine.org/en/stable/tutorials/shaders/shader_reference/canvas_item_shader.html
- **ImageMagick CLUT**: https://imagemagick.org/script/command-line-options.php#clut
- **PNG Palette Format**: https://www.w3.org/TR/PNG/#11Palette

## Troubleshooting

### Issue: Trees appear all one color
**Cause**: ColorTable may not be applied correctly
**Solution**: Verify indexed texture format with `identify -verbose`

### Issue: Shader not applying palette
**Cause**: Texture sampler not set or wrong UV calculation
**Solution**: Debug shader with solid color output first, then add palette lookup

### Issue: Pre-rendered trees too large file size
**Cause**: PNG not optimized
**Solution**: Use `pngcrush` or `optipng` to compress

```bash
optipng -o7 tree_pine_large_*.png
```

## Summary

For **immediate use**: Pre-rendered trees with spring green palette (Phase 1 - current)
For **future authenticity**: Implement shader-based palette system (Phase 3)
For **compromise**: Pre-render 3-5 color variants (Phase 2)

The palette system is well-documented and ready for implementation when needed. Current approach provides working trees with minimal complexity.
