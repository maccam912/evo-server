pub mod tick;

use crate::config::Config;
use crate::creature::{genome::Genome, Creature};
use crate::simulation::tick::Direction;
use crate::stats::SimulationMetrics;
use crate::world::World;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub struct SpatialIndex {
    width: usize,
    height: usize,
    cells: Vec<Option<u64>>,
}

impl SpatialIndex {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![None; width * height],
        }
    }

    #[inline]
    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Option<u64> {
        self.cells[self.idx(x, y)]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, creature_id: u64) {
        let idx = self.idx(x, y);
        self.cells[idx] = Some(creature_id);
    }

    #[inline]
    pub fn clear(&mut self, x: usize, y: usize) {
        let idx = self.idx(x, y);
        self.cells[idx] = None;
    }

    pub fn clear_all(&mut self) {
        self.cells.fill(None);
    }

    pub fn iter_box(
        &self,
        x_min: usize,
        y_min: usize,
        x_max: usize,
        y_max: usize,
    ) -> BoundingBoxIter<'_> {
        BoundingBoxIter {
            index: self,
            x: x_min,
            y: y_min,
            x_min,
            x_max,
            y_max,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

impl Default for SpatialIndex {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            cells: Vec::new(),
        }
    }
}

pub struct BoundingBoxIter<'a> {
    index: &'a SpatialIndex,
    x: usize,
    y: usize,
    x_min: usize,
    x_max: usize,
    y_max: usize,
}

impl<'a> Iterator for BoundingBoxIter<'a> {
    type Item = (usize, usize, u64);

    fn next(&mut self) -> Option<Self::Item> {
        while self.y <= self.y_max {
            if let Some(id) = self.index.get(self.x, self.y) {
                let result = (self.x, self.y, id);
                self.advance();
                return Some(result);
            }
            self.advance();
        }
        None
    }
}

impl<'a> BoundingBoxIter<'a> {
    fn advance(&mut self) {
        if self.x >= self.x_max {
            self.x = self.x_min;
            if self.y >= self.y_max {
                // Move beyond bounds to signal completion
                self.y = self.y_max + 1;
            } else {
                self.y += 1;
            }
        } else {
            self.x += 1;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationState {
    pub world: World,
    pub creatures: HashMap<u64, Creature>,
    #[serde(skip)]
    pub creature_positions: SpatialIndex,
    #[serde(skip)]
    pub attacks_last_tick: HashMap<u64, Vec<Direction>>,
    #[serde(skip)]
    pub recently_dead: VecDeque<Creature>,
    pub tick: u64,
    pub next_creature_id: u64,
    pub total_births: u64,
    pub total_deaths: u64,
}

impl SimulationState {
    pub fn new(config: &Config) -> Self {
        let mut world = World::new(config.world.width, config.world.height);
        world.initialize_food(
            config.world.initial_food_density,
            config.world.max_food_per_cell,
        );

        let mut creatures = HashMap::new();
        let mut creature_positions = SpatialIndex::new(config.world.width, config.world.height);
        let mut rng = rand::thread_rng();

        for id in 0..config.creature.initial_population {
            let x = rng.gen_range(0..config.world.width);
            let y = rng.gen_range(0..config.world.height);

            let genome = Genome::random(config.evolution.genome_size);
            let creature = Creature::new(
                id as u64,
                x,
                y,
                genome,
                config.creature.initial_energy,
                config.creature.max_energy,
                (
                    config.evolution.neural_net_inputs,
                    config.evolution.neural_net_hidden,
                    config.evolution.neural_net_outputs,
                ),
            );

            creature_positions.set(x, y, id as u64);
            creatures.insert(id as u64, creature);
        }

        Self::apply_population_cap(&mut creatures, config.creature.max_population);

        // Rebuild position index after population cap
        let mut creature_positions = SpatialIndex::new(config.world.width, config.world.height);
        for (id, creature) in &creatures {
            creature_positions.set(creature.x, creature.y, *id);
        }

        Self {
            world,
            creatures,
            creature_positions,
            attacks_last_tick: HashMap::new(),
            recently_dead: VecDeque::new(),
            tick: 0,
            next_creature_id: config.creature.initial_population as u64,
            total_births: 0,
            total_deaths: 0,
        }
    }

    pub fn creatures_vec(&self) -> Vec<Creature> {
        self.creatures.values().cloned().collect()
    }

    pub fn metrics(&self) -> SimulationMetrics {
        let creatures = self.creatures_vec();
        let total_food = self.world.total_food();
        SimulationMetrics::compute(
            self.tick,
            &creatures,
            total_food,
            self.total_births,
            self.total_deaths,
        )
    }

    pub fn apply_population_cap(creatures: &mut HashMap<u64, Creature>, max_population: usize) {
        if max_population == 0 || creatures.len() <= max_population {
            return;
        }

        let to_remove = creatures.len() - max_population;
        let mut rng = rand::thread_rng();
        let creature_ids: Vec<u64> = creatures.keys().copied().collect();

        use rand::seq::SliceRandom;
        let mut ids_to_remove = creature_ids;
        ids_to_remove.shuffle(&mut rng);

        for &id in ids_to_remove.iter().take(to_remove) {
            creatures.remove(&id);
        }

        log::info!("Population cap enforced: culled {} creatures", to_remove);
    }

    pub fn can_spawn_new_creature(&self, max_population: usize) -> bool {
        if max_population == 0 {
            return true;
        }
        self.creatures.len() < max_population
    }

    /// Get the creature ID at the specified position, if any
    pub fn creature_at(&self, x: usize, y: usize) -> Option<u64> {
        if x < self.creature_positions.width() && y < self.creature_positions.height() {
            self.creature_positions.get(x, y)
        } else {
            None
        }
    }

    /// Update creature position in spatial index
    pub fn update_creature_position(
        &mut self,
        creature_id: u64,
        old_x: usize,
        old_y: usize,
        new_x: usize,
        new_y: usize,
    ) {
        if old_x < self.creature_positions.width() && old_y < self.creature_positions.height() {
            self.creature_positions.clear(old_x, old_y);
        }
        self.creature_positions.set(new_x, new_y, creature_id);
    }

    /// Add creature to spatial index
    pub fn add_creature_to_position(&mut self, creature_id: u64, x: usize, y: usize) {
        self.creature_positions.set(x, y, creature_id);
    }

    /// Remove creature from spatial index
    pub fn remove_creature_from_position(&mut self, x: usize, y: usize) {
        if x < self.creature_positions.width() && y < self.creature_positions.height() {
            self.creature_positions.clear(x, y);
        }
    }

    /// Rebuild spatial index from creatures (for deserialization)
    pub fn rebuild_spatial_index(&mut self) {
        self.creature_positions = SpatialIndex::new(self.world.width(), self.world.height());
        for (id, creature) in &self.creatures {
            self.creature_positions.set(creature.x, creature.y, *id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_creation() {
        let config = Config::default();
        let sim = SimulationState::new(&config);

        assert_eq!(sim.tick, 0);
        assert_eq!(sim.creatures.len(), config.creature.initial_population);
        assert!(sim.world.total_food() > 0);
    }

    #[test]
    fn test_simulation_metrics() {
        let config = Config::default();
        let sim = SimulationState::new(&config);

        let metrics = sim.metrics();
        assert_eq!(metrics.tick, 0);
        assert_eq!(metrics.population, config.creature.initial_population);
    }

    #[test]
    fn test_spatial_index_basic_operations() {
        let mut index = SpatialIndex::new(4, 3);
        assert_eq!(index.get(1, 1), None);

        index.set(1, 1, 42);
        assert_eq!(index.get(1, 1), Some(42));

        index.clear(1, 1);
        assert_eq!(index.get(1, 1), None);

        index.set(2, 2, 7);
        index.set(3, 2, 8);

        let collected: Vec<_> = index.iter_box(0, 0, 3, 2).collect();
        assert!(collected.contains(&(2, 2, 7)));
        assert!(collected.contains(&(3, 2, 8)));
        assert_eq!(collected.len(), 2);

        index.clear_all();
        assert!(index.iter_box(0, 0, 3, 2).next().is_none());
    }
}
