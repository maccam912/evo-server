# Evolution Simulator

A real-time artificial life simulation powered by neural networks and genetic algorithms. Watch as simple creatures with randomly initialized brains evolve to find food, survive, and reproduce in a dynamic 2D world.

![Status](https://img.shields.io/badge/status-active-brightgreen)
![Language](https://img.shields.io/badge/language-Rust-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Features](#features)
- [How It Works](#how-it-works)
- [Documentation](#documentation)
- [Configuration](#configuration)
- [Development](#development)
- [Contributing](#contributing)

## Overview

This project simulates the evolution of artificial creatures that must learn to survive in a competitive, resource-limited environment. Each creature has:

- **A neural network brain** (16 inputs → 6 hidden neurons → 4 outputs) that controls its behavior
- **A genetic code** (150-byte genome) that encodes the neural network's weights
- **Dual resource systems**: Energy for actions and Health for survival
- **Combat capabilities**: Creatures can attack each other for territory and resources
- **Advanced sensors**: Detect nearby creatures, respond to attacks, monitor health
- **The ability to reproduce**, passing mutated copies of their genome to offspring

Over thousands of generations, natural selection shapes these creatures from random beginnings into complex agents capable of hunting, fleeing, healing, and strategic combat.

### Key Features

- **Combat system**: Creatures can attack, defend, and evolve combat strategies
- **Health and healing**: Separate health pool with energy-based regeneration
- **Food diversity**: Plant food (renewable) and meat food (from deceased creatures)
- **Advanced AI sensors**: 16 sensory inputs including threat detection and health awareness
- Real-time web-based visualization with energy-colored creatures
- Automatic checkpoint/resume system for long-running experiments
- Highly configurable simulation parameters (damage, healing, neural architecture)
- Built with Rust for maximum performance
- Docker support for easy deployment

## Quick Start

### Local Development

**Prerequisites**: Rust 1.70+ and Node.js (optional, for frontend development)

```bash
# Clone the repository
git clone <repository-url>
cd evo-server

# Build and run
cargo build --release
cargo run --release -- --config config.json

# Open browser to http://localhost:8080
```

### Docker (Production)

```bash
# Start the server
docker-compose up -d

# View logs
docker-compose logs -f

# Stop the server
docker-compose down
```

The simulation will:
1. Initialize 100 creatures with random genomes
2. Start the evolution process
3. Save checkpoints every hour
4. Stream real-time updates to the web UI at `http://localhost:8080`

## Features

Complete feature checklist organized in logical development order:

### Foundation ✅ Complete

1. ✅ **Grid-based world simulation** - 2D grid environment with configurable size
2. ✅ **Creature lifecycle system** - Birth, aging, and death mechanics
3. ✅ **Energy metabolism** - Tick-based energy consumption and starvation
4. ✅ **Food distribution system** - Cell-based food placement and regeneration
5. ✅ **Neural network brain** - 8-6-4 feedforward architecture with tanh activation
6. ✅ **Genetic encoding** - Neural weights derived from byte-array genome
7. ✅ **Mutation system** - Configurable mutation rate during reproduction

### Core Gameplay ✅ Complete

8. ✅ **Movement system** - 4-directional movement with boundary collision
9. ✅ **Food consumption** - Automatic eating when on food cell
10. ✅ **Asexual reproduction** - Energy-based reproduction with cooldown
11. ✅ **Population cap** - Configurable maximum population with random culling
12. ✅ **Death and removal** - Creatures die when energy reaches zero

### Visualization ✅ Mostly Complete

13. ✅ **WebSocket real-time updates** - 10 Hz state streaming to clients
14. ✅ **Canvas rendering** - Hardware-accelerated 2D visualization
15. ✅ **Energy-based color coding** - Red (low) → Yellow (medium) → Green (high)
16. ✅ **Creature selection** - Click to inspect individual creatures
17. ✅ **Pan and zoom controls** - Mouse wheel zoom, click-drag panning
18. 🚧 **Food grid visualization** - Food cells not rendered (only creatures visible)

### Statistics 🚧 Partially Complete

19. ✅ **Population metrics** - Real-time population count display
20. ✅ **Energy tracking** - Total and average energy monitoring
21. ✅ **Generation tracking** - Maximum generation reached display
22. 🚧 **Birth/death counters** - UI exists but server doesn't track events
23. 🚧 **Average age calculation** - UI exists but server doesn't send data

### Inspector 🚧 Partially Complete

24. ✅ **Basic creature info** - ID, position, energy, generation displayed
25. 🚧 **Genome visualization** - UI exists but genome data not transmitted
26. 🚧 **Neural network display** - UI exists but brain weights not transmitted

### Persistence ✅ Complete

27. ✅ **Checkpoint system** - Automatic hourly saves with state restoration
28. ✅ **Configuration system** - JSON-based config with command-line overrides
29. ✅ **Resume from checkpoint** - Automatic recovery from latest checkpoint

### Infrastructure ✅ Complete

30. ✅ **Docker containerization** - Multi-stage optimized Docker build
31. ✅ **Docker Compose setup** - One-command deployment with volumes

### Optimization 🚧 Planned

32. 🚧 **Regional updates** - Viewport-based partial world streaming (protocol exists, not implemented)
33. 🚧 **Spatial partitioning** - Efficient creature lookup for nearby queries
34. 🚧 **Parallel tick processing** - Multi-threaded simulation with Rayon

### Advanced Evolution 🚧 Planned

35. 🚧 **Additional neural inputs** - 3 unused input slots available for new sensors
36. 🚧 **Speciation tracking** - Genome similarity clustering (similarity function exists)
37. 🚧 **Sexual reproduction** - Two-parent genetic crossover
38. 🚧 **Predator-prey dynamics** - Multiple creature types with interactions

### Analysis Tools 🚧 Planned

39. 🚧 **Lineage tree visualization** - Evolutionary tree from reproduction history
40. 🚧 **Fitness graphs** - Generation-over-time performance metrics
41. 🚧 **Checkpoint replay** - Playback system for saved simulations
42. 🚧 **Data export** - CSV/JSON export for external analysis

**Legend**: ✅ Implemented | 🚧 Planned/Partial

## How It Works

### The Simulation Loop

Every tick (30 times per second by default):

1. **Food regenerates** - Random cells gain food based on regeneration rate
2. **Creatures think** - Each creature's neural network processes sensor inputs
3. **Actions execute** - Creatures move and eat based on neural network decisions
4. **Energy depletes** - All creatures lose energy from metabolism and movement
5. **Reproduction** - Eligible creatures create mutated offspring
6. **Death** - Creatures with zero energy are removed

### Neural Evolution

Creatures start with **completely random brains** - their initial behavior is essentially noise. However:

- Creatures that **accidentally find food** survive longer
- Creatures that **survive longer** have more chances to reproduce
- **Offspring inherit** their parent's genome with small random mutations
- Over generations, **food-finding behaviors** become more common in the population

The result: After thousands of generations, you'll observe creatures actively seeking food, avoiding empty areas, and efficiently managing their energy.

### Energy Economy

The simulation is driven by an energy-based economy:

| Event | Energy Change |
|-------|--------------|
| Per tick | -0.1 |
| Movement | -1.0 |
| Eating food | +20.0 |
| Reproduction | -50.0 |

Creatures must balance exploration (costly movement) with exploitation (staying near food sources).

## Documentation

Detailed documentation for understanding and extending the simulation:

- **[Simulation Mechanics](docs/SIMULATION.md)** - How the world works, tick cycle, energy system
- **[Neural Networks](docs/NEURAL_NETWORKS.md)** - Brain architecture, sensors, evolution of behavior
- **[UI Guide](docs/UI_GUIDE.md)** - Color coding, controls, statistics panel, inspector
- **[Configuration](docs/CONFIGURATION.md)** - All config options, tuning guide, parameters
- **[Architecture](docs/ARCHITECTURE.md)** - Technical design, protocols, code structure

## Configuration

The simulation is highly configurable via `config.json`:

```json
{
  "world": {
    "width": 100,
    "height": 100,
    "initial_food_density": 0.3,
    "food_regen_rate": 0.001
  },
  "creature": {
    "initial_population": 100,
    "max_energy": 200.0,
    "energy_per_food": 20.0
  },
  "evolution": {
    "mutation_rate": 0.01,
    "genome_size": 100
  }
}
```

See [Configuration Guide](docs/CONFIGURATION.md) for complete reference.

### Command-Line Options

```bash
# Use custom config file
cargo run -- --config my_config.json

# Disable checkpointing (for testing)
cargo run -- --no-checkpoint

# Run headless without web server
cargo run -- --no-server
```

## Development

### Project Structure

```
evo-server/
├── src/
│   ├── main.rs              # Entry point, CLI args
│   ├── config.rs            # Configuration system
│   ├── simulation/
│   │   ├── mod.rs           # Simulation state
│   │   ├── creature.rs      # Creature logic
│   │   ├── world.rs         # World grid
│   │   └── brain.rs         # Neural network
│   ├── server/
│   │   ├── mod.rs           # Axum web server
│   │   └── websocket.rs     # WebSocket handler
│   └── checkpoint.rs        # Save/load system
├── static/
│   ├── index.html           # Web UI
│   ├── app.js               # Canvas renderer
│   └── style.css            # UI styles
└── config.json              # Default configuration
```

### Building from Source

```bash
# Development build (faster compile, slower runtime)
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

### Testing Changes

```bash
# Quick iteration
cargo run -- --config config.json

# Watch mode (requires cargo-watch)
cargo watch -x run

# Performance profiling
cargo build --release
perf record ./target/release/evo-server
```

## Contributing

Contributions are welcome! Priority areas:

1. **Complete existing features** - Finish food visualization, birth/death tracking
2. **Performance optimization** - Spatial partitioning, parallel processing
3. **New neural inputs** - Add directional food gradient, energy trend sensors
4. **Analysis tools** - Fitness graphs, lineage trees, data export

### Development Workflow

1. Check the [Features](#features) list for 🚧 planned items
2. Review relevant documentation in `docs/`
3. Create a feature branch
4. Implement changes with tests
5. Update documentation
6. Submit pull request

## License

MIT License - See LICENSE file for details

## Acknowledgments

Inspired by:
- [Evolution Simulator](https://www.youtube.com/watch?v=N3tRFayqVtk) by Primer
- [Boxcar2D](http://boxcar2d.com/) genetic algorithm project
- [MarI/O](https://www.youtube.com/watch?v=qv6UVOQ0F44) neural network evolution

---

**Watch evolution in action**: Start the server and observe as random noise transforms into intelligent behavior over hundreds of generations. The best creatures you see today are the descendants of lucky accidents from earlier generations.
