use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metabolism {
    energy: f64,
    max_energy: f64,
    health: f64,
    max_health: f64,
}

impl Metabolism {
    pub fn new(initial_energy: f64, max_energy: f64) -> Self {
        let max_health = 100.0; // Default max health
        Self {
            energy: initial_energy.min(max_energy),
            max_energy,
            health: max_health,
            max_health,
        }
    }

    pub fn energy(&self) -> f64 {
        self.energy
    }

    pub fn max_energy(&self) -> f64 {
        self.max_energy
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    pub fn consume_energy(&mut self, amount: f64) -> bool {
        if self.energy >= amount {
            self.energy -= amount;
            true
        } else {
            self.energy = 0.0;
            false
        }
    }

    pub fn gain_energy(&mut self, amount: f64) {
        self.energy = (self.energy + amount).min(self.max_energy);
    }

    pub fn can_afford(&self, cost: f64) -> bool {
        self.energy >= cost
    }

    pub fn energy_ratio(&self) -> f64 {
        self.energy / self.max_energy
    }

    pub fn health(&self) -> f64 {
        self.health
    }

    pub fn max_health(&self) -> f64 {
        self.max_health
    }

    pub fn health_ratio(&self) -> f64 {
        self.health / self.max_health
    }

    pub fn take_damage(&mut self, amount: f64) {
        self.health = (self.health - amount).max(0.0);
    }

    pub fn heal(&mut self, amount: f64) {
        self.health = (self.health + amount).min(self.max_health);
    }

    /// Attempt to heal passively. Returns true if healing occurred.
    /// Costs energy_cost per healing_amount
    pub fn passive_heal(&mut self, healing_amount: f64, energy_cost: f64) -> bool {
        if self.health < self.max_health && self.energy >= energy_cost {
            self.heal(healing_amount);
            self.consume_energy(energy_cost);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metabolism_creation() {
        let metabolism = Metabolism::new(100.0, 200.0);
        assert_eq!(metabolism.energy(), 100.0);
        assert_eq!(metabolism.max_energy(), 200.0);
        assert!(metabolism.is_alive());
    }

    #[test]
    fn test_consume_energy() {
        let mut metabolism = Metabolism::new(100.0, 200.0);

        assert!(metabolism.consume_energy(50.0));
        assert_eq!(metabolism.energy(), 50.0);

        assert!(metabolism.consume_energy(50.0));
        assert_eq!(metabolism.energy(), 0.0);
        assert!(!metabolism.is_alive());

        assert!(!metabolism.consume_energy(10.0));
    }

    #[test]
    fn test_gain_energy() {
        let mut metabolism = Metabolism::new(100.0, 200.0);

        metabolism.gain_energy(50.0);
        assert_eq!(metabolism.energy(), 150.0);

        metabolism.gain_energy(100.0);
        assert_eq!(metabolism.energy(), 200.0);
    }

    #[test]
    fn test_can_afford() {
        let metabolism = Metabolism::new(100.0, 200.0);

        assert!(metabolism.can_afford(50.0));
        assert!(metabolism.can_afford(100.0));
        assert!(!metabolism.can_afford(150.0));
    }

    #[test]
    fn test_energy_ratio() {
        let metabolism = Metabolism::new(100.0, 200.0);
        assert_eq!(metabolism.energy_ratio(), 0.5);

        let metabolism2 = Metabolism::new(200.0, 200.0);
        assert_eq!(metabolism2.energy_ratio(), 1.0);
    }
}
