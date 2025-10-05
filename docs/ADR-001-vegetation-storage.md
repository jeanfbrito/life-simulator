# ADR-001: Vegetation Storage Layout

## Status

Accepted

## Context

The plant system needs to store biomass data for each tile in the world. We considered two main approaches:

1. **Dense Grid**: Store biomass for every tile in a 2D array
2. **Sparse HashMap**: Store only tiles that have vegetation using a hash map

## Decision

We chose **Sparse HashMap** storage for the vegetation system.

## Consequences

### Positive

1. **Memory Efficiency**: Only stores tiles that can support vegetation
   - Water tiles (Deep Water, Shallow Water) use 0 bytes
   - Desert/Mountain tiles use minimal space
   - Typical worlds: 10-50% vegetation coverage = 60-90% memory savings

2. **Performance**: Better cache locality for active tiles
   - Active tiles (recently grazed, regrowing) are hot in cache
   - Inactive tiles can be sampled rather than processed every cycle

3. **Scalability**: Linear memory usage with vegetation density
   - Handles large maps (1000x1000) without proportional memory growth
   - No wasted storage on uninhabitable terrain

4. **Integration**: Aligns with existing chunk-based architecture
   - Same chunk size (16×16) as terrain system
   - Sparse loading matches existing world loader patterns

### Negative

1. **Lookup Overhead**: HashMap lookup vs direct array access
   - Mitigated by limiting active tile updates to 1000 per cycle
   - Inactive tile sampling keeps overall overhead low

2. **Iteration Complexity**: Cannot iterate over all tiles efficiently
   - Worked around by maintaining active tile set
   - Sampling provides coverage of inactive tiles

3. **Memory Fragmentation**: HashMap allocation overhead
   - Acceptable trade-off for memory savings on sparse vegetation

## Implementation

```rust
pub struct VegetationGrid {
    tiles: HashMap<IVec2, TileVegetation>,  // Sparse storage
    active_tiles: HashMap<IVec2, u64>,      // Fast access to regrowing tiles
    total_suitable_tiles: usize,            // Statistics
}
```

## Performance Estimates

| World Size | Vegetation Coverage | Dense Grid | Sparse HashMap | Savings |
|------------|-------------------|------------|----------------|---------|
| 100×100    | 10%               | 40 KB      | 4 KB           | 90%     |
| 100×100    | 50%               | 40 KB      | 20 KB          | 50%     |
| 100×100    | 100%              | 40 KB      | 40 KB          | 0%      |
| 500×500    | 20%               | 1 MB       | 200 KB         | 80%     |

## Alternatives Considered

### Dense Grid

**Pros**:
- O(1) direct access by coordinate
- Simple iteration over all tiles
- Predictable memory usage

**Cons**:
- Wastes memory on water/deep water tiles
- Poor cache locality for sparse vegetation
- Fixed overhead regardless of vegetation density

### Hybrid Approach

**Idea**: Use dense grids for vegetated chunks, sparse storage elsewhere

**Rejected**: Added complexity for minimal benefit over pure sparse approach

## Future Considerations

1. **Region-based Storage**: Could switch to region-based hashing if performance issues arise
2. **Compression**: Consider bitmap compression for very large worlds
3. **Persistence**: Sparse storage easier to serialize efficiently

## Related Decisions

- [ADR-002]: Growth System Frequency (pending)
- [ADR-003]: Herbivore Integration Strategy (pending)

## References

- Original plant system plan: `docs/PLANT_SYSTEM_PLAN.md`
- Parameter specification: `docs/PLANT_SYSTEM_PARAMS.md`