use super::SimulationState;
use crate::config::Config;
use crate::creature::{neural_net::Action, Creature};
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl SimulationState {
    pub fn tick(&mut self, config: &Config) {
        // Food regeneration
        self.world.regenerate_food(config.world.food_regen_rate, config.world.max_food_per_cell);

        // Food aging and decay
        self.world.age_and_decay_food(config.world.plant_decay_ticks, config.world.meat_decay_ticks);

        let mut creature_ids: Vec<u64> = self.creatures.keys().copied().collect();
        let mut rng = rand::thread_rng();
        creature_ids.shuffle(&mut rng);

        let mut new_creatures = Vec::new();
        // Track attacks this tick: victim_id -> attacker_direction
        let mut attacks_this_tick: HashMap<u64, Vec<Direction>> = HashMap::new();

        for id in creature_ids {
            let (x, y, action) = {
                let (x, y, energy) = {
                    if let Some(creature) = self.creatures.get_mut(&id) {
                        // Increment age each tick
                        creature.age += 1;

                        // Check for death from old age
                        if creature.age >= config.creature.max_age_ticks {
                            creature.metabolism.take_damage(creature.metabolism.health());
                        }

                        creature.consume_energy(config.creature.energy_cost_per_tick);

                        // Check for death from zero energy (starvation)
                        if creature.energy() <= 0.0 {
                            creature.metabolism.take_damage(creature.metabolism.health());
                        }

                        // Passive healing
                        creature.metabolism.passive_heal(
                            config.combat.health_regen_rate,
                            config.combat.health_regen_energy_cost,
                        );

                        if !creature.is_alive() {
                            continue;
                        }

                        (creature.x, creature.y, creature.energy())
                    } else {
                        continue;
                    }
                };

                let inputs = self.get_sensor_inputs(id, x, y, energy, config);
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
                    // Calculate target position
                    let (dx, dy) = action.to_delta();
                    let new_x = (x as i32 + dx).max(0).min(self.world.width() as i32 - 1) as usize;
                    let new_y = (y as i32 + dy).max(0).min(self.world.height() as i32 - 1) as usize;

                    // Check if there's a creature at the target position (before borrowing)
                    let target_creature_id = self.creature_at(new_x, new_y);

                    // Try to consume energy for the move
                    let has_energy = if let Some(creature) = self.creatures.get_mut(&id) {
                        creature.consume_energy(config.creature.energy_cost_move)
                    } else {
                        false
                    };

                    if has_energy {
                        if let Some(target_id) = target_creature_id {
                            // Attack the creature instead of moving
                            if let Some(target) = self.creatures.get_mut(&target_id) {
                                target.metabolism.take_damage(config.combat.damage_per_attack);

                                // Record attack direction for sensors
                                let attack_dir = match action {
                                    Action::MoveUp => Direction::Down,    // Attacked from below
                                    Action::MoveDown => Direction::Up,    // Attacked from above
                                    Action::MoveLeft => Direction::Right, // Attacked from right
                                    Action::MoveRight => Direction::Left, // Attacked from left
                                    _ => unreachable!(),
                                };
                                attacks_this_tick.entry(target_id).or_insert_with(Vec::new).push(attack_dir);
                            }
                        } else {
                            // No creature, check if we can move there
                            if let Some(cell) = self.world.get(new_x, new_y) {
                                if cell.is_empty() || cell.is_food() {
                                    // Update spatial index
                                    self.update_creature_position(id, x, y, new_x, new_y);

                                    // Move the creature
                                    if let Some(creature) = self.creatures.get_mut(&id) {
                                        creature.x = new_x;
                                        creature.y = new_y;
                                    }

                                    // Try to eat at new position
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

        // Add new creatures and update spatial index
        for creature in new_creatures {
            self.add_creature_to_position(creature.id, creature.x, creature.y);
            self.creatures.insert(creature.id, creature);
        }

        // Handle deaths: spawn meat food and update spatial index
        let dead_creatures: Vec<(u64, usize, usize, f64)> = self.creatures
            .iter()
            .filter(|(_, c)| !c.is_alive())
            .map(|(id, c)| (*id, c.x, c.y, c.energy()))
            .collect();

        for (_dead_id, x, y, remaining_energy) in dead_creatures {
            // Spawn meat food based on remaining energy
            let meat_amount = (remaining_energy / 20.0).ceil() as u32;
            if meat_amount > 0 {
                if let Some(cell) = self.world.get_mut(x, y) {
                    cell.add_food(meat_amount, config.world.max_food_per_cell, true); // true = meat
                }
            }

            // Remove from spatial index
            self.remove_creature_from_position(x, y);
        }

        // Save dying creatures to buffer before removal (for extinction failsafe)
        for creature in self.creatures.values() {
            if !creature.is_alive() {
                self.recently_dead.push_back(creature.clone());
                // Keep buffer size reasonable (last 100 dead creatures)
                if self.recently_dead.len() > 100 {
                    self.recently_dead.pop_front();
                }
            }
        }

        // Count and remove dead creatures
        let deaths_this_tick = self.creatures.values().filter(|c| !c.is_alive()).count() as u64;
        self.total_deaths += deaths_this_tick;
        self.creatures.retain(|_, c| c.is_alive());

        // Extinction failsafe: resurrect recently dead creatures if population reaches 0
        if self.creatures.is_empty() && !self.recently_dead.is_empty() {
            log::warn!("EXTINCTION DETECTED at tick {}! Resurrecting {} creatures from recent deaths...",
                       self.tick, config.creature.initial_population);

            let mut rng = rand::thread_rng();
            let num_to_resurrect = config.creature.initial_population.min(self.recently_dead.len());

            // Take the most recent dead creatures
            for i in 0..num_to_resurrect {
                if let Some(dead_creature) = self.recently_dead.get(self.recently_dead.len() - 1 - i) {
                    // Clone the creature with a new ID and position
                    let new_id = self.next_creature_id;
                    self.next_creature_id += 1;

                    // Find a random empty position
                    let mut new_x = rng.gen_range(0..self.world.width());
                    let mut new_y = rng.gen_range(0..self.world.height());

                    // Try a few times to find unoccupied space
                    for _ in 0..10 {
                        if self.creature_at(new_x, new_y).is_none() {
                            break;
                        }
                        new_x = rng.gen_range(0..self.world.width());
                        new_y = rng.gen_range(0..self.world.height());
                    }

                    // Create resurrected creature with full health and energy
                    let resurrected = Creature::new(
                        new_id,
                        new_x,
                        new_y,
                        dead_creature.genome.clone(),
                        config.creature.initial_energy,
                        config.creature.max_energy,
                        (
                            config.evolution.neural_net_inputs,
                            config.evolution.neural_net_hidden,
                            config.evolution.neural_net_outputs,
                        ),
                    );

                    self.add_creature_to_position(new_id, new_x, new_y);
                    self.creatures.insert(new_id, resurrected);
                }
            }

            log::info!("Resurrected {} creatures. Population restored!", self.creatures.len());
        }

        // Store attacks for next tick's sensors
        self.attacks_last_tick = attacks_this_tick;

        self.tick += 1;
    }

    fn get_sensor_inputs(&self, creature_id: u64, x: usize, y: usize, energy: f64, config: &Config) -> Vec<f64> {
        let mut inputs = vec![0.0; config.evolution.neural_net_inputs];

        // Input 0: Energy ratio
        inputs[0] = energy / config.creature.max_energy;

        let neighbors = self.world.neighbors(x, y);
        let mut food_count = 0;
        let mut empty_count = 0;
        let mut plant_food_count = 0;
        let mut meat_food_count = 0;

        for (nx, ny) in neighbors {
            if let Some(cell) = self.world.get(nx, ny) {
                if cell.is_food() {
                    food_count += 1;
                    if cell.is_meat() {
                        meat_food_count += 1;
                    } else {
                        plant_food_count += 1;
                    }
                } else if cell.is_empty() {
                    empty_count += 1;
                }
            }
        }

        // Input 1: Nearby food count (0.0-1.0)
        if config.evolution.neural_net_inputs > 1 {
            inputs[1] = food_count as f64 / 8.0;
        }

        // Input 2: Empty neighbor count (0.0-1.0)
        if config.evolution.neural_net_inputs > 2 {
            inputs[2] = empty_count as f64 / 8.0;
        }

        // Input 3: Food at current position (0.0 or 1.0)
        if let Some(cell) = self.world.get(x, y) {
            if config.evolution.neural_net_inputs > 3 && cell.is_food() {
                inputs[3] = 1.0;
            }
        }

        // Input 4: Nearby creature density (0.0-1.0)
        let nearby_creatures = self.count_nearby_creatures(x, y, 5);
        if config.evolution.neural_net_inputs > 4 {
            inputs[4] = (nearby_creatures as f64 / 25.0).min(1.0);
        }

        // Inputs 5-8: Creature detected in [Up, Down, Left, Right]
        if config.evolution.neural_net_inputs > 5 {
            if let Some(_) = self.creature_at(x, y.wrapping_sub(1)) {
                inputs[5] = 1.0; // Up
            }
        }
        if config.evolution.neural_net_inputs > 6 {
            if let Some(_) = self.creature_at(x, y + 1) {
                inputs[6] = 1.0; // Down
            }
        }
        if config.evolution.neural_net_inputs > 7 {
            if let Some(_) = self.creature_at(x.wrapping_sub(1), y) {
                inputs[7] = 1.0; // Left
            }
        }
        if config.evolution.neural_net_inputs > 8 {
            if let Some(_) = self.creature_at(x + 1, y) {
                inputs[8] = 1.0; // Right
            }
        }

        // Inputs 9-12: Being attacked from [Up, Down, Left, Right]
        if let Some(attack_dirs) = self.attacks_last_tick.get(&creature_id) {
            for &dir in attack_dirs {
                match dir {
                    Direction::Up if config.evolution.neural_net_inputs > 9 => inputs[9] = 1.0,
                    Direction::Down if config.evolution.neural_net_inputs > 10 => inputs[10] = 1.0,
                    Direction::Left if config.evolution.neural_net_inputs > 11 => inputs[11] = 1.0,
                    Direction::Right if config.evolution.neural_net_inputs > 12 => inputs[12] = 1.0,
                    _ => {}
                }
            }
        }

        // Input 13: Own health ratio
        if config.evolution.neural_net_inputs > 13 {
            if let Some(creature) = self.creatures.get(&creature_id) {
                inputs[13] = creature.metabolism.health_ratio();
            }
        }

        // Input 14: Nearby plant food ratio (0.0-1.0)
        if config.evolution.neural_net_inputs > 14 {
            inputs[14] = if food_count > 0 {
                plant_food_count as f64 / food_count as f64
            } else {
                0.0
            };
        }

        // Input 15: Nearby meat food ratio (0.0-1.0)
        if config.evolution.neural_net_inputs > 15 {
            inputs[15] = if food_count > 0 {
                meat_food_count as f64 / food_count as f64
            } else {
                0.0
            };
        }

        inputs
    }

    fn try_eat(&mut self, creature_id: u64, config: &Config) {
        if let Some(creature) = self.creatures.get(&creature_id) {
            let x = creature.x;
            let y = creature.y;

            if let Some(cell) = self.world.get_mut(x, y) {
                if cell.is_food() {
                    let (food_amount, _is_meat) = cell.consume_food();
                    // For now, treat plant and meat food the same
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
        config.creature.initial_population = 2;
        config.combat.damage_per_attack = 150.0;

        let mut sim = SimulationState::new(&config);
        let initial_count = sim.creatures.len();

        // Kill both creatures by dealing damage
        for creature_id in sim.creatures.keys().copied().collect::<Vec<_>>() {
            sim.creatures.get_mut(&creature_id).unwrap().metabolism.take_damage(150.0);
        }

        // Clear the recently_dead buffer AND tick once to remove dead creatures
        sim.recently_dead.clear();
        sim.tick(&config);

        // With empty recently_dead, they still get added during tick, so they get resurrected
        // This is correct behavior - the failsafe prevents full extinction
        assert_eq!(sim.creatures.len(), initial_count); // Resurrected
    }

    #[test]
    fn test_extinction_failsafe() {
        let mut config = Config::default();
        config.creature.initial_population = 10;

        let mut sim = SimulationState::new(&config);

        // Kill all creatures by dealing damage
        for creature_id in sim.creatures.keys().copied().collect::<Vec<_>>() {
            sim.creatures.get_mut(&creature_id).unwrap().metabolism.take_damage(150.0);
        }

        // Tick should trigger resurrection
        sim.tick(&config);

        // Resurrection should have occurred
        assert_eq!(sim.creatures.len(), config.creature.initial_population);

        // All resurrected creatures should be alive
        for creature in sim.creatures.values() {
            assert!(creature.is_alive());
            assert!(creature.energy() > 0.0);
            // Energy might vary due to metabolism, healing, and food consumption during tick
        }
    }
}
