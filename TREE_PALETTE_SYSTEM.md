# 🎨 Stone-Kingdoms Tree Palette System

## Discovery

Stone-kingdoms trees use a **shader-based palette system** - very different from grass tiles!

### How It Works

1. **Grayscale Base Textures**: Trees are stored as grayscale/indexed color images in the atlas
2. **Color Palette Lookup Tables**: Each tree type has 10 color palette PNG files (1KB each)
3. **Shader Application**: At runtime, a shader applies the selected palette to the grayscale texture
4. **Variation**: `self.pallete` value (0-9) determines which color table to use

### Directory Structure

```
/Users/jean/Github/stone-kingdoms/colortables/
├── PineTree/
│   ├── ColorTable1.png  (1.2 KB)
│   ├── ColorTable2.png
│   ├── ...
│   └── ColorTable10.png
├── BirchTree/
│   ├── ColorTable1.png  (1.3 KB)
│   ├── ColorTable2.png
│   ├── ...
│   └── ColorTable10.png
└── ChestnutTree/
    ├── ColorTable1.png  (1.2 KB)
    ├── ColorTable2.png
    ├── ...
    └── ColorTable10.png
```

### Code References

**Object.lua (line 141):**
```lua
self.pallete = _G.ctables[_G.colortables[classname][number]] + 1
```

**Tree.lua (line 106):**
```lua
self.instancemesh:setVertex(self.vertId, x, y, self:inferZ(),
    qx, qy, qw, qh, 1, 1, 1, self.pallete)  -- Last param is palette index
```

### Extracted Trees So Far

We've extracted:
- **25 Pine tree variants** (75×157-161 px) - GRAYSCALE
- **22 Birch tree variants** (63×118-123 px) - GRAYSCALE

These are likely grayscale textures that need palette application!

## Implementation Options

### Option 1: Full Palette System (Complex but Authentic)

**Pros:**
- Exactly matches stone-kingdoms
- Memory efficient (1 texture + 10 palettes)
- Authentic color variation

**Cons:**
- Requires implementing shader system in Godot
- Need to extract grayscale textures correctly
- Need to load and apply color lookup tables
- Complex to implement

**Steps:**
1. Verify extracted textures are grayscale/indexed
2. Extract all color palette PNGs (10 per tree type)
3. Create Godot shader that applies palette lookup
4. Load palette textures as shader uniforms
5. Apply shader to tree sprites with palette index

**Godot Shader Example:**
```glsl
shader_type canvas_item;

uniform sampler2D color_palette : hint_albedo;
uniform float palette_index : hint_range(0, 9) = 0;

void fragment() {
    vec4 base_color = texture(TEXTURE, UV);
    float gray = base_color.r;  // Grayscale value
    vec2 palette_uv = vec2(gray, palette_index / 10.0);
    COLOR = texture(color_palette, palette_uv);
}
```

### Option 2: Pre-Rendered Textures (Simple but Memory Heavy)

**Pros:**
- No shader complexity
- Works like grass tiles (simple)
- Immediate results

**Cons:**
- Need to pre-render: 25 pine × 10 palettes = 250 PNG files
- Same for birch: 22 × 10 = 220 PNG files
- Much larger memory footprint
- Not authentic to original system

**Steps:**
1. Write script to apply each ColorTable to each tree texture
2. Save 250+ pre-rendered color variations
3. Load like grass tiles (TreeTextureManager)
4. Random selection from variants

### Option 3: Simplified Palette (Compromise)

**Pros:**
- Pick 2-3 best color palettes
- Still get variety without full complexity
- Reasonable file count

**Cons:**
- Less variety than full system
- Still need palette application or pre-rendering

**Steps:**
1. Identify best 2-3 color palettes (e.g., ColorTable1, ColorTable5, ColorTable9)
2. Pre-render: 25 pine × 3 palettes = 75 PNG files
3. More manageable than full 250 files

## Comparison to Grass Tiles

| Feature | Grass Tiles | Tree Textures |
|---------|-------------|---------------|
| **Storage** | Full-color RGB | Grayscale + Palette |
| **Variants** | 8 textures | 25 textures × 10 palettes |
| **Size** | 1.2 KB each | 5-7 KB base + 1KB palettes |
| **Rendering** | Direct texture | Shader palette lookup |
| **Complexity** | Simple | Complex |

## Recommendation

For **quick results**, I recommend **Option 3** (Simplified Palette):

1. **Extract 2-3 best color palettes** per tree type
2. **Pre-render combinations** using ImageMagick or Python
3. **Use existing TreeTextureManager** approach (like grass)
4. **Get immediate visual results** without shader complexity

Later, if needed, you can upgrade to the full palette shader system (Option 1) for authenticity.

## What Colors Look Like?

The ColorTable PNG files are small (256×1 pixel) lookup tables where:
- X-axis = input grayscale value (0-255)
- Pixel color = output RGB color

This allows the shader to map each gray shade to a specific color, creating variations like:
- Darker/lighter leaves
- Autumn colors (reds, yellows)
- Spring greens vs summer greens
- Dead/dry brown trees

## Next Steps

1. **Check extracted textures**: Verify if they're grayscale or already colored
2. **View ColorTable PNGs**: See what color variations exist
3. **Choose approach**: Option 1, 2, or 3?
4. **Implement**: Based on chosen approach

---

**Files to examine:**
- `godot-viewer/assets/tiles/trees/tree_pine_large_01.png` (check if grayscale)
- `/Users/jean/Github/stone-kingdoms/colortables/PineTree/ColorTable1.png` (view palette)
