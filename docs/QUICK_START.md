# Life Simulator Quick Start Guide

Get the ecosystem simulation running in 5 minutes.

## Prerequisites

- Rust 1.70+ installed
- Modern web browser for the viewer

## Step 1: Clone & Setup

```bash
git clone <repository-url>
cd life-simulator
```

## Step 2: Generate an Island

```bash
# Generate default island
cargo run --bin map_generator

# Or customize it
cargo run --bin map_generator -- --name spring_isle --seed 123456 --radius 12
```

This creates `maps/generated_world.ron` with a procedurally generated island.

## Step 3: Run the Simulation

```bash
cargo run --bin life-simulator
```

The server starts on `http://127.0.0.1:54321` with:
- Real-time ecosystem simulation
- HTTP API for data access
- Web viewer interface

## Step 4: Open the Web Viewer

Visit `http://127.0.0.1:54321/viewer.html`

**Viewer Controls:**
- **Drag**: Pan around the map
- **Scroll**: Zoom in/out
- **Click entities**: Inspect stats and behaviors
- **Toggle `CONFIG.showGrassDensity`**: See biomass overlay
- **Tab key**: Toggle statistics HUD

## What You'll See

- 🗺️ **Living Islands**: Beaches, forests, mountains, ponds with dynamic vegetation
- 🐇 **Rabbits**: Fast-reproducing herbivores with fear responses
- 🦌 **Deer**: Cautious grazers with fawns following mothers
- 🦝 **Raccoons**: Opportunistic foragers balancing hunger/thirst
- 🐺 **Wolves**: Pack hunters with scent marking and territory patrols
- 🦊 **Foxes**: Solitary hunters using scent tracking
- 🐻 **Bears**: Apex predators creating ecosystem-wide fear responses
- 🌿 **Biomass Visualization**: Real-time vegetation density showing grazing patterns

## Customization

Edit `config/spawn_config.ron` to:
- Adjust species populations
- Modify entity names and behaviors
- Change spawn locations and patterns

## Next Steps

- [Species Reference](SPECIES_REFERENCE.md) – Complete species catalog
- [Plant System](PLANT_SYSTEM_PARAMS.md) – Vegetation configuration
- [API Reference](../README.md#api-endpoints) – HTTP API documentation
- [Technical Overview](TECH_OVERVIEW.md) – Development guide

Happy simming! 🐾
