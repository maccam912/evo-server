# Configuration Guide

This document provides a complete reference for all configuration options in the evolution simulator, including parameter explanations, tuning guides, and example configurations.

## Table of Contents

- [Configuration File](#configuration-file)
- [Command-Line Arguments](#command-line-arguments)
- [World Configuration](#world-configuration)
- [Creature Configuration](#creature-configuration)
- [Evolution Configuration](#evolution-configuration)
- [Combat Configuration](#combat-configuration)
- [Simulation Configuration](#simulation-configuration)
- [Checkpoint Configuration](#checkpoint-configuration)
- [Server Configuration](#server-configuration)
- [Tuning Guide](#tuning-guide)
- [Example Configurations](#example-configurations)

## Configuration File

### Location

By default, the simulator looks for `config.json` in the working directory.

**Custom location**:
```bash
cargo run -- --config /path/to/my_config.json
```

### Format

The configuration file is JSON with the following top-level structure:

```json
{
  "world": { ... },
  "creature": { ... },
  "evolution": { ... },
  "simulation": { ... },
  "checkpoint": { ... },
  "server": { ... }
}
```

### Auto-Generation

If the config file doesn't exist, the simulator will create one with default values on first run.

## Command-Line Arguments

### Available Options

```bash
cargo run -- [OPTIONS]
```

| Flag | Description | Default |
|------|-------------|---------|
| `--config <PATH>` | Path to configuration file | `config.json` |
| `--no-checkpoint` | Disable checkpoint saving/loading | Enabled |
| `--no-server` | Run headless without web server | Enabled |

### Examples

**Use custom config**:
```bash
cargo run -- --config experiments/harsh_world.json
```

**Testing without checkpoints**:
```bash
cargo run -- --no-checkpoint
```

**Run simulation without web UI** (for data collection):
```bash
cargo run -- --no-server
```

**Combine options**:
```bash
cargo run -- --config test.json --no-checkpoint --no-server
```

## World Configuration

Controls the environment and food system.

### Complete Structure

```json
"world": {
  "width": 100,
  "height": 100,
  "initial_food_density": 0.3,
  "food_regen_rate": 0.001,
  "max_food_per_cell": 10,
  "plant_decay_ticks": 600,
  "meat_decay_ticks": 300
}
```

### Parameters

#### `width`

**Type**: Integer
**Default**: 100
**Range**: 10-1000 (recommended)

**Description**: Width of the world grid in cells.

**Effects**:
- Larger values = more space for creatures
- Affects total food capacity: `total_food_capacity = width × height × max_food_per_cell`
- Performance impact: Rendering scales with visible cells, not total size

**Examples**:
- `50`: Small world (2,500 cells) - cramped, high competition
- `100`: Default world (10,000 cells) - balanced
- `200`: Large world (40,000 cells) - room for spatial patterns

#### `height`

**Type**: Integer
**Default**: 100
**Range**: 10-1000 (recommended)

**Description**: Height of the world grid in cells.

**Note**: Can differ from width for rectangular worlds (e.g., 200×50 for "corridor" experiments).

#### `initial_food_density`

**Type**: Float
**Default**: 0.3
**Range**: 0.0-1.0

**Description**: Fraction of cells that start with 1 food unit.

**Examples**:
- `0.1`: Sparse (1,000 food units in 100×100 world) - hard start
- `0.3`: Default (3,000 food units) - moderate
- `0.5`: Abundant (5,000 food units) - easy start

**Effect on simulation**:
- Higher: Population grows faster initially
- Lower: Strong early selection pressure

#### `food_regen_rate`

**Type**: Float
**Default**: 0.001
**Range**: 0.0-1.0

**Description**: Probability per tick that each cell gains 1 food unit.

**Expected food generation per tick**:
```
food_per_tick = width × height × food_regen_rate
```

For default settings: `100 × 100 × 0.001 = 10 food units/tick`

**Examples**:
- `0.0005`: Harsh (5 food/tick in 100×100 world) - scarcity
- `0.001`: Default (10 food/tick) - balanced
- `0.002`: Abundant (20 food/tick) - less competition

**Carrying capacity estimation**:
```
max_population ≈ (width × height × food_regen_rate × energy_per_food) / energy_cost_per_tick
```

#### `max_food_per_cell`

**Type**: Integer
**Default**: 10
**Range**: 1-100 (recommended)

**Description**: Maximum food units that can accumulate in a single cell.

**Effects**:
- Prevents infinite food accumulation in unexplored areas
- Creates food "hotspots" where food concentrates
- Affects maximum stored energy in environment

**Examples**:
- `1`: No accumulation - cells have 0 or 1 food only
- `10`: Default - moderate accumulation
- `50`: High accumulation - rich "food patches"

#### `plant_decay_ticks`

**Type**: Integer
**Default**: 600
**Range**: 1-10000

**Description**: Number of ticks before plant food decays and disappears.

**Time at 30 TPS**:
- `300`: 10 seconds
- `600`: 20 seconds (default)
- `1800`: 60 seconds (1 minute)
- `0` or very low: Nearly instant decay (harsh environment)

**Effects**:
- Prevents infinite food accumulation in unexplored areas
- Creates dynamic food distribution
- Forces creatures to continually search for fresh food
- Balances with `food_regen_rate` to determine equilibrium food availability

**Tuning**:
- Higher: More stable food supply, easier survival
- Lower: Forces constant movement, higher selection pressure
- Should generally be higher than `meat_decay_ticks` (plant lasts longer)

#### `meat_decay_ticks`

**Type**: Integer
**Default**: 300
**Range**: 1-10000

**Description**: Number of ticks before meat food (from dead creatures) decays and disappears.

**Time at 30 TPS**:
- `150`: 5 seconds
- `300`: 10 seconds (default)
- `600`: 20 seconds
- `900`: 30 seconds

**Effects**:
- Creates urgency around scavenging dead creatures
- Prevents meat from accumulating indefinitely
- Faster decay than plant food encourages opportunistic feeding
- Rewards creatures that can quickly locate and consume fresh kills

**Strategic implications**:
- Lower values favor aggressive hunting (immediate consumption)
- Higher values allow more time for scavengers to find meat
- Default (300) is half of plant decay time (600), making meat a time-limited resource

**Relationship to combat**:
```
# Meat dropped per kill:
meat_amount = floor(victim_energy / 20)

# Time window to scavenge:
scavenge_window = meat_decay_ticks / ticks_per_second

# Default: 300 / 30 = 10 seconds to find and eat meat
```

## Creature Configuration

Controls creature properties and metabolism.

### Complete Structure

```json
"creature": {
  "initial_population": 100,
  "max_population": 1000,
  "initial_energy": 100.0,
  "max_energy": 200.0,
  "energy_per_food": 20.0,
  "energy_cost_per_tick": 0.1,
  "energy_cost_move": 1.0,
  "energy_cost_reproduce": 50.0,
  "min_reproduce_energy": 100.0,
  "reproduce_cooldown_ticks": 100
}
```

### Parameters

#### `initial_population`

**Type**: Integer
**Default**: 100
**Range**: 1-10000

**Description**: Number of creatures spawned at simulation start.

**Trade-offs**:
- Higher: Faster evolution (more mutations per generation)
- Lower: Watch recovery from near-extinction events
- Very low (1-10): High extinction risk early on

#### `max_population`

**Type**: Integer
**Default**: 1000
**Range**: 1-100000

**Description**: Maximum allowed creatures. Excess randomly culled.

**Purpose**:
- Performance limit (rendering and simulation)
- Competition enforcement
- Prevents exponential growth crashes

**Recommendation**: Set to 5-10× initial population for headroom.

#### `initial_energy`

**Type**: Float
**Default**: 100.0
**Range**: 0.1-1000.0

**Description**: Energy each creature starts with (generation 0 and offspring).

**Balance considerations**:
- Should allow 100-200 ticks of survival without food
- Must be ≥ `min_reproduce_energy` for reproduction to be possible
- Default 100.0 allows ~1000 ticks survival (33 seconds at 30 TPS)

#### `max_energy`

**Type**: Float
**Default**: 200.0
**Range**: 1.0-10000.0

**Description**: Energy cap - creatures cannot exceed this value.

**Effects**:
- Limits energy storage capacity
- Affects input neuron 0 normalization: `energy / max_energy`
- Typical: 2× initial energy for reproduction buffer

#### `energy_per_food`

**Type**: Float
**Default**: 20.0
**Range**: 0.1-1000.0

**Description**: Energy gained per food unit consumed.

**Energy budget calculation**:

```
# Ticks of survival per food unit:
survival_ticks = energy_per_food / (energy_cost_per_tick + average_moves_per_tick * energy_cost_move)

# For default settings with 50% movement rate:
survival_ticks = 20.0 / (0.1 + 0.5 * 1.0) ≈ 33 ticks
```

**Tuning**:
- Higher: Easier survival (each food more valuable)
- Lower: Harder survival (must eat frequently)

#### `energy_cost_per_tick`

**Type**: Float
**Default**: 0.1
**Range**: 0.0-10.0

**Description**: Energy lost every tick regardless of action (base metabolism).

**Effects**:
- Creates constant selection pressure
- Determines maximum survival time: `max_survival = max_energy / energy_cost_per_tick`
- Default 0.1: 2000 ticks maximum survival at full energy

**Special case**: Setting to 0.0 removes metabolism (creatures only lose energy from actions).

#### `energy_cost_move`

**Type**: Float
**Default**: 1.0
**Range**: 0.0-10.0

**Description**: Energy cost for each movement attempt (even if blocked).

**Ratio to metabolism**:
```
move_to_metabolism_ratio = energy_cost_move / energy_cost_per_tick
```

Default ratio: `1.0 / 0.1 = 10×` (moving costs 10× as much as existing)

**Effects**:
- Higher: Strong pressure for energy-efficient movement
- Lower: Exploration favored over conservation
- Zero: Free movement (only metabolism matters)

#### `energy_cost_reproduce`

**Type**: Float
**Default**: 50.0
**Range**: 0.0-1000.0

**Description**: Energy deducted from parent when offspring is created.

**Balance**:
- Should be significant (half of initial energy by default)
- Must be < `min_reproduce_energy` (otherwise impossible to reproduce)
- Creates trade-off: reproduce now vs. accumulate more energy

#### `min_reproduce_energy`

**Type**: Float
**Default**: 100.0
**Range**: 0.0-1000.0

**Description**: Minimum energy required to reproduce (checked before deducting cost).

**Typical setting**: Equal to `initial_energy` (must be "full" to reproduce).

**Effect on evolution**:
- Higher: Only very successful creatures reproduce
- Lower: More reproduction, faster population growth

#### `reproduce_cooldown_ticks`

**Type**: Integer
**Default**: 100
**Range**: 0-10000

**Description**: Ticks creature must wait after reproducing before reproducing again.

**Time at 30 TPS**:
- `30`: 1 second cooldown
- `100`: 3.3 seconds (default)
- `300`: 10 seconds

**Purpose**:
- Prevents exponential reproduction bursts
- Enforces maximum reproduction rate
- Creates generational structure

## Evolution Configuration

Controls genetic system and neural network architecture.

### Complete Structure

```json
"evolution": {
  "mutation_rate": 0.01,
  "genome_size": 150,
  "neural_net_inputs": 16,
  "neural_net_hidden": 6,
  "neural_net_outputs": 4
}
```

### Parameters

#### `mutation_rate`

**Type**: Float
**Default**: 0.01
**Range**: 0.0-1.0

**Description**: Probability per gene of random mutation during reproduction.

**Expected mutations per offspring**:
```
mutations_per_offspring = genome_size × mutation_rate
```

Default: `100 × 0.01 = 1 mutation per offspring` (average)

**Trade-offs**:
- **Low** (0.001-0.005): Slow, stable evolution - refinement over disruption
- **Medium** (0.01-0.02): Balanced - default range
- **High** (0.05-0.1): Fast, chaotic evolution - exploration over exploitation
- **Very high** (0.2+): Random drift dominates, little cumulative progress

**Recommendations**:
- Start with 0.01 for most experiments
- Increase for faster evolution at cost of stability
- Decrease for fine-tuning already-evolved populations

#### `genome_size`

**Type**: Integer
**Default**: 150
**Range**: 120-10000

**Description**: Number of bytes in each creature's genome.

**Minimum**: 120 genes (required for neural network encoding)
- Input → Hidden: 16 × 6 = 96 genes
- Hidden → Output: 6 × 4 = 24 genes

**Unused genes**: `genome_size - 120` genes available for future features

**Effects**:
- Larger: More potential complexity, slower evolution (larger search space)
- Smaller: Faster evolution, less complexity potential
- Default 150: 30 unused genes for future expansion

#### `neural_net_inputs`

**Type**: Integer
**Default**: 16
**Range**: 1-100

**Description**: Number of input neurons (sensors).

**Current implementation**: 16 inputs fully utilized:
- Input 0-4: Energy, food detection, movement options, creature density
- Input 5-8: Directional creature detection (combat awareness)
- Input 9-12: Directional attack detection (reactive combat)
- Input 13: Health ratio
- Input 14-15: Food type ratios (plant/meat)

See [NEURAL_NETWORKS.md](NEURAL_NETWORKS.md) for complete sensor documentation.

**Changing this**:
- Requires code changes in `src/simulation/tick.rs`
- Must implement new sensor logic
- Affects genome size requirement: `genome_size ≥ inputs × hidden + hidden × outputs`

#### `neural_net_hidden`

**Type**: Integer
**Default**: 6
**Range**: 1-100

**Description**: Number of hidden layer neurons.

**Trade-offs**:
- **Fewer** (2-4): Faster evolution, simpler behaviors, may limit capability
- **More** (10-20): Slower evolution, more complex behaviors possible
- **Many** (50+): Very slow evolution, risk of overfitting

**Genome size requirement**:
```
min_genome_size = (inputs × hidden) + (hidden × outputs)
```

For 8-6-4 architecture: `(8 × 6) + (6 × 4) = 48 + 24 = 72 genes`

**Experimentation**:
Try different values to find the sweet spot for your experiment:
- Start with 6 (default)
- Try 3 for faster evolution
- Try 12 for more complex worlds

#### `neural_net_outputs`

**Type**: Integer
**Default**: 4
**Range**: 1-100

**Description**: Number of output neurons (actions).

**Current implementation**: Fixed at 4 outputs (Up, Down, Left, Right)

**Changing this**:
- Requires code changes to add new actions
- Could add "Stay" action, "Eat" action, etc.
- Affects genome size requirement

## Combat Configuration

Controls combat mechanics, damage, and health regeneration.

### Complete Structure

```json
"combat": {
  "damage_per_attack": 20.0,
  "health_regen_rate": 2.0,
  "health_regen_energy_cost": 2.0
}
```

### Parameters

#### `damage_per_attack`

**Type**: Float
**Default**: 20.0
**Range**: 0.1-1000.0

**Description**: Health damage dealt when a creature attacks another.

**Combat duration (default health 100.0)**:
- `10.0`: 10 attacks to kill
- `20.0`: 5 attacks to kill (default)
- `50.0`: 2 attacks to kill
- `100.0`: 1-hit kills

**Trade-offs**:
- **Low** (5.0-15.0): Extended battles, more strategic combat, healing matters more
- **Medium** (20.0-30.0): Balanced combat (default range)
- **High** (50.0+): Quick decisive battles, less time to react

**Effect on evolution**:
- Higher: Favors aggressive "first strike" behavior
- Lower: Favors tactical positioning and retreat/heal strategies

#### `health_regen_rate`

**Type**: Float
**Default**: 2.0
**Range**: 0.0-50.0

**Description**: Health restored per tick when healing (if energy available).

**Healing time from 0 to 100 HP**:
- `1.0`: 100 ticks (~3.3 seconds at 30 TPS)
- `2.0`: 50 ticks (~1.7 seconds, default)
- `5.0`: 20 ticks (~0.7 seconds)
- `10.0`: 10 ticks (~0.3 seconds)

**Trade-offs**:
- **Slow** (0.5-1.0): Long recovery time, combat wounds persist
- **Medium** (2.0-5.0): Balanced (default range)
- **Fast** (10.0+): Nearly instant healing between fights

**Effect on evolution**:
- Slower: Encourages avoidance, careful engagement
- Faster: Encourages aggression, quick re-engagement

#### `health_regen_energy_cost`

**Type**: Float
**Default**: 2.0
**Range**: 0.0-50.0

**Description**: Energy consumed per tick when healing.

**Energy consumption during full heal (0 to 100 HP)**:
```
total_energy = (100 / health_regen_rate) × health_regen_energy_cost
```

**Examples**:
- `health_regen_rate=2.0, cost=2.0`: 50 ticks × 2.0 = 100 energy (default)
- `health_regen_rate=5.0, cost=1.0`: 20 ticks × 1.0 = 20 energy (cheap healing)
- `health_regen_rate=1.0, cost=5.0`: 100 ticks × 5.0 = 500 energy (expensive healing)

**Trade-offs**:
- **Low** (0.5-1.0): Healing is cheap, energy primarily for movement/reproduction
- **Medium** (2.0-5.0): Healing competes with other energy needs (default)
- **High** (10.0+): Healing very costly, creatures must choose carefully

**Effect on evolution**:
- Lower cost: Aggressive strategies viable (can afford to heal)
- Higher cost: Defensive strategies favored (avoid damage entirely)

**Relationship to other parameters**:
```
# Energy budget for full heal:
heal_cost = (max_health / health_regen_rate) × health_regen_energy_cost

# Compare to other costs:
move_cost = energy_cost_move              # Default: 1.0
reproduce_cost = energy_cost_reproduce    # Default: 50.0

# Default: heal_cost = (100/2.0) × 2.0 = 100 (same as reproduction)
```

## Simulation Configuration

Controls simulation execution and logging.

### Complete Structure

```json
"simulation": {
  "ticks_per_second": 30,
  "log_interval_ticks": 300
}
```

### Parameters

#### `ticks_per_second`

**Type**: Integer
**Default**: 30
**Range**: 1-1000

**Description**: Simulation speed in ticks per second.

**Effects**:
- Determines real-time speed of evolution
- Does NOT affect game logic (logic is tick-based)
- Higher values = faster evolution, higher CPU usage

**Examples**:
- `1`: Slow-motion (1 tick/second) - for observation
- `10`: Slow (10 ticks/second) - detailed watching
- `30`: Default - good balance
- `60`: Fast - 2× speed
- `300`: Very fast - 10× speed for long-term experiments

**Performance note**: If your CPU can't keep up, actual TPS will be lower. Monitor console output for actual rate.

#### `log_interval_ticks`

**Type**: Integer
**Default**: 300
**Range**: 1-100000

**Description**: Ticks between console log outputs.

**Time at default TPS**:
```
log_interval_seconds = log_interval_ticks / ticks_per_second
300 / 30 = 10 seconds
```

**Logged information**:
- Current tick
- Population count
- Average energy
- Max generation

**Examples**:
- `30`: Log every second (verbose)
- `300`: Log every 10 seconds (default)
- `3000`: Log every 100 seconds (quiet)

## Checkpoint Configuration

Controls automatic saving and resuming.

### Complete Structure

```json
"checkpoint": {
  "enabled": true,
  "interval_seconds": 3600,
  "directory": "checkpoints",
  "keep_last_n": 24
}
```

### Parameters

#### `enabled`

**Type**: Boolean
**Default**: true

**Description**: Whether checkpointing system is active.

**Override**: Command-line `--no-checkpoint` flag disables this.

**When to disable**:
- Short test runs
- Collecting performance metrics
- Disk space concerns

#### `interval_seconds`

**Type**: Integer
**Default**: 3600
**Range**: 60-86400

**Description**: Seconds between automatic checkpoint saves.

**Examples**:
- `300`: Every 5 minutes (frequent saving, high disk I/O)
- `3600`: Every hour (default, good balance)
- `86400`: Once per day (long-term experiments)

**Checkpoint time**: Saving takes ~0.1-1 second depending on world size. Simulation pauses briefly during save.

#### `directory`

**Type**: String
**Default**: "checkpoints"

**Description**: Directory path for checkpoint files (relative or absolute).

**File naming**: `checkpoint_YYYYMMDD_HHMMSS.json`

**Example**: `checkpoints/checkpoint_20250131_143022.json`

#### `keep_last_n`

**Type**: Integer
**Default**: 24
**Range**: 1-1000

**Description**: Number of recent checkpoints to keep (older ones deleted).

**Disk usage estimation**:
```
checkpoint_size_mb ≈ (population × 0.15 + world_cells × 0.001)

For 1000 creatures in 100×100 world:
≈ (1000 × 0.15 + 10000 × 0.001) = 160 MB per checkpoint

24 checkpoints ≈ 3.8 GB
```

**Examples**:
- `1`: Keep only latest (minimal disk usage)
- `24`: Keep 24 hours of hourly checkpoints (default)
- `168`: Keep 1 week of hourly checkpoints

## Server Configuration

Controls web server and WebSocket updates.

### Complete Structure

```json
"server": {
  "enabled": true,
  "address": "0.0.0.0",
  "port": 8080,
  "update_rate_hz": 10
}
```

### Parameters

#### `enabled`

**Type**: Boolean
**Default**: true

**Description**: Whether to start the web server.

**Override**: Command-line `--no-server` flag disables this.

**When to disable**:
- Headless simulations
- Data collection mode
- Performance benchmarking

#### `address`

**Type**: String
**Default**: "0.0.0.0"

**Description**: Network address to bind to.

**Options**:
- `"0.0.0.0"`: Listen on all interfaces (default, allows remote connections)
- `"127.0.0.1"`: Localhost only (restrict to local machine)
- Specific IP: Bind to specific network interface

**Security note**: Using `0.0.0.0` allows anyone on your network to connect.

#### `port`

**Type**: Integer
**Default**: 8080
**Range**: 1-65535

**Description**: TCP port for web server.

**URL**: `http://<address>:<port>`

**Common alternatives**:
- `3000`: Common Node.js port
- `8000`: Alternative HTTP port
- `8080`: Default (common alternative HTTP port)

**Note**: Ports below 1024 require admin/root privileges.

#### `update_rate_hz`

**Type**: Integer
**Default**: 10
**Range**: 1-60

**Description**: WebSocket updates per second sent to clients.

**Effects**:
- Higher: Smoother visualization, more bandwidth
- Lower: Choppier visualization, less bandwidth

**Bandwidth estimation**:
```
bytes_per_update ≈ population × 20 (creature data)
bandwidth_kbps = bytes_per_update × update_rate_hz × 8 / 1000

For 1000 creatures at 10 Hz:
≈ 20000 × 10 × 8 / 1000 = 1600 kbps (1.6 Mbps)
```

**Recommendations**:
- `5`: Low bandwidth (~800 kbps for 1000 creatures)
- `10`: Default balance
- `30`: Smooth visualization (matches simulation TPS)

## Tuning Guide

### Goal: Fast Evolution

**Objective**: See evolved behaviors quickly.

```json
{
  "world": {
    "width": 50,
    "height": 50,
    "food_regen_rate": 0.002
  },
  "creature": {
    "initial_population": 200,
    "energy_cost_per_tick": 0.2
  },
  "evolution": {
    "mutation_rate": 0.02,
    "neural_net_hidden": 4
  },
  "simulation": {
    "ticks_per_second": 100
  }
}
```

**Effects**:
- Smaller world (2,500 cells): Less searching
- More food: Easier survival, faster reproduction
- Higher mutation: Faster exploration of strategies
- Fewer hidden neurons: Smaller search space
- Higher TPS: 3.3× real-time speed

### Goal: Harsh Environment

**Objective**: Strong selection pressure, only the best survive.

```json
{
  "world": {
    "food_regen_rate": 0.0005,
    "initial_food_density": 0.1
  },
  "creature": {
    "energy_cost_per_tick": 0.2,
    "energy_cost_move": 2.0,
    "max_population": 200
  },
  "evolution": {
    "mutation_rate": 0.005
  }
}
```

**Effects**:
- Low food regen: Scarcity
- High metabolism: Must find food quickly
- High movement cost: Must move efficiently
- Low mutation: Refinement over disruption
- Low pop cap: Intense competition

### Goal: Spatial Dynamics

**Objective**: Observe migration, territory, clustering.

```json
{
  "world": {
    "width": 200,
    "height": 200,
    "food_regen_rate": 0.0008
  },
  "creature": {
    "initial_population": 50,
    "max_population": 2000
  },
  "evolution": {
    "neural_net_inputs": 8,
    "neural_net_hidden": 8
  }
}
```

**Effects**:
- Large world: Space for spatial patterns
- Low initial pop: Watch spreading and colonization
- High max pop: Allow large-scale clustering
- More hidden neurons: Complex spatial behaviors

### Goal: Long-Term Evolution

**Objective**: Run for weeks, observe long-term trends.

```json
{
  "simulation": {
    "ticks_per_second": 300
  },
  "checkpoint": {
    "interval_seconds": 1800,
    "keep_last_n": 336
  },
  "server": {
    "update_rate_hz": 5
  }
}
```

**Effects**:
- High TPS: 10× speed (10 hours of simulation per real hour)
- Frequent checkpoints: Every 30 minutes
- Keep 1 week of checkpoints (336 = 2 per hour × 24 hours × 7 days)
- Lower update rate: Reduce bandwidth for long-term monitoring

### Goal: Research/Analysis

**Objective**: Collect data for scientific analysis.

```json
{
  "simulation": {
    "ticks_per_second": 100,
    "log_interval_ticks": 100
  },
  "checkpoint": {
    "interval_seconds": 600,
    "keep_last_n": 1000
  },
  "server": {
    "enabled": false
  }
}
```

**Effects**:
- Headless (no web UI): Maximum performance
- Frequent logging: Detailed console output every second
- Many checkpoints: Full history for replay/analysis
- Fast TPS: Generate data quickly

## Example Configurations

### Beginner Friendly

```json
{
  "world": {
    "width": 75,
    "height": 75,
    "initial_food_density": 0.4,
    "food_regen_rate": 0.0015
  },
  "creature": {
    "initial_population": 150,
    "max_population": 800
  },
  "simulation": {
    "ticks_per_second": 20
  }
}
```

**Good for**: First-time users, watching evolution clearly

### Extreme Challenge

```json
{
  "world": {
    "width": 150,
    "height": 150,
    "initial_food_density": 0.05,
    "food_regen_rate": 0.0003,
    "max_food_per_cell": 3
  },
  "creature": {
    "initial_population": 20,
    "max_population": 100,
    "energy_cost_per_tick": 0.3,
    "energy_cost_move": 3.0
  },
  "evolution": {
    "mutation_rate": 0.008
  }
}
```

**Good for**: Observing survival against all odds

### Large-Scale Simulation

```json
{
  "world": {
    "width": 300,
    "height": 300
  },
  "creature": {
    "initial_population": 500,
    "max_population": 5000
  },
  "evolution": {
    "neural_net_hidden": 10
  },
  "server": {
    "update_rate_hz": 5
  }
}
```

**Good for**: Complex emergent behaviors, requires powerful CPU

---

**Next**: Learn about the [technical architecture](ARCHITECTURE.md) and code structure.
