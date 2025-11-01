use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub world: WorldConfig,
    pub creature: CreatureConfig,
    pub evolution: EvolutionConfig,
    pub combat: CombatConfig,
    pub simulation: SimulationConfig,
    pub checkpoint: CheckpointConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldConfig {
    pub width: usize,
    pub height: usize,
    pub initial_food_density: f64,
    pub food_regen_rate: f64,
    pub max_food_per_cell: u32,
    pub plant_decay_ticks: u32,
    pub meat_decay_ticks: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureConfig {
    pub initial_population: usize,
    pub max_population: usize,
    pub initial_energy: f64,
    pub max_energy: f64,
    pub energy_per_food: f64,
    pub energy_cost_per_tick: f64,
    pub energy_cost_move: f64,
    pub energy_cost_reproduce: f64,
    pub min_reproduce_energy: f64,
    pub reproduce_cooldown_ticks: u64,
    pub max_age_ticks: u64,
    pub energy_cost_sprint: f64,
    pub energy_share_amount: f64,
    pub rest_energy_multiplier: f64,
    pub rest_healing_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionConfig {
    pub mutation_rate: f64,
    pub genome_size: usize,
    pub neural_net_inputs: usize,
    pub neural_net_hidden: usize,
    pub neural_net_outputs: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatConfig {
    pub damage_per_attack: f64,
    pub damage_per_strong_attack: f64,
    pub health_regen_rate: f64,
    pub health_regen_energy_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub ticks_per_second: u64,
    pub log_interval_ticks: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointConfig {
    pub enabled: bool,
    pub interval_seconds: u64,
    pub directory: String,
    pub keep_last_n: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub enabled: bool,
    pub address: String,
    pub port: u16,
    pub update_rate_hz: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            world: WorldConfig {
                width: 300,
                height: 300,
                initial_food_density: 0.3,
                food_regen_rate: 0.001,
                max_food_per_cell: 10,
                plant_decay_ticks: 600, // ~20 seconds at 30 TPS
                meat_decay_ticks: 300,  // ~10 seconds at 30 TPS (faster decay)
            },
            creature: CreatureConfig {
                initial_population: 100,
                max_population: 10000,
                initial_energy: 100.0,
                max_energy: 200.0,
                energy_per_food: 20.0,
                energy_cost_per_tick: 0.1,
                energy_cost_move: 1.0,
                energy_cost_reproduce: 50.0,
                min_reproduce_energy: 100.0,
                reproduce_cooldown_ticks: 100,
                max_age_ticks: 10000,         // ~5.5 minutes at 30 TPS
                energy_cost_sprint: 2.0,      // 2x normal movement cost
                energy_share_amount: 20.0,    // Amount shared per action
                rest_energy_multiplier: 0.5,  // Reduced energy consumption when resting
                rest_healing_multiplier: 2.0, // Boosted healing when resting
            },
            evolution: EvolutionConfig {
                mutation_rate: 0.01,
                genome_size: 400,       // Expanded for ambitious sensor/action set
                neural_net_inputs: 34,  // 16 original + 14 sensors + 4 directional food sensors
                neural_net_hidden: 8,   // Increased for more complexity
                neural_net_outputs: 12, // 4 moves + 8 new actions
            },
            combat: CombatConfig {
                damage_per_attack: 20.0,
                damage_per_strong_attack: 40.0, // 2x normal attack damage
                health_regen_rate: 2.0,
                health_regen_energy_cost: 2.0,
            },
            simulation: SimulationConfig {
                ticks_per_second: 30,
                log_interval_ticks: 300,
            },
            checkpoint: CheckpointConfig {
                enabled: true,
                interval_seconds: 3600,
                directory: "checkpoints".to_string(),
                keep_last_n: 24,
            },
            server: ServerConfig {
                enabled: true,
                address: "0.0.0.0".to_string(),
                port: 8080,
                update_rate_hz: 10,
            },
        }
    }
}

impl Config {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                match serde_json::from_str(&content) {
                    Ok(config) => Ok(config),
                    Err(e) => {
                        // Deserialization error - backup the old file and create new default
                        log::error!(
                            "Failed to parse config file: {}. Creating backup and using defaults.",
                            e
                        );

                        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                        let backup_path = format!("{}.backup.{}", path, timestamp);

                        if let Err(rename_err) = std::fs::rename(path, &backup_path) {
                            log::error!("Failed to backup old config: {}", rename_err);
                        } else {
                            log::info!("Backed up old config to: {}", backup_path);
                        }

                        let default_config = Config::default();
                        if let Err(save_err) = default_config.save_to_file(path) {
                            log::error!("Failed to save new default config: {}", save_err);
                        } else {
                            log::info!("Created new default config at: {}", path);
                        }

                        Ok(default_config)
                    }
                }
            }
            Err(e) => Err(Box::new(e)),
        }
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.world.width, 300);
        assert_eq!(config.world.height, 300);
        assert!(config.checkpoint.enabled);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(config.world.width, deserialized.world.width);
    }
}
