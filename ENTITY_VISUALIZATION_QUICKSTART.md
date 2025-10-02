# Entity Visualization - Quick Start Guide

## What Was Added

The web viewer now displays wandering entities (creatures, people, etc.) as **colored circles** on the map that update in real-time!

### Features
- ✅ **Real-time Entity Display**: Entities appear as colored circles on the map
- ✅ **Entity Count**: Shows total entity count in sidebar statistics
- ✅ **Color-coded by Type**: Different colors for different entity types
- ✅ **Auto-refresh**: Updates every second
- ✅ **Viewport Culling**: Only renders visible entities for performance
- ✅ **Legend**: Visual guide showing entity types and colors

## How to Use

### 1. Start the Simulator
```bash
cargo run --bin life-simulator
```

### 2. Open the Web Viewer
Navigate to: **http://127.0.0.1:54321/viewer.html**

### 3. View the Entities
- Look for **colored circles** on the map (usually green for wanderers)
- Check the **"Entities"** stat card in the sidebar (shows count)
- Refer to the **"Entity Legend"** below the terrain legend

### 4. Force Refresh (if needed)
If you don't see entities:
1. Hard refresh the page: `Ctrl+Shift+R` (Windows/Linux) or `Cmd+Shift+R` (Mac)
2. Or clear cache and reload

## Entity Colors

| Color | Type | Emoji |
|-------|------|-------|
| 🟢 Green | Wanderer/Human | 🚶 |
| 🟠 Orange | Animal | 🐾 |
| 🔵 Blue | Person | 👤 |
| 🔴 Red | Other/Default | ❓ |

## Files Changed/Created

### Backend (Rust)
- `src/entities/entity_tracker.rs` - Updated JSON format to include `position` object

### Frontend (JavaScript)
- **NEW**: `web-viewer/js/entity-manager.js` - Manages entity data fetching
- `web-viewer/js/renderer.js` - Added `renderEntities()` method
- `web-viewer/js/app.js` - Integrated entity manager
- `web-viewer/viewer.html` - Added entity count and legend

## Testing the API

### Get All Entities
```bash
curl http://127.0.0.1:54321/api/entities
```

### Example Response
```json
{
  "entities": [
    {
      "id": 0,
      "name": "Wanderer_0",
      "entity_type": "Human",
      "position": {"x": 1, "y": -14}
    },
    {
      "id": 1,
      "name": "Wanderer_1",
      "entity_type": "Human",
      "position": {"x": 16, "y": 6}
    }
  ]
}
```

### Count Entities
```bash
curl -s http://127.0.0.1:54321/api/entities | jq '.entities | length'
```

## Troubleshooting

### Problem: Entities not visible on map

**Solution 1**: Hard refresh the browser
- Windows/Linux: `Ctrl + Shift + R`
- Mac: `Cmd + Shift + R`

**Solution 2**: Check the API
```bash
curl http://127.0.0.1:54321/api/entities
```
Should return JSON with entities array.

**Solution 3**: Check browser console
- Open Developer Tools (F12)
- Look for JavaScript errors in Console tab
- Look for network requests in Network tab

### Problem: Entity count shows 0

**Verify entities exist**:
```bash
curl -s http://127.0.0.1:54321/api/entities | jq '.entities | length'
```

If it returns 0, entities haven't been spawned yet. They spawn during world initialization.

### Problem: "undefined/api/entities" in logs

This means browser cache is serving old JavaScript files.
- Clear browser cache completely
- Hard refresh (Ctrl+Shift+R or Cmd+Shift+R)
- Try incognito/private browsing mode

## How It Works

### Data Flow
```
Bevy Simulation (Rust)
    ↓ (every frame)
Entity Tracker (Global State)
    ↓ (HTTP GET request)
Web Server (/api/entities)
    ↓ (JSON response)
Entity Manager (JavaScript)
    ↓ (every 1 second)
Renderer (Canvas)
    ↓ (60 FPS)
Your Screen!
```

### Performance Notes
- **Polling Interval**: 1000ms (1 second) - configurable
- **Rendering**: Only visible entities are drawn
- **Update Rate**: Entities update their positions at simulation tick rate (10 TPS)
- **Display Rate**: Screen refreshes at 60 FPS

## Configuration

### Change Polling Rate
Edit `web-viewer/js/app.js`:
```javascript
this.entityManager.startPolling(2000); // 2 seconds instead of 1
```

### Change Entity Colors
Edit `web-viewer/js/renderer.js`:
```javascript
const ENTITY_COLORS = {
    default: '#ff4444',      // red
    wanderer: '#44ff44',     // green
    animal: '#ffaa44',       // orange
    person: '#4444ff'        // blue
};
```

### Change Entity Size
Edit `web-viewer/js/renderer.js`:
```javascript
const ENTITY_RADIUS = Math.max(2, CONFIG.TILE_SIZE * 0.5); // Bigger entities
```

## Next Steps

Consider implementing:
1. **Entity Tooltips**: Hover to see entity name and stats
2. **Entity Selection**: Click to select and view details
3. **Movement Trails**: Show path history
4. **Entity Health Bars**: Visual health indicators
5. **Entity Labels**: Optional name display
6. **WebSocket Updates**: Real-time instead of polling

## More Information

See detailed documentation:
- [`docs/ENTITY_VISUALIZATION.md`](docs/ENTITY_VISUALIZATION.md) - Full technical documentation
- [`docs/WANDERING_ENTITIES.md`](docs/WANDERING_ENTITIES.md) - Entity AI system
- [`docs/MOVEMENT_INTEGRATION.md`](docs/MOVEMENT_INTEGRATION.md) - Movement system

## Quick Reference

| Component | Purpose |
|-----------|---------|
| `EntityTracker` | Backend: Tracks all entities globally |
| `EntityManager` | Frontend: Fetches entity data from API |
| `renderEntities()` | Frontend: Draws entities on canvas |
| `/api/entities` | API: Returns entity positions as JSON |

---

**Happy Simulating!** 🎮🗺️🎯
