# Complete RCT2 Grass Trees Collection ✅

**Status**: All 11 grass trees extracted and ready!
**Date**: October 13, 2025
**Source**: RollerCoaster Tycoon 2 ObjData directory

---

## 🌲 Complete Collection (11 Trees)

### Fir Trees (4 varieties)

1. **Caucasian Fir (TCF)** - `tree_fir_caucasian.png` (4.5 KB)
   - Dark green dense foliage
   - Brown trunk
   - Compact conifer shape
   - Height: 120 game units

2. **Red Fir (TRF)** - `tree_fir_red.png` (5.2 KB)
   - Medium green color
   - Classic fir shape
   - Standard conifer

3. **Red Fir variant 2 (TRF2)** - `tree_fir_red2.png` (4.3 KB)
   - Similar to TRF but slightly different shading
   - Lighter green tones
   - Narrower profile

4. **Red Fir variant 3 (TRF3)** - `tree_fir_red3.png` (5.3 KB)
   - Third variation of red fir
   - Medium-dark green
   - Fuller canopy

### Pine Trees (4 varieties)

5. **Scots Pine (TSP)** - `tree_pine_scots.png` (6.4 KB)
   - Tallest tree (156 game units)
   - Light green foliage
   - Very prominent brown trunk
   - Characteristic pine needle clusters

6. **Montezuma Pine (TMZP)** - `tree_pine_montezuma.png` (4.4 KB)
   - Dark green dense foliage
   - Round bushy canopy
   - Shorter trunk visibility

7. **Aleppo Pine (TAP)** - `tree_pine_aleppo.png` (3.6 KB)
   - Smaller file size
   - Compact pine shape
   - Mediterranean style

8. **Corsican Pine (TCRP)** - `tree_pine_corsican.png` (5.6 KB)
   - Tall elegant pine
   - Light-medium green
   - Vertical growth pattern

### Deciduous Trees (2 varieties)

9. **Black Poplar (TBP)** - `tree_poplar_black.png` (5.0 KB)
   - Round, full canopy
   - Dense leaf coverage
   - Classic deciduous shape
   - Dark green foliage

10. **European Larch (TEL)** - `tree_larch_european.png` (5.1 KB)
    - Light, airy foliage
    - Needle-like leaves (deciduous conifer!)
    - Unique layered branch structure

### Cedar Trees (1 variety)

11. **Cedar of Lebanon (TCL)** - `tree_cedar_lebanon.png` (11 KB)
    - **LARGEST FILE** - Most detailed tree
    - Distinctive horizontal branching
    - Multi-layered canopy structure
    - Wide spreading form
    - Ancient tree appearance

---

## 📊 Statistics

- **Total Trees**: 11
- **Total File Size**: ~55 KB
- **Average File Size**: 5 KB per tree
- **Isometric Views**: 4 per tree (44 sprites total)
- **Pixel Format**: PNG with transparency, 8-bit colormap
- **Source**: Chris Sawyer & Simon Foster (RCT2, 2002)

---

## 🎨 Visual Characteristics Summary

### By Foliage Density
- **Dense**: Caucasian Fir, Montezuma Pine, Black Poplar
- **Medium**: Red Fir variants, Corsican Pine
- **Light/Airy**: European Larch, Cedar of Lebanon

### By Height
- **Tall**: Scots Pine (156 units), Corsican Pine
- **Medium**: Most firs and pines (120-140 units)
- **Compact**: Aleppo Pine

### By Silhouette
- **Conical/Triangular**: All firs, most pines
- **Round/Oval**: Black Poplar, Montezuma Pine
- **Horizontal/Spread**: Cedar of Lebanon
- **Layered**: European Larch

### By Color Palette
- **Dark Green**: Caucasian Fir, Montezuma Pine, Black Poplar
- **Medium Green**: Red Fir variants, Corsican Pine
- **Light Green**: Scots Pine, European Larch, Cedar of Lebanon

---

## 🎮 Usage in Godot

### Loading All Trees

```gdscript
var grass_trees = {
    # Firs
    "CaucasianFir": preload("res://assets/sprites/vegetation/trees/tree_fir_caucasian.png"),
    "RedFir": preload("res://assets/sprites/vegetation/trees/tree_fir_red.png"),
    "RedFir2": preload("res://assets/sprites/vegetation/trees/tree_fir_red2.png"),
    "RedFir3": preload("res://assets/sprites/vegetation/trees/tree_fir_red3.png"),

    # Pines
    "ScotsPine": preload("res://assets/sprites/vegetation/trees/tree_pine_scots.png"),
    "MontezumaPine": preload("res://assets/sprites/vegetation/trees/tree_pine_montezuma.png"),
    "AleppoPine": preload("res://assets/sprites/vegetation/trees/tree_pine_aleppo.png"),
    "CorsicanPine": preload("res://assets/sprites/vegetation/trees/tree_pine_corsican.png"),

    # Deciduous
    "BlackPoplar": preload("res://assets/sprites/vegetation/trees/tree_poplar_black.png"),
    "EuropeanLarch": preload("res://assets/sprites/vegetation/trees/tree_larch_european.png"),

    # Cedar
    "CedarOfLebanon": preload("res://assets/sprites/vegetation/trees/tree_cedar_lebanon.png"),
}
```

### Random Tree Selection

```gdscript
func get_random_grass_tree() -> Texture2D:
    var tree_names = grass_trees.keys()
    var random_name = tree_names[randi() % tree_names.size()]
    return grass_trees[random_name]
```

### Weighted Random Selection (Realistic Distribution)

```gdscript
var tree_weights = {
    "ScotsPine": 20,        # Most common
    "CaucasianFir": 15,
    "RedFir": 15,
    "BlackPoplar": 10,
    "MontezumaPine": 10,
    "CorsicanPine": 8,
    "RedFir2": 7,
    "RedFir3": 7,
    "EuropeanLarch": 4,
    "AleppoPine": 3,
    "CedarOfLebanon": 1,    # Rare, majestic tree
}

func get_weighted_random_tree() -> Texture2D:
    var total_weight = 0
    for weight in tree_weights.values():
        total_weight += weight

    var random_value = randi() % total_weight
    var cumulative = 0

    for tree_name in tree_weights.keys():
        cumulative += tree_weights[tree_name]
        if random_value < cumulative:
            return grass_trees[tree_name]

    return grass_trees["ScotsPine"]  # Fallback
```

---

## 🌍 Biome Recommendations

### Temperate Forest
- Primary: Scots Pine, Caucasian Fir, Red Fir variants
- Secondary: European Larch, Black Poplar
- Rare: Cedar of Lebanon (ancient forest marker)

### Mediterranean
- Primary: Aleppo Pine, Cedar of Lebanon
- Secondary: Corsican Pine, Montezuma Pine

### Mountain Forest
- Primary: Red Fir variants, Caucasian Fir
- Secondary: Scots Pine, Corsican Pine

### Mixed Woodland
- Use weighted random selection with all varieties
- Cluster similar types for natural grouping
- Place Cedar of Lebanon as landmark trees

---

## 🔄 Next Steps

### Remaining Tree Categories to Extract

**Desert Trees** (4 varieties):
- TOAS.DAT - Oasis Palm Tree
- TLC.DAT - Lombardy Cypress Tree
- TMO.DAT - Mediterranean Oak Tree
- TWW.DAT - Weeping Willow Tree

**Snow Trees** (5 varieties):
- TCFS.DAT - Caucasian Fir (Snow)
- TRFS.DAT - Red Fir (Snow)
- TSP1.DAT - Scots Pine (Snow variant 1)
- TSP2.DAT - Scots Pine (Snow variant 2)
- TSPH.DAT - Scots Pine (Snow, Heavy)

---

## 📝 Extraction Command Reference

```bash
# Extract any tree from RCT2
DOTNET="/opt/homebrew/opt/dotnet@6/bin/dotnet"
OBJEXPORT="/Users/jean/Github/objects/tools/objexport/bin/Debug/net6.0/objexport.dll"
OBJDATA="/Users/jean/Downloads/RollerCoaster Tycoon 2 Triple Thrill Pack/ObjData"
OUTPUT="~/Downloads/RCT2-Tree-Sprites"

"$DOTNET" "$OBJEXPORT" "$OBJDATA/[TREE].DAT" "$OUTPUT/[TREE]" --png
```

Or use the automated script:
```bash
/Users/jean/Downloads/extract_rct2_trees.sh
```

---

## 🏆 Achievements Unlocked

- ✅ All 11 grass trees extracted
- ✅ Complete variety collection
- ✅ 4 isometric views per tree (44 sprites)
- ✅ High-quality PNG with transparency
- ✅ Authentic Chris Sawyer pixel art
- ✅ Ready for Godot integration
- ✅ Fully documented

---

## ❤️ Special Trees

### Cedar of Lebanon (TCL)
The crown jewel of this collection! At 11 KB (2x larger than average), this tree features:
- Most complex sprite structure
- Distinctive horizontal layered branches
- Ancient, majestic appearance
- Perfect for landmark/sacred grove locations
- Historical significance (Lebanon's national symbol)

### Scots Pine (TSP)
The tallest tree at 156 game units:
- Iconic RCT2 tree
- Visible from long distances
- Perfect for forests and skylines
- Light green foliage stands out

---

**Collection Status**: COMPLETE ✅
**Quality**: Authentic Chris Sawyer pixel art
**Ready for**: ResourceManager.gd integration
**Nostalgia Level**: MAXIMUM! 🌲🎮

---

**Last Updated**: 2025-10-13
**Total Extraction Time**: ~3 seconds (objexport is fast!)
**Next Mission**: Extract desert and snow trees!
