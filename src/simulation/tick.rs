use super::SimulationState;
use crate::config::Config;
use crate::creature::neural_net::Action;
use rand::seq::SliceRandom;

impl SimulationState {
    pub fn tick(&mut self, config: &Config) {
        self.world.regenerate_food(config.world.food_regen_rate, config.world.max_food_per_cell);

        let mut creature_ids: Vec<u64> = self.creatures.keys().copied().collect();
        let mut rng = rand::thread_rng();
        creature_ids.shuffle(&mut rng);

        let mut new_creatures = Vec::new();

        for id in creature_ids {
            let (x, y, action) = {
                let (x, y, energy) = {
                    if let Some(creature) = self.creatures.get_mut(&id) {
                        // Increment age each tick
                        creature.age += 1;

                        creature.consume_energy(config.creature.energy_cost_per_tick);

                        if !creature.is_alive() {
                            continue;
                        }

                        (creature.x, creature.y, creature.energy())
                    } else {
                        continue;
                    }
                };

                let inputs = self.get_sensor_inputs(x, y, energy, config);
                let action = if let Some(creature) = self.creatures.get(&id) {
                    creature.decide_action(&inputs)
                } else {
                    continue;
                };

                (x, y, action)
            };

            match action {
                Action::Stay => {
                    self.try_eat(id, config);
                }
                Action::MoveUp | Action::MoveDown | Action::MoveLeft | Action::MoveRight => {
                    if let Some(creature) = self.creatures.get_mut(&id) {
                        if creature.consume_energy(config.creature.energy_cost_move) {
                            let (dx, dy) = action.to_delta();
                            let new_x = (x as i32 + dx).max(0).min(self.world.width() as i32 - 1) as usize;
                            let new_y = (y as i32 + dy).max(0).min(self.world.height() as i32 - 1) as usize;

                            if let Some(cell) = self.world.get(new_x, new_y) {
                                if cell.is_empty() || cell.is_food() {
                                    creature.x = new_x;
                                    creature.y = new_y;
                                    drop(creature);
                                    self.try_eat(id, config);
                                }
                            }
                        }
                    }
                }
            }

            if let Some(creature) = self.creatures.get(&id) {
                if creature.is_alive() &&
                   creature.can_reproduce(
                       config.creature.min_reproduce_energy,
                       self.tick,
                       config.creature.reproduce_cooldown_ticks,
                   ) &&
                   self.can_spawn_new_creature(config.creature.max_population)
                {
                    let creature_x = creature.x;
                    let creature_y = creature.y;

                    if let Some(target_pos) = self.find_empty_neighbor(creature_x, creature_y) {
                        if let Some(parent) = self.creatures.get_mut(&id) {
                            if let Some(offspring) = parent.reproduce(
                                self.next_creature_id,
                                target_pos.0,
                                target_pos.1,
                                config.evolution.mutation_rate,
                                config.creature.energy_cost_reproduce,
                                config.creature.initial_energy,
                                config.creature.max_energy,
                                (
                                    config.evolution.neural_net_inputs,
                                    config.evolution.neural_net_hidden,
                                    config.evolution.neural_net_outputs,
                                ),
                                self.tick,
                            ) {
                                new_creatures.push(offspring);
                                self.next_creature_id += 1;
                                self.total_births += 1;
                            }
                        }
                    }
                }
            }
        }

        for creature in new_creatures {
            self.creatures.insert(creature.id, creature);
        }

        // Count deaths before removing dead creatures
        let deaths_this_tick = self.creatures.values().filter(|c| !c.is_alive()).count() as u64;
        self.total_deaths += deaths_this_tick;

        self.creatures.retain(|_, c| c.is_alive());

        self.tick += 1;
    }

    fn get_sensor_inputs(&self, x: usize, y: usize, energy: f64, config: &Config) -> Vec<f64> {
        let mut inputs = vec![0.0; config.evolution.neural_net_inputs];

        inputs[0] = energy / config.creature.max_energy;

        let neighbors = self.world.neighbors(x, y);
        let mut food_count = 0;
        let mut empty_count = 0;

        for (nx, ny) in neighbors {
            if let Some(cell) = self.world.get(nx, ny) {
                if cell.is_food() {
                    food_count += 1;
                } else if cell.is_empty() {
                    empty_count += 1;
                }
            }
        }

        if config.evolution.neural_net_inputs > 1 {
            inputs[1] = food_count as f64 / 8.0;
        }
        if config.evolution.neural_net_inputs > 2 {
            inputs[2] = empty_count as f64 / 8.0;
        }

        if let Some(cell) = self.world.get(x, y) {
            if config.evolution.neural_net_inputs > 3 && cell.is_food() {
                inputs[3] = 1.0;
            }
        }

        let nearby_creatures = self.count_nearby_creatures(x, y, 5);
        if config.evolution.neural_net_inputs > 4 {
            inputs[4] = (nearby_creatures as f64 / 25.0).min(1.0);
        }

        inputs
    }

    fn try_eat(&mut self, creature_id: u64, config: &Config) {
        if let Some(creature) = self.creatures.get(&creature_id) {
            let x = creature.x;
            let y = creature.y;

            if let Some(cell) = self.world.get_mut(x, y) {
                if cell.is_food() {
                    let food_amount = cell.consume_food();
                    let energy_gain = food_amount as f64 * config.creature.energy_per_food;

                    if let Some(creature) = self.creatures.get_mut(&creature_id) {
                        creature.gain_energy(energy_gain);
                    }
                }
            }
        }
    }

    fn find_empty_neighbor(&self, x: usize, y: usize) -> Option<(usize, usize)> {
        let empty = self.world.empty_neighbors(x, y);
        if empty.is_empty() {
            return None;
        }

        let mut rng = rand::thread_rng();
        empty.choose(&mut rng).copied()
    }

    fn count_nearby_creatures(&self, x: usize, y: usize, radius: usize) -> usize {
        let x_min = x.saturating_sub(radius);
        let x_max = (x + radius).min(self.world.width() - 1);
        let y_min = y.saturating_sub(radius);
        let y_max = (y + radius).min(self.world.height() - 1);

        self.creatures
            .values()
            .filter(|c| c.x >= x_min && c.x <= x_max && c.y >= y_min && c.y <= y_max)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_tick_increments() {
        let config = Config::default();
        let mut sim = SimulationState::new(&config);

        let initial_tick = sim.tick;
        sim.tick(&config);

        assert_eq!(sim.tick, initial_tick + 1);
    }

    #[test]
    fn test_creatures_consume_energy() {
        let mut config = Config::default();
        config.creature.initial_population = 1;
        config.creature.energy_cost_per_tick = 10.0;
        config.world.initial_food_density = 0.0;

        let mut sim = SimulationState::new(&config);
        let initial_energy = sim.creatures.values().next().unwrap().energy();

        sim.tick(&config);

        if let Some(creature) = sim.creatures.values().next() {
            assert!(creature.energy() < initial_energy);
        }
    }

    #[test]
    fn test_dead_creatures_removed() {
        let mut config = Config::default();
        config.creature.initial_population = 1;
        config.creature.initial_energy = 1.0;
        config.creature.energy_cost_per_tick = 10.0;

        let mut sim = SimulationState::new(&config);

        sim.tick(&config);

        assert_eq!(sim.creatures.len(), 0);
    }
}
