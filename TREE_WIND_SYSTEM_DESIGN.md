# 🌬️ Realistic Wind-Based Tree Animation System

## Problem with Random Animation

**Issue**: Trees with random speeds and random starting frames look chaotic and unrealistic.

**Why it's wrong**:
- Real wind affects all trees in an area simultaneously
- Trees should sway together as wind gusts pass through
- Random movement looks artificial and "gamey"
- No sense of unified environmental force

## Realistic Wind Behavior

### How Real Wind Works

1. **Uniform Gusts**: Wind affects all trees in view at roughly the same time
2. **Wave Propagation**: Wind travels across landscape (trees sway in sequence)
3. **Variable Strength**: Calm periods → moderate breeze → strong gusts → calm
4. **Smooth Transitions**: Natural sine/cosine oscillation, not linear

### What We Want

```
Calm:      🌲 🌲 🌲  (trees barely moving, frames 0-2)

Breeze:    🌲🌲 🌲   (gentle sway, frames 0-5)

Strong:   🌲  🌲🌲   (full sway, frames 0-9)

Wave:     🌲→ 🌲 →🌲  (wind travels left to right)
```

## Proposed Wind System Architecture

### 1. Global Wind State

A singleton resource tracking current wind conditions:

```gdscript
# WindManager.gd (autoload singleton)
extends Node

# Wind oscillation (sine wave)
var wind_time: float = 0.0
const WIND_CYCLE_DURATION = 8.0  # Seconds for full sway cycle (slow, natural)

# Wind strength (affects how much trees sway)
var wind_strength: float = 0.5  # 0.0 = calm, 1.0 = strong gust
const WIND_STRENGTH_CHANGE_RATE = 0.1  # How fast wind strength changes

# Wind wave propagation (creates traveling wave effect)
const WIND_WAVE_SPEED = 100.0  # Pixels per second wind travels

func _process(delta: float):
    # Advance wind time
    wind_time += delta

    # Vary wind strength over time (could use Perlin noise for more natural variation)
    wind_strength = 0.3 + 0.4 * sin(wind_time * 0.2)  # Oscillates 0.3-0.7

func get_wind_frame_for_position(world_pos: Vector2) -> int:
    """Calculate which animation frame a tree should be at based on wind."""

    # Calculate wind phase (0 to 2π)
    var base_phase = (wind_time / WIND_CYCLE_DURATION) * TAU

    # Add position offset for wave propagation effect
    var distance_offset = (world_pos.x + world_pos.y) / WIND_WAVE_SPEED
    var wind_phase = base_phase + distance_offset

    # Convert sine wave (-1 to +1) to frame (0 to 9)
    var wave_value = sin(wind_phase)  # -1 to +1
    var normalized = (wave_value + 1.0) / 2.0  # 0 to 1

    # Scale by wind strength
    var sway_amount = normalized * wind_strength

    # Map to frame (0 = still, 9 = max sway)
    var frame = int(sway_amount * 9.0)
    return clamp(frame, 0, 9)
```

### 2. Tree Animation (No Per-Tree State)

Trees become stateless - they just query global wind:

```gdscript
# ResourceManager.gd

func _process(delta: float):
    # Update all tree animations based on global wind
    for tree_data in animated_trees:
        var sprite = tree_data["sprite"]
        var tree_world_pos = sprite.global_position

        # Get frame from global wind system
        var frame = WindManager.get_wind_frame_for_position(tree_world_pos)

        # Update texture if frame changed
        if tree_data["current_frame"] != frame:
            tree_data["current_frame"] = frame
            _update_tree_texture(tree_data, frame)
```

**Simplified tree data**:
```gdscript
{
    "sprite": Sprite2D,
    "is_pine": bool,
    "current_frame": int  # Just for caching, not for timing
}
# No more: timer, direction, speed (all in global wind system)
```

### 3. Wave Propagation Effect

**Distance Offset**: Trees at different positions experience wind slightly offset in time:

```
Position (0, 0):    wind_phase = base_phase + 0.0
Position (100, 0):  wind_phase = base_phase + 1.0
Position (200, 0):  wind_phase = base_phase + 2.0
```

**Visual Result**: Creates traveling wave effect across forest:
```
Time 0:  🌲 🌲 🌲 🌲  (all still)
Time 1:  🌲🌲 🌲 🌲  (left trees sway first)
Time 2:  🌲 🌲🌲 🌲  (wave moves right)
Time 3:  🌲 🌲 🌲🌲  (wave continues)
Time 4:  🌲 🌲 🌲 🌲  (all return to still)
```

## Implementation Details

### Wind Cycle Duration

```gdscript
const WIND_CYCLE_DURATION = 8.0  # Full sway cycle in seconds

# Slow, natural:     10-15 seconds
# Moderate:          6-10 seconds
# Fast (storm):      3-5 seconds
```

**Recommended**: 8-10 seconds for calm, natural forest.

### Wind Strength Variation

**Option 1: Simple Sine Wave**
```gdscript
wind_strength = 0.3 + 0.4 * sin(wind_time * 0.2)
# Oscillates smoothly between 0.3 (calm) and 0.7 (moderate)
```

**Option 2: Perlin Noise (More Natural)**
```gdscript
var noise = OpenSimplexNoise.new()
noise.seed = randi()
noise.octaves = 2
noise.period = 20.0

func _process(delta):
    wind_time += delta
    wind_strength = 0.3 + 0.6 * noise.get_noise_1d(wind_time)
    # More natural, unpredictable variation
```

**Option 3: Gust System (Most Realistic)**
```gdscript
enum WindState { CALM, BUILDING, GUST, FADING }
var wind_state = WindState.CALM
var state_timer = 0.0

func _process(delta):
    state_timer += delta

    match wind_state:
        WindState.CALM:
            wind_strength = lerp(wind_strength, 0.2, delta * 0.5)
            if state_timer > randf_range(3.0, 8.0):
                wind_state = WindState.BUILDING
                state_timer = 0.0

        WindState.BUILDING:
            wind_strength = lerp(wind_strength, 0.8, delta * 2.0)
            if state_timer > 1.5:
                wind_state = WindState.GUST
                state_timer = 0.0

        WindState.GUST:
            wind_strength = 0.7 + 0.2 * sin(wind_time * 4.0)  # Fast oscillation
            if state_timer > randf_range(1.0, 3.0):
                wind_state = WindState.FADING
                state_timer = 0.0

        WindState.FADING:
            wind_strength = lerp(wind_strength, 0.3, delta * 1.0)
            if state_timer > 2.0:
                wind_state = WindState.CALM
                state_timer = 0.0
```

### Wave Speed

```gdscript
const WIND_WAVE_SPEED = 100.0  # Pixels per second

# Slow wave (trees sway in large groups):    50-100
# Medium wave (nice traveling effect):       100-200
# Fast wave (very visible wave motion):      200-400
# Instant (all trees sync):                  10000+
```

## Visual Examples

### Synchronized (No Wave)
```
WIND_WAVE_SPEED = 10000.0  # Effectively instant

All trees sway exactly together:
Frame 0: 🌲 🌲 🌲 🌲
Frame 3: 🌲🌲🌲🌲
Frame 6: 🌲 🌲 🌲 🌲
```

### Traveling Wave
```
WIND_WAVE_SPEED = 100.0  # Visible wave

Wave travels across forest:
t=0s:  🌲 🌲 🌲 🌲 🌲
t=1s:  🌲🌲 🌲 🌲 🌲
t=2s:  🌲 🌲🌲 🌲 🌲
t=3s:  🌲 🌲 🌲🌲 🌲
t=4s:  🌲 🌲 🌲 🌲🌲
t=5s:  🌲 🌲 🌲 🌲 🌲
```

### Variable Strength
```
Calm (strength=0.2):     Trees barely move (frames 0-2)
🌲 🌲 🌲  →  🌲🌲 🌲  →  🌲 🌲 🌲

Strong (strength=0.9):   Trees sway dramatically (frames 0-9)
🌲 🌲 🌲  →  🌲  🌲  →  🌲🌲🌲  →  🌲 🌲 🌲
```

## Benefits

✅ **Realistic**: Trees respond to unified wind force
✅ **Natural**: Smooth sine wave oscillation mimics real wind
✅ **Dynamic**: Variable wind strength creates interest
✅ **Performant**: Simple math, no complex per-tree state
✅ **Elegant**: Global wind state, trees just query it
✅ **Extensible**: Can add wind direction, gusts, storms

## Recommended Configuration

**For natural, calm forest**:
```gdscript
# WindManager
const WIND_CYCLE_DURATION = 10.0      # Slow, gentle sway
const WIND_WAVE_SPEED = 150.0         # Visible but subtle wave
const WIND_STRENGTH_MIN = 0.3         # Never completely still
const WIND_STRENGTH_MAX = 0.7         # Never too violent

# Simple sine variation
func _process(delta):
    wind_time += delta
    wind_strength = 0.5 + 0.2 * sin(wind_time * 0.15)
```

**For dramatic, stormy forest**:
```gdscript
const WIND_CYCLE_DURATION = 4.0       # Fast, dramatic sway
const WIND_WAVE_SPEED = 300.0         # Fast traveling gusts
const WIND_STRENGTH_MIN = 0.6         # Always moving
const WIND_STRENGTH_MAX = 1.0         # Full range

# Gust system with strong variation
```

## Implementation Plan

### Step 1: Create WindManager Singleton
- New file: `godot-viewer/scripts/WindManager.gd`
- Autoload as singleton
- Implement basic sine wave wind

### Step 2: Simplify Tree Animation
- Remove per-tree timing state (timer, direction, speed)
- Keep only sprite reference and current frame cache
- Query WindManager for frame calculation

### Step 3: Test and Tune
- Start with simple synchronized movement (high wave speed)
- Gradually reduce wave speed to see traveling wave effect
- Tune wind cycle duration and strength for desired feel

### Step 4: Optional Enhancements
- Add wind direction (trees sway in specific direction)
- Add gust system (calm → gust → calm cycles)
- Add debug visualization (wind strength indicator)
- Add weather integration (storms increase wind)

## Comparison: Random vs Wind

### Random System (Current - Bad)
```
Tree A: speed=0.3, frame=7, forward
Tree B: speed=0.8, frame=2, backward
Tree C: speed=0.5, frame=9, forward

Result: Chaotic, looks like glitch
```

### Wind System (Proposed - Good)
```
Global wind: phase=2.5, strength=0.6

Tree A at (100, 50): frame = 5
Tree B at (150, 50): frame = 6  (slightly ahead due to position)
Tree C at (200, 50): frame = 7  (further ahead)

Result: Unified, natural, realistic
```

## Summary

- ✅ Global wind state replaces per-tree randomness
- ✅ All trees respond to same wind force
- ✅ Wave propagation creates traveling effect
- ✅ Variable wind strength adds dynamism
- ✅ Smooth sine oscillation looks natural
- ✅ Simple, elegant, performant

**Next**: Implement WindManager singleton and update ResourceManager to use it.
