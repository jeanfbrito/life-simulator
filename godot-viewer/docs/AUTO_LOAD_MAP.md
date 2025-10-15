# Auto-Load Latest Map Feature

## Overview

The Godot viewer now automatically loads the most recently modified map file from the `maps/` directory when starting up or when pressing the refresh button.

## How It Works

1. **Automatic Detection**: On startup, the system scans the `maps/` directory for `.ron` files
2. **Latest File Selection**: It selects the file with the most recent modification time
3. **Auto-Loading**: The selected map is loaded automatically without requiring manual configuration
4. **Fallback**: If no map files are found, it falls back to the default backend API loading

## Usage

### Startup
- Simply run the simulation - it will automatically load the latest map
- The TopBar will show "🗺️ Latest Map" to indicate auto-loading is active

### Manual Refresh
- Press **R** key to reload the latest map
- Click the refresh button in the TopBar
- Use the menu option "Reload World"

### Adding New Maps
1. Create or copy a new `.ron` map file to the `maps/` directory
2. Restart the simulation or press refresh
3. The new map will be automatically detected and loaded

## Map File Format

Maps should be stored in RON format in the `maps/` directory with a `.ron` extension. Example:

```ron
(
    name: "My Map",
    description: "Map description",
    width: 50,
    height: 50,
    terrain: {
        default: "Grass",
        features: [...]
    },
    resources: {
        trees: [...],
        rocks: [...]
    }
)
```

## Implementation Details

- **ChunkManager.get_latest_map_file()**: Scans for the most recent `.ron` file
- **ChunkManager.load_specific_map()**: Loads and parses a specific map file
- **WorldRenderer.reload_latest_map()**: Public method to trigger reload
- **Keyboard Shortcut**: Press **R** to reload the latest map at any time

## Testing

The system includes a test map file `test_autoload_map.ron` that demonstrates the auto-loading functionality.