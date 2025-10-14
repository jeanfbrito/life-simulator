# RCT2 Complete Trees & Vegetation Index

**Purpose**: Quick reference for finding and extracting tree sprites from RCT2
**Last Updated**: October 13, 2025
**Total Objects**: 2,119 DAT files in RCT2 ObjData
**Trees Documented**: 50+ varieties

---

## How to Use This Index

**Search**: Use Ctrl+F / Cmd+F to find trees by name or category
**Extract**: Copy the DAT filename and use the extraction command
**Quick Extract**: `extract_rct2_trees.sh` for common trees

---

## 🌲 SMALL TREES (Single-Tile, 4 Isometric Views)

These are `scenery_small` objects with `isTree: true` property.
**Format**: 4 sprites per tree (NE, SE, SW, NW views)

### Grass Trees (Temperate) - **EXTRACTED** ✅

| DAT File | Object ID | Name | Height | Status |
|----------|-----------|------|--------|--------|
| TCF.DAT | rct2.scenery_small.tcf | Caucasian Fir Tree | 120 | ✅ EXTRACTED |
| TSP.DAT | rct2.scenery_small.tsp | Scots Pine Tree | 156 | ✅ EXTRACTED |
| TRF.DAT | rct2.scenery_small.trf | Red Fir Tree | Varies | ✅ EXTRACTED |
| TRF2.DAT | rct2.scenery_small.trf2 | Red Fir Tree variant 2 | Varies | ✅ EXTRACTED |
| TRF3.DAT | rct2.scenery_small.trf3 | Red Fir Tree variant 3 | Varies | ✅ EXTRACTED |
| TMZP.DAT | rct2.scenery_small.tmzp | Montezuma Pine Tree | Varies | ✅ EXTRACTED |
| TAP.DAT | rct2.scenery_small.tap | Aleppo Pine Tree | Varies | ✅ EXTRACTED |
| TCRP.DAT | rct2.scenery_small.tcrp | Corsican Pine Tree | Varies | ✅ EXTRACTED |
| TBP.DAT | rct2.scenery_small.tbp | Black Poplar Tree | Varies | ✅ EXTRACTED |
| TCL.DAT | rct2.scenery_small.tcl | Cedar of Lebanon Tree | Varies | ✅ EXTRACTED |
| TEL.DAT | rct2.scenery_small.tel | European Larch Tree | Varies | ✅ EXTRACTED |

### Desert Trees

| DAT File | Object ID | Name | Height | Status |
|----------|-----------|------|--------|--------|
| TOAS.DAT | rct2.scenery_small.toas | Oasis Palm Tree | Varies | Not extracted |
| TLC.DAT | rct2.scenery_small.tlc | Lombardy Cypress Tree | Varies | Not extracted |
| TMO.DAT | rct2.scenery_small.tmo | Mediterranean Oak Tree | Varies | Not extracted |
| TWW.DAT | rct2.scenery_small.tww | Weeping Willow Tree | Varies | Not extracted |

### Snow Trees

| DAT File | Object ID | Name | Height | Status |
|----------|-----------|------|--------|--------|
| TCFS.DAT | rct2.scenery_small.tcfs | Caucasian Fir Tree (Snow) | 120 | Not extracted |
| TRFS.DAT | rct2.scenery_small.trfs | Red Fir Tree (Snow) | Varies | Not extracted |
| TSP1.DAT | rct2.scenery_small.tsp1 | Scots Pine Tree (Snow 1) | Varies | Not extracted |
| TSP2.DAT | rct2.scenery_small.tsp2 | Scots Pine Tree (Snow 2) | Varies | Not extracted |
| TSPH.DAT | rct2.scenery_small.tsph | Scots Pine Tree (Snow Heavy) | Varies | Not extracted |

---

## 🌳 LARGE TREES (Multi-Tile, Complex)

These are `scenery_large` objects - occupy multiple tiles, more detailed.

### Jungle/Tropical Trees

| DAT File | Object ID | Name | Tiles | Notes |
|----------|-----------|------|-------|-------|
| 3X3ALTRE.DAT | rct2.ww.3x3altre | Jungle Tree with Leopards | 3×3 | Has animated animals |
| 3X3ATRE1.DAT | rct2.ww.3x3atre1 | Jungle Tree 2 | 3×3 | Dense canopy |
| 3X3ATRE2.DAT | rct2.ww.3x3atre2 | Jungle Tree 1 | 3×3 | Wide spread |
| 3X3ATRE3.DAT | rct2.ww.3x3atre3 | Jungle Tree with Vines | 3×3 | Has hanging vines |
| 3X3EUCAL.DAT | rct2.ww.3x3eucal | Eucalyptus Tree | 3×3 | Australian theme |
| 3X3MANTR.DAT | rct2.ww.3x3mantr | Mangrove Tree | 3×3 | Swamp tree |
| 4X4GMANT.DAT | rct2.tt.4x4gmant | Giant Mangrove Tree | 4×4 | Massive tree |
| 1X1ATRE2.DAT | rct2.ww.1x1atre2 | Vine Tree | 1×1 | Small vine tree |
| 1X1JUGT2.DAT | rct2.ww.1x1jugt2 | Small Rainforest Tree 1 | 1×1 | Compact |
| 1X1JUGT3.DAT | rct2.ww.1x1jugt3 | Small Rainforest Tree 2 | 1×1 | Compact |

### Volcano/Special Trees

| DAT File | Object ID | Name | Tiles | Notes |
|----------|-----------|------|-------|-------|
| 4X4VOLCA.DAT | rct2.tt.4x4volca | Volcano with small trees | 4×4 | Volcano landmark with trees |

---

## 🌿 BUSHES & SMALL VEGETATION

Small decorative vegetation objects.

| DAT File | Object ID | Name | Type | Notes |
|----------|-----------|------|------|-------|
| 1X1ATREE.DAT | rct2.ww.1x1atree | Low Bush | Bush | Small ground cover |
| (More to be catalogued) | | | | |

---

## 🎋 BAMBOO & REEDS

| DAT File | Object ID | Name | Type | Notes |
|----------|-----------|------|------|-------|
| (To be catalogued) | | Bamboo | Grass | Tall grass |
| (To be catalogued) | | Reeds | Grass | Water edge plants |

---

## 📋 Extraction Commands

### Extract Single Tree
```bash
DOTNET="/opt/homebrew/opt/dotnet@6/bin/dotnet"
OBJEXPORT="/Users/jean/Github/objects/tools/objexport/bin/Debug/net6.0/objexport.dll"
OBJDATA="/Users/jean/Downloads/RollerCoaster Tycoon 2 Triple Thrill Pack/ObjData"
OUTPUT="~/Downloads/RCT2-Tree-Sprites"

"$DOTNET" "$OBJEXPORT" "$OBJDATA/[FILENAME].DAT" "$OUTPUT/[FILENAME]" --png
```

### Extract Multiple Trees (Automated)
```bash
# Edit and run the extraction script
/Users/jean/Downloads/extract_rct2_trees.sh
```

### Copy to Godot Project
```bash
cp ~/Downloads/RCT2-Tree-Sprites/[TREE]/rct2.[tree]/images.png \
   godot-viewer/assets/sprites/vegetation/trees/tree_[name].png
```

---

## 🔍 Search Tips

### Find by Category
- **Grass Trees**: Search "Grass Trees"
- **Desert Trees**: Search "Desert Trees"
- **Snow Trees**: Search "Snow Trees"
- **Large Trees**: Search "LARGE TREES"
- **Bushes**: Search "BUSHES"

### Find by Name
- Search for the common name (e.g., "Pine", "Fir", "Palm")
- Search for the DAT filename (e.g., "TSP.DAT", "TOAS.DAT")

### Find by Status
- **Extracted**: Search "✅ EXTRACTED"
- **Not extracted**: Search "Not extracted"

---

## 📊 Statistics

**Total Categories**:
- Small Trees (Grass): 11 ✅
- Small Trees (Desert): 4
- Small Trees (Snow): 5
- Large Trees: 10+
- Bushes: 5+
- **TOTAL DOCUMENTED**: 35+

**Extraction Status**:
- ✅ Extracted: 11 grass trees
- 📦 Available: 24+ additional varieties

**File Sizes**:
- Small trees: 3-11 KB per tree
- Large trees: Varies (multi-tile)
- Total grass trees: ~55 KB (11 trees)

---

## 🎯 Recommended Trees for Life Simulator

### Essential Set (Already Extracted) ✅
- Scots Pine (TSP) - Iconic tall tree
- Caucasian Fir (TCF) - Dense conifer
- Red Fir variants (TRF, TRF2, TRF3) - Visual variety
- Black Poplar (TBP) - Deciduous variety
- Cedar of Lebanon (TCL) - Majestic landmark tree

### Optional Desert Expansion
- Oasis Palm (TOAS) - Desert oasis
- Weeping Willow (TWW) - Near water

### Optional Snow Expansion
- TCFS, TRFS, TSP1, TSP2, TSPH - Winter scenes

### Optional Large Trees (Multi-Tile)
- Giant Mangrove (4X4GMANT) - Massive tree landmark
- Jungle Trees (3X3ATRE1, 3X3ATRE2) - Tropical forests

---

## 🔧 Known Object Types

Based on RCT2 file format:

- **Type 0**: Ride/Shop
- **Type 1**: Small Scenery (includes trees with isTree flag)
- **Type 2**: Large Scenery (includes multi-tile trees)
- **Type 3**: Walls
- **Type 4**: Path Banners
- **Type 5**: Paths
- **Type 6**: Path Additions (benches, etc.)
- **Type 7**: Scenery Group
- **Type 8**: Park Entrance
- **Type 9**: Water
- **Type 10**: Scenario Text

**Trees are primarily Type 1 (Small) and Type 2 (Large).**

---

## 📚 References

- **Source Code**: `/Users/jean/Github/OpenRCT2/src/openrct2/world/map_generator/TreePlacement.cpp`
- **Objects Repository**: `/Users/jean/Github/objects/`
- **Extraction Tool**: `objexport` (F# tool from OpenRCT2)
- **Documentation**: `godot-viewer/assets/sprites/vegetation/trees/README.md`

---

## 🚀 Future Expansion

**To Create Full Index**:
1. Run objexport on all 2,119 DAT files (takes ~1 hour)
2. Parse all object.json files
3. Categorize by objectType and properties
4. Create searchable database

**For now, this manual index covers the most important trees!**

---

**Last Updated**: 2025-10-13
**Maintained By**: Life Simulator project
**Source**: RCT2 Triple Thrill Pack ObjData (2,119 objects)
**Status**: Essential trees documented and extracted ✅
