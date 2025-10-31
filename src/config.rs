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
                width: 100,
                height: 100,
                initial_food_density: 0.3,
                food_regen_rate: 0.001,
                max_food_per_cell: 10,
                plant_decay_ticks: 600,   // ~20 seconds at 30 TPS
                meat_decay_ticks: 300,    // ~10 seconds at 30 TPS (faster decay)
            },
            creature: CreatureConfig {
                initial_population: 100,
                max_population: 1000,
                initial_energy: 100.0,
                max_energy: 200.0,
                energy_per_food: 20.0,
                energy_cost_per_tick: 0.1,
                energy_cost_move: 1.0,
                energy_cost_reproduce: 50.0,
                min_reproduce_energy: 100.0,
                reproduce_cooldown_ticks: 100,
            },
            evolution: EvolutionConfig {
                mutation_rate: 0.01,
                genome_size: 150,  // Increased for more weights
                neural_net_inputs: 16,  // Expanded for combat sensors
                neural_net_hidden: 6,
                neural_net_outputs: 4,
            },
            combat: CombatConfig {
                damage_per_attack: 20.0,
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
        let content = std::fs::read_to_string(path)?;
        let config = serde_json::from_str(&content)?;
        Ok(config)
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
        assert_eq!(config.world.width, 100);
        assert_eq!(config.world.height, 100);
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
