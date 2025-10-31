pub mod tick;

use crate::config::Config;
use crate::creature::{genome::Genome, Creature};
use crate::stats::SimulationMetrics;
use crate::world::World;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationState {
    pub world: World,
    pub creatures: HashMap<u64, Creature>,
    pub tick: u64,
    pub next_creature_id: u64,
    pub total_births: u64,
    pub total_deaths: u64,
}

impl SimulationState {
    pub fn new(config: &Config) -> Self {
        let mut world = World::new(config.world.width, config.world.height);
        world.initialize_food(config.world.initial_food_density, config.world.max_food_per_cell);

        let mut creatures = HashMap::new();
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

            creatures.insert(id as u64, creature);
        }

        Self::apply_population_cap(&mut creatures, config.creature.max_population);

        Self {
            world,
            creatures,
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
        SimulationMetrics::compute(self.tick, &creatures, total_food, self.total_births, self.total_deaths)
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
}
