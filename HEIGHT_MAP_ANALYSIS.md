# Height Map System Analysis - OpenRCT2 Study

**Investigation Date:** 2025-01-13
**Purpose:** Research how OpenRCT2 handles height maps to apply similar concepts to life-simulator

---

## 🎯 Key Findings from OpenRCT2

### 1. Core Height Map Structure

**File:** `src/openrct2/world/map_generator/HeightMap.hpp`

```cpp
class HeightMap {
private:
    std::vector<uint8_t> _height;  // Single byte per tile!

public:
    uint16_t width;
    uint16_t height;
    uint8_t density;  // Interesting: allows higher resolution

    uint8_t& operator[](TileCoordsXY pos) {
        return _height[pos.y * width + pos.x];  // Simple 1D array access
    }
};
```

**Key Insights:**
- ✅ **Simple 1D vector** indexed as 2D: `_height[y * width + x]`
- ✅ **uint8_t** (0-255 range) sufficient for height - very memory efficient
- ✅ **Density parameter** allows super-sampling for smoother results
- ✅ **Operator overload** makes access clean: `heightMap[{x, y}]`

---

### 2. Surface Element with Height

**File:** `src/openrct2/world/tile_element/SurfaceElement.h`

```cpp
struct SurfaceElement {
    uint8_t Slope;           // Bitfield for corner heights
    uint8_t WaterHeight;     // Separate water level
    uint8_t GrassLength;     // Dynamic grass growth!
    uint8_t Ownership;
    uint8_t SurfaceStyle;    // Terrain texture
    uint8_t EdgeObjectIndex;
};
```

**Key Insights:**
- ✅ Each tile has **base height + slope** (corners can be raised)
- ✅ **Separate water height** allows underwater terrain
- ✅ **Grass growth system** (7 levels: mowed → clumps)
- ✅ **Compact structure** - everything fits in 16 bytes

---

### 3. Slope System (Brilliant!)

**File:** `src/openrct2/world/tile_element/Slope.h`

```cpp
// Bitfield for corner heights
constexpr uint8_t kTileSlopeFlat = 0;
constexpr uint8_t kTileSlopeNCornerUp = 0b00000001;  // North corner raised
constexpr uint8_t kTileSlopeECornerUp = 0b00000010;  // East corner raised
constexpr uint8_t kTileSlopeSCornerUp = 0b00000100;  // South corner raised
constexpr uint8_t kTileSlopeWCornerUp = 0b00001000;  // West corner raised
constexpr uint8_t kTileSlopeDiagonalFlag = 0b00010000;  // Diagonal slopes

// Combinations
constexpr uint8_t kTileSlopeNESideUp = kTileSlopeNCornerUp | kTileSlopeECornerUp;
constexpr uint8_t kTileSlopeWEValley = kTileSlopeECornerUp | kTileSlopeWCornerUp;
```

**Key Insights:**
- ✅ **4 bits = 4 corners** - can represent 16 slope combinations
- ✅ Enables **smooth terrain transitions** without cliffs
- ✅ **Diagonal flag** for even more natural slopes
- ✅ Easy to calculate relative corner heights

---

### 4. Height Generation Algorithms

**File:** `src/openrct2/world/map_generator/MapGen.h`

```cpp
enum class Algorithm {
    blank,
    simplexNoise,    // Procedural generation
    heightmapImage,  // Import from PNG!
};

struct Settings {
    int32_t waterLevel = 6;
    int32_t heightmapLow = 14;   // Min height
    int32_t heightmapHigh = 60;  // Max height

    // Simplex parameters
    int32_t simplex_base_freq = 175;
    int32_t simplex_octaves = 6;

    // Smoothing
    bool smooth_height_map = true;
    uint32_t smooth_strength = 1;
    bool normalize_height = true;
    bool smoothTileEdges = true;  // Removes cliffs
};
```

**Key Insights:**
- ✅ **Simplex noise** with configurable frequency and octaves
- ✅ **PNG import** - can paint height maps in image editors!
- ✅ **Smoothing algorithms** prevent harsh transitions
- ✅ **Normalization** spreads heights across full range
- ✅ **Edge smoothing** ensures no vertical cliffs

---

### 5. Simplex Noise Implementation

**File:** `src/openrct2/world/map_generator/SimplexNoise.cpp`

```cpp
static void generateSimplexNoise(Settings* settings, HeightMap& heightMap) {
    float freq = settings->simplex_base_freq / 100.0f * (1.0f / heightMap.width);
    int32_t octaves = settings->simplex_octaves;

    int32_t low = settings->heightmapLow / 2;
    int32_t high = settings->heightmapHigh / 2 - low;

    for (int32_t y = 0; y < heightMap.height; y++) {
        for (int32_t x = 0; x < heightMap.width; x++) {
            // Fractal noise = multiple octaves of simplex
            float noiseValue = std::clamp(
                FractalNoise(x, y, freq, octaves, 2.0f, 0.65f),
                -1.0f, 1.0f
            );

            // Normalize to [0, 1]
            float normalisedNoiseValue = (noiseValue + 1.0f) / 2.0f;

            // Map to height range
            heightMap[{x, y}] = low + static_cast<int32_t>(normalisedNoiseValue * high);
        }
    }
}

static void smoothHeightMap(int32_t iterations, HeightMap& heightMap) {
    for (auto i = 0; i < iterations; i++) {
        auto copyHeight = heightMap;
        for (auto y = 1; y < heightMap.height - 1; y++) {
            for (auto x = 1; x < heightMap.width - 1; x++) {
                // 3×3 box blur averaging
                auto avg = 0;
                for (auto yy = -1; yy <= 1; yy++) {
                    for (auto xx = -1; xx <= 1; xx++) {
                        avg += copyHeight[{y + yy, x + xx}];
                    }
                }
                avg /= 9;  // Average of 9 pixels
                heightMap[{x, y}] = avg;
            }
        }
    }
}

void generateSimplexMap(Settings* settings) {
    const auto density = 2;  // 2x super-sampling for smoother results
    auto heightMap = HeightMap(mapSize.x, mapSize.y, density);

    generateSimplexNoise(settings, heightMap);
    smoothHeightMap(2 + (UtilRand() % 6), heightMap);  // Random smoothing 2-7 iterations

    setMapHeight(settings, heightMap);  // Apply to actual world tiles

    if (settings->smoothTileEdges) {
        smoothMap(settings->mapSize, smoothTileStrong);  // Remove cliffs
    }

    setWaterLevel(settings->waterLevel);
}
```

**Key Insights:**
- ✅ **Fractal noise** = multiple octaves for natural variation
- ✅ **2x density** super-sampling then downsampling for smoothness
- ✅ **Box blur smoothing** with random iterations (2-7)
- ✅ **Post-processing** to remove cliffs and set water
- ✅ **Configurable parameters** allow different landscapes

---

### 6. PNG Height Map Import

**File:** `src/openrct2/world/map_generator/PngTerrainGenerator.cpp`

```cpp
void GenerateFromHeightmapImage(Settings* settings) {
    HeightMap dest = _heightMapData;  // Copy loaded PNG data

    if (settings->smooth_height_map) {
        SmoothHeightmap(dest, settings->smooth_strength);
    }

    if (settings->normalize_height) {
        // Find min/max values in image
        uint8_t maxValue = 0;
        uint8_t minValue = 255;
        for (auto y = 0; y < dest.height; y++) {
            for (auto x = 0; x < dest.width; x++) {
                maxValue = std::max(maxValue, dest[{x, y}]);
                minValue = std::min(minValue, dest[{x, y}]);
            }
        }

        // Normalize to use full range
        for (auto y = 0; y < dest.height; y++) {
            for (auto x = 0; x < dest.width; x++) {
                uint8_t oldValue = dest[{x, y}];
                float normalisedValue = static_cast<float>(oldValue - minValue)
                                      / static_cast<float>(maxValue - minValue);
                dest[{x, y}] = settings->heightmapLow
                             + static_cast<uint8_t>(normalisedValue * rangeSize);
            }
        }
    }

    setMapHeight(settings, dest);
}
```

**Key Insights:**
- ✅ **Import PNG grayscale** directly as height data
- ✅ **Normalization** spreads values across desired range
- ✅ **Smoothing** can be applied after import
- ✅ Enables **artistic control** - paint height maps in Photoshop/GIMP!

---

## 🧬 How This Could Work in Life Simulator

### Current State
- ✅ Chunk-based world (16×16 tiles)
- ✅ RON serialization with multi-layer support
- ✅ TerrainType enum (Grass, Forest, Water, etc.)
- ❌ No height/elevation data
- ❌ No slopes or 3D terrain
- ❌ Flat world currently

### Proposed Architecture

#### 1. Add Height Layer to Chunks

```rust
// In src/serialization.rs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SerializedChunk {
    pub x: i32,
    pub y: i32,
    pub layers: HashMap<String, Vec<Vec<String>>>,
    // NEW: Height data for each tile in chunk
    pub heights: Vec<Vec<u8>>,  // 16×16 array of heights (0-255)
}

impl SerializedChunk {
    pub fn set_height(&mut self, local_x: usize, local_y: usize, height: u8) {
        self.heights[local_y][local_x] = height;
    }

    pub fn get_height(&self, local_x: usize, local_y: usize) -> u8 {
        self.heights[local_y][local_x]
    }
}
```

#### 2. Height Map Generator

```rust
// In src/map_generator.rs or new src/tilemap/height_generator.rs

use noise::{NoiseFn, Simplex, Fbm};

pub struct HeightMapConfig {
    pub min_height: u8,      // e.g., 20
    pub max_height: u8,      // e.g., 200
    pub base_frequency: f64, // e.g., 1.5
    pub octaves: usize,      // e.g., 6
    pub smoothing_passes: u32, // e.g., 3
    pub water_level: u8,     // e.g., 50
}

pub struct HeightMapGenerator {
    noise: Fbm<Simplex>,
    config: HeightMapConfig,
}

impl HeightMapGenerator {
    pub fn new(seed: u64, config: HeightMapConfig) -> Self {
        let simplex = Simplex::new(seed as u32);
        let noise = Fbm::new(seed as u32)
            .set_octaves(config.octaves)
            .set_frequency(config.base_frequency)
            .set_lacunarity(2.0)
            .set_persistence(0.5);

        Self { noise, config }
    }

    pub fn generate_for_chunk(&self, chunk_x: i32, chunk_y: i32) -> Vec<Vec<u8>> {
        let mut heights = vec![vec![0u8; 16]; 16];

        for local_y in 0..16 {
            for local_x in 0..16 {
                let world_x = chunk_x * 16 + local_x as i32;
                let world_y = chunk_y * 16 + local_y as i32;

                // Sample noise at world coordinates
                let noise_value = self.noise.get([
                    world_x as f64,
                    world_y as f64
                ]);

                // Normalize from [-1, 1] to [0, 1]
                let normalized = (noise_value + 1.0) / 2.0;

                // Map to height range
                let range = self.config.max_height - self.config.min_height;
                let height = self.config.min_height
                           + (normalized * range as f64) as u8;

                heights[local_y][local_x] = height;
            }
        }

        // Apply smoothing
        self.smooth_heights(&mut heights, self.config.smoothing_passes)
    }

    fn smooth_heights(&self, heights: &mut Vec<Vec<u8>>, passes: u32) -> Vec<Vec<u8>> {
        for _ in 0..passes {
            let copy = heights.clone();
            for y in 1..15 {
                for x in 1..15 {
                    // 3×3 box blur
                    let mut sum = 0u32;
                    for dy in -1..=1 {
                        for dx in -1..=1 {
                            sum += copy[(y + dy) as usize][(x + dx) as usize] as u32;
                        }
                    }
                    heights[y][x] = (sum / 9) as u8;
                }
            }
        }
        heights.clone()
    }
}
```

#### 3. Integration with Terrain Generation

```rust
// In src/map_generator.rs

fn generate_world_with_height(
    world_name: &str,
    seed: u64,
    radius: i32,
) -> SerializedWorld {
    let mut world = SerializedWorld::new(world_name, seed);

    // Create height map generator
    let height_config = HeightMapConfig {
        min_height: 20,
        max_height: 200,
        base_frequency: 1.5,
        octaves: 6,
        smoothing_passes: 3,
        water_level: 50,
    };
    let height_gen = HeightMapGenerator::new(seed, height_config);

    // Create terrain generator (existing)
    let terrain_gen = WorldGenerator::new(seed, radius);

    for chunk_x in -radius..=radius {
        for chunk_y in -radius..=radius {
            // Generate terrain layer (existing)
            let terrain = terrain_gen.generate_chunk(chunk_x, chunk_y);

            // Generate height layer (NEW)
            let heights = height_gen.generate_for_chunk(chunk_x, chunk_y);

            // Create chunk with both layers
            let mut chunk = SerializedChunk::new(chunk_x, chunk_y);
            chunk.set_layer("terrain", terrain);
            chunk.heights = heights;

            // Adjust terrain based on height (water at low elevations)
            adjust_terrain_by_height(&mut chunk, height_config.water_level);

            world.add_chunk(chunk);
        }
    }

    world
}

fn adjust_terrain_by_height(chunk: &mut SerializedChunk, water_level: u8) {
    for y in 0..16 {
        for x in 0..16 {
            let height = chunk.heights[y][x];

            // Override terrain based on height
            if height < water_level - 10 {
                chunk.set_terrain(x, y, TerrainType::DeepWater);
            } else if height < water_level {
                chunk.set_terrain(x, y, TerrainType::ShallowWater);
            } else if height < water_level + 5 {
                chunk.set_terrain(x, y, TerrainType::Sand); // Beach
            } else if height > 180 {
                chunk.set_terrain(x, y, TerrainType::Mountains);
            } else if height > 150 {
                chunk.set_terrain(x, y, TerrainType::Stone);
            }
            // Otherwise keep generated terrain type
        }
    }
}
```

#### 4. River Generation Using Height

```rust
// Rivers flow downhill from high to low elevation

pub struct RiverGenerator {
    height_map: Arc<HashMap<(i32, i32), Vec<Vec<u8>>>>,
}

impl RiverGenerator {
    pub fn generate_rivers(&self, num_sources: usize) -> Vec<RiverPath> {
        let mut rivers = Vec::new();

        for _ in 0..num_sources {
            // Find high elevation starting point
            let source = self.find_high_elevation_point();

            // Flow downhill using pathfinding
            let river_path = self.flow_downhill(source);

            rivers.push(river_path);
        }

        rivers
    }

    fn flow_downhill(&self, start: IVec2) -> RiverPath {
        let mut path = vec![start];
        let mut current = start;

        while let Some(next) = self.find_lowest_neighbor(current) {
            path.push(next);
            current = next;

            // Stop at water level or ocean
            if self.get_height(current) < WATER_LEVEL {
                break;
            }
        }

        RiverPath { tiles: path }
    }

    fn find_lowest_neighbor(&self, pos: IVec2) -> Option<IVec2> {
        let current_height = self.get_height(pos);

        let neighbors = [
            pos + IVec2::new(-1, 0),
            pos + IVec2::new(1, 0),
            pos + IVec2::new(0, -1),
            pos + IVec2::new(0, 1),
        ];

        neighbors.iter()
            .filter(|&&n| self.get_height(n) < current_height)
            .min_by_key(|&&n| self.get_height(n))
            .copied()
    }
}
```

#### 5. Pathfinding with Elevation Cost

```rust
// In src/pathfinding.rs

impl PathfindingGrid {
    pub fn calculate_movement_cost_with_elevation(
        &self,
        from: IVec2,
        to: IVec2,
    ) -> u32 {
        // Base terrain cost
        let terrain_cost = self.get_cost(to);

        // Get height difference
        let from_height = self.get_height(from);
        let to_height = self.get_height(to);
        let height_diff = (to_height as i32 - from_height as i32).abs();

        // Add elevation penalty
        let elevation_cost = match height_diff {
            0..=2 => 0,           // Flat - no penalty
            3..=5 => 1,           // Gentle slope
            6..=10 => 3,          // Moderate slope
            11..=20 => 8,         // Steep slope
            _ => 20,              // Very steep / cliff
        };

        // Uphill is harder than downhill
        let direction_multiplier = if to_height > from_height {
            2  // Going uphill costs 2x
        } else {
            1  // Going downhill normal cost
        };

        terrain_cost + (elevation_cost * direction_multiplier)
    }
}
```

#### 6. Godot Viewer Integration

```gdscript
# In godot-viewer/scripts/TerrainTileMap.gd

func render_chunk_with_elevation(chunk_data: Dictionary):
    var chunk_x = chunk_data.x
    var chunk_y = chunk_data.y
    var terrain = chunk_data.layers.terrain
    var heights = chunk_data.heights  # NEW

    for local_y in range(16):
        for local_x in range(16):
            var tile_coord = Vector2i(
                chunk_x * 16 + local_x,
                chunk_y * 16 + local_y
            )

            var terrain_type = terrain[local_y][local_x]
            var height = heights[local_y][local_x]

            # Get atlas coords for terrain
            var atlas_coords = terrain_to_atlas[terrain_type]

            # Adjust color based on height (shading)
            var brightness = 0.7 + (height / 255.0) * 0.6
            var modulate_color = Color(brightness, brightness, brightness)

            # Paint tile with height-based shading
            set_cell(0, tile_coord, 0, atlas_coords)

            # Optional: Add height visualization layer
            if Config.show_elevation:
                visualize_elevation(tile_coord, height)

func visualize_elevation(tile_coord: Vector2i, height: int):
    # Draw contour lines every 20 height units
    if height % 20 < 2:
        draw_contour_line(tile_coord)

    # Or use color gradient overlay
    var elevation_color = get_elevation_color(height)
    add_overlay_tile(tile_coord, elevation_color)

func get_elevation_color(height: int) -> Color:
    # Gradient from dark blue (low) to white (high)
    var t = height / 255.0
    return Color(t * 0.5, t * 0.7, 1.0, 0.3)  # Semi-transparent overlay
```

---

## 📊 Benefits of Height System

### For Ecology & Gameplay

1. **Realistic Water Bodies**
   - Rivers flow naturally downhill
   - Lakes form in valleys
   - Waterfalls at elevation changes
   - Proper drainage patterns

2. **Biome Distribution**
   - Alpine biomes at high elevation
   - Valleys have different climate
   - Rain shadow effects (future)
   - Temperature gradient by altitude

3. **Animal Behavior**
   - Deer prefer meadows in valleys
   - Mountain goats at high elevation (future)
   - Bears in foothills
   - Birds of prey in mountains (future)

4. **Strategic Gameplay** (future humans)
   - Settlements on hills (defensive)
   - Farms in fertile valleys
   - Roads avoid steep slopes
   - Bridges at low river crossings

5. **Pathfinding Realism**
   - Animals avoid steep slopes
   - Predators ambush from high ground
   - Energy cost for climbing
   - Natural territorial boundaries

### Technical Benefits

1. **Memory Efficient**
   - Only 1 byte per tile (256 height levels)
   - For 100×100 chunks = 10,000 tiles = 10KB

2. **Simple to Implement**
   - Just add `heights: Vec<Vec<u8>>` to chunks
   - Noise library already exists (noise-rs)
   - Smoothing is basic 3×3 blur

3. **Easy Serialization**
   - RON handles Vec<Vec<u8>> perfectly
   - No special encoding needed

4. **Godot Compatible**
   - Can visualize with shading
   - Can add contour lines
   - Can use for 3D rendering later

5. **PNG Import/Export** (like OpenRCT2)
   - Paint height maps in image editors
   - Share custom terrains
   - Version control friendly

---

## ⚠️ Challenges & Considerations

### 1. Existing World Compatibility
**Problem:** Old worlds don't have height data
**Solution:**
- Add migration system to generate heights for old worlds
- Default height = 100 (middle value) for flat worlds
- Or regenerate height from terrain type heuristics

### 2. Chunk Boundaries
**Problem:** Height might not match between chunks
**Solution:**
- Generate all chunks in one pass (already doing this)
- Smooth across boundaries after generation
- Use continuous noise (already seamless)

### 3. Performance
**Problem:** Extra data to serialize/deserialize
**Solution:**
- 1 byte per tile is minimal
- Can compress height data (RLE for flat areas)
- Lazy loading already implemented for chunks

### 4. Visual Representation
**Problem:** Isometric view makes height harder to show
**Solution:**
- Use shading (lighter = higher)
- Add contour lines option
- Optional elevation overlay layer
- Height display on hover

### 5. Pathfinding Complexity
**Problem:** Elevation adds pathfinding cost
**Solution:**
- Keep elevation cost separate from terrain cost
- Cache common paths
- Diagonal movement already helps
- Most terrain is gently sloping

---

## 🎯 Implementation Strategy

### Phase 1: Foundation (Week 1)
1. Add `heights: Vec<Vec<u8>>` to `SerializedChunk`
2. Add `HeightMapGenerator` with simplex noise
3. Integrate into `map_generator` binary
4. Test world generation with heights

### Phase 2: Terrain Integration (Week 2)
5. Use height to influence terrain type (water at low, mountains at high)
6. Add height-based biome selection
7. Implement height smoothing algorithms
8. Test various height configurations

### Phase 3: River Generation (Week 3)
9. Implement downhill flow algorithm
10. Generate river sources at high elevation
11. Create river paths following height gradient
12. Add rivers to world serialization

### Phase 4: Visualization (Week 4)
13. Update Godot viewer to read height data
14. Add height-based shading/coloring
15. Optional contour line overlay
16. Height tooltip display

### Phase 5: Pathfinding (Week 5)
17. Add elevation cost to pathfinding
18. Test with animals navigating slopes
19. Balance uphill/downhill costs
20. Optimize performance

### Phase 6: Polish (Week 6)
21. Add PNG import/export for height maps
22. Migration system for old worlds
23. Configuration UI for height parameters
24. Documentation and examples

---

## 📝 Configuration Example

```rust
// In src/map_generator.rs

pub struct WorldGenConfig {
    // Existing
    pub seed: u64,
    pub radius: i32,
    pub name: String,

    // NEW: Height configuration
    pub height_config: HeightMapConfig,
}

pub struct HeightMapConfig {
    // Basic range
    pub min_height: u8,          // Default: 20
    pub max_height: u8,          // Default: 200
    pub water_level: u8,         // Default: 50

    // Noise parameters
    pub frequency: f64,          // Default: 1.5 (higher = more variation)
    pub octaves: usize,          // Default: 6 (layers of detail)
    pub lacunarity: f64,         // Default: 2.0 (frequency multiplier per octave)
    pub persistence: f64,        // Default: 0.5 (amplitude multiplier per octave)

    // Post-processing
    pub smoothing_passes: u32,   // Default: 3
    pub normalize: bool,         // Default: true

    // Terrain influence
    pub beach_height_range: u8,  // Default: 5 (tiles above water)
    pub mountain_height: u8,     // Default: 180
    pub hill_height: u8,         // Default: 150
}

impl Default for HeightMapConfig {
    fn default() -> Self {
        Self {
            min_height: 20,
            max_height: 200,
            water_level: 50,
            frequency: 1.5,
            octaves: 6,
            lacunarity: 2.0,
            persistence: 0.5,
            smoothing_passes: 3,
            normalize: true,
            beach_height_range: 5,
            mountain_height: 180,
            hill_height: 150,
        }
    }
}
```

---

## 🔬 Testing Plan

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_height_generation() {
        let config = HeightMapConfig::default();
        let gen = HeightMapGenerator::new(12345, config);
        let heights = gen.generate_for_chunk(0, 0);

        assert_eq!(heights.len(), 16);
        assert_eq!(heights[0].len(), 16);
        assert!(heights[0][0] >= config.min_height);
        assert!(heights[0][0] <= config.max_height);
    }

    #[test]
    fn test_height_smoothing() {
        // Create rough heights
        let mut heights = vec![vec![0u8; 16]; 16];
        heights[8][8] = 255;  // Spike in middle

        let config = HeightMapConfig::default();
        let gen = HeightMapGenerator::new(12345, config);
        let smoothed = gen.smooth_heights(&mut heights, 5);

        // Spike should be reduced
        assert!(smoothed[8][8] < 255);
        // Neighbors should be elevated
        assert!(smoothed[7][8] > 0);
    }

    #[test]
    fn test_river_flows_downhill() {
        let river_gen = RiverGenerator::new(height_map);
        let source = IVec2::new(100, 100);  // High point
        let path = river_gen.flow_downhill(source);

        // Each step should be lower or equal
        for i in 1..path.tiles.len() {
            let prev_height = river_gen.get_height(path.tiles[i-1]);
            let curr_height = river_gen.get_height(path.tiles[i]);
            assert!(curr_height <= prev_height);
        }
    }
}
```

### Integration Tests
```rust
#[test]
fn test_full_world_generation_with_heights() {
    let world = generate_world_with_height("test_world", 12345, 5);

    // Check all chunks have heights
    for (_, chunk) in world.chunks.iter() {
        assert_eq!(chunk.heights.len(), 16);
        assert_eq!(chunk.heights[0].len(), 16);

        // Heights should be in valid range
        for y in 0..16 {
            for x in 0..16 {
                let h = chunk.heights[y][x];
                assert!(h >= 20 && h <= 200);
            }
        }
    }

    // Check terrain matches height
    // Low areas should be water, high areas should be mountains
    let center_chunk = world.get_chunk(0, 0).unwrap();
    for y in 0..16 {
        for x in 0..16 {
            let height = center_chunk.heights[y][x];
            let terrain = center_chunk.get_terrain(x, y);

            if height < 40 {
                assert!(matches!(terrain, TerrainType::DeepWater | TerrainType::ShallowWater));
            } else if height > 180 {
                assert!(matches!(terrain, TerrainType::Mountains | TerrainType::Stone));
            }
        }
    }
}
```

---

## 🎨 Visual Examples (Godot Viewer)

### Height Visualization Modes

**1. Shading Mode**
```
Lower elevation = darker
Higher elevation = lighter

Example:
  Dark blue  ▓▓  = Deep water (height: 20-40)
  Light blue ▒▒  = Shallow water (height: 40-50)
  Green      ░░  = Grass (height: 50-150)
  Brown      ▒▒  = Hills (height: 150-180)
  Gray       ▓▓  = Mountains (height: 180-200)
```

**2. Contour Lines Mode**
```
Draw lines every 20 height units
Creates topographic map effect

   ╱───╲
  ╱     ╲
 ╱   ·   ╲  ← Peak
╱─────────╲
```

**3. Color Gradient Mode**
```
Rainbow gradient from low to high
  Blue → Green → Yellow → Orange → Red

  🔵 Low elevation
  🟢 Medium low
  🟡 Medium
  🟠 Medium high
  🔴 High elevation
```

---

## 💡 Future Enhancements

### 1. Dynamic Height Changes
- Erosion simulation (rivers carve valleys)
- Landslides on steep slopes
- Volcanic activity (raise terrain)
- Humans terraform (future)

### 2. 3D Visualization
- True 3D rendering in Godot
- Mesh generation from height map
- Realistic shadows and lighting
- Camera fly-through

### 3. Weather Effects
- Rain flows downhill (water simulation)
- Snow accumulates at high elevation
- Fog in valleys
- Clouds at mountain peaks

### 4. Advanced Rivers
- Width based on flow accumulation
- Meandering in flat areas
- Deltas at river mouths
- Waterfalls at steep drops

### 5. Caves & Underground
- Negative heights for caves
- Underground water table
- Cave systems following terrain
- Mineral deposits at depth

---

## 📚 References

**OpenRCT2 Code:**
- `src/openrct2/world/map_generator/HeightMap.hpp` - Core height map structure
- `src/openrct2/world/map_generator/MapGen.h` - Generation settings
- `src/openrct2/world/map_generator/SimplexNoise.cpp` - Noise generation
- `src/openrct2/world/tile_element/SurfaceElement.h` - Tile height storage
- `src/openrct2/world/tile_element/Slope.h` - Slope bitfield system

**Rust Libraries:**
- [noise-rs](https://github.com/Razaekel/noise-rs) - Simplex/Perlin noise
- [image-rs](https://github.com/image-rs/image) - PNG import/export

**Algorithms:**
- Simplex noise (Ken Perlin, 2001)
- Fractal Brownian Motion (fBm)
- Box blur smoothing
- Flow accumulation for rivers

---

## ✅ Conclusion

**Is it worth implementing?**

**YES - For these reasons:**

1. ✅ **Scientifically Sound** - Real wildlife ecosystems have elevation gradients
2. ✅ **Relatively Simple** - Only 1 byte per tile, proven algorithms exist
3. ✅ **Natural Rivers** - Rivers flowing downhill is essential for realism
4. ✅ **Better Biomes** - Alpine, valley, and riparian zones need elevation
5. ✅ **Richer Gameplay** - Animals behave differently at different elevations
6. ✅ **Visual Appeal** - Height adds depth to isometric view
7. ✅ **Future-Proof** - Enables 3D rendering, caves, weather, etc.

**Implementation Effort:**
- **Low** - Only need height array + simplex noise
- **Medium** - River generation needs pathfinding
- **High** - 3D rendering (future)

**Recommended Approach:**
Start with **Phase 1-2** (height generation + terrain integration) which gives immediate value with minimal effort. Rivers and visualization can come in later phases.

The OpenRCT2 approach is proven, efficient, and well-suited to our chunk-based architecture. The memory overhead is negligible (1 byte per tile) and the visual/gameplay benefits are substantial.

**Next Step:** Add height generation to Phase 3 of the PRD roadmap.
