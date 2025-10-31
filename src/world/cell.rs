use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellType {
    Empty,
    Food(u32),
}

impl CellType {
    pub fn is_empty(&self) -> bool {
        matches!(self, CellType::Empty)
    }

    pub fn is_food(&self) -> bool {
        matches!(self, CellType::Food(_))
    }

    pub fn food_amount(&self) -> u32 {
        match self {
            CellType::Food(amount) => *amount,
            _ => 0,
        }
    }

    pub fn add_food(&mut self, amount: u32, max: u32) {
        match self {
            CellType::Food(current) => {
                *current = (*current + amount).min(max);
            }
            CellType::Empty => {
                *self = CellType::Food(amount.min(max));
            }
        }
    }

    pub fn consume_food(&mut self) -> u32 {
        match self {
            CellType::Food(amount) => {
                let consumed = *amount;
                *self = CellType::Empty;
                consumed
            }
            CellType::Empty => 0,
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
        let cell = CellType::Food(5);
        assert!(!cell.is_empty());
        assert!(cell.is_food());
        assert_eq!(cell.food_amount(), 5);
    }

    #[test]
    fn test_add_food() {
        let mut cell = CellType::Empty;
        cell.add_food(5, 10);
        assert_eq!(cell.food_amount(), 5);

        cell.add_food(3, 10);
        assert_eq!(cell.food_amount(), 8);

        cell.add_food(5, 10);
        assert_eq!(cell.food_amount(), 10);
    }

    #[test]
    fn test_consume_food() {
        let mut cell = CellType::Food(5);
        let consumed = cell.consume_food();
        assert_eq!(consumed, 5);
        assert!(cell.is_empty());

        let consumed_again = cell.consume_food();
        assert_eq!(consumed_again, 0);
    }
}
