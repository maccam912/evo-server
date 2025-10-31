pub mod cell;
pub mod resources;

use cell::CellType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    width: usize,
    height: usize,
    grid: Vec<CellType>,
}

impl World {
    pub fn new(width: usize, height: usize) -> Self {
        let grid = vec![CellType::Empty; width * height];
        Self {
            width,
            height,
            grid,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&CellType> {
        if x >= self.width || y >= self.height {
            return None;
        }
        self.grid.get(y * self.width + x)
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut CellType> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = y * self.width + x;
        self.grid.get_mut(idx)
    }

    pub fn set(&mut self, x: usize, y: usize, cell: CellType) {
        if let Some(c) = self.get_mut(x, y) {
            *c = cell;
        }
    }

    pub fn neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut result = Vec::new();
        let x = x as i32;
        let y = y as i32;

        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let nx = x + dx;
                let ny = y + dy;

                if nx >= 0 && nx < self.width as i32 && ny >= 0 && ny < self.height as i32 {
                    result.push((nx as usize, ny as usize));
                }
            }
        }

        result
    }

    pub fn empty_neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        self.neighbors(x, y)
            .into_iter()
            .filter(|(nx, ny)| {
                self.get(*nx, *ny)
                    .map(|c| c.is_empty())
                    .unwrap_or(false)
            })
            .collect()
    }

    pub fn count_cells(&self, predicate: impl Fn(&CellType) -> bool) -> usize {
        self.grid.iter().filter(|c| predicate(c)).count()
    }

    pub fn total_food(&self) -> u64 {
        self.grid
            .iter()
            .map(|c| c.food_amount() as u64)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_creation() {
        let world = World::new(10, 10);
        assert_eq!(world.width(), 10);
        assert_eq!(world.height(), 10);
        assert_eq!(world.count_cells(|c| c.is_empty()), 100);
    }

    #[test]
    fn test_world_get_set() {
        let mut world = World::new(10, 10);
        assert!(world.get(5, 5).unwrap().is_empty());

        world.set(5, 5, CellType::Food(10));
        assert_eq!(world.get(5, 5).unwrap().food_amount(), 10);
    }

    #[test]
    fn test_world_neighbors() {
        let world = World::new(10, 10);

        let neighbors = world.neighbors(5, 5);
        assert_eq!(neighbors.len(), 8);

        let neighbors = world.neighbors(0, 0);
        assert_eq!(neighbors.len(), 3);

        let neighbors = world.neighbors(9, 9);
        assert_eq!(neighbors.len(), 3);
    }

    #[test]
    fn test_world_empty_neighbors() {
        let mut world = World::new(10, 10);
        world.set(4, 4, CellType::Food(5));
        world.set(5, 4, CellType::Food(5));

        let empty = world.empty_neighbors(5, 5);
        assert_eq!(empty.len(), 6);
    }

    #[test]
    fn test_world_total_food() {
        let mut world = World::new(10, 10);
        world.set(0, 0, CellType::Food(5));
        world.set(1, 1, CellType::Food(10));

        assert_eq!(world.total_food(), 15);
    }
}
