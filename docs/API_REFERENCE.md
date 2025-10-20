# Life Simulator API Reference

The HTTP server provides comprehensive real-time data access for the ecosystem simulation.

**Base URL**: `http://127.0.0.1:54321`

## World Data Endpoints

### World Information
```http
GET /api/world_info
```
Returns world metadata including name, seed, chunk count, and boundaries.

**Response:**
```json
{
  "name": "generated_world",
  "seed": 12345,
  "chunk_count": 121,
  "bounds": {"min_x": -6, "max_x": 6, "min_y": -6, "max_y": 6}
}
```

### Current World Details
```http
GET /api/world/current
```
Returns detailed information about the currently loaded world.

### List Available Worlds
```http
GET /api/worlds
```
Returns a list of all generated worlds available for loading.

### Select World
```http
POST /api/world/select
Content-Type: application/json

{"world_name": "my_world"}
```
Switches to a different world for the simulation.

## Entity & Species Data

### Real-time Entity Data
```http
GET /api/entities
```
Returns current positions, stats, and behaviors for all entities.

**Response:**
```json
{
  "entities": [
    {
      "id": 123,
      "species": "rabbit",
      "position": {"x": 10.5, "y": 15.2},
      "health": 85,
      "hunger": 30,
      "thirst": 45,
      "current_action": "grazing",
      "fear_level": 0.2
    }
  ]
}
```

### Species Metadata
```http
GET /api/species
```
Returns species information including behaviors, reproduction stats, and AI parameters.

## Vegetation & Ecosystem

### Biomass Density
```http
GET /api/vegetation/biomass
```
Returns real-time biomass density heatmap for the entire world.

**Response:**
```json
{
  "biomass_data": [
    {"x": 0, "y": 0, "density": 0.85},
    {"x": 1, "y": 0, "density": 0.72}
  ],
  "total_biomass": 1250000,
  "coverage_percentage": 78.5
}
```

### Vegetation Performance
```http
GET /api/vegetation/performance
```
Returns performance metrics for the ResourceGrid vegetation system.

### Memory Usage
```http
GET /api/vegetation/memory
```
Memory usage analysis for the vegetation storage system.

### Ecosystem Statistics
```http
GET /api/vegetation/stats
```
Comprehensive ecosystem health metrics and statistics.

### Performance Metrics
```http
GET /api/vegetation/metrics
```
Performance dashboard with tick times, memory usage, and system health.

## Performance & Benchmarking

### Quick Benchmark
```http
GET /api/vegetation/benchmark/quick
```
Runs a quick performance benchmark of the vegetation system.

### Comprehensive Benchmark
```http
GET /api/vegetation/benchmark/phase4
```
Runs comprehensive system benchmarking across all components.

### Current Performance Rating
```http
GET /api/vegetation/benchmark/current
```
Returns current performance rating and health status.

### Historical Performance
```http
GET /api/vegetation/benchmark/history
```
Returns historical performance trends and analysis.

## Terrain & Chunks

### Web Viewer
```http
GET /viewer.html
```
Interactive web viewer with biomass overlays and entity tracking.

### Specific Chunks
```http
GET /api/chunks?coords=x1,y1&coords=x2,y2
```
Returns terrain data for specific chunk coordinates.

### Multi-layer Chunk Data
```http
GET /api/chunks?center_x=0&center_y=0&radius=3&layers=true
```
Returns multi-layer chunk data with batching to prevent timeouts.

**Parameters:**
- `center_x`, `center_y`: Center chunk coordinates
- `radius`: Number of chunks around center (default: 1)
- `layers`: Include detailed layer information (default: false)

**Response:**
```json
{
  "chunks": [
    {
      "coord": {"x": 0, "y": 0},
      "terrain": [["grass", "forest", "water"]],
      "resources": [["tree", null, "rock"]],
      "heights": [[56, 58, 50]],
      "entities": [{"id": 123, "species": "rabbit"}]
    }
  ]
}
```

## API Usage Examples

### Basic Operations
```bash
# List available worlds
curl http://127.0.0.1:54321/api/worlds

# Select different world
curl -X POST http://127.0.0.1:54321/api/world/select \
  -H "Content-Type: application/json" \
  -d '{"world_name": "my_world"}'

# Get real-time biomass data
curl http://127.0.0.1:54321/api/vegetation/biomass

# Run performance benchmark
curl http://127.0.0.1:54321/api/vegetation/benchmark/quick
```

### Monitoring
```bash
# Watch entity positions
watch -n 1 'curl -s http://127.0.0.1:54321/api/entities | jq ".entities | length"'

# Monitor ecosystem health
curl -s http://127.0.0.1:54321/api/vegetation/stats | jq ".health_score"

# Check system performance
curl -s http://127.0.0.1:54321/api/vegetation/metrics | jq ".avg_tick_time"
```

### Data Analysis
```bash
# Export biomass data for analysis
curl http://127.0.0.1:54321/api/vegetation/biomass | jq '.biomass_data' > biomass.json

# Get species populations
curl -s http://127.0.0.1:54321/api/entities | \
  jq -r '.entities | group_by(.species) | map({species: .[0].species, count: length})'

# Performance history
curl -s http://127.0.0.1:54321/api/vegetation/benchmark/history | \
  jq '.historical_data | last(10)'
```

## Response Format

All API responses use JSON format with the following conventions:

- **Success**: HTTP 200 with JSON body
- **Error**: HTTP 4xx/5xx with error message
- **Arrays**: Use plural names (`entities`, `chunks`, `biomass_data`)
- **Metadata**: Include counts, totals, and timestamps
- **Coordinates**: Use `{"x": 0, "y": 0}` format

## Rate Limiting & Performance

- **No explicit rate limiting** (local development)
- **Batched requests** for large data to prevent timeouts
- **CORS enabled** for web viewer access
- **Compressed responses** for large datasets

## Real-time Updates

For real-time updates, use:
- **Polling**: Request `/api/entities` every 200-500ms
- **WebSockets**: Not implemented (consider for future)
- **Server-Sent Events**: Not implemented (consider for future)

## Error Handling

Common error responses:
```json
{
  "error": "World not found",
  "code": "WORLD_NOT_FOUND",
  "details": "The specified world does not exist"
}
```

Error codes:
- `WORLD_NOT_FOUND`: World file doesn't exist
- `INVALID_COORDINATES`: Chunk coordinates out of bounds
- `BENCHMARK_RUNNING`: Another benchmark is in progress
- `SYSTEM_ERROR`: Internal simulation error
