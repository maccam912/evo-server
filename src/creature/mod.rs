pub mod genome;
pub mod metabolism;
pub mod neural_net;
pub mod reproduction;

use genome::Genome;
use metabolism::Metabolism;
use neural_net::{Action, NeuralNetwork};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Creature {
    pub id: u64,
    pub x: usize,
    pub y: usize,
    pub genome: Genome,
    pub brain: NeuralNetwork,
    pub metabolism: Metabolism,
    pub last_reproduce_tick: u64,
    pub age: u64,
}

impl Creature {
    pub fn new(
        id: u64,
        x: usize,
        y: usize,
        genome: Genome,
        initial_energy: f64,
        max_energy: f64,
        nn_config: (usize, usize, usize),
    ) -> Self {
        let brain = NeuralNetwork::from_genome(&genome, nn_config.0, nn_config.1, nn_config.2);
        let metabolism = Metabolism::new(initial_energy, max_energy);

        Self {
            id,
            x,
            y,
            genome,
            brain,
            metabolism,
            last_reproduce_tick: 0,
            age: 0,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.metabolism.is_alive()
    }

    pub fn decide_action(&self, inputs: &[f64]) -> Action {
        self.brain.decide_action(inputs)
    }

    pub fn consume_energy(&mut self, amount: f64) -> bool {
        self.metabolism.consume_energy(amount)
    }

    pub fn gain_energy(&mut self, amount: f64) {
        self.metabolism.gain_energy(amount)
    }

    pub fn energy(&self) -> f64 {
        self.metabolism.energy()
    }

    pub fn can_reproduce(&self, min_energy: f64, current_tick: u64, cooldown: u64) -> bool {
        self.metabolism.can_afford(min_energy) &&
        (current_tick - self.last_reproduce_tick) >= cooldown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creature_creation() {
        let genome = Genome::random(100);
        let creature = Creature::new(1, 10, 20, genome, 100.0, 200.0, (8, 6, 4));

        assert_eq!(creature.id, 1);
        assert_eq!(creature.x, 10);
        assert_eq!(creature.y, 20);
        assert!(creature.is_alive());
        assert_eq!(creature.energy(), 100.0);
    }

    #[test]
    fn test_creature_energy() {
        let genome = Genome::random(100);
        let mut creature = Creature::new(1, 10, 20, genome, 100.0, 200.0, (8, 6, 4));

        creature.consume_energy(30.0);
        assert_eq!(creature.energy(), 70.0);

        creature.gain_energy(50.0);
        assert_eq!(creature.energy(), 120.0);

        creature.consume_energy(200.0);
        assert_eq!(creature.energy(), 0.0);
        assert!(!creature.is_alive());
    }

    #[test]
    fn test_creature_decide_action() {
        let genome = Genome::random(100);
        let creature = Creature::new(1, 10, 20, genome, 100.0, 200.0, (8, 6, 4));

        let inputs = vec![0.5, 0.3, 0.1, 0.9, 0.2, 0.7, 0.4, 0.6];
        let action = creature.decide_action(&inputs);

        assert!(matches!(
            action,
            Action::MoveUp | Action::MoveDown | Action::MoveLeft | Action::MoveRight | Action::Stay
        ));
    }

    #[test]
    fn test_creature_can_reproduce() {
        let genome = Genome::random(100);
        let creature = Creature::new(1, 10, 20, genome, 150.0, 200.0, (8, 6, 4));

        assert!(creature.can_reproduce(100.0, 1000, 100));
        assert!(!creature.can_reproduce(200.0, 1000, 100));
        assert!(!creature.can_reproduce(100.0, 50, 100));
    }
}
