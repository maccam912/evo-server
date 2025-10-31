use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellType {
    Empty,
    Food { amount: u32, is_meat: bool },
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
            CellType::Food { amount: current, is_meat: current_is_meat } => {
                // If trying to add different food type, replace it
                if *current_is_meat != is_meat {
                    *self = CellType::Food { amount: amount.min(max), is_meat };
                } else {
                    *current = (*current + amount).min(max);
                }
            }
            CellType::Empty => {
                *self = CellType::Food { amount: amount.min(max), is_meat };
            }
        }
    }

    /// Consumes food and returns (amount, is_meat)
    pub fn consume_food(&mut self) -> (u32, bool) {
        match self {
            CellType::Food { amount, is_meat } => {
                let consumed = (*amount, *is_meat);
                *self = CellType::Empty;
                consumed
            }
            CellType::Empty => (0, false),
        }
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
        let cell = CellType::Food { amount: 5, is_meat: false };
        assert!(!cell.is_empty());
        assert!(cell.is_food());
        assert_eq!(cell.food_amount(), 5);
        assert!(!cell.is_meat());

        let meat_cell = CellType::Food { amount: 3, is_meat: true };
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
        let mut cell = CellType::Food { amount: 5, is_meat: false };
        let (amount, is_meat) = cell.consume_food();
        assert_eq!(amount, 5);
        assert!(!is_meat);
        assert!(cell.is_empty());

        let (amount_again, _) = cell.consume_food();
        assert_eq!(amount_again, 0);

        // Test meat consumption
        let mut meat_cell = CellType::Food { amount: 3, is_meat: true };
        let (meat_amount, meat_flag) = meat_cell.consume_food();
        assert_eq!(meat_amount, 3);
        assert!(meat_flag);
    }
}
