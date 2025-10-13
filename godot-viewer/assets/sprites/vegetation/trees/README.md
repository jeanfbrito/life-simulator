# RCT2 Tree Sprites

**Source:** RollerCoaster Tycoon 2 g1.dat sprites

## Available Trees

| File | RCT2 Sprite | Size | Description |
|------|-------------|------|-------------|
| `tree_pine.png` | 1283 | 14×18 | Small dark green pine/evergreen tree |
| `tree_deciduous.png` | 1351 | 25×39 | Large brown deciduous/autumn tree |

## Visual Comparison

**tree_pine.png (1283):**
- Small conifer/evergreen
- Dark green color
- Compact, symmetric shape
- Good for dense forests

**tree_deciduous.png (1351):**
- Large broad-leaf tree
- Brown/autumn coloring
- Fuller, wider canopy
- Good for variety and landmarks

## Usage in Godot

These sprites replace the temporary stronghold textures previously used.

**In ResourceManager.gd:**
```gdscript
var tree_textures = {
    "Wood": preload("res://assets/sprites/vegetation/trees/tree_pine.png"),
    "WoodDeciduous": preload("res://assets/sprites/vegetation/trees/tree_deciduous.png"),
}
```

**Rendering:**
- Position using isometric tile coordinates (map_to_local)
- Y-sort for proper depth layering
- Scale as needed (default 1.0)

## More Tree Options Available

From RCT2 sprite library (~170 tree sprites total):

**Evergreens/Conifers:**
- 1280-1320: Small pine variations (different angles/seasons)
- 1400-1420: Blue/teal conifers (unique coloring)

**Deciduous/Broad-leaf:**
- 1350-1370: Brown/autumn trees (various sizes)
- 1380-1400: Green/summer deciduous trees

**Special:**
- Palm trees, dead trees, snowy trees, and more

See `docs/RCT2_SPRITE_LIBRARY.md` for full catalog.

## Extraction Process

Trees extracted from permanent sprite library:
```bash
cp ~/RCT2-Sprites/1283.png godot-viewer/assets/sprites/vegetation/trees/tree_pine.png
cp ~/RCT2-Sprites/1351.png godot-viewer/assets/sprites/vegetation/trees/tree_deciduous.png
```

For additional trees, copy from `~/RCT2-Sprites/<sprite_number>.png`

## Nostalgia Notes

These iconic RCT2 trees bring authentic Chris Sawyer pixel art to the life simulator! 🌲

The small pine (1283) and brown deciduous (1351) are among the most recognizable trees from RCT2 parks, appearing in countless player-created scenarios and theme parks.
