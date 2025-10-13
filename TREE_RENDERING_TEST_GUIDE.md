# Tree Rendering Testing Guide

## What Was Changed

### 1. Tree Extraction with Green Palette
- **Old**: Trees extracted with autumn/red palette (ColorTable from atlas)
- **New**: Trees extracted with spring green palette (ColorTable1)
- **Script**: `scripts/extract_tree_textures_with_palette.sh`
- **Method**: Custom Python script implements stone-kingdoms shader algorithm

### 2. TreeTextureManager Implementation
- **File**: `godot-viewer/scripts/TreeTextureManager.gd`
- **Loads**: 47 tree textures (25 pine + 22 birch)
- **Method**: `get_random_tree_texture(tree_type)` returns random variant

### 3. ResourceManager Integration
- **File**: `godot-viewer/scripts/ResourceManager.gd`
- **Changes**:
  - Detects tree resources (Wood, Pine, Birch, TreeOak, TreePine, TreeBirch)
  - Renders trees with Sprite2D instead of emoji labels
  - Non-tree resources (rocks, bushes, flowers) still use emojis

### 4. Tree Sprite Positioning
Current implementation:
```gdscript
var sprite = Sprite2D.new()
sprite.texture = tree_texture
sprite.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST

# sprite.centered = true (default)
# Texture center is at sprite.position (0, 0 in container)

var texture_height = tree_texture.get_height()

# Move sprite UP by half height to align bottom with container origin
sprite.offset.y = -texture_height / 2.0

# Container is positioned at tile center
container.position = pixel_pos + config.offset
```

**Visual effect:**
- Tree base (bottom of texture) aligns with tile center
- Tree extends upward from tile
- Config offset (typically negative Y) raises tree slightly above ground

## Testing Instructions

### Step 1: Start Backend
```bash
cd /Users/jean/Github/life-simulator
cargo run --bin life-simulator
```

Wait for: `✅ Web server running on http://127.0.0.1:54321`

### Step 2: Launch Godot Viewer
```bash
cd godot-viewer
/Applications/Godot.app/Contents/MacOS/Godot --path .
# Press F5 to run
```

### Step 3: Verify Tree Loading
Check Godot console output:
```
✅ Loaded 47 tree textures (Pine: 25, Birch: 22)
🌳 Rendered X resources for chunk Y,Z
```

### Step 4: Visual Inspection

**Check for:**
1. ✅ Trees appear green (spring palette)
2. ✅ Tree bases align with grass/ground
3. ✅ Trees not floating or clipping into ground
4. ✅ Trees centered on tiles
5. ✅ No texture artifacts or clipping
6. ✅ Pixel-perfect rendering (no blurring)

**Expected appearance:**
```
     🌲  ← Tree top/foliage
    🌲🌲
   🌲🌲🌲
    🌲🌲
     ||  ← Trunk
    ====  ← Ground (tile center should be here)
```

### Step 5: Known Issues to Report

If trees appear incorrectly positioned, note:
- **Too high?** Floating above ground
- **Too low?** Trunk clipping into terrain
- **Offset horizontally?** Not centered on tile
- **Cut off?** Top or bottom clipped

## Current Positioning Logic

### Coordinate System
- **Container position**: Tile center in pixel coordinates
- **Sprite position**: (0, 0) relative to container
- **Sprite.offset**: Shifts texture within sprite

### Formula
```
Sprite Center Y = Container Y (tile center)
Sprite offset.y = -texture_height / 2.0 (moves texture UP)
Final Sprite Bottom Y = Container Y + (-texture_height/2) + (texture_height/2) = Container Y
```

**Result**: Tree bottom aligns with tile center

### Config Offset
Trees have `offset_y = -0.3` to `-0.5` in Config.gd:
```gdscript
"TreePine": {
    "size_multiplier": 1.6,
    "offset_x": 0.0,
    "offset_y": -0.5  // Raises tree above ground by 0.5 tiles
}
```

This moves the container up by `32 * -0.5 = -16 pixels`, raising tree slightly.

## Alternative Positioning Methods

If current method doesn't work, try:

### Method 1: centered = false with position offset
```gdscript
sprite.centered = false
sprite.position = Vector2(-texture_width / 2.0, -texture_height)
```

### Method 2: Different offset calculation
```gdscript
sprite.offset.y = 0  # No offset, use config offset_y only
```

### Method 3: Anchor at tile base
```gdscript
# Move container down to tile base instead of center
pixel_pos.y += Config.TILE_SIZE / 2  # Move to bottom of tile
sprite.offset.y = -texture_height    # Align bottom with position
```

## Files to Check

**Tree Textures:**
```bash
ls -lh godot-viewer/assets/tiles/trees/
# Should show 47 PNG files, each 6-7 KB, green foliage
```

**Godot Scripts:**
- `godot-viewer/scripts/TreeTextureManager.gd` - Texture loading
- `godot-viewer/scripts/ResourceManager.gd` - Rendering logic
- `godot-viewer/scripts/Config.gd` - Resource config with offsets

**Backend Logs:**
```bash
tail -f /tmp/life-simulator.log
# Watch for chunk loading, no errors
```

## Debug Console Commands

In Godot console, check:
```gdscript
# ResourceManager debug
get_node("WorldRenderer/ResourceManager").debug_print_status()

# Check tree texture manager
get_node("WorldRenderer/ResourceManager/TreeTextureManager").get_texture_count()
# Should return: 47
```

## Troubleshooting

### Trees not appearing at all
**Check**:
- TreeTextureManager loaded correctly
- `tree_texture_manager.has_textures()` returns true
- World has tree resources in loaded chunks

### Trees appear as emojis
**Check**:
- `_is_tree_resource()` function returning true for tree types
- Tree textures loading successfully (not null)

### Trees positioned incorrectly
**This is the current issue to debug**

Try modifying `sprite.offset.y` value in ResourceManager.gd:
```gdscript
# Current
sprite.offset.y = -texture_height / 2.0

# Try: No offset
sprite.offset.y = 0

# Try: Full height offset
sprite.offset.y = -texture_height

# Try: Different fraction
sprite.offset.y = -texture_height * 0.75
```

## Expected Output (Working)

When working correctly:
1. Trees render with green foliage
2. Trunk base sits on grass/ground
3. Trees taller than grass blades
4. No visual artifacts or clipping
5. Multiple tree variants provide visual variety
6. Rocks/bushes/flowers still use emojis

## Reporting Issues

When reporting positioning problems, include:
1. Screenshot of incorrect rendering
2. Expected vs actual position description
3. Console output from Godot
4. Specific tree types affected (all trees, only pine, only birch?)
5. Does changing sprite.offset.y value fix it?

## Summary

The tree rendering system is implemented and loads correctly (47 textures, no errors). The remaining issue is fine-tuning the sprite positioning to align correctly with tiles. Current implementation uses `sprite.offset.y = -texture_height / 2.0` to align tree base with tile center, plus config offset to raise slightly above ground.

Test the viewer and provide visual feedback on tree positioning to determine if adjustments are needed.
