# Resource System Balancing Analysis

## Current Parameters Analysis

### 1. Resource Spawn Densities (ResourceConfig::default)

**Current Values:**
- Trees: 8% (0.08) - HIGH
- Berry Bushes: 3% (0.03) - HIGH
- Hazel Shrubs: 2% (0.02) - HIGH
- Mushroom Patches: 1.5% (0.015) - HIGH
- Wild Roots: 1% (0.01) - HIGH
- Rocks: 3% (0.03) - OK
- Bushes: 5% (0.05) - HIGH
- Flowers: 4% (0.04) - OK

**Issues:**
- Overall resource density is ~27.5% per tile (1 in 4 tiles has something)
- This is too dense for natural appearance and gameplay balance
- Shrubs (5% total) are too common relative to trees
- Collectables (2.5% total) are too common for "rare" resources

### 2. Biomass & Growth Parameters

**Trees (e.g., Oak):**
- Biomass Cap: 100.0 - GOOD
- Growth Rate: 0.8 - GOOD (slow growth for large biomass)
- Regrowth: 1000 ticks - GOOD (long term resource)

**Shrubs (e.g., Berry Bush):**
- Biomass Cap: 25.0 - GOOD
- Growth Rate: 1.2 - GOOD (faster than trees)
- Regrowth: 500 ticks - GOOD (medium term)
- Nutritional Value: 15.0 - GOOD

**Collectables (e.g., Mushroom):**
- Biomass Cap: 8.0 - GOOD (low biomass for small plants)
- Growth Rate: 1.5 - GOOD (fast growth)
- Regrowth: 300 ticks - GOOD (short term, renewable)
- Nutritional Value: 8.0 - OK

### 3. Herbivore Diet Preferences

**Rabbit (HerbivoreDiet::rabbit()):**
- Grass Preference: 0.3 - LOW (good for balance)
- Shrub Preference: 0.8 - HIGH (rabbits love shrubs)
- Min Biomass: 12.0 - GOOD

**Deer (HerbivoreDiet::deer()):**
- Grass Preference: 0.7 - HIGH (deer graze)
- Shrub Preference: 0.4 - MEDIUM (deer also browse)
- Min Biomass: 15.0 - GOOD

**Raccoon:**
- Grass Preference: 0.6 - MEDIUM
- Shrub Preference: 0.8 - HIGH
- Min Biomass: 12.0 - GOOD

## Recommended Balancing Changes

### 1. Reduce Base Spawn Densities

**New Recommended Values:**
- Trees: 6% → 5% (0.08 → 0.05)
- Berry Bushes: 3% → 1.5% (0.03 → 0.015)
- Hazel Shrubs: 2% → 1% (0.02 → 0.01)
- Mushroom Patches: 1.5% → 0.8% (0.015 → 0.008)
- Wild Roots: 1% → 0.6% (0.01 → 0.006)
- Bushes: 5% → 3% (0.05 → 0.03)
- Flowers: 4% → 2.5% (0.04 → 0.025)

**Result:** Overall density drops from 27.5% to ~15.4% (1 in 6.5 tiles)

### 2. Improve Biome Distribution

**Forest Biome Enhancements:**
- Tree multiplier: 2.0 → 2.5 (more forested)
- Shrub multiplier: 1.5 → 1.2 (fewer shrubs in dense forest)
- Collectable multiplier: 1.8 → 2.2 (more mushrooms in forests)

**Swamp Biome Refinements:**
- Tree multiplier: 0.8 → 0.6 (fewer trees in swamps)
- Collectable multiplier: 2.2 → 3.0 (excellent mushroom habitat)
- Add special "swamp_shrub_multiplier: 1.8"

**Plains Biome Balance:**
- Tree multiplier: 0.4 → 0.3 (more open plains)
- Shrub multiplier: 1.2 → 1.5 (more shrubs in plains)
- Flower multiplier: 1.5 → 2.0 (more flowers in meadows)

### 3. Adjust Resource Priority System

**Current Priority:** Trees > Shrubs > Collectables > Flowers > Bushes

**Issue:** Collectables are too common for their intended rarity.

**Solution:** Adjust probability distribution within combined chances:

```rust
// Current: Trees 40%, Shrubs 60%, Collectables 60%
// New: Trees 50%, Shrubs 35%, Collectables 15%

if tree_roll < 0.5 { // Oak
if tree_roll < 0.8 { // Pine
else { // Birch }

if shrub_roll < 0.7 { // Berry Bush
else { // Hazel Shrub }

if collectable_roll < 0.7 { // Mushroom Patch
else { // Wild Root }
```

### 4. Fine-tune Nutritional Values

**Current Issues:**
- Wild Roots (12.0) > Mushrooms (8.0) but roots should be harder to obtain
- Berries (15.0) are very generous for common resource

**Recommendations:**
- Mushroom Patch: 8.0 → 6.0 (moderate nutrition)
- Wild Root: 12.0 → 10.0 (good but not overpowered)
- Berry Bush: 15.0 → 12.0 (still good food source)
- Hazel Shrub: 20.0 → 18.0 (premium but not game-breaking)

### 5. Terrain-Specific Adjustments

**Forest Terrain:**
- Reduce tree density slightly (2.0x → 1.5x multiplier)
- Increase mushroom chance (2.5x → 3.0x multiplier)

**Swamp Terrain:**
- Major mushroom boost (3.0x → 4.0x multiplier)
- Add rare special resources in future

**Dirt Terrain:**
- Increase wild root chance (1.5x → 2.0x multiplier)
- Decrease rock chance (0.5x → 0.3x multiplier)

## Implementation Priority

### High Priority (Critical Balance)
1. ✅ Reduce base spawn densities (affects overall game balance)
2. ✅ Adjust tree/shrub/collectable priority distribution
3. ✅ Fine-tune nutritional values

### Medium Priority (Biome Variety)
4. ✅ Refine biome multipliers for better distribution
5. ✅ Adjust terrain-specific modifiers

### Low Priority (Future Enhancements)
6. Add rare resource variants
7. Seasonal resource variations
8. Resource depletion mechanics

## Testing Plan

### Density Validation
- Generate test world with new parameters
- Count resources per 100x100 area
- Target: ~15% coverage with good distribution

### Biome Testing
- Test each biome type
- Verify characteristic resources appear appropriately
- Ensure swamps have mushrooms, forests have trees, etc.

### Gameplay Testing
- Spawn herbivores and verify foraging behavior
- Test resource sustainability with current population
- Adjust if resources deplete too quickly/slowly

## Expected Outcomes

### Visual Improvements
- Less crowded appearance
- More natural resource clustering
- Clearer biome differentiation

### Gameplay Balance
- Resources feel valuable but not scarce
- Herbivores have meaningful foraging choices
- Collectables feel like special discoveries

### Performance
- Slightly fewer resources to process
- Better chunk loading performance
- Smoother visualization

## Migration Strategy

1. Update ResourceConfig::default() values
2. Test with existing world generation
3. Regenerate worlds with new parameters
4. Verify web viewer displays correctly
5. Run entity behavior tests

This balancing pass should create a more natural and engaging resource distribution while maintaining the ecosystem's functionality.