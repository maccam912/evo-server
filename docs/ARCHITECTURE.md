# Technical Architecture

This document explains the technical design of the evolution simulator, including code structure, communication protocols, data flow, and implementation details.

## Table of Contents

- [Technology Stack](#technology-stack)
- [Project Structure](#project-structure)
- [Core Components](#core-components)
- [Communication Protocol](#communication-protocol)
- [Data Flow](#data-flow)
- [Concurrency Model](#concurrency-model)
- [Serialization](#serialization)
- [Performance Considerations](#performance-considerations)
- [Testing](#testing)
- [Deployment](#deployment)

## Technology Stack

### Backend

| Component | Technology | Purpose |
|-----------|------------|---------|
| Language | **Rust** (edition 2021) | Performance, safety, concurrency |
| Web Framework | **Axum** | Async HTTP server and routing |
| WebSocket | **tokio-tungstenite** | Real-time bidirectional communication |
| Async Runtime | **Tokio** | Async I/O and task scheduling |
| Serialization | **serde + serde_json** | JSON encoding/decoding |
| Parallelism | **Rayon** | Data parallelism (available but not currently used) |
| Logging | **env_logger** | Console logging |

### Frontend

| Component | Technology | Purpose |
|-----------|------------|---------|
| UI | **Vanilla JavaScript** | No framework overhead |
| Rendering | **HTML5 Canvas** | 2D graphics with hardware acceleration |
| Communication | **WebSocket API** | Receive real-time updates |
| Styling | **CSS3** | UI layout and styling |

### Infrastructure

| Component | Technology | Purpose |
|-----------|------------|---------|
| Containerization | **Docker** | Isolated deployment environment |
| Orchestration | **Docker Compose** | Multi-container management |
| Build System | **Cargo** | Rust package manager and build tool |

## Project Structure

```
evo-server/
├── src/
│   ├── main.rs                    # Entry point, CLI parsing, main loop
│   ├── config.rs                  # Configuration loading and defaults
│   ├── checkpoint.rs              # Save/load system
│   ├── simulation/
│   │   ├── mod.rs                 # Simulation state and tick logic
│   │   ├── creature.rs            # Creature struct and behavior
│   │   ├── world.rs               # World grid and food system
│   │   └── brain.rs               # Neural network implementation
│   └── server/
│       ├── mod.rs                 # Axum server setup
│       └── websocket.rs           # WebSocket handler and streaming
├── static/
│   ├── index.html                 # Main web page
│   ├── app.js                     # Canvas rendering and client logic
│   └── style.css                  # UI styling
├── checkpoints/                   # Saved simulation states (gitignored)
├── config.json                    # Configuration file (auto-generated)
├── Cargo.toml                     # Rust dependencies
├── Dockerfile                     # Container build instructions
├── docker-compose.yml             # Container orchestration
└── docs/                          # Documentation (this file)
```

### Key Files

#### `src/main.rs` (267 lines)

**Purpose**: Application entry point

**Responsibilities**:
- Parse command-line arguments (clap)
- Load or create configuration
- Load checkpoint or initialize new simulation
- Spawn server task (if enabled)
- Run main simulation loop
- Handle periodic checkpointing

**Key functions**:
- `main()`: Entry point, orchestrates startup
- Simulation loop: Ticks at configured rate

#### `src/config.rs` (120 lines)

**Purpose**: Configuration management

**Types**:
- `Config`: Root configuration struct
- `WorldConfig`: World parameters
- `CreatureConfig`: Creature parameters
- `EvolutionConfig`: Evolution parameters
- `SimulationConfig`: Simulation parameters
- `CheckpointConfig`: Checkpoint parameters
- `ServerConfig`: Server parameters

**Features**:
- Default values with `Default` trait
- JSON serialization/deserialization
- Auto-generation of missing config file

#### `src/simulation/mod.rs` (300+ lines)

**Purpose**: Core simulation state and logic

**Key types**:
- `State`: Complete simulation state (world, creatures, metrics)
- `SimulationMetrics`: Statistics tracking
- `CreatureSnapshot`: Serializable creature data

**Key methods**:
- `State::new()`: Initialize new simulation
- `State::tick()`: Execute one simulation step
- `State::get_snapshot()`: Create client-safe view of state

**Tick logic flow**:
```rust
pub fn tick(&mut self) {
    self.world.regenerate_food();
    self.creatures.shuffle();

    for creature in &mut self.creatures {
        creature.energy -= self.config.energy_cost_per_tick;
        if creature.energy <= 0 { continue; }

        let inputs = self.gather_inputs(creature);
        let action = creature.brain.decide(inputs);
        self.execute_action(creature, action);

        if self.can_reproduce(creature) {
            let offspring = self.reproduce(creature);
            self.pending_births.push(offspring);
        }
    }

    self.creatures.extend(self.pending_births.drain(..));
    self.creatures.retain(|c| c.energy > 0);
    self.enforce_population_cap();
    self.tick += 1;
    self.update_metrics();
}
```

#### `src/simulation/creature.rs` (150+ lines)

**Purpose**: Creature implementation

**Key types**:
- `Creature`: Creature struct with ID, position, energy, genome, brain, generation

**Key fields**:
```rust
pub struct Creature {
    pub id: u32,
    pub x: usize,
    pub y: usize,
    pub energy: f32,
    pub genome: Vec<u8>,
    pub brain: NeuralNetwork,
    pub generation: u32,
    pub reproduction_cooldown: u32,
}
```

**Key methods**:
- `Creature::new()`: Create creature with random or specified genome
- `reproduce()`: Create mutated offspring

#### `src/simulation/world.rs` (100+ lines)

**Purpose**: World grid and food system

**Key types**:
- `World`: 2D grid of cells
- `Cell`: Food count

**Key methods**:
- `World::new()`: Initialize world with initial food
- `regenerate_food()`: Stochastic food regeneration
- `get_food()` / `set_food()`: Cell access
- `count_food_in_neighbors()`: Sensor helper

#### `src/simulation/brain.rs` (200+ lines)

**Purpose**: Neural network implementation

**Key types**:
- `NeuralNetwork`: Weights and architecture

**Key methods**:
- `NeuralNetwork::from_genome()`: Derive weights from genome
- `decide()`: Forward pass through network, return action
- `activate_tanh()`: Activation function

**Network structure**:
```rust
pub struct NeuralNetwork {
    input_hidden_weights: Vec<Vec<f32>>,  // [8][6]
    hidden_output_weights: Vec<Vec<f32>>, // [6][4]
}
```

**Forward pass**:
```rust
pub fn decide(&self, inputs: [f32; 8]) -> usize {
    // Hidden layer
    let mut hidden = vec![0.0; 6];
    for h in 0..6 {
        for i in 0..8 {
            hidden[h] += inputs[i] * self.input_hidden_weights[i][h];
        }
        hidden[h] = hidden[h].tanh();
    }

    // Output layer
    let mut outputs = vec![0.0; 4];
    for o in 0..4 {
        for h in 0..6 {
            outputs[o] += hidden[h] * self.hidden_output_weights[h][o];
        }
        outputs[o] = outputs[o].tanh();
    }

    // Argmax
    outputs.iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(idx, _)| idx)
        .unwrap()
}
```

#### `src/checkpoint.rs` (100+ lines)

**Purpose**: Persistence system

**Key functions**:
- `save_checkpoint()`: Serialize state to JSON file
- `load_latest_checkpoint()`: Find and load most recent checkpoint
- `cleanup_old_checkpoints()`: Delete old checkpoint files

**File format**: JSON (pretty-printed)
**Filename pattern**: `checkpoint_YYYYMMDD_HHMMSS.json`

#### `src/server/mod.rs` (100+ lines)

**Purpose**: HTTP and WebSocket server

**Routes**:
- `GET /`: Serve index.html
- `GET /app.js`: Serve JavaScript
- `GET /style.css`: Serve CSS
- `GET /ws`: WebSocket upgrade endpoint

**Server setup**:
```rust
pub async fn run_server(state: Arc<RwLock<State>>, config: ServerConfig) {
    let app = Router::new()
        .route("/", get(serve_index))
        .route("/app.js", get(serve_js))
        .route("/style.css", get(serve_css))
        .route("/ws", get(websocket_handler))
        .layer(Extension(state));

    let addr = format!("{}:{}", config.address, config.port);
    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

#### `src/server/websocket.rs` (200+ lines)

**Purpose**: WebSocket communication

**Key functions**:
- `websocket_handler()`: Upgrade HTTP to WebSocket
- `handle_websocket()`: Main WebSocket loop
- `StateStream`: Async stream of state snapshots

**Message flow**:
```rust
async fn handle_websocket(socket: WebSocket, state: Arc<RwLock<State>>) {
    let (mut sender, mut receiver) = socket.split();

    // Send initial full state
    let full_state = state.read().await.get_full_state();
    sender.send(Message::Text(serde_json::to_string(&full_state)?)).await?;

    // Stream updates
    let mut stream = StateStream::new(state.clone(), update_rate_hz);
    while let Some(update) = stream.next().await {
        sender.send(Message::Text(serde_json::to_string(&update)?)).await?;
    }
}
```

#### `static/app.js` (500+ lines)

**Purpose**: Client-side rendering and interaction

**Key classes**:
- `SimulationRenderer`: Canvas drawing
- `WebSocketClient`: Server communication

**Key functions**:
- `connect()`: Establish WebSocket connection
- `handleMessage()`: Process server messages
- `render()`: Draw world and creatures
- `handleClick()`: Creature selection
- `handleZoom()`: Zoom controls

**Rendering loop**:
```javascript
function render() {
    // Clear canvas
    ctx.fillStyle = '#0a0a0a';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // Draw grid
    drawGrid();

    // Draw creatures
    for (const creature of creatures) {
        const color = energyToColor(creature.energy);
        drawCreature(creature.x, creature.y, color);
    }

    // Draw selection
    if (selectedCreature) {
        drawSelectionOutline(selectedCreature.x, selectedCreature.y);
    }

    requestAnimationFrame(render);
}
```

## Communication Protocol

### Message Types

#### Server → Client

##### 1. FullState (Initial Connection)

```json
{
  "type": "full_state",
  "tick": 12345,
  "world_width": 100,
  "world_height": 100,
  "metrics": {
    "tick": 12345,
    "population": 456,
    "total_energy": 12345.67,
    "avg_energy": 27.05,
    "avg_generation": 12.3,
    "max_generation": 45,
    "total_food": 3000
  },
  "creatures": [
    {
      "id": 1,
      "x": 45,
      "y": 67,
      "energy": 123.4,
      "generation": 12
    },
    ...
  ]
}
```

**Sent**: Once on initial connection

**Purpose**: Client initialization with complete world state

##### 2. Update (Periodic)

```json
{
  "type": "update",
  "metrics": {
    "tick": 12346,
    "population": 457,
    "total_energy": 12400.0,
    "avg_energy": 27.13,
    "avg_generation": 12.3,
    "max_generation": 45,
    "total_food": 3010
  },
  "creatures": [
    {
      "id": 1,
      "x": 45,
      "y": 68,
      "energy": 122.3,
      "generation": 12
    },
    ...
  ]
}
```

**Sent**: At `update_rate_hz` frequency (default 10 Hz)

**Purpose**: Real-time state updates

**Bandwidth**: ~20 bytes per creature × population × update_rate_hz

##### 3. WorldRegion (Not Implemented)

```json
{
  "type": "world_region",
  "x": 0,
  "y": 0,
  "width": 50,
  "height": 50,
  "cells": [0, 1, 0, 5, 2, 0, ...]
}
```

**Status**: Protocol defined, server handler not implemented

**Purpose**: Send partial world updates (viewport-based optimization)

#### Client → Server

##### 1. GetState

```json
{
  "type": "get_state"
}
```

**Purpose**: Request full state update

**Response**: Server sends `FullState` message

##### 2. GetRegion (Not Implemented)

```json
{
  "type": "get_region",
  "x": 0,
  "y": 0,
  "width": 50,
  "height": 50
}
```

**Status**: Protocol defined, server handler logs "not yet implemented"

**Purpose**: Request specific world region

### Protocol Design Decisions

**Why JSON over Binary?**
- Human-readable debugging
- Simple serialization with serde
- WebSocket text frames (widely supported)
- Performance acceptable for target scale (< 10,000 creatures)

**Binary protocol** would provide:
- ~50% size reduction
- Slightly faster parsing
- Added complexity

**Trade-off**: JSON chosen for simplicity and debuggability.

## Data Flow

### Startup Sequence

```
1. Parse CLI arguments
   ↓
2. Load config.json (or create with defaults)
   ↓
3. Check for existing checkpoint
   ↓
4a. If checkpoint exists:
    - Load state from checkpoint
    - Resume from saved tick

4b. If no checkpoint:
    - Create new State
    - Initialize world with food
    - Spawn initial creatures with random genomes
   ↓
5. If server enabled:
    - Wrap State in Arc<RwLock<State>>
    - Spawn server task on tokio runtime
    - Server starts listening
   ↓
6. Enter main simulation loop
```

### Main Loop

```
Loop every (1.0 / ticks_per_second) seconds:
    1. Acquire write lock on State
    2. Call state.tick()
       - Food regeneration
       - Creature processing (think, move, eat, reproduce)
       - Population update (births, deaths)
       - Metrics update
    3. Release write lock

    4. If log_interval elapsed:
       - Print metrics to console

    5. If checkpoint_interval elapsed:
       - Save checkpoint to disk
       - Cleanup old checkpoints

    6. Sleep until next tick
```

### Server Data Flow

```
Client connects:
    1. HTTP GET /ws
    2. Upgrade to WebSocket
    3. Create StateStream (read-only view)
    4. Acquire read lock on State
    5. Send FullState message
    6. Release read lock

Update loop (every 1/update_rate_hz seconds):
    1. Acquire read lock on State
    2. Get snapshot (metrics + creature list)
    3. Release read lock
    4. Serialize to JSON
    5. Send Update message to client
    6. Sleep until next update
```

**Key insight**: Read locks allow concurrent client updates while simulation continues.

### Client Rendering Flow

```
On receive Update message:
    1. Parse JSON
    2. Update local state (creatures, metrics)
    3. Update statistics panel DOM elements

Render loop (requestAnimationFrame, ~60 FPS):
    1. Clear canvas
    2. Apply zoom and pan transforms
    3. Draw grid lines
    4. For each creature:
       - Calculate screen position
       - Calculate color from energy
       - Draw circle
    5. If creature selected:
       - Draw selection outline
       - Ensure inspector panel visible
```

## Concurrency Model

### Threading Architecture

```
Main Thread:
    - Simulation loop (State::tick())
    - Checkpoint saving

Tokio Runtime (if server enabled):
    - HTTP server
    - WebSocket handlers (one task per client)
    - StateStream updates
```

### Synchronization

**State access**:
- Type: `Arc<RwLock<State>>`
- **Write lock**: Held only during `tick()` (brief, ~0.1-10ms depending on population)
- **Read locks**: Held during client updates (concurrent, non-blocking each other)

**Lock hierarchy**:
```
Main thread:           Tokio tasks:
  ↓                      ↓
state.write()          state.read()  (client 1)
  (simulation tick)    state.read()  (client 2)
  ↓                    state.read()  (client 3)
release                  ↓
                       release
```

**No deadlocks**: Only one lock type, clear ownership.

### Parallelism Opportunities

**Currently sequential**:
- Creature processing (loop over creatures)
- Food regeneration (loop over cells)

**Potential parallelization** (Rayon available but unused):
```rust
use rayon::prelude::*;

// Parallel creature processing
self.creatures.par_iter_mut().for_each(|creature| {
    // Process creature
});
```

**Challenges**:
- Shared mutable state (world grid)
- Creature interactions (reproduction requires finding empty cells)

**Possible with refactoring**:
- Partition world into regions
- Process regions in parallel
- Synchronize border interactions

## Serialization

### Serde Usage

**State serialization** (checkpoints):
```rust
#[derive(Serialize, Deserialize)]
pub struct State {
    pub tick: u64,
    pub creatures: Vec<Creature>,
    pub world: World,
    pub config: Config,
}
```

**Checkpoint save**:
```rust
let json = serde_json::to_string_pretty(&state)?;
std::fs::write(path, json)?;
```

**Checkpoint load**:
```rust
let json = std::fs::read_to_string(path)?;
let state: State = serde_json::from_str(&json)?;
```

### Custom Serialization

**Genome encoding**:
- Direct byte array: `Vec<u8>`
- JSON representation: Array of integers `[142, 67, 203, ...]`

**Neural network**:
- Not serialized in messages (reconstructed from genome)
- Checkpoint includes genome only

## Performance Considerations

### Bottlenecks

**Simulation tick** (main bottleneck for large populations):
- Time complexity: O(creatures × (sensor_computation + neural_network_forward))
- Typical: ~0.1ms per creature for 8-6-4 network
- 1000 creatures: ~100ms per tick (10 TPS maximum)

**Sensor computation**:
- `count_food_in_neighbors()`: O(1) (8 cells)
- `count_creatures_in_radius(5)`: O(creatures_in_radius) (up to 78 cells)
- Most expensive sensor: creature density

**Neural network forward pass**:
- Input → Hidden: 8 × 6 = 48 multiplications + 6 tanh
- Hidden → Output: 6 × 4 = 24 multiplications + 4 tanh
- Total: 72 multiplications, 10 tanh calls

### Optimizations Implemented

1. **Viewport culling** (client): Only render visible creatures
2. **Update throttling**: Client updates at 10 Hz, not simulation rate
3. **Read-write lock**: Concurrent client reads during simulation
4. **Canvas hardware acceleration**: GPU-accelerated rendering

### Optimization Opportunities

**Spatial partitioning**:
```rust
// Current: O(n) creature lookup in radius
for creature in creatures {
    if distance(center, creature.pos) <= radius {
        count += 1;
    }
}

// Optimized: O(1) lookup with grid partitioning
let cells_in_radius = get_cells_in_radius(center, radius);
for cell in cells_in_radius {
    count += cell.creature_count;
}
```

**Parallel processing** (Rayon):
```rust
creatures.par_iter_mut().for_each(|creature| {
    // Process independently
});
```

**Incremental rendering**:
```rust
// Only send changed creatures
let update = creatures
    .filter(|c| c.moved_this_tick)
    .collect();
```

### Scalability

**Current limits** (single-threaded):
- ~1000 creatures at 30 TPS (smooth)
- ~5000 creatures at 10 TPS (acceptable)
- ~10000 creatures at 3 TPS (sluggish)

**With optimizations**:
- Spatial partitioning: 2-3× improvement
- Parallel processing: 4-8× improvement (on multi-core)
- Combined: ~50,000 creatures at 30 TPS (estimated)

## Testing

### Unit Tests

**Location**: Inline with source files (`#[cfg(test)]` modules)

**Coverage**:
- ✅ Neural network forward pass
- ✅ Genome to weights conversion
- ✅ World food regeneration
- ✅ Creature reproduction and mutation
- ✅ Configuration loading

**Run tests**:
```bash
cargo test
```

### Integration Tests

**Currently not implemented** (opportunity for contribution)

**Potential tests**:
- Full simulation tick cycle
- WebSocket message protocol
- Checkpoint save/load roundtrip

### Manual Testing

**Typical workflow**:
1. Start simulation with test config
2. Observe behavior in web UI
3. Check console logs for metrics
4. Verify checkpoint creation
5. Test resume from checkpoint

## Deployment

### Docker Build

**Multi-stage build** (Dockerfile):

```dockerfile
# Stage 1: Build Rust binary
FROM rust:1.70 as builder
WORKDIR /app
COPY Cargo.* ./
COPY src ./src
RUN cargo build --release

# Stage 2: Runtime image
FROM debian:bullseye-slim
WORKDIR /app
COPY --from=builder /app/target/release/evo-server .
COPY static ./static
COPY config.json .
EXPOSE 8080
CMD ["./evo-server"]
```

**Benefits**:
- Small final image (Rust binary only, no toolchain)
- Reproducible builds
- Isolated environment

### Docker Compose

```yaml
version: '3.8'
services:
  evo-server:
    build: .
    ports:
      - "8080:8080"
    volumes:
      - ./checkpoints:/app/checkpoints
      - ./data:/app/data
    restart: unless-stopped
```

**Features**:
- Port mapping: 8080:8080
- Volume persistence: Checkpoints survive container restart
- Auto-restart: Resilience to crashes

### Production Considerations

**Monitoring**:
- Console logs (Docker logs)
- Metrics endpoint (not implemented, opportunity)
- Health checks (not implemented, opportunity)

**Scaling**:
- Current: Single instance (stateful)
- Horizontal scaling: Not possible (shared state required)
- Vertical scaling: More CPU/RAM for larger populations

**Security**:
- No authentication (web UI publicly accessible)
- No HTTPS (use reverse proxy like Nginx for TLS)
- File system access (checkpoints writable by process)

---

**Related Documentation**:
- [Simulation Mechanics](SIMULATION.md)
- [Neural Networks](NEURAL_NETWORKS.md)
- [Configuration](CONFIGURATION.md)
