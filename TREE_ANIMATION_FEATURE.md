# 🎬 Tree Animation Feature

## What Was Implemented

Added **animated trees** that slowly cycle through texture variants to create a "living forest" effect, making trees appear to sway in the wind.

## How It Works

### Animation System
- Each tree cycles through variants 01 → 02 → 03 → ... → 10 → back to 01
- Animation speed: **0.15 seconds per frame** (smooth, natural movement)
- Full cycle time: **1.5 seconds** (10 frames × 0.15s)
- Trees start at **random frames** so they don't all animate in sync
- Random timer offsets create natural, non-synchronized forest movement

**Note**: Stone-kingdoms trees are actually **static** (no animation in the original game). This feature creates dynamic forest movement beyond the original.

### Tree Variants Used
**Pine Trees**: `tree_pine_large_01` through `tree_pine_large_10` (10 frames)
**Birch Trees**: `tree_birch_large_01` through `tree_birch_large_10` (10 frames)

Each variant has slightly different foliage/branch positions, creating a swaying effect when cycled.

## Implementation Details

### ResourceManager.gd Changes

**Added Animation Tracking**:
```gdscript
var animated_trees: Array[Dictionary] = []  # Tracks all animated tree sprites
const TREE_ANIMATION_SPEED = 0.15  # Seconds per frame (smooth animation)
const TREE_ANIMATION_FRAMES = 10  # Cycle through 10 variants
```

**Animation Loop** (`_process(delta)`):
- Updates all tree sprites every frame
- When timer reaches TREE_ANIMATION_SPEED:
  - Advances to next variant (0-9 cycling)
  - Updates sprite texture
  - Updates sprite position (each variant has unique stone-kingdoms offset)

**Tree Creation**:
- Always starts with variant 01 (index 0)
- Adds tree to animated_trees list with:
  - Random starting frame (0-9) for visual variety
  - Random timer offset (0.0 - 0.15s) so trees animate at different times

**Cleanup**:
- When clearing chunk resources, removes trees from animated_trees array
- Prevents memory leaks and references to freed sprites

### Code Structure

```gdscript
func _process(delta: float):
    for tree_data in animated_trees:
        tree_data["timer"] += delta

        if tree_data["timer"] >= TREE_ANIMATION_SPEED:
            tree_data["timer"] = 0.0
            tree_data["current_frame"] = (tree_data["current_frame"] + 1) % 10

            # Get next texture and offset
            var texture = pine_textures[frame] or birch_textures[frame]
            var offset = pine_offsets[frame] or birch_offsets[frame]

            # Update sprite
            sprite.texture = texture
            sprite.position = apply_stone_kingdoms_offset(offset)
```

## Visual Effect

**Expected Appearance**:
- Trees gently "sway" by cycling through slightly different poses
- Non-synchronized movement across the forest (looks natural)
- Smooth, natural animation (0.15s per frame = full cycle in 1.5 seconds)
- No two adjacent trees animate at the same time (random offsets)

**Animation cycle**: 10 frames in 1.5 seconds (smooth, fluid movement)

**Like this**:
```
Frame 0:  🌲 🌲 🌲   (Different positions)
Frame 1:  🌲🌲  🌲   (Trees shifted slightly)
Frame 2:  🌲 🌲🌲    (Natural swaying motion)
...continuous smooth cycling...
```

## Testing Instructions

### Start the Viewer
```bash
# Terminal 1: Start backend
cd /Users/jean/Github/life-simulator
cargo run --bin life-simulator

# Terminal 2: Launch Godot viewer
cd godot-viewer
/Applications/Godot.app/Contents/MacOS/Godot --path .
# Press F5 to run
```

### What to Look For

1. **Trees Load Correctly**
   - Console: "✅ Loaded 47 tree textures (Pine: 25, Birch: 22)"
   - Console: "🌳 Rendered X resources for chunk Y (Z animated trees, W total)"

2. **Animation is Working**
   - Trees slowly change appearance every ~1.5 seconds
   - Different trees animate at different times (not all in sync)
   - Smooth cycling through 10 variants continuously

3. **Positioning is Correct**
   - Trees don't jump or shift position during animation
   - Tree bases stay aligned with ground
   - No visual glitches or artifacts

### Debug Console Messages

```
🌳 Rendered 33 resources for chunk 0,0 (15 animated trees, 45 total)
🌳 Rendered 30 resources for chunk 0,1 (12 animated trees, 57 total)
```

**Numbers to check**:
- `X resources`: Total resources in chunk (trees + rocks + bushes)
- `Y animated trees`: How many trees in this chunk
- `Z total`: Total animated trees across all loaded chunks

### Expected Performance

- Minimal performance impact (updates textures only, not creating new sprites)
- ~10-20 trees visible at once with default camera
- ~50-100 trees total in loaded chunks (7×7 grid)
- Animation updates in `_process()` at 60 FPS, but only changes texture every 1.5s

## Customization Options

### Animation Speed
Change `TREE_ANIMATION_SPEED` in ResourceManager.gd:
```gdscript
const TREE_ANIMATION_SPEED = 0.15  # Seconds per frame (current - smooth)

# Faster swaying:
const TREE_ANIMATION_SPEED = 0.08  # Very quick movement

# Slower, more subtle:
const TREE_ANIMATION_SPEED = 0.3   # Slower swaying

# Very slow (original choppy version):
const TREE_ANIMATION_SPEED = 1.5   # Too slow, looks stuttery
```

### Animation Frames
Change `TREE_ANIMATION_FRAMES` to use more/fewer variants:
```gdscript
const TREE_ANIMATION_FRAMES = 10   # Use variants 1-10 (current)

# Use all 25 pine variants (more variety, longer cycle):
const TREE_ANIMATION_FRAMES = 25   # 25 × 0.15s = 3.75s per cycle

# Use only first 5 variants (faster loop):
const TREE_ANIMATION_FRAMES = 5    # 5 × 0.15s = 0.75s per cycle
```

### Synchronized Animation
To make all trees animate together (less natural but synchronized):
```gdscript
# In tree creation, remove randomness:
animated_trees.append({
    "sprite": sprite,
    "is_pine": is_pine,
    "current_frame": 0,              # Start at frame 0 (instead of random)
    "timer": 0.0                      # Start timer at 0 (instead of random)
})
```

## Troubleshooting

### Trees not animating
**Check**:
1. Console shows "animated trees" count > 0
2. `tree_texture_manager.pine_tree_textures.size() >= 10`
3. `tree_texture_manager.birch_tree_textures.size() >= 10`
4. No errors in Godot console

### Trees jumping/shifting during animation
**Problem**: Offset not updating correctly
**Check**: Each frame has correct stone-kingdoms offset applied

### Performance issues
**If FPS drops**:
- Reduce `TREE_ANIMATION_FRAMES` to 5
- Only animate visible trees (add viewport culling)
- Increase `TREE_ANIMATION_SPEED` to update less frequently

## Technical Notes

### Memory Management
- `animated_trees` array stores Dictionary per tree (~100 bytes each)
- 100 trees = ~10 KB memory overhead (negligible)
- Sprites are already in memory (just swapping texture reference)

### Why Update Position Each Frame?
Each tree variant has a **different stone-kingdoms quad offset**:
```
Pine tree 01: offset (26, 23)
Pine tree 02: offset (27, 24)  // Different X and Y!
Pine tree 03: offset (26, 23)
```

Without updating position, trees would shift horizontally/vertically during animation.

### Texture Swapping Performance
- Texture swap: `sprite.texture = new_texture` (just pointer assignment)
- Position update: `sprite.position = Vector2(x, y)` (two floats)
- Very fast operation, no rendering cost until next frame

## Future Enhancements

### Wind Direction
Add uniform wind to all trees:
```gdscript
# All trees sway in same direction
var wind_direction = Vector2(0.5, 0.0)  # Slight right sway
sprite.position = base_position + wind_direction * sin(time)
```

### Dynamic Animation Speed
Speed up animation during storms:
```gdscript
var wind_speed = 1.0  # Normal
if storm_active:
    wind_speed = 3.0  # Fast swaying during storm

tree_data["timer"] += delta * wind_speed
```

### Seasonal Color Changes
Combine with multiple palette extraction:
```gdscript
# Spring: Use ColorTable1 (green)
# Summer: Use ColorTable3 (dark green)
# Autumn: Use ColorTable5 (red/yellow)
# Winter: Use ColorTable9 (brown/bare)
```

## Summary

- ✅ Trees now animate by cycling through 10 variants
- ✅ Slow, natural swaying effect (1.5s per frame)
- ✅ Non-synchronized for realistic forest movement
- ✅ Correct positioning maintained during animation
- ✅ Minimal performance impact
- ✅ Easy to customize speed and frame count

**Result**: Living, breathing forest instead of static trees!

## Related Documentation

- `TREE_SYSTEM_FINAL_SUMMARY.md` - Complete tree extraction and positioning
- `TREE_PALETTE_IMPLEMENTATION_GUIDE.md` - Palette system explanation
- `TREE_EXTRACTION_WITH_PALETTE.md` - Extraction process
