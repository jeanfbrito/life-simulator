# Spatial Mate Matching - Quick Reference

## Using the Spatial Index for Mate Matching

### For Developers

If you need to add spatial optimization to another system (e.g., predator hunting):

#### Step 1: Import
```rust
use crate::entities::reproduction::mate_matching_system_with_spatial;
use crate::entities::{SpatialEntityIndex, SpatialEntityType};
```

#### Step 2: Update System Signature
```rust
pub fn your_system(
    // ... other parameters ...
    spatial_index: Res<SpatialEntityIndex>,
) {
    // Your code
}
```

#### Step 3: Use Spatial Query
```rust
let nearby = spatial_index.entities_in_radius(
    center_position,
    search_radius,
    Some(SpatialEntityType::Herbivore),  // Filter by type
);

// Now iterate only nearby entities instead of all entities
for nearby_entity in nearby {
    // Process only k entities instead of N
}
```

---

## Entity Type Classification

```
Herbivores:
├── Rabbit     → SpatialEntityType::Herbivore
└── Deer       → SpatialEntityType::Herbivore

Predators:
├── Fox        → SpatialEntityType::Predator
└── Wolf       → SpatialEntityType::Predator

Omnivores:
├── Bear       → SpatialEntityType::Omnivore
└── Raccoon    → SpatialEntityType::Omnivore
```

---

## Performance Comparison

### Original Linear Search
```rust
for female in females {
    for male in all_males {  // O(N) iteration
        let dist = distance(female, male);
        if dist <= radius {
            // Found mate
        }
    }
}
// Complexity: O(M * N)
```

### Spatial Index Query
```rust
for female in females {
    let nearby = spatial_index.entities_in_radius(  // O(k) query
        female.pos,
        radius,
        Some(EntityType),
    );
    for male in nearby {  // O(k) iteration where k << N
        let dist = distance(female, male);
        if dist <= radius {
            // Found mate
        }
    }
}
// Complexity: O(M * k) where k << N
```

---

## Key Parameters

### search_radius
Each species has configurable mating search radius:
```rust
pub struct ReproductionConfig {
    pub mating_search_radius: i32,  // In tiles
    // ...
}
```

Current values:
- Rabbit: 50 tiles
- Deer: 60 tiles
- Fox: 120 tiles
- Wolf: 160 tiles
- Bear: 100 tiles (varies with config)
- Raccoon: 100 tiles

### EntityType Filter
Must match species classification:
- Rabbits + Deer → `SpatialEntityType::Herbivore`
- Fox + Wolf → `SpatialEntityType::Predator`
- Bear + Raccoon → `SpatialEntityType::Omnivore`

---

## Testing

### Run Integration Tests
```bash
cargo test --test spatial_mate_integration_test
```

### Run All Tests
```bash
cargo test --lib
```

### Run Single Species Test
```bash
cargo test rabbit_mate  # Tests rabbit mate matching
```

---

## Troubleshooting

### Issue: "unresolved import"
**Solution**: Ensure `SpatialEntityIndex` and `SpatialEntityType` are in scope
```rust
use crate::entities::{SpatialEntityIndex, SpatialEntityType};
```

### Issue: Mate matching not working
**Solution**: Verify spatial index resource is initialized in Bevy app
- Check that spatial_index is added as a resource
- Ensure entity positions are registered in spatial index
- Verify EntityType filter matches entity classification

### Issue: Type mismatch with SpatialEntityType
**Solution**: Use correct variant for species:
- Herbivores: `SpatialEntityType::Herbivore` (not Herbivores)
- Predators: `SpatialEntityType::Predator` (singular)
- Omnivores: `SpatialEntityType::Omnivore` (singular)

---

## Current Status

**All 6 species integrated**: ✅
- Rabbit ✅
- Deer ✅
- Fox ✅
- Wolf ✅
- Bear ✅
- Raccoon ✅

**Tests passing**: 279 total
- 268 existing library tests
- 11 new spatial integration tests

---

## Files Modified

Core implementation:
- `src/entities/reproduction.rs` - New spatial function

Species wrappers (all following identical pattern):
- `src/entities/types/rabbit.rs`
- `src/entities/types/deer.rs`
- `src/entities/types/fox.rs`
- `src/entities/types/wolf.rs`
- `src/entities/types/bear.rs`
- `src/entities/types/raccoon.rs`

Module exports:
- `src/entities/mod.rs`

Tests:
- `tests/spatial_mate_integration_test.rs`

---

## Documentation Links

- **Full Delivery**: `SPATIAL_MATE_MATCHING_DELIVERY.md`
- **Spatial Index**: `src/entities/spatial_index.rs`
- **Reproduction System**: `src/entities/reproduction.rs`
- **Species Documentation**: `docs/SPECIES_REFERENCE.md`
