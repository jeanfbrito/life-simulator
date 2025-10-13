# OpenRCT2 Terrain Sprite Extraction & Replacement Guide

**Purpose:** Extract OpenRCT2 terrain sprites as placeholders while creating custom art, then gradually replace them.

**Philosophy:** OpenRCT2 is excellence in tile-based terrain rendering. We copy their approach exactly, then customize with our own art.

---

## 🎯 OpenRCT2 Terrain System Overview

### Core Structure

From `src/openrct2/object/TerrainSurfaceObject.h`:

```cpp
class TerrainSurfaceObject {
    static constexpr auto kNumImagesInEntry = 19;  // ← 19 slope variations!

    uint32_t PatternBaseImageId;     // Base sprite ID
    uint32_t EntryBaseImageId;       // Entry sprite ID
    uint32_t NumEntries;             // Number of texture variations
    colour_t Colour;                 // Base color
    uint8_t Rotations;               // Rotation variants
};
```

**Key Insight:** Each terrain type has **19 images** representing different slope combinations!

---

## 🗺️ The 19 Slope Variations

Based on `src/openrct2/world/tile_element/Slope.h`:

```
INDEX  SLOPE TYPE           DESCRIPTION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
0      Flat                 [____]  All corners same height

1      N corner up          [/¯¯¯]  North raised
2      E corner up          [¯¯¯\]  East raised
3      NE side up           [/¯¯\]  North-East slope
4      S corner up          [___/]  South raised
5      NS valley            [/¯_¯\] North-South valley
6      SE side up           [¯¯\_]  South-East slope
7      NSE (3 corners up)   [/¯¯_]  North-South-East raised

8      W corner up          [\___]  West raised
9      NW side up           [\¯¯_]  North-West slope
10     EW valley            [¯\_/¯] East-West valley
11     NWE (3 corners up)   [\¯¯\]  North-West-East raised
12     SW side up           [_/¯¯]  South-West slope
13     SWN (3 corners up)   [_/¯_]  South-West-North raised
14     SWE (3 corners up)   [¯_/¯]  South-West-East raised
15     All corners up       [/¯¯\]  All raised (plateau)

16-18  Diagonal variations  Diagonal slopes (advanced)
```

**Visual Reference:**

```
Flat (0):      NE Slope (3):    Valley (5):      All Up (15):
   [____]         [/¯¯\]           [/¯_¯\]          [/¯¯¯\]
```

---

## 📦 Where OpenRCT2 Sprites Are

### Option 1: Extract from Game Assets

OpenRCT2 stores sprites in proprietary format (.DAT files from original RCT2).

**Tools to extract:**
- [OpenRCT2/OpenRCT2 Object Editor](https://github.com/OpenRCT2/OpenRCT2)
- [ParkitecTools](https://github.com/Parkitect/ParkitectAssetEditor) (similar format)
- [RCT2 Sprite Extractor](https://www.rctspace.com/topic_4488.html)

### Option 2: Use Community Extracted Sprites

OpenRCT2 community has already extracted many sprites to PNG:
- [OpenRCT2 Objects Repository](https://github.com/OpenRCT2/objects)
- Check `objects/terrain` directory

### Option 3: Screenshot Method (Quick & Dirty)

For rapid prototyping:

1. **Run OpenRCT2** with scenario editor
2. **Open Godot** or image editor
3. **Screenshot individual terrain tiles** at 1:1 zoom
4. **Crop to exact tile size** (isometric diamond)
5. **Organize by slope type** (0-18)

---

## 🎨 Tile Specifications

### Isometric Diamond Dimensions

Based on typical RCT2 tiles:

```
Standard Tile Size: 32×16 pixels (isometric diamond)

Larger tiles (with height): 32×32 pixels
- Accommodates vertical walls
- Extra height for slopes

Actual OpenRCT2 dimensions:
- Base: 32×16 (flat tile)
- With height: 32×32 (allows up to 16px height variation)
```

**Your Current Godot Tiles:** 32×16 pixels (OpenRCT2 original size)
- Perfect! Use RCT2 sprites directly, no scaling needed

### Color Palette

OpenRCT2 uses indexed color palette:
- Each terrain has base color
- Slope variations are shaded versions
- Grass: Multiple greens (seasonal)
- Sand: Tan/beige tones
- Water: Blues with transparency

---

## 🔧 Extraction Process

### Step 1: Get Base Terrain Types

**Terrain Types to Extract:**
```
Priority (matching life-simulator needs):
1. Grass (green, lush)
2. Forest floor (dark green/brown)
3. Sand (beaches)
4. Dirt (brown)
5. Stone/Rock (gray)
6. Water (shallow blue)
7. Deep water (dark blue)
8. Snow (white/light blue) [future]
9. Mountain rock (gray-white)
```

### Step 2: Extract All 19 Slope Variations

For EACH terrain type, extract sprites for slopes 0-18.

**File Naming Convention:**
```
terrain_{type}_{slope}.png

Examples:
terrain_grass_00.png    # Flat grass
terrain_grass_01.png    # Grass N-up
terrain_grass_03.png    # Grass NE-up
terrain_grass_15.png    # Grass all-up

terrain_sand_00.png     # Flat sand
terrain_sand_03.png     # Sand NE-up
...
```

### Step 3: Organize in Godot Project (No Scaling Needed!)

```
godot-viewer/
  assets/
    tiles/
      terrain/
        openrct2_placeholder/  ← Temporary extracted sprites
          grass/
            slope_00.png
            slope_01.png
            ...slope_18.png
          sand/
            slope_00.png
            ...
          stone/
            ...

        custom/                ← Your drawn replacements
          grass/
            slope_00.png       ← Gradually replace these!
            slope_01.png
            ...
```

---

## 🎮 Implementation in Godot

### Phase 1: Use Placeholders

```gdscript
# In TerrainTileMap.gd

const TERRAIN_SPRITE_PATH = {
    "Grass": "res://assets/tiles/terrain/openrct2_placeholder/grass/",
    "Sand": "res://assets/tiles/terrain/openrct2_placeholder/sand/",
    "Stone": "res://assets/tiles/terrain/openrct2_placeholder/stone/",
    # ... more terrain types
}

func get_slope_sprite_path(terrain_type: String, slope_index: int) -> String:
    var base_path = TERRAIN_SPRITE_PATH.get(terrain_type, "")
    return base_path + "slope_%02d.png" % slope_index

func render_tile_with_slope(pos: Vector2i, terrain: String, height: int):
    # Calculate slope from neighbor heights
    var slope_index = calculate_slope_index(pos, height)

    # Load appropriate sprite
    var sprite_path = get_slope_sprite_path(terrain, slope_index)
    var texture = load(sprite_path)

    # Render to TileMap or as Sprite2D
    # ... rendering code
```

### Phase 2: Calculate Slope Index

```gdscript
func calculate_slope_index(pos: Vector2i, height: int) -> int:
    # Get neighbor heights
    var h_n = get_height(pos + Vector2i(0, -1))  # North
    var h_e = get_height(pos + Vector2i(1, 0))   # East
    var h_s = get_height(pos + Vector2i(0, 1))   # South
    var h_w = get_height(pos + Vector2i(-1, 0))  # West

    # Calculate corner raises (1 if neighbor higher, 0 if same/lower)
    var n_up = 1 if h_n > height else 0
    var e_up = 1 if h_e > height else 0
    var s_up = 1 if h_s > height else 0
    var w_up = 1 if h_w > height else 0

    # Build slope bitfield (matches OpenRCT2)
    var slope = (n_up << 0) | (e_up << 1) | (s_up << 2) | (w_up << 3)

    # Map bitfield to slope index (0-15 for basic slopes)
    # Slope 0 = 0000 (flat)
    # Slope 1 = 0001 (N up)
    # Slope 3 = 0011 (NE up)
    # ... etc

    return slope
```

### Phase 3: Fallback to Shading

```gdscript
func render_tile_with_slope(pos: Vector2i, terrain: String, height: int):
    var slope_index = calculate_slope_index(pos, height)
    var sprite_path = get_slope_sprite_path(terrain, slope_index)

    # Try to load slope sprite
    if ResourceLoader.exists(sprite_path):
        var texture = load(sprite_path)
        render_sprite(pos, texture)
    else:
        # Fallback: Use flat tile with height shading
        var flat_path = get_slope_sprite_path(terrain, 0)
        var texture = load(flat_path)

        # Apply brightness modulation
        var brightness = 0.6 + (height / 255.0) * 0.8
        render_sprite_modulated(pos, texture, Color(brightness, brightness, brightness))
```

---

## 🎨 Custom Art Replacement Workflow

### Step 1: Analyze Placeholder

1. Open `openrct2_placeholder/grass/slope_00.png` in image editor
2. Study the style:
   - Color palette
   - Shading direction (light from NW typically)
   - Texture detail
   - Edge treatment

### Step 2: Create Custom Tile

**Recommended Tools:**
- **Aseprite** - Excellent for pixel art and isometric
- **GIMP** - Free, powerful
- **Photoshop** - Industry standard
- **Krita** - Free, brush-focused

**Process:**
1. Create 32×16 canvas (OpenRCT2 original size)
2. Draw isometric diamond shape
3. Fill with terrain texture
4. Add shading (NW light source)
5. Export as PNG

**For Slopes:**
- Start with flat tile (slope_00)
- Duplicate and add raised corners
- Adjust shading for height
- Maintain consistent light source

### Step 3: Replace Placeholder

```bash
# Move completed art to custom folder
mv custom_grass_flat.png godot-viewer/assets/tiles/terrain/custom/grass/slope_00.png

# Update code to check custom first
```

**Updated Code:**
```gdscript
const TERRAIN_SPRITE_PATHS = {
    "Grass": [
        "res://assets/tiles/terrain/custom/grass/",          # Check custom first
        "res://assets/tiles/terrain/openrct2_placeholder/grass/",  # Fallback
    ],
}

func get_slope_sprite_path(terrain_type: String, slope_index: int) -> String:
    var paths = TERRAIN_SPRITE_PATHS.get(terrain_type, [])
    var filename = "slope_%02d.png" % slope_index

    for base_path in paths:
        var full_path = base_path + filename
        if ResourceLoader.exists(full_path):
            return full_path  # Use first found (custom overrides placeholder)

    return ""  # Not found
```

### Step 4: Track Progress

**Create Replacement Checklist:**

```
## Grass Terrain
- [ ] slope_00 (flat) ← Start here, most common
- [ ] slope_01 (N up)
- [ ] slope_02 (E up)
- [ ] slope_03 (NE up) ← Do corners next
- [ ] slope_04 (S up)
- [ ] slope_06 (SE up)
- [ ] slope_08 (W up)
- [ ] slope_09 (NW up)
- [ ] slope_12 (SW up)
- [ ] slope_05 (NS valley)
- [ ] slope_10 (EW valley)
- [ ] slope_07 (NSE)
- [ ] slope_11 (NWE)
- [ ] slope_13 (SWN)
- [ ] slope_14 (SWE)
- [ ] slope_15 (all up)
- [ ] slope_16-18 (diagonals)

## Sand Terrain
- [ ] slope_00
- [ ] ... (repeat for all slopes)

## Stone Terrain
- [ ] slope_00
- [ ] ...
```

---

## 🚀 Implementation Phases

### Phase 1: Extraction & Setup (Week 1)
**Goal:** Get placeholders working in Godot

**Tasks:**
1. Extract 19 slope sprites for Grass terrain from OpenRCT2
2. Copy directly to Godot (no scaling - use 32×16 original size)
3. Organize in `openrct2_placeholder/grass/` folder
4. Implement slope calculation in Godot
5. Render single terrain type with all slopes
6. Test: Generate world with hills and valleys - grass renders with proper slopes

**Success:** Grass terrain shows natural-looking slopes using OpenRCT2 sprites

---

### Phase 2: All Terrain Types (Week 2)
**Goal:** Extract all needed terrain types

**Tasks:**
1. Extract sprites for: Sand, Stone, Dirt, Water (shallow), Water (deep)
2. Organize in appropriate folders
3. Update Godot terrain mapping
4. Test: World with multiple terrains all showing proper slopes
5. Implement fallback to flat + shading if slope missing

**Success:** All terrain types render with slopes, or gracefully fall back

---

### Phase 3: First Custom Art (Week 3)
**Goal:** Replace Grass flat tile with custom art

**Tasks:**
1. Draw custom grass tile (flat, slope_00)
2. Match OpenRCT2 style but with your own flair
3. Place in `custom/grass/slope_00.png`
4. Test in-game
5. Iterate based on look
6. Document art style guide

**Success:** Custom grass flat tile looks great in-game, matches placeholders

---

### Phase 4: Gradual Replacement (Weeks 4-8)
**Goal:** Replace 1-2 terrain types per week

**Suggested Order:**
1. **Week 4:** Finish Grass (all 19 slopes)
   - Most visible terrain type
   - Set the art style standard

2. **Week 5:** Sand terrain (all 19 slopes)
   - Beaches are prominent
   - Different texture than grass (good variety test)

3. **Week 6:** Stone/Rock terrain (all 19 slopes)
   - Mountains need to look good
   - Practice with gray tones

4. **Week 7:** Water (shallow + deep)
   - Special case: animated or static?
   - Transparency considerations

5. **Week 8:** Dirt, Snow, etc. (remaining types)
   - Less common, lower priority

**Success:** Each week, one more terrain type is fully custom

---

### Phase 5: Polish & Variations (Week 9+)
**Goal:** Add visual richness

**Tasks:**
1. Create 2-3 variations per slope (randomize placement)
2. Add seasonal variations (spring grass, autumn grass)
3. Add detail textures (small rocks, flowers)
4. Implement biome-specific coloring
5. Add edge blending between terrain types

**Success:** World looks vibrant with natural variation

---

## 📐 Art Style Guidelines

### Consistency Rules

**To Match OpenRCT2 Excellence:**

1. **Isometric Perspective:**
   - 26.565° angle (true isometric)
   - Consistent across all tiles

2. **Light Source:**
   - Light from North-West (standard game convention)
   - Consistent shading direction

3. **Color Palette:**
   - Limited palette per terrain (4-8 colors)
   - Consistent saturation levels
   - Good contrast for readability

4. **Detail Level:**
   - Readable at 1:1 zoom
   - Not too busy (avoid visual noise)
   - Clear silhouettes

5. **Edge Treatment:**
   - Clean edges where tiles meet
   - Avoid harsh lines
   - Slight blur for natural look

### Example: Grass Terrain

**Color Palette:**
```
Base Green:     #4CAF50
Dark Shadow:    #2E7D32
Light Highlight:#81C784
Very Dark:      #1B5E20
Mid-tone:       #66BB6A
```

**Texture Elements:**
- Small grass tufts (1-2 pixels)
- Occasional tiny flowers (accent color)
- Subtle ground texture (noise)
- No hard edges (organic feel)

### Slope Shading Guidelines

**Flat tiles (slope_00):**
- Even shading, subtle variation
- Light from NW (top-left lighter)

**Raised corners (slopes 1-15):**
- Raised edges are lighter (catch more light)
- Lowered edges are darker (in shadow)
- Vertical faces (sides) are darkest

**Example - NE Slope (slope_03):**
```
     North (high)
       /¯¯¯¯\       ← Light (top surface)
      │      \      ← Very dark (E vertical face)
West  │       \     East
      │        \
       \________\
          South (high)
```

---

## 🔍 Quality Checklist

Before replacing a placeholder, check:

### Technical Quality
- [ ] Correct dimensions (32×16 - OpenRCT2 original size)
- [ ] Transparent background (if needed)
- [ ] PNG format, indexed color or RGB+Alpha
- [ ] File size reasonable (<10KB per tile)
- [ ] Filename matches convention exactly

### Visual Quality
- [ ] Matches isometric angle
- [ ] Light source consistent (NW)
- [ ] Colors harmonize with other tiles
- [ ] Readable at 1:1 zoom
- [ ] Edges align properly with neighbors
- [ ] No stray pixels or artifacts

### In-Game Quality
- [ ] Looks good in context (surrounded by other tiles)
- [ ] Transitions smoothly between slopes
- [ ] Maintains consistent style with other tiles
- [ ] Performance impact negligible
- [ ] Works with all height levels

---

## 🛠️ Tools & Resources

### Extraction Tools
- **OpenRCT2 Object Editor** - Official tool
- **ParkitectAssetEditor** - Similar game format
- **ImageMagick** - Batch processing
- **FFmpeg** - Animation if needed

### Art Creation Tools
- **Aseprite** ($20) - BEST for pixel art isometric
- **Krita** (Free) - Great brushes
- **GIMP** (Free) - Powerful, but clunky UI
- **Photoshop** - Industry standard (expensive)
- **Procreate** (iPad, $10) - Touch-friendly

### Reference Resources
- **OpenRCT2 Subreddit** - Community art
- **RCT2 Wiki** - Original game assets
- **PixelJoint** - Isometric tutorials
- **Lospec** - Color palette generator

---

## 📊 Progress Tracking

### Recommended Approach

**Create GitHub Issues for Each Terrain:**

```markdown
## Grass Terrain Replacement

**Goal:** Replace all 19 OpenRCT2 grass slopes with custom art

**Checklist:**
- [ ] slope_00 (flat)
- [ ] slope_01 (N up)
- [ ] slope_02 (E up)
- ...
- [ ] slope_18 (diagonal)

**Art Style:**
- Lush green grass
- Small flower accents (yellow, white)
- Organic texture
- Slightly cartoonish (not photorealistic)

**Target:** Week 4 completion

**Preview:**
![Grass Flat](preview_grass_00.png)
```

### Milestones

**Milestone 1: Extraction Complete**
- All OpenRCT2 placeholders extracted
- Organized in folders
- Working in Godot

**Milestone 2: First Custom Terrain**
- Grass terrain 100% custom
- Art style guide documented
- Process refined

**Milestone 3: Core Terrains Done**
- Grass, Sand, Stone, Water all custom
- Game has unique visual identity
- Ready for public showcase

**Milestone 4: All Terrains Complete**
- No more placeholders
- Seasonal variations added
- Polish pass complete

---

## ⚠️ Legal & Attribution

### Using OpenRCT2 Sprites as Placeholders

**IMPORTANT:**

1. **Placeholders Only:**
   - OpenRCT2/RCT2 sprites are for PROTOTYPING
   - Must be replaced before any public release
   - Do NOT distribute game with RCT2 assets

2. **Attribution:**
   - If showing development screenshots publicly, note "placeholder art"
   - Credit OpenRCT2 project for inspiration
   - Make clear your final art is original

3. **License:**
   - OpenRCT2 code is GPL v3
   - Original RCT2 assets are © Atari/Chris Sawyer
   - Your custom art is yours (license as you wish)

4. **Safe Approach:**
   - Study RCT2 sprites, don't copy
   - Recreate style, not exact pixels
   - Make it "inspired by" not "copied from"

### Recommended Disclaimer

**In your README/docs while using placeholders:**

```markdown
## Asset Attribution

**Terrain Sprites:** Currently using placeholder sprites inspired by
OpenRCT2/RollerCoaster Tycoon 2 for prototyping. These will be
replaced with original artwork before release.

Original RCT2 assets © Chris Sawyer / Atari
OpenRCT2 Project: https://github.com/OpenRCT2/OpenRCT2

All final game art will be original work licensed under [Your License].
```

---

## 🎯 TL;DR Quick Start

**5-Minute Setup:**

1. **Find OpenRCT2 grass sprites** (flat + slopes)
2. **Copy directly** (no scaling - use 32×16 original size)
3. **Save as:** `godot-viewer/assets/tiles/terrain/openrct2_placeholder/grass/slope_00.png` (etc.)
4. **Update TerrainTileMap.gd** to load sprites based on slope
5. **Test in-game** - should see sloped terrain!

**Then iterate:**
- Extract more terrain types
- Draw custom replacements
- Gradually replace placeholders
- Track progress with checklists

---

## 📚 Next Steps

1. **Read this guide** ✅ (you are here!)
2. **Extract first terrain type** (grass) - see Phase 1
3. **Implement slope rendering** in Godot - see Implementation section
4. **Test with height map data** from Phase 3.1
5. **Start drawing custom art** - see Art Style Guidelines
6. **Replace incrementally** - one terrain type at a time

**Ready to start extracting?** Next document should be:
`GODOT_SLOPE_RENDERING_IMPLEMENTATION.md` - detailed Godot code for rendering slopes!

---

**Remember:** OpenRCT2 shows us the way. We follow their proven approach, then make it uniquely ours with custom art. This is the path to excellence! 🎨✨
