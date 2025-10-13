# 🔬 Tree Palette System - Analysis Results

## What We Found

### Extracted Tree Textures

**Format:** 8-bit indexed/palette PNG
**Type:** `PaletteAlpha` (not full RGB!)
**Example:** `tree_pine_large_01.png` - 75×157 pixels, 8-bit colormap

```bash
$ file tree_pine_large_01.png
PNG image data, 75 x 157, 8-bit colormap, non-interlaced

$ identify -verbose tree_pine_large_01.png | grep Type
Type: PaletteAlpha
```

**This confirms:** The trees use an indexed color palette system, NOT direct RGB colors.

### Color Palette Tables

**Format:** 320×80 pixel PNG
**Type:** `PaletteAlpha` with 221 colors
**Location:** `/Users/jean/Github/stone-kingdoms/colortables/{TreeType}/ColorTable1-10.png`

```bash
$ identify ColorTable1.png
PNG 320x80 320x80+0+0 8-bit sRGB 1168B

$ identify -verbose ColorTable1.png | grep Colors
Colors: 221
```

**This means:** Each color table is a 320×80 image (not a simple 1D lookup) with 221 unique colors.

## How Stone-Kingdoms Renders Trees

1. **Base Texture**: 8-bit palette/indexed image (e.g., `tree_pine_large_01.png`)
2. **Palette Selection**: Runtime selects one of 10 color tables (`ColorTable1-10.png`)
3. **Shader Lookup**: Shader uses the color table to map palette indices → RGB colors
4. **Result**: Same tree texture renders in 10 different color variations

## Current Status

✅ **Extracted:** 47 tree textures (25 pine, 22 birch)
✅ **Identified:** 10 color palettes per tree type
✅ **Confirmed:** Palette-based rendering system
❌ **Not Implemented:** Godot palette shader system

## Options for Implementation

### Option 1: Full Palette Shader System ⭐ (Authentic)

**Complexity:** High
**Authenticity:** 100% matches stone-kingdoms
**Memory:** Low (1 texture + 10 small palettes)

**What to do:**
1. Keep extracted palette-indexed PNGs as-is
2. Extract all 10 color table PNGs per tree type
3. Implement Godot shader that:
   - Reads tree texture (palette indices)
   - Reads color table texture
   - Maps palette index → color table → final RGB
4. Pass `palette_index` (0-9) as shader uniform per tree instance

**Pros:**
- Exact stone-kingdoms behavior
- Low memory (25 textures + 10 palettes = 35 files for pine)
- Authentic color variation system
- Can add more palettes easily

**Cons:**
- Requires custom shader development
- More complex to implement
- Need to understand palette mapping logic

### Option 2: Pre-Render All Combinations (Simple)

**Complexity:** Low
**Authenticity:** Visual match, not technical match
**Memory:** High (250 PNG files for pine alone)

**What to do:**
1. Write script to apply each color table to each tree texture
2. Generate: 25 pine × 10 palettes = 250 pre-rendered PNGs
3. Use TreeTextureManager like GrassTextureManager
4. Random selection from 250 variants

**Pros:**
- No shader complexity
- Works exactly like grass tiles
- Immediate implementation
- Guaranteed to work

**Cons:**
- 250 files per tree type (pine, birch, chestnut = 750+ files!)
- Larger memory footprint
- Not technically authentic
- Hard to add new palette variations later

### Option 3: Selective Pre-Render (Compromise) ⭐ (Recommended)

**Complexity:** Low-Medium
**Authenticity:** Good visual match
**Memory:** Medium (75-150 files per tree type)

**What to do:**
1. Examine all 10 color tables
2. Select best 3-5 palettes (e.g., spring green, summer, autumn, winter)
3. Pre-render: 25 pine × 3 palettes = 75 PNGs
4. Use TreeTextureManager approach

**Pros:**
- Manageable file count (75 vs 250)
- Good color variety
- Simple implementation
- Can always add more later

**Cons:**
- Less variety than full 10 palettes
- Still pre-rendering overhead
- Not technically authentic

## Comparison Table

| Aspect | Option 1 (Shader) | Option 2 (Full Pre-render) | Option 3 (Selective) |
|--------|-------------------|---------------------------|----------------------|
| Files (Pine) | 35 | 250 | 75 |
| Memory | ~200 KB | ~1.8 MB | ~540 KB |
| Complexity | High | Low | Low-Medium |
| Authenticity | 100% | Visual only | Visual only |
| Flexibility | High | Low | Medium |
| Implementation Time | 2-3 hours | 30 mins | 1 hour |

## Recommended Next Steps

### Immediate (Option 3 - Recommended)

1. **View all 10 color tables** to see color variations:
   ```bash
   open /Users/jean/Github/stone-kingdoms/colortables/PineTree/*.png
   ```

2. **Select 3 best palettes** (e.g., ColorTable1, ColorTable5, ColorTable9)

3. **Create pre-rendering script** using ImageMagick or Python PIL:
   ```python
   # Pseudo-code
   for tree in tree_textures:
       for palette in selected_palettes:
           apply_palette(tree, palette)
           save(f"{tree}_{palette}.png")
   ```

4. **Integrate like grass tiles**:
   - TreeTextureManager.gd
   - Load 75 pre-rendered variants
   - Random selection per tree placement

### Later (Option 1 - For Authenticity)

If you want the full palette system:

1. **Study stone-kingdoms shader code** for palette mapping logic
2. **Create Godot canvas_item shader** with palette lookup
3. **Load palette textures** as shader parameters
4. **Apply per-instance** with different palette indices

## Files to Examine

**Extracted trees (palette-indexed):**
```
godot-viewer/assets/tiles/trees/
├── tree_pine_large_01.png  (75×157, indexed)
├── tree_pine_large_02.png
├── ...
└── tree_birch_large_01.png
```

**Color palettes:**
```
/Users/jean/Github/stone-kingdoms/colortables/
├── PineTree/ColorTable1-10.png  (320×80 each)
├── BirchTree/ColorTable1-10.png
└── ChestnutTree/ColorTable1-10.png
```

## Question for You

**Which approach do you prefer?**

A. **Option 1 (Shader)** - Full palette system, authentic, complex
B. **Option 2 (Full Pre-render)** - Simple, heavy, 750+ files
C. **Option 3 (Selective)** - Compromise, 3-5 palettes, manageable ⭐

I recommend **Option 3** for now (quick results), then upgrade to **Option 1** later if you want the full authentic system.
