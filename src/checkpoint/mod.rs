pub mod storage;

use crate::config::Config;
use crate::simulation::SimulationState;
use std::fs;
use std::path::Path;

pub fn save_checkpoint(state: &SimulationState, config: &Config) -> Result<String, Box<dyn std::error::Error>> {
    let dir = Path::new(&config.checkpoint.directory);
    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }

    let checkpoint_path = storage::create_checkpoint_path(&config.checkpoint.directory);
    let json = serde_json::to_string_pretty(state)?;
    fs::write(&checkpoint_path, json)?;

    storage::cleanup_old_checkpoints(&config.checkpoint.directory, config.checkpoint.keep_last_n)?;

    Ok(checkpoint_path.to_string_lossy().to_string())
}

pub fn load_checkpoint(config: &Config) -> Result<Option<SimulationState>, Box<dyn std::error::Error>> {
    if let Some(checkpoint_path) = storage::find_latest_checkpoint(&config.checkpoint.directory) {
        log::info!("Loading checkpoint from: {:?}", checkpoint_path);

        match fs::read_to_string(&checkpoint_path) {
            Ok(content) => {
                match serde_json::from_str::<SimulationState>(&content) {
                    Ok(mut state) => {
                        // Rebuild spatial index since it's not serialized
                        state.rebuild_spatial_index();
                        Ok(Some(state))
                    }
                    Err(e) => {
                        // Deserialization error - backup the old checkpoint and start fresh
                        log::error!("Failed to parse checkpoint file: {}. Creating backup and starting fresh.", e);

                        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                        let backup_path = format!("{}.backup.{}", checkpoint_path.display(), timestamp);

                        if let Err(rename_err) = fs::rename(&checkpoint_path, &backup_path) {
                            log::error!("Failed to backup old checkpoint: {}", rename_err);
                        } else {
                            log::info!("Backed up old checkpoint to: {}", backup_path);
                        }

                        Ok(None)
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to read checkpoint file: {}", e);
                Ok(None)
            }
        }
    } else {
        log::info!("No checkpoint found");
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_load_checkpoint() {
        let config = Config {
            checkpoint: crate::config::CheckpointConfig {
                enabled: true,
                interval_seconds: 3600,
                directory: "test_checkpoints_temp".to_string(),
                keep_last_n: 5,
            },
            ..Config::default()
        };

        let state = SimulationState::new(&config);

        let path = save_checkpoint(&state, &config).unwrap();
        assert!(Path::new(&path).exists());

        let loaded_state = load_checkpoint(&config).unwrap();
        assert!(loaded_state.is_some());

        let loaded = loaded_state.unwrap();
        assert_eq!(loaded.tick, state.tick);
        assert_eq!(loaded.creatures.len(), state.creatures.len());

        let _ = fs::remove_dir_all("test_checkpoints_temp");
    }
}
