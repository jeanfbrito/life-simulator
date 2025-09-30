# Life Simulator

A life simulation game built with Bevy 0.16, drawing inspiration from advanced AI systems and simulation engines.

## Overview

This project demonstrates various game development and simulation concepts using the Bevy game engine. It incorporates ideas from:

- **[Big Brain](https://github.com/zkat/big-brain)** - Utility AI library for Bevy
- **[World Simulator](https://github.com/jeanfbrito/world-simulator)** - Medieval economy simulation engine
- **[DOGOAP](https://github.com/victorb/dogoap)** - Referenced for GOAP implementation examples

## Features

- **Bevy 0.16**: Built with the latest version of the Bevy game engine
- **AI Systems**: Integration examples for Utility AI with GOAP demonstration
- **ECS Architecture**: Entity-Component-System based design
- **Performance Optimized**: Configured for both development and release builds

## Project Structure

```
life-simulator/
├── src/
│   ├── main.rs              # Main application entry point
│   ├── lib.rs               # Library exports
│   ├── ai/                  # AI-related systems and components
│   ├── simulation/          # Core simulation logic
│   └── components/          # Bevy components and resources
├── Cargo.toml               # Project configuration
└── README.md               # This file
```

## Quick Start

### Prerequisites

- Rust 1.70+ (recommended to use [rustup](https://rustup.rs/))
- Git

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd life-simulator
```

2. Run the project:
```bash
cargo run
```

3. For development with faster compilation:
```bash
cargo run --features dynamic_linking
```


## Development

### Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

### Code Style

This project follows standard Rust formatting:
```bash
cargo fmt
cargo clippy
```

## Architecture

The project is organized around Bevy's ECS architecture:

- **Components**: Data attached to entities
- **Systems**: Logic that operates on components
- **Resources**: Global data and configuration
- **Plugins**: Bundled systems and components

### AI Integration

The project will include AI approaches:

1. **Utility AI**
   - Score-based decision making
   - Inspired by the Big Brain library

2. **GOAP (Goal-Oriented Action Planning)**
   - Planning-based AI for complex, multi-step behaviors
   - Demonstrated as an example system

## Dependencies

- `bevy` 0.16 - Main game engine
- `rand` 0.8 - Random number generation

## Performance Configuration

The project includes optimized build configurations:

- Development builds balance compilation speed with performance
- Release builds use LTO (Link Time Optimization) for maximum performance
- Dynamic linking is available for faster development iteration

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

## References and Inspiration

- [Bevy Game Engine](https://bevyengine.org/) - The game engine powering this project
- [DOGOAP](https://github.com/victorb/dogoap) - Data-Oriented GOAP library
- [Big Brain](https://github.com/zkat/big-brain) - Utility AI for Bevy
- [World Simulator](https://github.com/jeanfbrito/world-simulator) - Medieval economy simulation

## Documentation

For detailed documentation on specific topics:

- Check inline documentation in the source code
- Refer to Bevy's official documentation for engine-specific questions