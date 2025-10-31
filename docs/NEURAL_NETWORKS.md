# Neural Networks

This document provides a comprehensive guide to how neural networks control creature behavior, including architecture, weight encoding, decision making, and how behavior evolves over time.

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [Network Structure](#network-structure)
- [Input Layer (Sensors)](#input-layer-sensors)
- [Hidden Layer](#hidden-layer)
- [Output Layer (Actions)](#output-layer-actions)
- [Weight Encoding from Genome](#weight-encoding-from-genome)
- [Decision Making Process](#decision-making-process)
- [Activation Functions](#activation-functions)
- [How Behavior Evolves](#how-behavior-evolves)
- [Network Topology Details](#network-topology-details)

## Architecture Overview

Each creature has a **feedforward neural network** brain with fixed architecture:

```
INPUT LAYER          HIDDEN LAYER         OUTPUT LAYER
  (16 neurons)        (6 neurons)          (4 neurons)

     [0]  ────────┐    ┌──► [H0] ────┐    ┌──► [Up]
     [1]  ────────┤    │              │    │
     [2]  ────────┤    │    [H1] ────┤    │    [Down]
     [3]  ────────┤    │              │    ├──►
     [4]  ────────┤    │    [H2] ────┤    │
     [5]  ────────┼────┤              ├────┤    [Left]
     [6]  ────────┤    │    [H3] ────┤    │
     [7]  ────────┤    │              │    │    [Right]
     [8]  ────────┤    │    [H4] ────┤    └──►
     [9]  ────────┤    │              │
    [10]  ────────┤    └──► [H5] ────┘
    [11]  ────────┤
    [12]  ────────┤
    [13]  ────────┤       tanh
    [14]  ────────┤       activation
    [15]  ────────┘

  Sensors              tanh              tanh
  (normalized)         activation        activation
```

**Key characteristics**:
- **Fully connected**: Every input connects to every hidden neuron, every hidden connects to every output
- **Feedforward only**: No recurrent connections (no memory)
- **Fixed topology**: Architecture never changes (only weights evolve)
- **Deterministic**: Same inputs always produce same outputs for a given creature

## Network Structure

### Layer Sizes

| Layer | Neurons | Activation | Purpose |
|-------|---------|------------|---------|
| Input | 16 | None (pass-through) | Sensor data from environment (energy, food, combat, health) |
| Hidden | 6 | Hyperbolic tangent (tanh) | Feature extraction and decision logic |
| Output | 4 | Hyperbolic tangent (tanh) | Action selection |

### Total Parameters

**Weight counts**:
- Input → Hidden: 16 × 6 = **96 weights**
- Hidden → Output: 6 × 4 = **24 weights**
- **Total: 120 weights**

**Bias neurons**: Currently **not implemented** (no bias terms)

**Genome usage**: 120 genes are used from the 150-byte genome to encode all weights. The remaining 30 bytes are available for future traits or wrap around if needed.

## Input Layer (Sensors)

The input layer provides the creature with information about its environment. Each input is normalized to the range [0.0, 1.0].

### Input 0: Energy Ratio

```rust
input[0] = creature.energy / max_energy
```

- **Range**: 0.0 to 1.0
- **Meaning**: Self-awareness of energy state
- **Example values**:
  - 0.0 = Empty (dead)
  - 0.5 = Half energy (moderate danger)
  - 1.0 = Full energy (safe, can reproduce)

**Purpose**: Allows creatures to adjust behavior based on energy level (e.g., be more cautious when low).

### Input 1: Nearby Food Count

```rust
neighbors = 8 cells surrounding creature
food_count = count cells with food > 0 in neighbors
input[1] = food_count / 8.0
```

- **Range**: 0.0 to 1.0
- **Quantization**: 9 discrete values (0/8, 1/8, 2/8, ..., 8/8)
- **Meaning**: Food density in immediate neighborhood

**Purpose**: Primary food detection sensor - enables creatures to sense food without being directly on it.

### Input 2: Empty Space Count

```rust
neighbors = 8 cells surrounding creature
empty_count = count cells with no creatures in neighbors
input[2] = empty_count / 8.0
```

- **Range**: 0.0 to 1.0
- **Meaning**: Available movement directions

**Purpose**: Helps creatures avoid getting trapped or crowded. Useful for exploration behavior.

### Input 3: Food At Current Position

```rust
input[3] = if cell.food > 0 { 1.0 } else { 0.0 }
```

- **Range**: 0.0 or 1.0 (binary)
- **Meaning**: Food available right now

**Purpose**: Direct signal that food can be eaten immediately. Enables "stop and eat" behavior.

### Input 4: Nearby Creature Density

```rust
radius = 5 cells
max_cells_in_radius = 78  // π × 5² ≈ 78 (excluding center)
creature_count = count creatures within radius
input[4] = min(creature_count / 78.0, 1.0)
```

- **Range**: 0.0 to 1.0
- **Meaning**: Population pressure in local area

**Purpose**: Enables density-dependent behavior like:
- Avoiding crowded areas (less food competition)
- Seeking crowded areas (social behavior, if beneficial)
- Migration patterns

### Input 5: Creature Detected Up

```rust
input[5] = if creature_at(x, y - 1) { 1.0 } else { 0.0 }
```

- **Range**: 0.0 or 1.0 (binary)
- **Meaning**: Enemy presence directly above

**Purpose**: Enables combat awareness and tactical positioning. Creatures can detect threats before moving.

### Input 6: Creature Detected Down

```rust
input[6] = if creature_at(x, y + 1) { 1.0 } else { 0.0 }
```

- **Range**: 0.0 or 1.0 (binary)
- **Meaning**: Enemy presence directly below

### Input 7: Creature Detected Left

```rust
input[7] = if creature_at(x - 1, y) { 1.0 } else { 0.0 }
```

- **Range**: 0.0 or 1.0 (binary)
- **Meaning**: Enemy presence directly to the left

### Input 8: Creature Detected Right

```rust
input[8] = if creature_at(x + 1, y) { 1.0 } else { 0.0 }
```

- **Range**: 0.0 or 1.0 (binary)
- **Meaning**: Enemy presence directly to the right

**Purpose (Inputs 5-8)**: These four directional sensors enable creatures to:
- Detect immediate threats in cardinal directions
- Choose combat vs avoidance strategies
- Coordinate attacks from specific directions
- Learn to flee when surrounded

### Input 9: Attacked From Up (Last Tick)

```rust
input[9] = if attacked_from_up_last_tick { 1.0 } else { 0.0 }
```

- **Range**: 0.0 or 1.0 (binary)
- **Meaning**: Took damage from above on previous tick

**Purpose**: Enables reactive combat behavior. Creatures can learn to counter-attack or flee from attackers.

### Input 10: Attacked From Down (Last Tick)

```rust
input[10] = if attacked_from_down_last_tick { 1.0 } else { 0.0 }
```

- **Range**: 0.0 or 1.0 (binary)
- **Meaning**: Took damage from below on previous tick

### Input 11: Attacked From Left (Last Tick)

```rust
input[11] = if attacked_from_left_last_tick { 1.0 } else { 0.0 }
```

- **Range**: 0.0 or 1.0 (binary)
- **Meaning**: Took damage from the left on previous tick

### Input 12: Attacked From Right (Last Tick)

```rust
input[12] = if attacked_from_right_last_tick { 1.0 } else { 0.0 }
```

- **Range**: 0.0 or 1.0 (binary)
- **Meaning**: Took damage from the right on previous tick

**Purpose (Inputs 9-12)**: These attack sensors enable:
- Retaliation behavior (attack back in the direction of the attacker)
- Escape behavior (flee away from danger)
- Learning to predict multi-tick combat sequences
- Differentiating between proactive hunting and defensive responses

### Input 13: Health Ratio

```rust
input[13] = creature.health / max_health
```

- **Range**: 0.0 to 1.0
- **Meaning**: Current health as fraction of maximum
- **Example values**:
  - 0.0 = Near death (1 hit from dying)
  - 0.5 = Half health (moderate danger)
  - 1.0 = Full health (safe for combat)

**Purpose**: Self-awareness of combat readiness. Enables:
- Fleeing when wounded
- Attacking when healthy
- Seeking food/safety to heal
- Risk assessment in combat situations

### Input 14: Nearby Plant Food Ratio

```rust
plant_count = count plant food in 8 neighbors
food_count = total food in 8 neighbors
input[14] = if food_count > 0 { plant_count / food_count } else { 0.0 }
```

- **Range**: 0.0 to 1.0
- **Meaning**: Proportion of nearby food that is plant-based
- **Example values**:
  - 0.0 = All nearby food is meat
  - 0.5 = Equal mix of plant and meat
  - 1.0 = All nearby food is plant

**Purpose**: Enables dietary preference evolution. Currently both food types provide equal energy, but this sensor allows future differentiation.

### Input 15: Nearby Meat Food Ratio

```rust
meat_count = count meat food in 8 neighbors
food_count = total food in 8 neighbors
input[15] = if food_count > 0 { meat_count / food_count } else { 0.0 }
```

- **Range**: 0.0 to 1.0
- **Meaning**: Proportion of nearby food that is meat
- **Note**: input[14] + input[15] = 1.0 (when food present)

**Purpose**: Complementary to input 14. Could enable:
- Scavenger behavior (seeking meat from dead creatures)
- Herbivore behavior (avoiding meat, seeking plants)
- Opportunistic feeding (eating whatever is available)

**Potential future sensors**:
- Directional food gradient (which direction has most food)
- Energy trend (is energy rising or falling)
- Age/generation awareness
- Reproduction cooldown status
- Distance to nearest food
- Distance to world boundary

## Hidden Layer

The hidden layer performs **feature extraction and transformation** of sensor inputs.

### Structure

- **6 neurons** (configurable via `neural_net_hidden`)
- **Activation**: Hyperbolic tangent (tanh)
- **Inputs**: All 8 input sensors (fully connected)
- **Outputs**: Connect to all 4 output neurons

### Computation

For each hidden neuron `h`:

```rust
// Weighted sum of inputs
z[h] = sum(input[i] × weight[i][h] for i in 0..8)

// Activation function
hidden[h] = tanh(z[h])
```

### Why 6 Neurons?

The choice of 6 hidden neurons is a balance:

- **Too few** (e.g., 2): Not enough capacity to learn complex strategies
- **Too many** (e.g., 20): Slower evolution (more genes to optimize)
- **6 neurons**: Provides ~48 learnable parameters for input processing

**Experimentation**: This can be tuned in `config.json` - try different values!

## Output Layer (Actions)

The output layer determines which action the creature will take.

### Structure

- **4 neurons** (one per possible action)
- **Activation**: Hyperbolic tangent (tanh)
- **Range**: -1.0 to +1.0 per output

### Output Meanings

| Output Index | Action | Movement | Energy Cost |
|--------------|--------|----------|-------------|
| 0 | Move Up | (x, y-1) | 1.0 |
| 1 | Move Down | (x, y+1) | 1.0 |
| 2 | Move Left | (x-1, y) | 1.0 |
| 3 | Move Right | (x+1, y) | 1.0 |

### Computation

For each output neuron `o`:

```rust
// Weighted sum of hidden layer
z[o] = sum(hidden[h] × weight[h][o] for h in 0..6)

// Activation function
output[o] = tanh(z[o])
```

### Why Tanh for Outputs?

Tanh outputs are in range [-1.0, +1.0], which provides:
- **Magnitude**: Absolute value indicates "confidence" in action
- **Sign**: Positive values still allow argmax selection
- **Differentiation**: Clear preferences when one output is much higher

The actual range doesn't matter since only **relative ordering** (argmax) is used.

## Weight Encoding from Genome

This is the key mechanism that links **genetics** to **behavior**.

### Genome Structure

- **Type**: Array of bytes (u8 values)
- **Size**: 100 bytes (default, configurable)
- **Range**: Each byte is 0-255

### Weight Derivation

Weights are deterministically extracted from the genome:

```rust
fn genome_to_weights(genome: &[u8]) -> Vec<f32> {
    genome.iter()
        .map(|&byte| {
            // Normalize to 0.0-1.0
            let normalized = byte as f32 / 255.0;
            // Scale to -1.0 to +1.0
            (normalized * 2.0) - 1.0
        })
        .collect()
}
```

**Example**:
- Genome byte `0` → Weight `-1.0`
- Genome byte `127` → Weight `-0.004`
- Genome byte `255` → Weight `+1.0`

### Weight Assignment

Weights are assigned in order:

```rust
// First 48 genes: Input → Hidden weights
for i in 0..8:           // For each input
    for h in 0..6:       // For each hidden neuron
        weight[i][h] = genome[i * 6 + h]

// Next 24 genes: Hidden → Output weights
for h in 0..6:           // For each hidden neuron
    for o in 0..4:       // For each output
        weight[h][o] = genome[48 + h * 4 + o]
```

**Genome usage**: Genes 0-71 encode the neural network (72 genes total).

**Unused genes**: Genes 72-99 are currently unused (28 genes). These could be used for:
- Bias terms
- Additional layers
- Non-neural traits (speed, size, etc.)

### Why This Encoding?

**Advantages**:
1. **Deterministic**: Same genome always produces same behavior
2. **Gradual evolution**: Single gene mutation = single weight change
3. **Searchable**: 100-dimensional search space (manageable for evolution)

**Alternatives not used**:
- Binary encoding (too discrete, hard to evolve)
- Floating-point genome (harder to mutate meaningfully)
- Indirect encoding (more complex, possibly slower evolution)

## Decision Making Process

Every tick, each creature goes through this decision process:

### Step 1: Gather Inputs

```rust
let inputs = [
    creature.energy / max_energy,
    count_food_neighbors() / 8.0,
    count_empty_neighbors() / 8.0,
    if has_food_here() { 1.0 } else { 0.0 },
    count_nearby_creatures() / 78.0,
    0.0,
    0.0,
    0.0,
];
```

### Step 2: Compute Hidden Layer

```rust
let mut hidden = [0.0; 6];

for h in 0..6 {
    let mut sum = 0.0;
    for i in 0..8 {
        sum += inputs[i] * weights_input_hidden[i][h];
    }
    hidden[h] = tanh(sum);
}
```

### Step 3: Compute Output Layer

```rust
let mut outputs = [0.0; 4];

for o in 0..4 {
    let mut sum = 0.0;
    for h in 0..6 {
        sum += hidden[h] * weights_hidden_output[h][o];
    }
    outputs[o] = tanh(sum);
}
```

### Step 4: Select Action (Argmax)

```rust
let action = outputs
    .iter()
    .enumerate()
    .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
    .map(|(index, _)| index)
    .unwrap();

// action is 0 (Up), 1 (Down), 2 (Left), or 3 (Right)
```

**Tie-breaking**: If multiple outputs have the same value, the lowest index wins (implementation detail).

### Step 5: Execute Action

The selected action is executed (see [Simulation Mechanics](SIMULATION.md#action-execution) for details).

## Activation Functions

### Hyperbolic Tangent (tanh)

Both hidden and output layers use **tanh** activation:

```rust
fn tanh(x: f32) -> f32 {
    let e2x = (2.0 * x).exp();
    (e2x - 1.0) / (e2x + 1.0)
}
```

**Properties**:
- **Range**: -1.0 to +1.0
- **Symmetric**: tanh(-x) = -tanh(x)
- **Non-linear**: Enables complex decision boundaries
- **Saturating**: Large inputs compress to ±1.0

### Tanh Behavior

| Input (x) | Output tanh(x) | Interpretation |
|-----------|----------------|----------------|
| -∞ to -3  | ≈ -1.0 | Strong negative |
| -1.0      | -0.76 | Moderate negative |
| 0.0       | 0.0 | Neutral |
| +1.0      | +0.76 | Moderate positive |
| +3 to +∞  | ≈ +1.0 | Strong positive |

### Why Tanh?

**Advantages over other activations**:
- **vs ReLU**: Symmetric (no dead neurons), bounded output
- **vs Sigmoid**: Centered at 0 (better gradient flow)
- **vs Linear**: Non-linearity enables complex behavior

**Historical note**: Tanh is a classic activation function, well-suited for evolutionary algorithms where gradient-based learning is not used.

## How Behavior Evolves

### Generation 0: Random Chaos

Initial creatures have **completely random genomes** → **random weights** → **random behavior**:

```
Example genome: [142, 67, 203, 89, 12, 255, 71, ...]
         ↓
Example weights: [0.12, -0.47, 0.59, -0.30, -0.91, 1.0, -0.44, ...]
         ↓
Random neural network behavior
```

**Typical behavior**: Wander aimlessly, no correlation with food location, most die quickly.

### Generations 1-10: Lucky Survivors

A few creatures **accidentally** have weight combinations that:
- Respond positively to `input[1]` (nearby food)
- Bias outputs toward moving onto food

**Example beneficial pattern**:
```rust
// If this weight is large and positive:
weights_input_hidden[1][0] = 0.8  // Food sensor → Hidden neuron

// And this weight pattern exists:
weights_hidden_output[0][0] = 0.6  // Hidden → Up
weights_hidden_output[0][1] = -0.3 // Hidden → Down

// Then: High food signal → Activate hidden[0] → Prefer Up movement
```

These "lucky" creatures:
- Survive longer (find food)
- Reproduce more (pass genes to offspring)
- **Dominate the next generation**

### Generations 10-50: Refinement

Offspring inherit "food-sensing" genes with small mutations:

```rust
// Parent genome: [142, 67, 203, 89, ...]
//                  ↓ mutation (1% chance per gene)
// Offspring:      [142, 67, 189, 89, ...]
//                       ^^^^
//                    Changed from 203 → 189
```

**Result**: Gradual refinement of food-seeking behavior

**Beneficial mutations**: Make food-response stronger or more consistent
**Neutral mutations**: No effect on survival
**Harmful mutations**: Reduce food-seeking, these lineages die out

### Generations 50-200: Optimization

Population now consists **entirely** of descendants from the best early creatures.

**Selection pressure shifts** to:
- More efficient movement (less energy waste)
- Better handling of edge cases (boundaries, crowding)
- Optimal reproduction timing

**Diminishing returns**: Easy gains are already made, progress slows.

### Generations 200+: Equilibrium

Population reaches **near-optimal** food-finding behavior given:
- The 8-input sensors available
- The 6-hidden-neuron capacity
- The environmental parameters

**Further evolution** is driven by:
- Competition between creatures (indirect selection)
- Rare beneficial mutations (small incremental gains)
- Genetic drift (neutral mutations accumulating)

### Observable Evolutionary Patterns

You can observe evolution by watching creatures develop:

1. **Directional movement toward food** (vs random wandering)
2. **Avoiding boundaries** (vs getting stuck at edges)
3. **Energy conservation** (vs constant movement)
4. **Clustering near food sources** (vs uniform distribution)

**Time scale**: Expect to see clear improvements after ~50-100 generations, which can take 10-30 minutes of real time depending on simulation speed.

## Network Topology Details

### Why Fully Connected?

Every input connects to every hidden neuron, and every hidden connects to every output.

**Advantages**:
- **Maximum flexibility**: Any input can influence any output
- **Redundancy**: Multiple pathways for important signals
- **Evolvability**: Many weight combinations can produce same behavior

**Disadvantages**:
- **More parameters**: 72 weights vs. ~20-30 for sparse network
- **Slower evolution**: Larger search space

### Why No Recurrence?

The network has **no memory** - decisions are based only on current sensory input.

**Implications**:
- Cannot learn sequences or temporal patterns
- Cannot remember where food was previously
- Cannot plan ahead or anticipate future states

**Potential enhancement**: Add recurrent connections for memory-based behavior.

### Why No Bias Terms?

Bias neurons are **not implemented** - all neurons start from zero weighted sum.

**Effect**:
- Slightly reduces network capacity
- 72 parameters instead of 72 + 6 + 4 = 82

**Potential enhancement**: Add bias terms for each hidden and output neuron.

### Network Capacity

With 72 weights and tanh activations, the network can represent:
- **Linear decision boundaries**: Simple rules like "move toward food"
- **Non-linear behaviors**: Complex strategies involving multiple sensor combinations
- **XOR-like logic**: Hidden layer enables non-linearly-separable decisions

**Not possible**:
- Temporal logic (no memory)
- Very complex strategies requiring >6 hidden features
- Learning during lifetime (weights are fixed from birth)

## Experimentation Ideas

### Modify Network Architecture

Try different configurations in `config.json`:

```json
"evolution": {
  "neural_net_inputs": 8,
  "neural_net_hidden": 12,    // More capacity
  "neural_net_outputs": 4
}
```

**Effects to observe**:
- More hidden neurons: Slower evolution but possibly more complex behavior
- Fewer hidden neurons: Faster evolution but limited capability

### Add New Input Sensors

Modify `src/simulation/brain.rs` to add new inputs (indices 5-7):

**Directional food gradient**:
```rust
input[5] = count_food_in_direction(Up) - count_food_in_direction(Down);
input[6] = count_food_in_direction(Left) - count_food_in_direction(Right);
```

**Energy trend**:
```rust
input[7] = (current_energy - last_tick_energy) / max_energy;
```

### Analyze Evolved Brains

Export creature genomes and visualize weight matrices to see what patterns evolution discovers.

---

**Next**: Learn about [UI elements and visualization](UI_GUIDE.md).
