use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellType {
    Empty,
    Food { amount: u32, is_meat: bool, age: u32 },
}

impl CellType {
    pub fn is_empty(&self) -> bool {
        matches!(self, CellType::Empty)
    }

    pub fn is_food(&self) -> bool {
        matches!(self, CellType::Food { .. })
    }

    pub fn food_amount(&self) -> u32 {
        match self {
            CellType::Food { amount, .. } => *amount,
            _ => 0,
        }
    }

    pub fn is_meat(&self) -> bool {
        match self {
            CellType::Food { is_meat, .. } => *is_meat,
            _ => false,
        }
    }

    pub fn add_food(&mut self, amount: u32, max: u32, is_meat: bool) {
        match self {
            CellType::Food { amount: current, is_meat: current_is_meat, age } => {
                // If trying to add different food type, replace it
                if *current_is_meat != is_meat {
                    *self = CellType::Food { amount: amount.min(max), is_meat, age: 0 };
                } else {
                    *current = (*current + amount).min(max);
                    // Reset age when adding more food
                    *age = 0;
                }
            }
            CellType::Empty => {
                *self = CellType::Food { amount: amount.min(max), is_meat, age: 0 };
            }
        }
    }

    /// Consumes food and returns (amount, is_meat)
    pub fn consume_food(&mut self) -> (u32, bool) {
        match self {
            CellType::Food { amount, is_meat, .. } => {
                let consumed = (*amount, *is_meat);
                *self = CellType::Empty;
                consumed
            }
            CellType::Empty => (0, false),
        }
    }

    /// Increments food age by 1 tick
    pub fn age_food(&mut self) {
        if let CellType::Food { age, .. } = self {
            *age += 1;
        }
    }

    /// Checks if food should decay based on age thresholds
    pub fn should_decay(&self, plant_decay_ticks: u32, meat_decay_ticks: u32) -> bool {
        match self {
            CellType::Food { is_meat, age, .. } => {
                if *is_meat {
                    *age >= meat_decay_ticks
                } else {
                    *age >= plant_decay_ticks
                }
            }
            CellType::Empty => false,
        }
    }

    /// Decays food, making the cell empty
    pub fn decay(&mut self) {
        *self = CellType::Empty;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_type_empty() {
        let cell = CellType::Empty;
        assert!(cell.is_empty());
        assert!(!cell.is_food());
        assert_eq!(cell.food_amount(), 0);
    }

    #[test]
    fn test_cell_type_food() {
        let cell = CellType::Food { amount: 5, is_meat: false, age: 0 };
        assert!(!cell.is_empty());
        assert!(cell.is_food());
        assert_eq!(cell.food_amount(), 5);
        assert!(!cell.is_meat());

        let meat_cell = CellType::Food { amount: 3, is_meat: true, age: 0 };
        assert!(meat_cell.is_food());
        assert!(meat_cell.is_meat());
        assert_eq!(meat_cell.food_amount(), 3);
    }

    #[test]
    fn test_add_food() {
        let mut cell = CellType::Empty;
        cell.add_food(5, 10, false);
        assert_eq!(cell.food_amount(), 5);
        assert!(!cell.is_meat());

        cell.add_food(3, 10, false);
        assert_eq!(cell.food_amount(), 8);

        cell.add_food(5, 10, false);
        assert_eq!(cell.food_amount(), 10);

        // Test meat food
        let mut meat_cell = CellType::Empty;
        meat_cell.add_food(7, 10, true);
        assert_eq!(meat_cell.food_amount(), 7);
        assert!(meat_cell.is_meat());
    }

    #[test]
    fn test_consume_food() {
        let mut cell = CellType::Food { amount: 5, is_meat: false, age: 10 };
        let (amount, is_meat) = cell.consume_food();
        assert_eq!(amount, 5);
        assert!(!is_meat);
        assert!(cell.is_empty());

        let (amount_again, _) = cell.consume_food();
        assert_eq!(amount_again, 0);

        // Test meat consumption
        let mut meat_cell = CellType::Food { amount: 3, is_meat: true, age: 5 };
        let (meat_amount, meat_flag) = meat_cell.consume_food();
        assert_eq!(meat_amount, 3);
        assert!(meat_flag);
    }

    #[test]
    fn test_food_aging_and_decay() {
        let mut cell = CellType::Food { amount: 5, is_meat: false, age: 0 };

        // Age the food
        cell.age_food();
        if let CellType::Food { age, .. } = cell {
            assert_eq!(age, 1);
        }

        // Test decay check for plant food
        assert!(!cell.should_decay(100, 50)); // Not old enough

        // Age to decay threshold
        for _ in 0..99 {
            cell.age_food();
        }
        assert!(cell.should_decay(100, 50)); // Now should decay

        // Test meat decay (faster)
        let mut meat = CellType::Food { amount: 3, is_meat: true, age: 49 };
        assert!(!meat.should_decay(100, 50)); // Not old enough
        meat.age_food();
        assert!(meat.should_decay(100, 50)); // Now should decay

        // Test actual decay
        meat.decay();
        assert!(meat.is_empty());
    }
}
