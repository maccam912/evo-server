# UI Guide

This document explains all the visual elements of the evolution simulator's web interface, including color meanings, controls, statistics, and the creature inspector.

## Table of Contents

- [Overview](#overview)
- [Main Canvas](#main-canvas)
- [Color Coding System](#color-coding-system)
- [Controls and Interaction](#controls-and-interaction)
- [Statistics Panel](#statistics-panel)
- [Creature Inspector](#creature-inspector)
- [Connection Status](#connection-status)
- [Visual Elements Reference](#visual-elements-reference)

## Overview

The web UI is located at `http://localhost:8080` (default) and consists of:

1. **Main canvas** - Displays the world, creatures, and grid
2. **Top-right statistics panel** - Real-time metrics
3. **Bottom-left creature inspector** - Details about selected creature
4. **Top-right connection indicator** - Server connection status
5. **Top-left control buttons** - Zoom and view controls

## Main Canvas

### World Visualization

The simulation world is rendered as a 2D grid on an HTML5 canvas:

- **Background**: Black (#0a0a0a) for high contrast
- **Grid lines**: Dark gray (#222222) every 10 cells
- **World border**: Lighter gray (#333333) outline
- **Dynamic viewport**: Pan and zoom to explore large worlds

### Rendering Features

- **Real-time updates**: 10 frames per second (configurable server-side)
- **Hardware accelerated**: Uses canvas 2D context for smooth rendering
- **Smooth scaling**: Bilinear filtering when zoomed in
- **Responsive**: Canvas fills browser window

### What's Rendered

**Currently visible**:
- ✅ Creatures (colored circles)
- ✅ Selected creature highlight
- ✅ Grid lines
- ✅ World boundaries

**Not currently rendered** (🚧 planned):
- 🚧 Food distribution (food cells not shown)
- 🚧 Creature trails/history
- 🚧 Species color coding

## Color Coding System

### Creature Colors (Energy Levels)

Creatures are colored based on their **current energy** using an HSL (Hue, Saturation, Lightness) color scheme:

```javascript
// Energy → Color formula
hue = min(120, (energy / 100.0) * 120)
saturation = 80%
lightness = 50%
```

#### Color Spectrum

| Energy Level | Hue (°) | Color | Visual | Meaning |
|--------------|---------|-------|--------|---------|
| 0-10 | 0-12 | **Red** | 🔴 | Dying (critical) |
| 10-25 | 12-30 | **Orange-Red** | 🟠 | Very low (danger) |
| 25-50 | 30-60 | **Orange** | 🟠 | Low (warning) |
| 50-75 | 60-90 | **Yellow** | 🟡 | Moderate (caution) |
| 75-100 | 90-120 | **Yellow-Green** | 🟢 | Good (safe) |
| 100-200 | 120 | **Green** | 🟢 | Excellent (can reproduce) |

**Key thresholds**:
- **Red** (hue 0°): Below 10 energy - likely to die soon
- **Yellow** (hue 60°): ~50 energy - sustainable but risky
- **Green** (hue 120°): 100+ energy - healthy, can reproduce

#### Why This Color Scheme?

The red → yellow → green gradient:
- **Intuitive**: Matches traffic light colors (danger → caution → safe)
- **Color-blind friendly**: Varying hues distinguishable with most color vision deficiencies
- **High contrast**: Colors stand out against black background
- **Continuous**: Smooth transition shows gradual energy changes

### Selection Highlighting

When a creature is clicked:
- **Yellow outline**: 4-pixel stroke around selected creature
- **Outline color**: Yellow (#ffff00) for high visibility
- **Persists**: Remains until another creature is selected or deselected

### Connection Status Colors

The connection indicator in the top-right corner shows:

| Status | Color | Animation | Meaning |
|--------|-------|-----------|---------|
| Connected | Green (#22c55e) | Solid glow | WebSocket active |
| Connecting | Orange (#f97316) | Pulsing | Attempting connection |
| Disconnected | Red (#ef4444) | Solid | Connection lost |

## Controls and Interaction

### Mouse Controls

#### Pan (Click and Drag)

```
Action: Click and hold left mouse button, then drag
Effect: Moves viewport around the world
```

**Implementation details**:
- Drag threshold: Minimum movement to prevent accidental drags
- Smooth panning: Canvas offset updates in real-time
- No bounds: Can pan beyond world edges (centering is preserved)

#### Zoom (Mouse Wheel)

```
Action: Scroll mouse wheel up/down
Effect: Zoom in/out centered on mouse position
```

**Zoom parameters**:
- **Factor**: 1.1× per wheel notch (10% zoom per scroll)
- **Minimum zoom**: 0.1× (view very large areas)
- **Maximum zoom**: 10× (view individual cells in detail)
- **Center point**: Zooms toward cursor position

#### Select Creature (Click)

```
Action: Click on a creature
Effect: Shows creature inspector panel with details
```

**Selection behavior**:
- Highlights selected creature with yellow outline
- Opens inspector panel (slides up from bottom-left)
- Click background to deselect (closes inspector)

### Button Controls

Located in top-left corner:

#### Zoom In Button [+]

```
Action: Click the [+] button
Effect: Zoom in by 1.2× centered on viewport center
```

Equivalent to 2 mouse wheel scrolls forward.

#### Zoom Out Button [-]

```
Action: Click the [-] button
Effect: Zoom out by 0.8× centered on viewport center
```

Equivalent to 2 mouse wheel scrolls backward.

#### Reset View Button [↺]

```
Action: Click the [↺] reset button
Effect: Returns to default zoom and centers the world
```

**Reset behavior**:
- Zoom: 1.0× (100% scale)
- Position: World centered in viewport
- Selection: Preserved (creature remains selected)

### Keyboard Shortcuts

**Currently not implemented** (🚧 potential feature):
- Arrow keys for panning
- +/- keys for zooming
- Space bar for pause/resume
- R for reset view

## Statistics Panel

Located in top-right corner, shows real-time simulation metrics:

### Population

```
Population: 456
```

**Meaning**: Current number of living creatures

**Typical values**:
- **0-50**: Near extinction (rare unless harsh settings)
- **50-200**: Stable small population
- **200-800**: Healthy equilibrium
- **800-1000**: Near carrying capacity
- **1000**: Population cap reached (culling active)

**What to watch for**:
- Declining population: Environment too harsh or creatures poorly adapted
- Growing population: Abundant food or newly evolved efficiency
- Stable population: Equilibrium between births and deaths

### Generation

```
Generation: 45
```

**Meaning**: Maximum generation number reached by any creature

**Interpretation**:
- **0-10**: Early stage, random behavior dominates
- **10-50**: Learning phase, food-seeking emerges
- **50-200**: Optimization phase, refinement ongoing
- **200+**: Mature population, near-optimal behavior

**Important**: This is the **maximum** generation, not the average. The average generation is also tracked but represents the evolutionary age of the current population.

### Tick

```
Tick: 12,345
```

**Meaning**: Simulation time steps elapsed since start (or last checkpoint load)

**Time conversion** (at default 30 TPS):
- 30 ticks = 1 second
- 1,800 ticks = 1 minute
- 108,000 ticks = 1 hour

**Use cases**:
- Measure experiment duration
- Correlate events with specific time points
- Estimate evolutionary speed (generations per tick)

### Average Energy

```
Avg Energy: 87.3
```

**Meaning**: Mean energy across all living creatures

**Interpretation**:
- **0-50**: Population struggling (poor food access)
- **50-100**: Moderate health (sustainable but competitive)
- **100-150**: High health (efficient food-finding)
- **150-200**: Excellent health (abundant food)

**Trends to observe**:
- Rising: Creatures evolving better strategies
- Falling: Resource depletion or population overgrowth
- Stable: Equilibrium reached

### Total Energy

```
Total Energy: 40,000
```

**Meaning**: Sum of all creature energy in the simulation

**Use cases**:
- Ecosystem vitality indicator
- Correlates with population size
- Shows total "stored resources" in creature form

**Formula**: `total_energy = sum(creature.energy for all creatures)`

### World Size

```
World Size: 100 × 100
```

**Meaning**: Grid dimensions (width × height)

**Static value**: Configured at startup, doesn't change during simulation

**Total cells**: width × height (e.g., 100 × 100 = 10,000 cells)

### Total Food

```
Total Food: 3,000
```

**Meaning**: Sum of food units across all cells in the world

**Interpretation**:
- **Low** (< 1% of cells): Severe scarcity, high competition
- **Medium** (1-5% of cells): Moderate availability, selective pressure
- **High** (> 5% of cells): Abundant food, population can grow

**Dynamics**:
- Food regenerates at `food_regen_rate` per tick
- Food consumed instantly when creatures eat
- Balance determines carrying capacity

### 🚧 Incomplete Statistics

These appear in the UI but are **not currently tracked**:

#### Births (🚧 Not Implemented)

```
Births: ---
```

**Intended meaning**: Total reproduction events since start

**Status**: UI placeholder exists, but server doesn't track birth events

#### Deaths (🚧 Not Implemented)

```
Deaths: ---
```

**Intended meaning**: Total creature deaths since start

**Status**: UI placeholder exists, but server doesn't track death events

#### Average Age (🚧 Not Implemented)

```
Avg Age: ---
```

**Intended meaning**: Mean creature age (in ticks or generations)

**Status**: UI placeholder exists, but server doesn't calculate or send this metric

## Creature Inspector

Located in bottom-left corner, appears when a creature is selected.

### Basic Information (✅ Implemented)

#### ID

```
ID: 42
```

**Meaning**: Unique identifier for this creature

**Properties**:
- Assigned at birth
- Never reused (incrementing counter)
- Persists in checkpoints

#### Position

```
Position: (45, 67)
```

**Meaning**: Current (x, y) coordinates in the world grid

**Coordinate system**:
- Origin (0, 0): Top-left corner
- X-axis: Increases rightward
- Y-axis: Increases downward

#### Energy

```
Energy: 123.4
```

**Meaning**: Current energy level

**Range**: 0.0 to `max_energy` (default 200.0)

**Critical values**:
- ≤ 0: Death
- < 50: Danger zone (can't reproduce)
- ≥ 100: Can reproduce (if cooldown elapsed)

#### Generation

```
Generation: 12
```

**Meaning**: Number of reproduction events from generation 0

**Interpretation**:
- **0**: Original random creature (rare after early simulation)
- **1-10**: Early descendants
- **10-50**: Evolved creatures
- **50+**: Highly refined lineages

### 🚧 Incomplete Features

These sections appear in the inspector but **don't show data**:

#### Genome (🚧 Not Implemented)

**Intended display**: Visual representation of the creature's 100-byte genome

**Status**:
- UI section exists
- Server doesn't send genome data in `CreatureSnapshot`
- Would require protocol extension to include `genome: Vec<u8>`

**Potential visualization**:
```
Genome (100 bytes):
█▓▒░█▓▒░█▓▒░█▓▒░█▓▒░ (byte values as grayscale or color)
```

#### Neural Network (🚧 Not Implemented)

**Intended display**: Visualization of neural network weights and structure

**Status**:
- UI section exists
- Server doesn't send brain data
- Would require protocol extension to include weight matrices

**Potential visualization**:
```
Neural Network (8-6-4):
Input → Hidden weights: [0.23, -0.45, 0.89, ...]
Hidden → Output weights: [0.12, 0.67, -0.34, ...]
```

### Using the Inspector

**To open**:
1. Click on any creature in the canvas
2. Inspector slides up from bottom-left
3. Selected creature gets yellow outline

**To close**:
1. Click on empty space in the canvas
2. Click on same creature again (toggles off)
3. Inspector slides down

**Selection persistence**:
- Selection stays active even if creature moves
- If selected creature dies, inspector closes automatically
- Zooming/panning preserves selection

## Connection Status

Located in top-right corner above statistics panel.

### Status Indicator

A colored circle with text:

```
● Connected
```

### Status Types

#### Connected (Green)

```
● Connected
```

**Meaning**: WebSocket connection active, receiving updates

**Visual**: Solid green circle with subtle glow effect

**Behavior**: Updates streaming at 10 Hz (default)

#### Connecting (Orange)

```
● Connecting...
```

**Meaning**: Attempting to establish WebSocket connection

**Visual**: Pulsing orange circle (animated)

**Typical duration**: 1-3 seconds on startup

**Causes**:
- Initial page load
- Server restart
- Network interruption (automatic reconnection attempt)

#### Disconnected (Red)

```
● Disconnected
```

**Meaning**: WebSocket connection lost

**Visual**: Solid red circle

**Behavior**:
- Canvas freezes (no updates)
- Statistics stop updating
- Automatic reconnection attempts every 3 seconds

**Common causes**:
- Server stopped or crashed
- Network connection lost
- Server overloaded (not responding)

### Reconnection Behavior

The client automatically attempts to reconnect when disconnected:

```
1. Detect disconnection
2. Wait 3 seconds
3. Attempt reconnection
4. If fails, repeat from step 2
```

**User action**: Usually none required - just wait for reconnection

## Visual Elements Reference

### Colors Cheat Sheet

| Element | Color | Hex Code | Usage |
|---------|-------|----------|-------|
| Background | Black | #0a0a0a | Canvas background |
| Grid lines | Dark gray | #222222 | 10-cell grid lines |
| Border | Gray | #333333 | World outline |
| Creature (0 energy) | Red | HSL(0°, 80%, 50%) | Dying |
| Creature (50 energy) | Yellow | HSL(60°, 80%, 50%) | Moderate |
| Creature (100+ energy) | Green | HSL(120°, 80%, 50%) | Healthy |
| Selection outline | Yellow | #ffff00 | Selected creature |
| Connected | Green | #22c55e | Status indicator |
| Connecting | Orange | #f97316 | Status indicator |
| Disconnected | Red | #ef4444 | Status indicator |

### Size and Scale

| Element | Size | Notes |
|---------|------|-------|
| Creature radius | 60% of cell size | Scales with zoom |
| Selection outline | 4 pixels | Fixed width |
| Grid lines | 1 pixel | Fixed width |
| World border | 2 pixels | Fixed width |

### Rendering Order (Z-Index)

Layers rendered from back to front:

1. **Background** - Black fill
2. **Grid lines** - 10-cell intervals
3. **World border** - Perimeter outline
4. **Creatures** - Colored circles
5. **Selection outline** - Yellow ring (if applicable)

### Performance Notes

**Optimization features**:
- Only visible creatures are rendered (viewport culling)
- Canvas uses hardware acceleration where available
- Grid lines use efficient line drawing
- Updates throttled to 10 FPS (configurable server-side)

**Performance tips**:
- Larger worlds require more rendering time
- Higher populations increase draw calls
- Zooming out draws more creatures (slower)
- Zooming in draws fewer creatures (faster)

## Accessibility

### Current Support

- ✅ High contrast colors (black background, bright creatures)
- ✅ Large click targets (creatures scale with zoom)
- ✅ Clear status indicators (color + text)

### 🚧 Potential Improvements

- Keyboard navigation support
- Screen reader compatibility
- Alternative color schemes (protanopia, deuteranopia modes)
- Adjustable font sizes for statistics
- Texture patterns in addition to colors (for color blindness)

## Troubleshooting

### Creatures not moving

**Cause**: Disconnected from server or server paused

**Solution**: Check connection status indicator (top-right)

### Canvas is black/empty

**Causes**:
1. Simulation not started yet
2. All creatures died (extinction)
3. Connection not established

**Solution**: Check statistics panel for population count and connection status

### Can't select creatures

**Cause**: Creatures too small (zoomed out too far)

**Solution**: Zoom in using mouse wheel or [+] button

### Inspector shows wrong creature

**Cause**: Creature ID persists even if creature dies and new creature spawned at same ID

**Solution**: This is expected behavior - click another creature to update

---

**Next**: Learn about [Configuration options](CONFIGURATION.md) for customizing the simulation.
