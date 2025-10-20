# Life Simulator 🦌🐇🦝🦁🐺

Life Simulator is a rich ecosystem simulation where herbivores and predators live, hunt, and evolve on procedurally generated islands. The core sim runs in Bevy 0.16 without graphics; you peek inside through a lightweight web viewer or by calling the HTTP API.

## ✨ What makes this special

- **Dynamic Predator-Prey Ecosystems** – Wolves hunt rabbits, foxes patrol territories, while bears dominate the landscape. Herbivores exhibit realistic fear responses including fleeing, reduced feeding, and altered grazing patterns.
- **Advanced Plant System** – Real-time vegetation growth with ResourceGrid technology, biomass density visualization, and ecosystem-wide nutrient cycling.
- **Complex Animal Behaviors** – Animals form pairs, carry pregnancies, birth litters, exhibit fear responses, and adapt their behavior based on predator presence and resource availability.
- **Modular Species Architecture** – Each species (rabbits, deer, raccoons, wolves, foxes, bears) has its own module with configurable behaviors, reproduction strategies, and AI patterns.
- **Living Worlds** – Vegetation grows, gets consumed, and regenerates based on grazing pressure. Carcasses from predators enrich the soil, creating realistic ecosystem dynamics.
- **Real-time Web Visualization** – Watch the ecosystem evolve through an interactive web viewer with biomass overlays, entity tracking, and live statistics.

## Tour of the experience

| Feature | What you'll see |
| --- | --- |
| 🗺️ Living Islands | Sand-lined beaches leading into forests, mountains, and ponds, with dynamic vegetation that responds to grazing pressure. |
| 🐇 Rabbits | Fast-reproducing herbivores that graze aggressively and exhibit realistic fear responses when predators are nearby. |
| 🦌 Deer | Majestic creatures whose fawns trail their mothers until adulthood, with cautious grazing behaviors. |
| 🦝 Raccoons | Opportunistic foragers that balance thirst and hunger while staying alert to danger. |
| 🐺 Wolves | Pack hunters that patrol territories, mark scents, and create coordinated hunting strategies. |
| 🦊 Foxes | Cunning solitary hunters that use scent tracking and territory patrol techniques. |
| 🐻 Bears | Powerful apex predators that dominate territories and create fear responses across the ecosystem. |
| 🌿 Biomass Visualization | Real-time grass density overlay showing vegetation health and grazing patterns. |
| 🌐 Advanced Web Viewer | Interactive HTML/JS viewer with entity tracking, fear response visualization, and ecosystem statistics. |

## 🚀 Quick Start

Get the ecosystem simulation running in 5 minutes.

**📖 Detailed Guide**: See [Quick Start Guide](docs/QUICK_START.md) for complete setup instructions.

```bash
# 1. Clone & setup
git clone <repository-url>
cd life-simulator

# 2. Generate island (creates maps/generated_world.ron)
cargo run --bin map_generator
# Optional: cargo run --bin map_generator -- --name spring_isle --seed 123456

# 3. Run simulation
cargo run --bin life-simulator

# 4. Open viewer at http://127.0.0.1:54321/viewer.html
```

**What you'll see**: Living islands with rabbits, deer, wolves, foxes, bears, and raccoons exhibiting realistic predator-prey dynamics and fear responses.

## 🏗️ Architecture

Built on **Bevy 0.16 ECS** with modular, performance-optimized systems:

- **ResourceGrid Vegetation** - High-performance sparse vegetation storage with LOD
- **Fear & Predator System** - Spatial predator detection and behavioral responses
- **Event-Driven AI** - Trigger-based decision making with utility scoring
- **Modular Species Architecture** - Each species has configurable behaviors and reproduction

**📖 Technical Details**: See [Development Guide](docs/DEVELOPMENT_GUIDE.md) for complete architecture overview.

## 🌐 Visualization

Two viewer options for monitoring the ecosystem:

- **Web Viewer** - Interactive HTML/JS viewer with real-time entity tracking and biomass overlays
- **Godot Viewer** - 3D viewer with OpenRCT2 sprite integration

**📖 API Reference**: See [API Documentation](docs/API_REFERENCE.md) for all endpoints and usage examples.

## 📚 Documentation

**For Development**:
- [Quick Start Guide](docs/QUICK_START.md) - Get running in 5 minutes
- [Development Guide](docs/DEVELOPMENT_GUIDE.md) - Complete development setup and architecture
- [API Reference](docs/API_REFERENCE.md) - HTTP API endpoints and usage

**For Features**:
- [Species Reference](docs/SPECIES_REFERENCE.md) - Complete species catalog with behaviors and stats
- [Plant System](docs/PLANT_SYSTEM_PARAMS.md) - Vegetation, ResourceGrid, and ecosystem configuration
- [AI Architecture](docs/EVENT_DRIVEN_PLANNER_IMPLEMENTATION.md) - AI planning and fear systems

**For Deep Dives**:
- [Technical Overview](docs/TECH_OVERVIEW.md) - System architecture and technical details
- [Movement System](docs/ENTITY_MOVEMENT_EXPLAINED.md) - Entity movement and pathfinding
- [Tick System](docs/TICK_SYSTEM_QUICKSTART.md) - Simulation timing and update cycles

## 🧪 Testing & Performance

- **Comprehensive test suite** covering plant system, predator-prey dynamics, and AI behaviors
- **Performance benchmarks** for vegetation system validation
- **Integration tests** for fear mechanics and ecosystem interactions

Run tests with `cargo test`. See [Development Guide](docs/DEVELOPMENT_GUIDE.md) for testing guidelines.

## 🔧 Extending

Want to add new species, behaviors, or ecosystem features?

**🦁 Adding Species**: Copy existing modules in `src/entities/types/`, implement behaviors, configure stats, and register in the species registry.

**🌿 Plant System**: Extend ResourceGrid with new resource types, adjust growth parameters, and add vegetation interactions.

**🧠 AI Features**: Add new trigger types, create behavior trees, extend fear systems, and optimize performance.

**📖 Complete Guide**: See [Development Guide](docs/DEVELOPMENT_GUIDE.md) for step-by-step instructions.

## 🤝 Contributing

1. Fork repository and create feature branch
2. Read [Development Guide](docs/DEVELOPMENT_GUIDE.md) for architecture understanding
3. Run `cargo test` to ensure tests pass
4. Add tests for new features
5. Update documentation
6. Submit pull request with clear description

## 📦 Dependencies

- `bevy` 0.16 - ECS game engine
- `rand` 0.8 - Deterministic random generation
- `serde` & `ron` - Data serialization
- `tokio` - Async web server runtime

## 📄 License

Dual-licensed under MIT License ([LICENSE-MIT](LICENSE-MIT)) or Apache License 2.0 ([LICENSE-APACHE](LICENSE-APACHE)).

## 🔗 References

- [Bevy Game Engine](https://bevyengine.org/) - ECS engine powering this project
- `/Users/jean/Github/world-simulator` - Terrain generation inspiration
- `/Users/jean/Github/dogoap` - AI behavior tree reference
- `/Users/jean/Github/big-brain` - AI planning reference

---

**Ready to explore living ecosystems?** Start with the [Quick Start Guide](docs/QUICK_START.md) and watch predator-prey dynamics unfold in real-time! 🐾
