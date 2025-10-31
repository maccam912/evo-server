use super::Creature;
use super::genome::Genome;

impl Creature {
    pub fn reproduce(
        &mut self,
        offspring_id: u64,
        target_x: usize,
        target_y: usize,
        mutation_rate: f64,
        energy_cost: f64,
        initial_energy: f64,
        max_energy: f64,
        nn_config: (usize, usize, usize),
        current_tick: u64,
    ) -> Option<Creature> {
        if !self.metabolism.can_afford(energy_cost) {
            return None;
        }

        self.consume_energy(energy_cost);
        self.last_reproduce_tick = current_tick;

        let offspring_genome = Genome::from_parent(&self.genome, mutation_rate);

        Some(Creature::new(
            offspring_id,
            target_x,
            target_y,
            offspring_genome,
            initial_energy,
            max_energy,
            nn_config,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reproduce() {
        let genome = Genome::random(100);
        let mut parent = Creature::new(1, 10, 20, genome, 150.0, 200.0, (8, 6, 4));

        let offspring = parent.reproduce(2, 11, 20, 0.01, 50.0, 100.0, 200.0, (8, 6, 4), 100);

        assert!(offspring.is_some());
        let child = offspring.unwrap();
        assert_eq!(child.id, 2);
        assert_eq!(child.x, 11);
        assert_eq!(child.y, 20);
        assert_eq!(parent.energy(), 100.0);
        assert_eq!(parent.last_reproduce_tick, 100);
        assert_eq!(child.genome.generation, parent.genome.generation + 1);
    }

    #[test]
    fn test_reproduce_insufficient_energy() {
        let genome = Genome::random(100);
        let mut parent = Creature::new(1, 10, 20, genome, 40.0, 200.0, (8, 6, 4));

        let offspring = parent.reproduce(2, 11, 20, 0.01, 50.0, 100.0, 200.0, (8, 6, 4), 100);

        assert!(offspring.is_none());
        assert_eq!(parent.energy(), 40.0);
    }

    #[test]
    fn test_offspring_has_mutations() {
        let genome = Genome::random(100);
        let mut parent = Creature::new(1, 10, 20, genome.clone(), 150.0, 200.0, (8, 6, 4));

        let offspring = parent.reproduce(2, 11, 20, 0.5, 50.0, 100.0, 200.0, (8, 6, 4), 100);

        assert!(offspring.is_some());
        let child = offspring.unwrap();

        let differences = parent.genome.genes
            .iter()
            .zip(&child.genome.genes)
            .filter(|(a, b)| a != b)
            .count();

        assert!(differences > 0);
    }
}
