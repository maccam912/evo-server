use super::World;
use rand::Rng;

impl World {
    pub fn initialize_food(&mut self, density: f64, max_per_cell: u32) {
        let mut rng = rand::thread_rng();

        for y in 0..self.height() {
            for x in 0..self.width() {
                if rng.gen::<f64>() < density {
                    let amount = rng.gen_range(1..=max_per_cell);
                    if let Some(cell) = self.get_mut(x, y) {
                        cell.add_food(amount, max_per_cell, false); // Plant food
                    }
                }
            }
        }
    }

    pub fn regenerate_food(&mut self, rate: f64, max_per_cell: u32) {
        // OPTIMIZATION: Instead of checking all cells (width * height iterations),
        // calculate expected number of cells to regenerate and randomly sample them.
        // At 300x300 with rate=0.001: checks ~90 cells instead of 90,000 (1000x faster)
        let total_cells = self.width() * self.height();
        let num_cells_to_regen = ((total_cells as f64) * rate).round() as usize;

        let mut rng = rand::thread_rng();
        for _ in 0..num_cells_to_regen {
            let x = rng.gen_range(0..self.width());
            let y = rng.gen_range(0..self.height());
            if let Some(cell) = self.get_mut(x, y) {
                cell.add_food(1, max_per_cell, false); // Plant food regeneration
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_food() {
        let mut world = World::new(100, 100);
        world.initialize_food(0.3, 10);

        let food_count = world.count_cells(|c| c.is_food());
        assert!(food_count > 0);
        assert!(food_count < 10000);

        let avg_food = food_count as f64 / 10000.0;
        assert!(avg_food > 0.2 && avg_food < 0.4);
    }

    #[test]
    fn test_regenerate_food() {
        let mut world = World::new(100, 100);
        world.initialize_food(0.1, 10);

        let initial_food = world.total_food();

        for _ in 0..100 {
            world.regenerate_food(0.01, 10);
        }

        let final_food = world.total_food();
        assert!(final_food >= initial_food);
    }

    #[test]
    fn test_food_cap() {
        let mut world = World::new(10, 10);

        for _ in 0..1000 {
            world.regenerate_food(1.0, 5);
        }

        for y in 0..10 {
            for x in 0..10 {
                if let Some(cell) = world.get(x, y) {
                    assert!(cell.food_amount() <= 5);
                }
            }
        }
    }
}
