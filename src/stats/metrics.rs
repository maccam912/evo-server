use crate::creature::Creature;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationMetrics {
    pub tick: u64,
    pub population: usize,
    pub total_energy: f64,
    pub avg_energy: f64,
    pub avg_generation: f64,
    pub max_generation: u64,
    pub total_food: u64,
}

impl SimulationMetrics {
    pub fn compute(tick: u64, creatures: &[Creature], total_food: u64) -> Self {
        let population = creatures.len();

        if population == 0 {
            return Self {
                tick,
                population: 0,
                total_energy: 0.0,
                avg_energy: 0.0,
                avg_generation: 0.0,
                max_generation: 0,
                total_food,
            };
        }

        let total_energy: f64 = creatures.iter().map(|c| c.energy()).sum();
        let avg_energy = total_energy / population as f64;

        let total_generation: u64 = creatures.iter().map(|c| c.genome.generation).sum();
        let avg_generation = total_generation as f64 / population as f64;

        let max_generation = creatures
            .iter()
            .map(|c| c.genome.generation)
            .max()
            .unwrap_or(0);

        Self {
            tick,
            population,
            total_energy,
            avg_energy,
            avg_generation,
            max_generation,
            total_food,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creature::genome::Genome;

    #[test]
    fn test_metrics_empty_population() {
        let metrics = SimulationMetrics::compute(100, &[], 50);

        assert_eq!(metrics.tick, 100);
        assert_eq!(metrics.population, 0);
        assert_eq!(metrics.total_energy, 0.0);
    }

    #[test]
    fn test_metrics_with_creatures() {
        let genome1 = Genome::random(100);
        let genome2 = Genome {
            genes: genome1.genes.clone(),
            generation: 5,
        };

        let c1 = Creature::new(1, 0, 0, genome1, 100.0, 200.0, (8, 6, 4));
        let c2 = Creature::new(2, 1, 1, genome2, 150.0, 200.0, (8, 6, 4));

        let creatures = vec![c1, c2];
        let metrics = SimulationMetrics::compute(100, &creatures, 50);

        assert_eq!(metrics.tick, 100);
        assert_eq!(metrics.population, 2);
        assert_eq!(metrics.total_energy, 250.0);
        assert_eq!(metrics.avg_energy, 125.0);
        assert_eq!(metrics.max_generation, 5);
    }
}
