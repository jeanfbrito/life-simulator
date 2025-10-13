# 🌲 Tree Ping-Pong Animation System

## How It Works

Trees animate by **ping-ponging** through their 10 texture variants:
```
01 → 02 → 03 → 04 → 05 → 06 → 07 → 08 → 09 → 10
                                              ↓
01 ← 02 ← 03 ← 04 ← 05 ← 06 ← 07 ← 08 ← 09 ← 10
```

This creates a realistic swaying effect - trees sway one direction, then sway back, like wind pushing them.

## Key Features

### Random Speed Per Tree
- Each tree gets random animation speed between **0.3 - 0.9 seconds per frame**
- Some trees sway faster, some slower
- Creates natural variation across the forest

### Random Starting State
- **Random starting frame**: 0-9 (could start at any point in cycle)
- **Random direction**: Forward (01→10) or backward (10→01)
- **Random timer offset**: 0.0 - speed seconds

Result: Trees never all animate together - looks completely natural!

### Position Updates
- Each frame has different stone-kingdoms offset
- Position updates with texture to prevent jumping/shifting
- Maintains perfect tree base alignment with ground

## Implementation

### Constants
```gdscript
const TREE_ANIMATION_FRAMES = 10          // Use 10 variants
const TREE_ANIMATION_SPEED_MIN = 0.3      // Fastest sway (seconds per frame)
const TREE_ANIMATION_SPEED_MAX = 0.9      // Slowest sway (seconds per frame)
```

### Tree Data Structure
```gdscript
{
    "sprite": Sprite2D,           // The tree sprite
    "is_pine": bool,              // Pine or birch
    "current_frame": int,         // Current frame index (0-9)
    "direction": int,             // 1 = forward, -1 = backward
    "speed": float,               // Random speed for this tree (0.3-0.9)
    "timer": float                // Current animation timer
}
```

### Animation Loop
```gdscript
func _process(delta: float):
    for tree_data in animated_trees:
        tree_data["timer"] += delta

        if tree_data["timer"] >= tree_data["speed"]:
            tree_data["timer"] = 0.0

            # Ping-pong logic
            if tree_data["direction"] == 1:  # Moving forward
                current += 1
                if current >= 9:  # Reached end
                    tree_data["direction"] = -1  # Reverse
            else:  # Moving backward
                current -= 1
                if current <= 0:  # Reached start
                    tree_data["direction"] = 1  # Forward again

            # Update texture and position
            update_tree_sprite(tree_data, current)
```

## Visual Effect

**What you'll see**:
- Trees gently sway back and forth
- Different speeds create wave-like motion through forest
- Non-synchronized movement looks completely natural
- Smooth, continuous animation (no jarring loops)

**Example forest view**:
```
Frame 0:  🌲 🌲 🌲  (Trees at different positions)
Frame 1:  🌲🌲 🌲   (Some swayed right)
Frame 2:   🌲🌲🌲   (Different trees swaying)
Frame 3:  🌲 🌲🌲   (Swaying back left)
Frame 4:  🌲 🌲 🌲  (Natural back-and-forth motion)
```

## Customization

### Change Speed Range
```gdscript
# Faster, more energetic swaying:
const TREE_ANIMATION_SPEED_MIN = 0.1
const TREE_ANIMATION_SPEED_MAX = 0.3

# Slower, barely noticeable:
const TREE_ANIMATION_SPEED_MIN = 1.0
const TREE_ANIMATION_SPEED_MAX = 2.0
```

### Synchronized Animation
Make all trees sway together (uniform wind):
```gdscript
# Give all trees same speed and direction:
var anim_speed = 0.5  # Fixed speed
var start_direction = 1  # All start forward

animated_trees.append({
    # ... other fields ...
    "direction": start_direction,
    "speed": anim_speed
})
```

### Use More Frames
Cycle through all 25 pine variants instead of just 10:
```gdscript
const TREE_ANIMATION_FRAMES = 25  // Longer animation cycle
```

## Performance

**Overhead per tree**:
- Dictionary entry: ~100 bytes
- Texture swap: O(1) pointer assignment
- Position update: 2 float assignments

**With 100 trees**:
- Memory: ~10 KB
- Per-frame cost: Minimal (only updates when timer triggers)
- No FPS impact expected

## Testing

### What to Check
1. ✅ Trees sway smoothly back and forth
2. ✅ Different trees have different speeds
3. ✅ Trees don't all animate in sync
4. ✅ No position jumping or shifting
5. ✅ Animation loops continuously

### Console Output
```
🌲 Loading tree textures...
✅ Loaded 47 tree textures (Pine: 25, Birch: 22)
🌳 Rendered 30 resources for chunk 0,0 (15 animated trees, 45 total)
```

### Debugging
If animation isn't working:
```gdscript
# Add to _process():
if animated_trees.size() > 0 and frame_count % 60 == 0:
    var tree = animated_trees[0]
    print("Tree 0: frame=%d dir=%d speed=%.2f" % [
        tree["current_frame"],
        tree["direction"],
        tree["speed"]
    ])
```

## Comparison: Cycling vs Ping-Pong

### Old: Cycling (01→10→01→10)
```
01 → 02 → ... → 10 → 01 → 02 → ... → 10 → 01
                     ↑
                Jump back (not natural)
```
- Sudden jump from frame 10 back to 01
- Looks like teleporting/glitch
- Not realistic

### New: Ping-Pong (01→10→01)
```
01 → 02 → ... → 10 → 09 → ... → 01 → 02
                     ↓
                Smooth reversal (natural)
```
- Smooth direction reversal
- Mimics real tree swaying in wind
- Looks realistic and natural

## Summary

- ✅ Ping-pong animation (01→10→01)
- ✅ Random speed per tree (0.3-0.9 seconds)
- ✅ Random starting frames and directions
- ✅ Smooth, continuous swaying motion
- ✅ Natural, non-synchronized forest movement
- ✅ Minimal performance impact

**Result**: Realistic forest that feels alive with natural tree movement!
