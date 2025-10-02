# Entity Visualization in Web Viewer

## Overview
This document describes the entity visualization system that displays wandering entities (creatures, people, etc.) on the web-based map viewer.

## Architecture

### Backend Components

#### Entity Tracker (`src/entities/entity_tracker.rs`)
- **Global Singleton**: Thread-safe tracker for all entities in the simulation
- **Bevy System Integration**: Automatically syncs entity data every frame
- **JSON Export**: Provides entity data in JSON format for the web API
- **Data Structure**:
  ```rust
  pub struct EntityData {
      pub entity_id: u32,
      pub name: String,
      pub species: String,
      pub position: IVec2,
  }
  ```

#### Web API Endpoint (`src/web_server_simple.rs`)
- **Endpoint**: `GET /api/entities`
- **Response Format**:
  ```json
  {
    "entities": [
      {
        "id": 0,
        "name": "Wanderer_0",
        "entity_type": "Human",
        "position": {"x": 1, "y": -14}
      }
    ]
  }
  ```

### Frontend Components

#### Entity Manager (`web-viewer/js/entity-manager.js`)
- **Polling System**: Fetches entity data from `/api/entities` every 1000ms
- **Caching**: Maintains the last known entity state to handle temporary network issues
- **Filtering**: Provides methods to filter entities by bounds
- **Features**:
  - `startPolling(intervalMs)` - Begin polling for entities
  - `stopPolling()` - Stop polling
  - `getEntities()` - Get all entities
  - `getEntitiesInBounds(minX, minY, maxX, maxY)` - Get entities in specific area
  - `getEntityCount()` - Get total entity count

#### Renderer Updates (`web-viewer/js/renderer.js`)
- **renderEntities()**: New method that renders entities as colored circles
- **Entity Colors**:
  - 🟢 Wanderer/Human: `#44ff44` (green)
  - 🟠 Animal: `#ffaa44` (orange)
  - 🔵 Person: `#4444ff` (blue)
  - 🔴 Other/Default: `#ff4444` (red)
- **Visual Style**:
  - Circular entities with white borders
  - Dynamic sizing based on zoom level
  - Subtle shadow for depth
  - Only visible entities are rendered (viewport culling)

#### App Integration (`web-viewer/js/app.js`)
- Initializes EntityManager on startup
- Starts polling at 1000ms intervals
- Passes entity data to renderer each frame
- Updates entity count in sidebar statistics
- Properly cleans up on destroy

### User Interface

#### Statistics Display
- **Entity Count Card**: Shows total number of entities in the simulation
- Located in the sidebar stats grid
- Updates in real-time as entities are added/removed

#### Entity Legend
- **Color-coded Legend**: Shows what each entity type looks like
- Circular color swatches matching entity rendering
- Located in sidebar below terrain legend
- Lists:
  - 🚶 Wanderer (green)
  - 🐾 Animal (orange)
  - 👤 Person (blue)
  - ❓ Other (red)

## How It Works

### Data Flow
1. **Bevy Simulation** (10 TPS tick rate):
   - Entities move and update positions
   - `sync_entities_to_tracker` system runs every frame
   - Updates global `EntityTracker` with current positions

2. **Web Server** (HTTP/1.1):
   - Receives GET request to `/api/entities`
   - Reads from `EntityTracker` (thread-safe read lock)
   - Returns JSON response with all entity data

3. **Web Viewer** (JavaScript):
   - `EntityManager` polls every 1 second
   - Fetches entity data via HTTP
   - Stores entities in local state
   - Main app passes entities to renderer

4. **Rendering** (60 FPS):
   - Renderer receives entities each frame
   - Converts world coordinates to screen coordinates
   - Culls entities outside viewport
   - Draws visible entities as colored circles

### Performance Considerations
- **Polling Interval**: 1000ms is chosen to balance responsiveness and server load
- **Viewport Culling**: Only entities within view bounds are rendered
- **Frame-by-frame Sync**: Backend tracker updates every frame for accuracy
- **Separate Layer**: Entities render after terrain and resources for correct z-ordering

## Configuration

### Polling Interval
To change the entity polling rate, modify `app.js`:
```javascript
this.entityManager.startPolling(1000); // milliseconds
```

### Entity Colors
To customize entity colors, edit `renderer.js`:
```javascript
const ENTITY_COLORS = {
    default: '#ff4444',
    wanderer: '#44ff44',
    animal: '#ffaa44',
    person: '#4444ff'
};
```

### Entity Size
Entity radius is calculated dynamically based on tile size:
```javascript
const ENTITY_RADIUS = Math.max(2, CONFIG.TILE_SIZE * 0.3);
```

## Future Enhancements

### Potential Improvements
1. **WebSocket Support**: Real-time updates instead of polling
2. **Entity Tooltips**: Show entity details on hover
3. **Entity Selection**: Click to select and view entity info
4. **Entity Trails**: Visual path history for wandering entities
5. **Entity Health/Status**: Color intensity based on health/hunger
6. **Entity Labels**: Optional name display for entities
7. **Entity Filtering**: Toggle visibility by entity type
8. **Entity Count by Type**: Breakdown of entity types in stats
9. **Entity Movement Animation**: Interpolate between positions for smooth movement
10. **Entity Sprites**: Use images/emojis instead of circles

### Known Limitations
1. **Polling Delay**: Up to 1 second lag in entity position updates
2. **No Interpolation**: Entities teleport between positions
3. **Limited Detail**: No health, inventory, or status information
4. **No Interaction**: Cannot select or command entities from viewer

## Testing

### Manual Testing Steps
1. **Start the Simulator**:
   ```bash
   cargo run --bin life-simulator
   ```

2. **Open Web Viewer**:
   - Navigate to `http://127.0.0.1:54321/viewer.html`

3. **Verify Entity Display**:
   - Look for colored circles on the map
   - Check entity count in sidebar stats
   - Verify entity legend is visible

4. **Test API Endpoint**:
   ```bash
   curl http://127.0.0.1:54321/api/entities
   ```

5. **Verify Movement**:
   - Watch entities move over time
   - Should update approximately every second
   - Movement should be smooth with no jumps

### API Testing
```bash
# Get all entities
curl -s http://127.0.0.1:54321/api/entities | jq

# Check entity count
curl -s http://127.0.0.1:54321/api/entities | jq '.entities | length'

# Get first entity
curl -s http://127.0.0.1:54321/api/entities | jq '.entities[0]'
```

## Troubleshooting

### Entities Not Visible
1. **Check API**: Verify `/api/entities` returns data
2. **Check Console**: Look for JavaScript errors
3. **Check Zoom**: Entities might be too small at low zoom
4. **Verify Initialization**: Ensure `EntityTracker::init()` is called

### Wrong Positions
1. **Coordinate System**: Ensure world ↔ screen conversion is correct
2. **Camera Offset**: Verify drag offset is applied properly
3. **Chunk Boundaries**: Check for off-by-one errors in tile calculations

### Performance Issues
1. **Reduce Polling Rate**: Increase interval to 2000ms or more
2. **Limit Entity Count**: Spawn fewer entities for testing
3. **Viewport Culling**: Verify only visible entities are rendered
4. **Browser DevTools**: Use Performance profiler to find bottlenecks

## Related Documentation
- [`WANDERING_ENTITIES.md`](WANDERING_ENTITIES.md) - Entity AI and behavior
- [`MOVEMENT_INTEGRATION.md`](MOVEMENT_INTEGRATION.md) - Movement system
- [`TICK_SYSTEM_ANALYSIS.md`](TICK_SYSTEM_ANALYSIS.md) - Simulation timing
- [`WEB_VIEWER_ARCHITECTURE.md`](WEB_VIEWER_ARCHITECTURE.md) - Web viewer design

## Credits
- Entity tracking system inspired by ECS patterns
- Polling approach based on simple HTTP server architecture
- Visual design follows game UI conventions
