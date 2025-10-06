# Phase 6: Cleanup & Legacy Removal Plan

## Legacy Components to Remove/Deprecate

### 1. VegetationGrid (Old System)
- **Location**: `src/vegetation/mod.rs:235-272`
- **Issue**: Replaced by ResourceGrid + ChunkLODManager
- **Status**: Should be deprecated and eventually removed

### 2. TileVegetation (Old Data Structure)
- **Location**: `src/vegetation/mod.rs:63-100`
- **Issue**: Replaced by GrazingCell in ResourceGrid
- **Status**: Should be deprecated

### 3. Chunk States System
- **Location**: `src/vegetation/mod.rs:265` (chunk_states field)
- **Issue**: Replaced by ChunkLODManager state tracking
- **Status**: Should be removed

### 4. Legacy Growth Systems
- **Location**: `src/vegetation/mod.rs` (various growth functions)
- **Issue**: Replaced by event-driven ResourceGrid system
- **Status**: Should be deprecated

### 5. Active Tiles Arrays
- **Location**: `src/vegetation/mod.rs` (active_tiles fields)
- **Issue**: Replaced by sparse ResourceGrid storage
- **Status**: Should be removed

### 6. Legacy Metrics Dashboard
- **Location**: `src/vegetation/mod.rs:274-322`
- **Issue**: Replaced by ResourceGrid + ChunkLOD metrics
- **Status**: Should be deprecated

### 7. Old Heatmap System
- **Location**: `src/vegetation/mod.rs:2046-2059`
- **Issue**: Replaced by Phase 5 ResourceGrid-based heatmap
- **Status**: Should be removed

## Migration Strategy

1. **Phase 1**: Mark legacy components as deprecated
2. **Phase 2**: Update all references to use new systems
3. **Phase 3**: Remove deprecated code
4. **Phase 4**: Update documentation
5. **Phase 5**: Final validation

## Systems Using Legacy Code

### Affected Files:
- `src/vegetation/mod.rs` - Main vegetation module
- `src/vegetation/memory_optimization.rs` - Memory optimization
- Tests and benchmarks that use VegetationGrid

### Systems to Update:
- VegetationPlugin resource initialization
- Setup and startup systems
- Benchmark systems
- Web API calls that reference old systems

## New Architecture (Post-Cleanup)

### Core Components:
- **ResourceGrid**: Sparse, event-driven vegetation storage
- **ChunkLODManager**: Proximity-based level-of-detail management
- **HeatmapRefreshManager**: On-demand heatmap refresh

### Systems:
- ResourceGrid event processing
- ChunkLOD update systems
- Phase 5 API endpoints
- Performance monitoring

## Validation Requirements

1. **Code Quality**: cargo check, cargo fmt, cargo clippy clean
2. **Performance**: Steady tick <20ms with grazing activity
3. **Functionality**: All vegetation features work with new systems
4. **Documentation**: Updated to reflect new architecture
5. **Tests**: All existing tests pass with new systems