# Loading OpenRCT2 Park Files (.park)

## Current Status

The Godot viewer currently supports:
- ✅ `.ron` files (custom format) - loads directly from `maps/` directory
- ❌ `.park` files (OpenRCT2 save format) - not yet supported
- ❌ `.sv6` files (RollerCoaster Tycoon 2 format) - not yet supported

## How to Load a .park File

### Option 1: Convert to .sv6 using OpenRCT2 (Recommended)

1. Open OpenRCT2
2. Load the park file: `File → Load Game → good-generated-map.park`
3. Save as `.sv6` format: `File → Save Game As → Classic (.sv6)`
4. Place the `.sv6` file in the backend's expected location
5. Update the backend configuration to load this map

### Option 2: View in OpenRCT2 Directly

Since the Godot viewer aims to replicate OpenRCT2's rendering:
1. Open `good-generated-map.park` in OpenRCT2 directly
2. Compare the rendering with Godot viewer's output
3. Use existing maps in Godot to verify edge rendering works

### Option 3: Test with Current Map

The Godot viewer is already working and rendering edge faces. To see it in action:
1. The viewer is currently running with map data from the backend
2. Edge faces are rendering successfully (see terminal output: "🏔️ Painted edge faces")
3. All 4 terrain edge types are available (rock, wood_black, wood_red, ice)

## Future Development

To add `.park` file support:

### Approach A: Rust Backend Integration
Add OpenRCT2 park file parsing to the Rust backend:
- Use OpenRCT2's park file format specification
- Parse `.park` files directly in Rust
- Convert to internal RON format

### Approach B: OpenRCT2 CLI Integration
Use OpenRCT2's command-line interface:
```bash
/Applications/OpenRCT2.app/Contents/MacOS/OpenRCT2 \
  --headless \
  export \
  good-generated-map.park \
  output.sv6
```

### Approach C: Direct JSON Export
OpenRCT2 can export park data as JSON:
1. Use OpenRCT2's plugin API to export park data
2. Convert JSON to RON format
3. Load in Godot viewer

## Current Workaround

The Godot viewer is fully functional with the current backend-generated maps. The edge rendering system is complete and working:
- ✅ All terrain edge sprites loaded (336 total)
- ✅ Terrain-based edge selection working
- ✅ Height-based edge rendering working
- ✅ Proper isometric positioning

To see the edge rendering in action, the viewer is currently running and successfully rendering chunks with edge faces!

