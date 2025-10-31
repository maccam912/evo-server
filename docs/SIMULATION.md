# Simulation Mechanics

This document explains how the evolution simulator works at a detailed level, including the tick cycle, energy system, food mechanics, and population dynamics.

## Table of Contents

- [World Structure](#world-structure)
- [The Tick Cycle](#the-tick-cycle)
- [Energy System](#energy-system)
- [Health System](#health-system)
- [Combat Mechanics](#combat-mechanics)
- [Food Mechanics](#food-mechanics)
- [Reproduction System](#reproduction-system)
- [Death and Population Control](#death-and-population-control)
- [Selection Pressures](#selection-pressures)
- [Evolutionary Dynamics](#evolutionary-dynamics)

## World Structure

### Grid System

The world is a 2D grid of cells:

- **Default size**: 100×100 cells (10,000 total cells)
- **Configurable**: Can be adjusted in `config.json`
- **Coordinate system**: (0,0) is top-left, (width-1, height-1) is bottom-right
- **Boundaries**: Hard boundaries - creatures cannot move outside the grid

### Cell Contents

Each cell can contain:
- **Food**: 0 to `max_food_per_cell` units (default: 10 units)
  - **Plant food**: Naturally regenerating food (green in UI)
  - **Meat food**: Dropped when creatures die (red in UI)
- **Creatures**: **At most one creature per cell**

Note: Combat occurs when a creature attempts to move into an occupied cell.

### Initial State

When a simulation starts:
1. **Food placement**: `initial_food_density` (default 30%) of cells receive 1 food unit
2. **Creature placement**: `initial_population` (default 100) creatures spawn at random positions
3. **Initial energy**: Each creature starts with `initial_energy` (default 100.0)
4. **Genomes**: All generation-0 creatures have completely random genomes

## The Tick Cycle

The simulation runs at a configurable rate (default: 30 ticks per second). Each tick follows this exact sequence:

### 1. Food Regeneration and Decay Phase

```rust
# Regenerate food
for each cell in world:
    if random() < food_regen_rate:
        if cell.food < max_food_per_cell:
            cell.food += 1

# Age and decay food
for each cell in world:
    cell.age_food()  # Increment age by 1
    if cell.should_decay(plant_decay_ticks, meat_decay_ticks):
        cell.decay()  # Remove food
```

**Regeneration**:
- **Rate**: `food_regen_rate` (default 0.001 = 0.1% of cells per tick)
- **Result**: ~10 cells gain food per tick in a 100×100 world
- **Cap**: Cells cannot exceed `max_food_per_cell` food units

**Decay**:
- **Plant food**: Decays after `plant_decay_ticks` (default 600 = ~20 seconds at 30 TPS)
- **Meat food**: Decays after `meat_decay_ticks` (default 300 = ~10 seconds at 30 TPS, faster)
- **Age tracking**: Each food cell tracks ticks since creation/refresh
- **Purpose**: Prevents infinite food accumulation and creates urgency for scavenging

### 2. Creature Processing Phase

Creatures are processed in **random order** each tick (shuffled for fairness):

```
shuffle(creatures)

for each creature in shuffled_list:
    # 2a. Base metabolism
    creature.energy -= energy_cost_per_tick  # -0.1

    # 2b. Check survival
    if creature.energy <= 0:
        mark_for_death(creature)
        continue

    # 2c. Gather sensor inputs
    inputs = gather_inputs(creature)

    # 2d. Neural network decision
    action = creature.brain.decide(inputs)

    # 2e. Execute action
    execute_action(creature, action)

    # 2f. Check reproduction eligibility
    if can_reproduce(creature):
        offspring = reproduce(creature)
        add_to_pending(offspring)
```

#### 2a. Base Metabolism

Every living creature loses `energy_cost_per_tick` (default 0.1) energy.

**Purpose**: Creates constant selection pressure - creatures must actively find food.

#### 2b. Survival Check

If energy drops to or below 0, the creature is marked for death and skips the rest of its turn.

#### 2c. Sensor Input Gathering

The creature's neural network receives 8 inputs from the environment:

| Index | Sensor | Range | Calculation |
|-------|--------|-------|-------------|
| 0 | Energy ratio | 0.0-1.0 | `current_energy / max_energy` |
| 1 | Nearby food | 0.0-1.0 | `count_food_in_neighbors() / 8.0` |
| 2 | Empty spaces | 0.0-1.0 | `count_empty_neighbors() / 8.0` |
| 3 | Food here | 0.0 or 1.0 | `if food > 0 then 1.0 else 0.0` |
| 4 | Creature density | 0.0-1.0 | `count_creatures_in_radius(5) / 78.0` |
| 5-7 | Unused | 0.0 | Reserved for future sensors |

**Neighbors**: The 8 cells surrounding the creature (Moore neighborhood)

**Density radius**: 5-cell radius = 78 cells maximum (excluding center)

#### 2d. Neural Network Decision

The brain processes inputs through:
1. Input layer (8 neurons) → Hidden layer (6 neurons, tanh activation)
2. Hidden layer → Output layer (4 neurons, tanh activation)
3. Argmax selection (highest output value determines action)

See [Neural Networks](NEURAL_NETWORKS.md) for details.

#### 2e. Action Execution

Based on the neural network output, one of 4 actions is attempted:

| Output Index | Action | Energy Cost | Behavior |
|--------------|--------|-------------|----------|
| 0 | Move Up | 1.0 | Attempt to move to (x, y-1) |
| 1 | Move Down | 1.0 | Attempt to move to (x, y+1) |
| 2 | Move Left | 1.0 | Attempt to move to (x-1, y) |
| 3 | Move Right | 1.0 | Attempt to move to (x+1, y) |

**Movement rules**:
- Target cell must be within world boundaries
- Target cell must not contain other creatures
- If movement fails (blocked), creature stays in place but still pays energy cost
- After movement (or staying), check for food at current position

**Food consumption**:
```rust
if cell.food > 0:
    creature.energy += energy_per_food * cell.food  # +20.0 per unit
    creature.energy = min(creature.energy, max_energy)  # Cap at 200.0
    cell.food = 0  # All food consumed
```

#### 2f. Reproduction Check

A creature can reproduce if ALL conditions are met:

1. **Energy threshold**: `energy >= min_reproduce_energy` (default 100.0)
2. **Cooldown elapsed**: `ticks_since_last_reproduction >= reproduce_cooldown_ticks` (default 100)
3. **Population cap**: `current_population < max_population` (default 1000)
4. **Empty neighbor**: At least one adjacent cell is empty

**Reproduction process**:
```rust
if eligible:
    # Find empty adjacent cell
    adjacent_cell = find_empty_neighbor(creature)

    if adjacent_cell exists:
        # Create offspring
        offspring = Creature {
            position: adjacent_cell,
            energy: initial_energy,  # 100.0
            genome: mutate(parent.genome),
            generation: parent.generation + 1,
            reproduction_cooldown: reproduce_cooldown_ticks
        }

        # Pay reproduction cost
        parent.energy -= energy_cost_reproduce  # -50.0
        parent.reproduction_cooldown = reproduce_cooldown_ticks

        # Add to pending births
        pending_births.push(offspring)
```

### 3. Population Update Phase

```rust
# Add all offspring born this tick
creatures.extend(pending_births)
pending_births.clear()

# Remove all dead creatures
creatures.retain(|c| c.energy > 0)
```

### 4. Population Cap Enforcement

If population exceeds `max_population`:

```rust
if creatures.len() > max_population:
    # Randomly cull excess creatures
    excess = creatures.len() - max_population
    for _ in 0..excess:
        random_index = random(0, creatures.len())
        creatures.remove(random_index)
```

**Note**: This is a hard cap to prevent unbounded growth and performance degradation.

### 5. Logging and Checkpoints

Every `log_interval_ticks` (default 300 = 10 seconds):
- Print population, average energy, max generation to console

Every `checkpoint_interval_seconds` (default 3600 = 1 hour):
- Save complete simulation state to disk
- Cleanup old checkpoints (keep last 24)

### 6. Termination Check

The simulation ends if:
- Population reaches 0 (extinction)
- User manually stops the process

## Energy System

### Energy Budget

Every creature must balance energy income and expenses:

**Income**:
- Food consumption: +20.0 per food unit eaten (plant or meat)

**Expenses**:
- Metabolism: -0.1 per tick (always)
- Movement: -1.0 per move attempt (even if blocked)
- Reproduction: -50.0 per offspring created
- **Passive healing**: -2.0 per tick (when health < max and energy available)

### Energy Constraints

- **Minimum**: 0.0 (no direct death, but prevents healing and actions)
- **Maximum**: 200.0 (default `max_energy`)
- **Starting**: 100.0 (default `initial_energy`)

**Note**: Unlike previous versions, running out of energy does NOT kill creatures. Only health depletion causes death.

### Energy Strategies

Successful creatures must develop strategies like:

1. **Food seeking**: Move toward cells with food
2. **Energy conservation**: Avoid unnecessary movement while healing
3. **Reproduction timing**: Wait for high energy before reproducing
4. **Combat consideration**: Maintain energy reserves for healing after battles
5. **Exploration vs exploitation**: Balance searching new areas vs staying near food

These strategies **emerge naturally** through evolution - they are not programmed.

## Health System

### Health Pool

Each creature has a separate health pool independent of energy:

**Stats**:
- **Maximum**: 100.0 (default `max_health`)
- **Starting**: 100.0 (full health)
- **Death threshold**: 0.0 (health ≤ 0 causes death)

### Health Regeneration

Creatures passively heal each tick if:
1. Current health < max health
2. Sufficient energy available (2.0 energy cost)

**Healing rate**: +2.0 health per tick (default `health_regen_rate`)
**Energy cost**: -2.0 energy per tick (default `health_regen_energy_cost`)

Example:
```
Tick 1: health=80, energy=100 → health=82, energy=98
Tick 2: health=82, energy=98  → health=84, energy=96
Tick 3: health=84, energy=1.5 → health=84, energy=1.5 (not enough energy)
```

### Health vs Energy

Key distinction:
- **Energy**: Fuels actions (movement, reproduction, healing)
- **Health**: Determines survival (only source of death)

This creates strategic tension: creatures must balance action efficiency with combat readiness.

## Combat Mechanics

### Spatial Collisions

**One creature per cell**: Attempting to move into an occupied cell triggers combat instead of movement.

### Attack Resolution

When creature A moves toward creature B's cell:

1. **Energy deduction**: Attacker pays movement cost (-1.0 energy)
2. **Position**: Attacker remains in original cell (no movement)
3. **Damage**: Target takes damage (default: 20.0 health)
4. **One-sided**: Only attacker deals damage this tick

Example combat sequence:
```
Tick 0:
  Creature A: pos=(5,5), health=100, energy=50
  Creature B: pos=(5,6), health=100, energy=50

Tick 1: A decides MoveDown (toward B)
  - A pays 1.0 energy
  - A stays at (5,5)
  - B takes 20.0 damage
  Result:
    A: pos=(5,5), health=100, energy=49
    B: pos=(5,6), health=80, energy=50

Tick 2: B decides MoveUp (toward A)
  - B pays 1.0 energy
  - B stays at (5,6)
  - A takes 20.0 damage
  Result:
    A: pos=(5,5), health=80, energy=49
    B: pos=(5,6), health=80, energy=49
```

### Death and Meat Food

When a creature's health ≤ 0:

1. **Death**: Creature is removed from simulation
2. **Meat drop**: `floor(remaining_energy / 20)` meat food spawns at death location
3. **Spatial index update**: Cell becomes unoccupied

Example:
```
Creature dies with 45.0 energy remaining
→ Spawns 2 meat food at death position
→ Other creatures can eat this meat for energy
```

### Combat Sensors

Creatures have 8 combat-related sensors (see [Neural Networks](NEURAL_NETWORKS.md)):

**Creature detection** (inputs 5-8):
- Up, Down, Left, Right: 1.0 if creature present, 0.0 otherwise

**Attack detection** (inputs 9-12):
- Up, Down, Left, Right: 1.0 if attacked from that direction last tick, 0.0 otherwise

**Health awareness** (input 13):
- Own health ratio: `current_health / max_health` (0.0 to 1.0)

These sensors enable creatures to:
- Detect nearby threats
- React to being attacked
- Make health-aware decisions (flee when wounded, attack when healthy)

## Food Mechanics

### Food Types

**Plant Food** (Green):
- **Source**: Natural regeneration
- **Regeneration rate**: 0.1% of cells per tick (default)
- **Decay time**: 600 ticks (~20 seconds at 30 TPS)
- **Properties**: Slower decay, stable resource

**Meat Food** (Red):
- **Source**: Dropped when creatures die
- **Amount**: `floor(creature_energy / 20)` units
- **Decay time**: 300 ticks (~10 seconds at 30 TPS, faster than plant)
- **Properties**: High-value but time-limited, encourages scavenging

### Food Properties

- **Regeneration**: Stochastic (random cells gain food each tick)
- **Consumption**: All-or-nothing (entire cell's food eaten at once)
- **Decay**: Age-based removal after type-specific threshold
- **Age tracking**: Food age resets to 0 when more food is added to cell
- **Distribution**: Initially random, then governed by regeneration and decay rates

### Food Decay System

Each food cell tracks its age in ticks:

```rust
# Every tick:
food.age += 1

# Check decay threshold:
if food.is_meat:
    should_decay = (food.age >= meat_decay_ticks)  # 300 ticks
else:
    should_decay = (food.age >= plant_decay_ticks)  # 600 ticks

# Remove if expired:
if should_decay:
    cell = Empty
```

**Strategic implications**:
- **Meat creates urgency**: Must scavenge within ~10 seconds
- **Plant is stable**: Reliable long-term food source
- **Prevents hoarding**: No infinite food accumulation
- **Dynamic equilibrium**: Food availability balances regeneration vs decay

### Food Dynamics

**Early simulation**:
- Food is plentiful (30% initial density)
- Creatures easily find food by random movement
- Population grows rapidly

**Mid simulation**:
- Food becomes scarce as population increases
- Competition intensifies
- Selection pressure for efficient food-finding increases

**Equilibrium**:
- Population stabilizes around carrying capacity
- Food regeneration rate balances consumption rate
- Only efficient creatures survive

### Carrying Capacity

The theoretical maximum sustainable population is:

```
carrying_capacity ≈ (world_cells × food_regen_rate × energy_per_food) / energy_cost_per_tick
```

For default settings:
```
= (10000 × 0.001 × 20.0) / 0.1
= 2000 creatures
```

However, the `max_population` cap (1000) typically limits growth before this is reached.

## Reproduction System

### Requirements

Reproduction is **expensive** and **exclusive**:

- **Energy cost**: 50.0 (half of initial energy)
- **Minimum energy**: 100.0 (must have enough to survive after cost)
- **Cooldown**: 100 ticks (3.33 seconds at 30 TPS)
- **Space requirement**: Adjacent empty cell

### Genetic Inheritance

Offspring receive a **mutated copy** of the parent genome:

```rust
fn mutate(parent_genome: &[u8]) -> Vec<u8> {
    let mut offspring_genome = parent_genome.to_vec();

    for gene in offspring_genome.iter_mut() {
        if random() < mutation_rate {  // default 1%
            *gene = random(0..256);
        }
    }

    offspring_genome
}
```

- **Mutation rate**: 1% per gene (default)
- **Mutation type**: Complete random replacement (not incremental change)
- **Average mutations**: ~1 gene per offspring (100 genes × 0.01 rate)

### Generation Counter

Each creature tracks its generation number:
- **Generation 0**: Original random creatures
- **Generation N**: N reproduction events from generation 0

This metric indicates **evolutionary age** - higher generations have been refined by more selection events.

## Death and Population Control

### Death Causes

1. **Starvation**: Energy drops to 0 or below
2. **Random culling**: Population exceeds cap

### Death Timing

Creatures are removed from the simulation:
- **Within tick**: If energy drops to 0 during their turn
- **End of tick**: During population update phase

### Population Control

The `max_population` parameter serves multiple purposes:

1. **Performance**: Limits computational load
2. **Competition**: Forces creatures to compete for limited space
3. **Stability**: Prevents exponential growth crashes

## Selection Pressures

### Natural Selection Mechanisms

The simulation creates selection pressure through:

1. **Resource competition**: Limited food creates scarcity
2. **Energy requirements**: Constant metabolism demands food-finding
3. **Reproduction threshold**: Only high-energy creatures reproduce
4. **Survival time**: Longer survival = more reproduction opportunities

### What Gets Selected

Traits that increase reproductive success:

- **Food detection**: Creatures that move toward food
- **Energy efficiency**: Creatures that minimize wasteful movement
- **Exploration**: Creatures that search new areas when local food depletes
- **Energy management**: Creatures that reproduce at optimal times

### What Gets Eliminated

Traits that decrease survival:

- **Random movement**: No correlation with food location
- **Edge fixation**: Getting stuck at boundaries
- **Energy waste**: Excessive movement without eating
- **Poor timing**: Reproducing too early (death) or too late (missed opportunity)

## Evolutionary Dynamics

### Phase 1: Random Noise (Generations 0-10)

- Creatures have random, uncorrelated behavior
- Most die quickly without reproducing
- A few get "lucky" and stumble onto food
- Population declines rapidly

### Phase 2: Lucky Survivors (Generations 10-50)

- Surviving lineages have slightly better-than-random food-finding
- Reproduction begins to sustain population
- "Accidental" food-seeking behavior emerges
- Population stabilizes at low level

### Phase 3: Optimization (Generations 50-200)

- Mutations refine existing food-seeking strategies
- Population grows as efficiency increases
- Competition intensifies at carrying capacity
- Diminishing returns on further optimization

### Phase 4: Equilibrium (Generations 200+)

- Population near carrying capacity
- Evolutionary "arms race" against each other
- Incremental improvements via rare beneficial mutations
- Long-term stable population with slow generation increase

### Observing Evolution

You can watch evolution in action by monitoring:

- **Max generation**: Evolutionary time (higher = more refined)
- **Average energy**: Efficiency (higher = better food-finding)
- **Population**: Sustainability (stable = good adaptation)
- **Creature behavior**: Visual patterns in movement

**Tip**: Fast-forward (increase `ticks_per_second`) to see evolutionary trends over thousands of generations.

## Tuning the Simulation

### Making Evolution Faster

- **Increase mutation rate**: More variation per generation
- **Decrease genome size**: Smaller search space
- **Increase food regeneration**: Less selection pressure (faster population growth)
- **Increase ticks per second**: More generations per real-time minute

### Making It Harder

- **Decrease food regeneration**: Harsher environment
- **Increase energy costs**: Tighter energy budget
- **Decrease energy per food**: Less reward per food unit
- **Increase world size**: More searching required

### Making It More Interesting

- **Larger world**: Spatial patterns and migration
- **Lower initial population**: Watch recovery from near-extinction
- **Extreme mutation rate**: Faster evolution but less stability
- **Very low food regen**: Intense competition dynamics

See [Configuration Guide](CONFIGURATION.md) for parameter details.

---

**Next**: Learn how the [Neural Networks](NEURAL_NETWORKS.md) control creature behavior.
