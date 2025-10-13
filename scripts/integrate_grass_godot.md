# Integrating Stone-Kingdoms Grass Textures into Godot Viewer

## Overview

This guide explains how to extract and use the beautiful grass textures from the stone-kingdoms project in your life-simulator Godot viewer.

## Architecture

### Stone-Kingdoms Texture System

- **Texture Atlas**: Single 52MB packed PNG (8192x16384 pixels)
- **Quad Definitions**: Lua file with rectangular regions (x, y, width, height)
- **Macro Tiles**: Optimized rendering with 1x1, 2x2, 3x3, 4x4 tile variants
- **Rendering**: Love2D framework with quad-based sprite batching

### Adaptation for Godot

We have three approaches to adapt this system:

## Approach 1: Extract Individual Tiles (Recommended for Start)

**Pros:**
- Simple to implement
- Full control over each tile
- Easy to understand and debug
- Works with existing TileMap system

**Cons:**
- More files to manage
- Slightly more memory usage

**Steps:**

1. **Extract tiles** using the Python script:
   ```bash
   # Install dependencies
   pip install pillow

   # Extract grass tiles
   python3 scripts/extract_grass_textures.py
   ```

2. **Import into Godot**:
   - Godot will auto-import PNGs in `godot-viewer/assets/tiles/grass/`
   - Check import settings: Filter=Nearest, Mipmaps=Off (for pixel art)

3. **Create TileSet** (Option A: Manual in Editor):
   ```
   1. Create new TileSet resource
   2. Add each tile as a source
   3. Configure collision, terrain bits, etc.
   4. Save as grass_tileset.tres
   ```

4. **Create TileSet** (Option B: Programmatic):
   ```gdscript
   # GrassTextureManager.gd
   extends Node

   const GRASS_TILES = [
       "abundant_grass_1x1_01",
       "abundant_grass_1x1_02",
       # ... etc
   ]

   func create_tile_set() -> TileSet:
       var tile_set = TileSet.new()
       var source_id = 0

       for tile_name in GRASS_TILES:
           var texture_path = "res://assets/tiles/grass/%s.png" % tile_name
           var texture = load(texture_path)

           # Create atlas source
           var atlas_source = TileSetAtlasSource.new()
           atlas_source.texture = texture
           atlas_source.texture_region_size = Vector2i(30, 18)  # Adjust per tile

           # Add to tile set
           tile_set.add_source(atlas_source, source_id)
           source_id += 1

       return tile_set
   ```

5. **Use in TerrainTileMap.gd**:
   ```gdscript
   # Update create_colored_tile_source() to use grass textures
   func create_grass_tile_source() -> TileSetAtlasSource:
       var source = TileSetAtlasSource.new()

       # Load one of the extracted grass textures
       var grass_texture = load("res://assets/tiles/grass/abundant_grass_1x1_01.png")
       source.texture = grass_texture
       source.texture_region_size = Vector2i(64, 32)  # Your isometric tile size

       # Create atlas coordinates
       source.create_tile(Vector2i(0, 0))

       return source
   ```

## Approach 2: Use Full Atlas with AtlasTexture

**Pros:**
- Single texture file (less I/O)
- Matches stone-kingdoms architecture
- Better GPU batching

**Cons:**
- Large texture in memory (52MB uncompressed)
- More complex coordinate management
- Need to parse quad definitions

**Steps:**

1. **Copy the full atlas**:
   ```bash
   cp /Users/jean/Github/stone-kingdoms/assets/tiles/stronghold_assets_packed_v12-hd.png \
      godot-viewer/assets/tiles/stronghold_atlas.png
   ```

2. **Create quad definitions** in GDScript:
   ```gdscript
   # grass_quads.gd
   extends Node

   const ATLAS_SIZE = Vector2(8192, 16384)

   const GRASS_QUADS = {
       "abundant_grass_1x1_01": { "x": 4995, "y": 198, "w": 30, "h": 18 },
       "abundant_grass_1x1_02": { "x": 5029, "y": 198, "w": 30, "h": 18 },
       "abundant_grass_2x2_01": { "x": 4562, "y": 952, "w": 62, "h": 34 },
       # ... etc (converted from object_quads.lua)
   }

   func get_atlas_texture(quad_name: String) -> AtlasTexture:
       var quad = GRASS_QUADS[quad_name]
       var atlas_texture = AtlasTexture.new()

       atlas_texture.atlas = load("res://assets/tiles/stronghold_atlas.png")
       atlas_texture.region = Rect2(quad.x, quad.y, quad.w, quad.h)

       return atlas_texture
   ```

3. **Use in rendering**:
   ```gdscript
   # TerrainTileMap.gd
   var grass_manager = preload("res://scripts/grass_quads.gd").new()

   func create_grass_tile_source() -> TileSetAtlasSource:
       var source = TileSetAtlasSource.new()

       # Get random grass variant
       var variants = ["abundant_grass_1x1_01", "abundant_grass_1x1_02", "abundant_grass_1x1_03"]
       var chosen = variants[randi() % variants.size()]

       var atlas_tex = grass_manager.get_atlas_texture(chosen)
       source.texture = atlas_tex

       return source
   ```

## Approach 3: Hybrid (Recommended for Production)

**Best of both worlds:**

1. **Extract commonly used tiles** (1x1 variants) for variety
2. **Use macro tiles** (2x2, 3x3, 4x4) for large grass areas (performance)
3. **Create tile variations** in Godot for different biomes

**Implementation:**

```gdscript
# grass_manager.gd (autoload singleton)
extends Node

const TILE_1x1_DIR = "res://assets/tiles/grass/1x1/"
const TILE_2x2_DIR = "res://assets/tiles/grass/2x2/"
const TILE_3x3_DIR = "res://assets/tiles/grass/3x3/"
const TILE_4x4_DIR = "res://assets/tiles/grass/4x4/"

var tile_cache = {}

func get_random_grass_1x1() -> Texture2D:
    var variants = [
        "abundant_grass_1x1_01",
        "abundant_grass_1x1_02",
        "abundant_grass_1x1_03",
        "abundant_grass_1x1_04",
    ]
    var chosen = variants[randi() % variants.size()]
    return _load_cached_texture(TILE_1x1_DIR + chosen + ".png")

func get_macro_tile(size: int, variant: int) -> Texture2D:
    var path = ""
    match size:
        2: path = TILE_2x2_DIR + "abundant_grass_2x2_%02d.png" % variant
        3: path = TILE_3x3_DIR + "abundant_grass_3x3_%02d.png" % variant
        4: path = TILE_4x4_DIR + "abundant_grass_4x4_%02d.png" % variant

    return _load_cached_texture(path)

func _load_cached_texture(path: String) -> Texture2D:
    if not tile_cache.has(path):
        tile_cache[path] = load(path)
    return tile_cache[path]
```

## Performance Considerations

### Stone-Kingdoms Uses Macro Tiles Because:

1. **Reduced draw calls**: One 4x4 tile = 16 individual tiles rendered at once
2. **Better batching**: GPU can batch similar-sized quads
3. **Less state changes**: Fewer texture switches

### For Your Godot Viewer:

- **For large grass areas**: Use 4x4 macro tiles
- **For boundaries/edges**: Use 1x1 tiles for fine control
- **For variety**: Randomly select from multiple variants

## Integration Workflow

### Step 1: Extract Tiles
```bash
python3 scripts/extract_grass_textures.py
```

### Step 2: Test in Godot
```gdscript
# Test script in World.tscn
extends Node2D

func _ready():
    var grass_tex = load("res://assets/tiles/grass/abundant_grass_1x1_01.png")
    var sprite = Sprite2D.new()
    sprite.texture = grass_tex
    sprite.position = Vector2(400, 300)
    add_child(sprite)
```

### Step 3: Integrate with TileMap

Update `TerrainTileMap.gd` to use grass textures instead of solid colors:

```gdscript
func _get_terrain_texture(terrain: String) -> Texture2D:
    match terrain:
        "Grass":
            return GrassManager.get_random_grass_1x1()
        "Forest":
            return GrassManager.get_random_grass_1x1()  # Could use darker variant
        _:
            return null  # Fall back to colored tiles

func create_colored_tile_source_with_texture(terrain: String, color: Color) -> int:
    var source_id = tile_set.get_next_source_id()
    var source = TileSetAtlasSource.new()

    # Try to get texture first
    var texture = _get_terrain_texture(terrain)

    if texture:
        source.texture = texture
        source.texture_region_size = TILE_SIZE
    else:
        # Fall back to colored tile
        source.texture = _create_colored_texture(color)
        source.texture_region_size = TILE_SIZE

    source.create_tile(Vector2i(0, 0))
    tile_set.add_source(source, source_id)

    return source_id
```

## Tile Size Conversion

### Stone-Kingdoms → Life-Simulator

- **Stone-Kingdoms**: 30x17 pixel tiles (isometric)
- **Your Godot Viewer**: 128x64 pixel tiles (isometric)

**Options:**

1. **Scale up**: Use `texture.scale = Vector2(4.27, 3.76)` to match your tile size
2. **Resample**: Pre-process tiles to 128x64 using ImageMagick:
   ```bash
   for f in godot-viewer/assets/tiles/grass/*.png; do
       convert "$f" -interpolate Nearest -filter point -resize 128x64! "${f%.png}_scaled.png"
   done
   ```
3. **Use as-is**: Keep original size, adjust your tile size to match

## Color Variation System

Stone-kingdoms uses **lighting variants** (light_1, light_2, etc.) for depth. You can:

1. **Use directly**: Different lighting variants for shadowed tiles
2. **Modulate in Godot**: Use `CanvasItem.modulate` to tint base tiles
3. **Combine**: Base texture + Godot color modulation for biome variation

## Example: Biome-Specific Grass

```gdscript
# grass_manager.gd
func get_grass_for_biome(biome: String) -> Texture2D:
    match biome:
        "abundant_grass":
            return get_random_grass_1x1()
        "scarce_grass":
            return get_light_variant(2)  # Use lighter variant
        "yellow_grass":
            var base = get_random_grass_1x1()
            # Could tint yellow in shader or use different extracted tiles
            return base
        _:
            return get_random_grass_1x1()
```

## Memory Considerations

- **Full Atlas**: ~256MB in GPU memory (8192x16384 RGBA)
- **Extracted Tiles**: ~50KB per tile × 28 tiles = 1.4MB
- **Recommendation**: Use extracted tiles unless you need 100+ tile types

## Next Steps

1. Run extraction script
2. Test one grass tile in Godot viewer
3. Create GrassManager singleton
4. Update TerrainTileMap to use grass textures
5. Add random variation
6. Consider macro tiles for performance

## References

- Stone-Kingdoms: `/Users/jean/Github/stone-kingdoms`
- Quad definitions: `objects/object_quads.lua`
- Atlas: `assets/tiles/stronghold_assets_packed_v12-hd.png`
- Terrain rendering: `terrain/terrain.lua`
