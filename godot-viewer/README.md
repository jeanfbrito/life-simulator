# Godot Viewer

Godot client for isometric viewing of the Life Simulator world.

## Setup

This project uses Godot 4.4.1.stable.official.49a5bc7b6

### Running

```bash
# Run from command line
"/Applications/Godot.app/Contents/MacOS/Godot" --path godot-viewer

# Headless mode for testing
"/Applications/Godot.app/Contents/MacOS/Godot" --headless --path godot-viewer
```

## Project Structure

- `scenes/` - Scene files (.tscn)
- `scripts/` - GDScript files (.gd)
- `resources/` - Resource files (.tres, .import)
- `addons/` - Godot addons/plugins

## Development

The project connects to the Rust backend running on port 54321 to fetch world data and render it isometrically.