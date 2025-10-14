# RCT2 Tree Sprites - COMPLETE EXTRACTION GUIDE

**SUCCESS!** These are the REAL tree sprites from RollerCoaster Tycoon 2 object DAT files!

## Current Tree Sprites

All trees include **4 isometric views** (NE, SE, SW, NW directions) in a single atlas PNG.

### Caucasian Fir Tree (TCF)
- **File**: `tree_fir_caucasian.png`
- **Source DAT**: `TCF.DAT` from RCT2 ObjData directory
- **Object ID**: `rct2.scenery_small.tcf`
- **Dimensions**: 4 views, each ~36×60 pixels
- **Atlas Size**: ~144×60 pixels (4 trees side-by-side)
- **Style**: Dark green conifer with brown trunk
- **Height**: 120 game units
- **Authors**: Chris Sawyer, Simon Foster

### Scots Pine Tree (TSP)
- **File**: `tree_pine_scots.png`
- **Source DAT**: `TSP.DAT` from RCT2 ObjData directory
- **Object ID**: `rct2.scenery_small.tsp`
- **Dimensions**: 4 views, each ~40×78 pixels (taller tree)
- **Atlas Size**: ~160×78 pixels
- **Style**: Light green foliage with prominent brown trunk
- **Height**: 156 game units
- **Authors**: Chris Sawyer, Simon Foster

### Red Fir Tree (TRF)
- **File**: `tree_fir_red.png`
- **Source DAT**: `TRF.DAT` from RCT2 ObjData directory
- **Object ID**: `rct2.scenery_small.trf`
- **Dimensions**: 4 views, each ~36×60 pixels
- **Atlas Size**: ~144×60 pixels
- **Style**: Medium green conifer with brown trunk
- **Height**: Varies
- **Authors**: Chris Sawyer, Simon Foster

---

## 🎯 CRITICAL: Tree Sprites Are NOT in g1.dat!

**Important Discovery:** Tree sprites are stored in **individual object DAT files** in RCT2's `ObjData` directory, NOT in the main `g1.dat` sprite archive!

**Wrong Path (What We Tried First):**
- ❌ Searched g1.dat sprites (29,284 total) for trees
- ❌ Found wrong sprites: 1283, 1351 (terrain tiles), 15048, 15140 (roller coaster tracks)
- ❌ These were NOT trees!

**Correct Path (What Actually Worked):**
- ✅ Tree sprites are in separate DAT files: `TCF.DAT`, `TSP.DAT`, `TRF.DAT`, etc.
- ✅ Located in RCT2's `ObjData` directory
- ✅ Each tree DAT file contains 4 isometric view sprites
- ✅ Extracted using OpenRCT2's `objexport` tool

---

## 📋 Complete Extraction Process (STEP-BY-STEP)

### Prerequisites

1. **RollerCoaster Tycoon 2 Installation**
   - Path: `/Users/jean/Downloads/RollerCoaster Tycoon 2 Triple Thrill Pack/ObjData/`
   - Contains: TCF.DAT, TSP.DAT, TRF.DAT, and many more object files

2. **OpenRCT2 Objects Repository**
   - Clone: `git clone https://github.com/OpenRCT2/objects.git /Users/jean/Github/objects`
   - Contains: `objexport` tool in `tools/objexport/`

3. **.NET 6.0 SDK**
   ```bash
   brew install dotnet@6
   ```

### Step 1: Build the objexport Tool

```bash
cd /Users/jean/Github/objects/tools/objexport
dotnet build
```

**Expected Output:**
- `objexport.dll` in `bin/Debug/net6.0/`
- Warnings about ImageSharp vulnerabilities (ignore for local use)
- "Build succeeded"

### Step 2: Extract Tree Sprites

```bash
# Set up paths
OBJDATA_DIR="/Users/jean/Downloads/RollerCoaster Tycoon 2 Triple Thrill Pack/ObjData"
OUTPUT_DIR="~/Downloads/RCT2-Tree-Sprites"
OBJEXPORT="/opt/homebrew/opt/dotnet@6/bin/dotnet /Users/jean/Github/objects/tools/objexport/bin/Debug/net6.0/objexport.dll"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Extract Caucasian Fir Tree (TCF)
$OBJEXPORT "$OBJDATA_DIR/TCF.DAT" "$OUTPUT_DIR/TCF" --png

# Extract Scots Pine Tree (TSP)
$OBJEXPORT "$OBJDATA_DIR/TSP.DAT" "$OUTPUT_DIR/TSP" --png

# Extract Red Fir Tree (TRF)
$OBJEXPORT "$OBJDATA_DIR/TRF.DAT" "$OUTPUT_DIR/TRF" --png
```

**Expected Output:**
```
RCT2 object to .json / .parkobj exporter
Exporting object from '.../TCF.DAT' to '~/Downloads/RCT2-Tree-Sprites/TCF'
Exporting TCF to ~/Downloads/RCT2-Tree-Sprites/TCF/rct2.tcf/object.json
Exporting images.png...
Object exported in 0.3s
```

### Step 3: Copy to Godot Project

```bash
# Copy extracted sprites to godot-viewer
cp ~/Downloads/RCT2-Tree-Sprites/TCF/rct2.tcf/images.png \
   godot-viewer/assets/sprites/vegetation/trees/tree_fir_caucasian.png

cp ~/Downloads/RCT2-Tree-Sprites/TSP/rct2.tsp/images.png \
   godot-viewer/assets/sprites/vegetation/trees/tree_pine_scots.png

cp ~/Downloads/RCT2-Tree-Sprites/TRF/rct2.trf/images.png \
   godot-viewer/assets/sprites/vegetation/trees/tree_fir_red.png
```

---

## 🌲 Available Tree DAT Files (RCT2 ObjData)

From OpenRCT2's `TreePlacement.cpp`, here are the tree object IDs:

### Grass Trees (Common)
- `TCF.DAT` - Caucasian Fir Tree ✅ **Extracted**
- `TSP.DAT` - Scots Pine Tree ✅ **Extracted**
- `TRF.DAT` - Red Fir Tree ✅ **Extracted**
- `TRF2.DAT` - Red Fir Tree (variant)
- `TRF3.DAT` - Red Fir Tree (variant)
- `TMZP.DAT` - Montezuma Pine Tree
- `TAP.DAT` - Aleppo Pine Tree
- `TCRP.DAT` - Corsican Pine Tree
- `TBP.DAT` - Black Poplar Tree
- `TCL.DAT` - Cedar of Lebanon Tree
- `TEL.DAT` - European Larch Tree

### Desert Trees
- `TOAS.DAT` - Oasis Palm Tree
- `TLC.DAT` - Lombardy Cypress Tree
- `TMO.DAT` - Mediterranean Oak Tree
- `TWW.DAT` - Weeping Willow Tree

### Snow Trees
- `TCFS.DAT` - Caucasian Fir Tree (Snow)
- `TRFS.DAT` - Red Fir Tree (Snow)
- `TSP1.DAT` - Scots Pine Tree (Snow variant 1)
- `TSP2.DAT` - Scots Pine Tree (Snow variant 2)
- `TSPH.DAT` - Scots Pine Tree (Snow, Heavy)

**To extract any tree:**
```bash
$OBJEXPORT "$OBJDATA_DIR/[TREE].DAT" "$OUTPUT_DIR/[TREE]" --png
```

---

## 📚 Technical Documentation

### DAT File Format Structure

Based on RCT2 file format documentation (September 18, 2004):

1. **DAT File Header** (16 bytes)
   - Flags (4 bytes) - Low nibble = object type (1 = Small Scenery/Trees)
   - Filename (8 bytes) - ASCII, space-padded
   - Checksum (4 bytes) - Rotated XOR checksum

2. **Encoded Data Chunk**
   - Encoding byte (0 = no compression, 1 = RLE)
   - Size (4 bytes)
   - Compressed data

3. **Decoded Data Contains:**
   - **Object Header** (0x1C = 28 bytes for Type 1/Small Scenery)
   - **String Table** (multilingual names, null-terminated, 0xFF = end)
   - **Group Info** (16 bytes - scenery grouping)
   - **Optional Animation Sequence** (if T1_ANIMDATA flag set, ends with 0xFF)
   - **Image Directory:**
     - Number of images (4 bytes) - Always 4 for trees
     - Graphics data size (4 bytes)
     - Image entries (16 bytes each):
       - StartAddress (4 bytes) - Offset to scan line data
       - Width (2 bytes signed)
       - Height (2 bytes signed)
       - XOffset (2 bytes signed) - Drawing offset
       - YOffset (2 bytes signed) - Drawing offset
       - Flags (2 bytes)
       - Unused (2 bytes)
   - **Graphics Data** (RLE-encoded pixel data with RCT2 256-color palette)

### RCT2 Palette

The objexport tool uses the complete 256-color RCT2 palette defined in `ImageExporter.fs`:
- Color 0: Transparent (0, 0, 0, 0)
- Colors 1-255: RGB values with full alpha (255)
- Includes remappable color ranges for customization

### objexport Tool Usage

```bash
objexport <objects path> <output path> [options]
          <object path> <output path> [options]

Options:
  --author <author>   Specify an author (multiple use)
  --id                Specify the id of the target object
  --language <dir>    Specify directory for language files
  --type <type>       Specify type of object to export
  --split             Split footpath into surface and railing objects
  --png               Store images as .png instead of gx file
  -j                  Multithreaded
  -z                  Create .parkobj files
```

**For trees, always use `--png` to get PNG files with transparency!**

---

## 🎮 Usage in Godot

### In ResourceManager.gd

```gdscript
var tree_textures = {
    "CaucasianFir": preload("res://assets/sprites/vegetation/trees/tree_fir_caucasian.png"),
    "ScotsPine": preload("res://assets/sprites/vegetation/trees/tree_pine_scots.png"),
    "RedFir": preload("res://assets/sprites/vegetation/trees/tree_fir_red.png"),
}
```

### Rendering Isometric Views

Each atlas contains 4 views arranged horizontally:
- View 0 (left): Northeast (NE)
- View 1: Southeast (SE)
- View 2: Southwest (SW)
- View 3 (right): Northwest (NW)

**To select the correct view based on camera angle:**
```gdscript
func get_tree_view_for_direction(direction: Vector2) -> int:
    var angle = direction.angle()
    if angle < -PI * 3/4:
        return 3  # NW
    elif angle < -PI/4:
        return 0  # NE
    elif angle < PI/4:
        return 1  # SE
    else:
        return 2  # SW
```

---

## 🏆 Attribution

- **Original Pixel Art**: Chris Sawyer & Simon Foster (RollerCoaster Tycoon 2, 2002)
- **Extraction Tool**: OpenRCT2 Team (`objexport` F# tool)
- **RCT2 Palette**: 256-color indexed palette from original game
- **File Format Documentation**: RCT2 community (2004)
- **Integration**: Life Simulator project (2025)

---

## ❤️ Nostalgia

These authentic Chris Sawyer trees bring the iconic RCT2 pixel art to the life simulator! 🌲🌳

The Caucasian Fir (TCF) and Scots Pine (TSP) are among the most recognizable trees from RCT2 parks, appearing in countless player-created scenarios and theme parks since 2002.

**Project Timestamp**: Extracted October 13, 2025 (Tahoe macOS)

---

## 🔧 Troubleshooting

### "No such file or directory" for DAT files
- Check RCT2 installation path
- Verify `ObjData` directory exists
- Ensure DAT filenames are uppercase (TCF.DAT not tcf.dat)

### ".NET 6.0 not found"
- Install via Homebrew: `brew install dotnet@6`
- Use correct path: `/opt/homebrew/opt/dotnet@6/bin/dotnet`

### "Object exported" but no images.png
- Verify `--png` flag is used
- Check output directory: `~/Downloads/RCT2-Tree-Sprites/[TREE]/rct2.[tree]/images.png`

### Trees appear blank/corrupted in Godot
- Verify PNG transparency is preserved
- Check file size (should be 4-6 KB for tree atlases)
- Ensure Godot import settings use "Lossless" compression

---

**Last Updated**: 2025-10-13
**Tested On**: macOS Sequoia (Tahoe), Godot 4.3+, .NET 6.0
**Status**: ✅ WORKING - DO NOT LOSE THIS DOCUMENTATION!
