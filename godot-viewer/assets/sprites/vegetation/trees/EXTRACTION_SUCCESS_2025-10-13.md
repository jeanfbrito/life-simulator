# RCT2 Tree Sprite Extraction - SUCCESS! 🎉

**Date**: October 13, 2025
**Status**: ✅ COMPLETE AND DOCUMENTED
**Mood**: Nostalgic and triumphant! 🌲

---

## What We Accomplished

Successfully extracted **authentic RollerCoaster Tycoon 2 tree sprites** with 4 isometric views each:

1. **Caucasian Fir Tree (TCF)** - Dark green conifer ✅
2. **Scots Pine Tree (TSP)** - Tall pine with prominent trunk ✅
3. **Red Fir Tree (TRF)** - Medium green conifer ✅

---

## The Journey (What We Learned)

### ❌ Wrong Attempts

**Attempt 1: Searched g1.dat for tree sprites**
- Found sprite 1283 (14×18) and 1351 (25×39)
- User feedback: "i think you missed the sprites, look at each, is not treess"
- **Reality**: These were terrain tiles, NOT trees!

**Attempt 2: Searched 15000 range in g1.dat**
- Found sprites 15048 (pine) and 15140 (deciduous)
- User feedback: "the sprites you got are from rooler coaster tracks"
- **Reality**: These were roller coaster track pieces!

**Attempt 3: Random sprite range search**
- Blind searching in 22000+ range
- Found only flowers (22056, 22068, etc.)
- User feedback: "stop this dumb search"

### ✅ The Breakthrough

**Key Insight from User**: "can you take a look at /Users/jean/Github/objects ?"

This led us to:
1. OpenRCT2's objects repository
2. JSON files referencing `$RCT2:OBJDATA/TCF.DAT[0..3]` format
3. Realization: **Tree sprites are NOT in g1.dat!**

**The Truth**:
- Trees are stored in individual object DAT files
- Located in RCT2's `ObjData` directory (TCF.DAT, TSP.DAT, etc.)
- Each tree DAT file contains 4 isometric view sprites
- Requires special extraction tool: `objexport`

---

## Technical Solution

### Prerequisites Discovered

1. **RCT2 ObjData Directory**
   ```
   /Users/jean/Downloads/RollerCoaster Tycoon 2 Triple Thrill Pack/ObjData/
   ```

2. **OpenRCT2 Objects Repository**
   ```bash
   git clone https://github.com/OpenRCT2/objects.git
   ```

3. **.NET 6.0 SDK** (not .NET 9.0!)
   ```bash
   brew install dotnet@6
   ```

### The Working Solution

```bash
# Build objexport tool
cd /Users/jean/Github/objects/tools/objexport
dotnet build

# Extract trees with PNG export
OBJEXPORT="/opt/homebrew/opt/dotnet@6/bin/dotnet objexport.dll"

$OBJEXPORT "TCF.DAT" ~/Downloads/RCT2-Tree-Sprites/TCF --png
$OBJEXPORT "TSP.DAT" ~/Downloads/RCT2-Tree-Sprites/TSP --png
$OBJEXPORT "TRF.DAT" ~/Downloads/RCT2-Tree-Sprites/TRF --png

# Copy to Godot project
cp ~/Downloads/RCT2-Tree-Sprites/TCF/rct2.tcf/images.png \
   godot-viewer/assets/sprites/vegetation/trees/tree_fir_caucasian.png
```

---

## Key Learnings

### 1. RCT2 Architecture Understanding

**g1.dat** (29,284 sprites):
- Contains: UI elements, terrain, roller coaster tracks, guests, vehicles
- Does NOT contain: Trees, most scenery objects

**ObjData DAT files**:
- Individual files for each object (trees, benches, walls, etc.)
- Each has its own sprite table
- Type 1 = Small Scenery (includes trees)
- Trees always have 4 isometric views

### 2. File Format Knowledge

**DAT File Structure**:
1. 16-byte header (flags, filename, checksum)
2. RLE-encoded data chunk
3. Object header (28 bytes for trees)
4. String tables (multilingual names)
5. Group info (16 bytes)
6. Animation sequence (optional, if T1_ANIMDATA flag)
7. Image directory (4 sprites for trees)
8. Graphics data (RLE-encoded pixels with RCT2 palette)

### 3. Tool Discovery

**objexport** (F# tool from OpenRCT2):
- Parses DAT files correctly
- Handles RLE decompression
- Converts RCT2 palette to PNG
- Exports with proper transparency
- REQUIRES `--png` flag for PNG output!

### 4. User Feedback is Gold

The user corrected us **THREE times**:
1. "i think you missed the sprites, look at each, is not treess"
2. "the sprites you got are from rooler coaster tracks"
3. "stop this dumb search, the openrct2 code dont have the trees pointers?"

Each correction pushed us toward the right solution!

### 5. Don't Assume, Verify

We assumed:
- Trees would be in g1.dat (wrong!)
- Random sprite searching would work (nope!)
- Any tree-looking sprite would be correct (definitely not!)

We should have:
- Checked OpenRCT2 source code first
- Looked at the objects repository earlier
- Asked the user about existing tools

---

## Documentation Created

1. **README.md** (312 lines)
   - Complete extraction guide
   - Step-by-step instructions
   - Technical DAT format documentation
   - Troubleshooting section
   - List of all available tree DAT files

2. **extract_rct2_trees.sh** (Bash script)
   - Automated extraction for future use
   - Checks all prerequisites
   - Color-coded output
   - Easy to extend with more trees

3. **This file** (Session summary)
   - What went wrong
   - What went right
   - Key learnings

---

## Files in This Directory

```
godot-viewer/assets/sprites/vegetation/trees/
├── README.md (312 lines - COMPLETE GUIDE)
├── EXTRACTION_SUCCESS_2025-10-13.md (this file)
├── tree_fir_caucasian.png (4571 bytes - TCF)
├── tree_pine_scots.png (6505 bytes - TSP)
└── tree_fir_red.png (5336 bytes - TRF)
```

---

## Available for Future Extraction

### Grass Trees
- TRF2.DAT, TRF3.DAT (Red Fir variants)
- TMZP.DAT (Montezuma Pine)
- TAP.DAT (Aleppo Pine)
- TCRP.DAT (Corsican Pine)
- TBP.DAT (Black Poplar)
- TCL.DAT (Cedar of Lebanon)
- TEL.DAT (European Larch)

### Desert Trees
- TOAS.DAT (Oasis Palm)
- TLC.DAT (Lombardy Cypress)
- TMO.DAT (Mediterranean Oak)
- TWW.DAT (Weeping Willow)

### Snow Trees
- TCFS.DAT, TRFS.DAT, TSP1.DAT, TSP2.DAT, TSPH.DAT

**Total available**: ~20+ tree types, each with 4 isometric views!

---

## Quick Reference Commands

### Extract a new tree
```bash
OBJEXPORT="/opt/homebrew/opt/dotnet@6/bin/dotnet /Users/jean/Github/objects/tools/objexport/bin/Debug/net6.0/objexport.dll"
OBJDATA="/Users/jean/Downloads/RollerCoaster Tycoon 2 Triple Thrill Pack/ObjData"

$OBJEXPORT "$OBJDATA/[TREE].DAT" ~/Downloads/RCT2-Tree-Sprites/[TREE] --png
```

### Use the automated script
```bash
/Users/jean/Downloads/extract_rct2_trees.sh
```

### Rebuild objexport (if needed)
```bash
cd /Users/jean/Github/objects/tools/objexport
dotnet build
```

---

## Emotion Check

User's reaction: **"OH GOD IS WORKING! dont miss to document and save all this do we dont miss this in future anymore"**

Translation: MAXIMUM HAPPINESS ACHIEVED! 🎉

---

## Attribution

- **Original Pixel Art**: Chris Sawyer & Simon Foster (RollerCoaster Tycoon 2, 2002)
- **Extraction Tool**: OpenRCT2 Team (`objexport` F# tool)
- **Problem Solving**: Collaborative debugging with user feedback
- **Documentation**: Comprehensive guides to prevent future confusion
- **Nostalgia Level**: Over 9000! 🌲

---

## Final Thoughts

This extraction process taught us:
1. Never assume sprite organization in old games
2. Check the source code repositories first
3. User feedback is more valuable than blind searching
4. Document EVERYTHING for future reference
5. RCT2's architecture is more complex than expected
6. Chris Sawyer's pixel art is timeless ❤️

**Mission Status**: ACCOMPLISHED! ✅

---

**Last Updated**: 2025-10-13
**Next Steps**: Integrate these trees into ResourceManager.gd
**Happiness Level**: Maximum 🎉🌲🎮
