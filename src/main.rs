mod checkpoint;
mod config;
mod creature;
mod evolution;
mod server;
mod simulation;
mod stats;
mod world;

use clap::Parser;
use config::Config;
use simulation::SimulationState;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration, Instant};

#[derive(Parser, Debug)]
#[command(name = "evo-server")]
#[command(about = "Evolution Simulator Server", long_about = None)]
struct Args {
    #[arg(short, long, default_value = "config.json")]
    config: String,

    #[arg(long)]
    no_checkpoint: bool,

    #[arg(long)]
    no_server: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Args::parse();

    let config = if std::path::Path::new(&args.config).exists() {
        log::info!("Loading config from: {}", args.config);
        Config::load_from_file(&args.config)?
    } else {
        log::info!("Config file not found, using defaults and saving to: {}", args.config);
        let config = Config::default();
        config.save_to_file(&args.config)?;
        config
    };

    log::info!("Initializing simulation...");
    let state = if !args.no_checkpoint && config.checkpoint.enabled {
        if let Ok(Some(loaded_state)) = checkpoint::load_checkpoint(&config) {
            log::info!("Resumed from checkpoint at tick {}", loaded_state.tick);
            loaded_state
        } else {
            SimulationState::new(&config)
        }
    } else {
        SimulationState::new(&config)
    };

    let state = Arc::new(RwLock::new(state));

    if !args.no_server && config.server.enabled {
        let server_state = state.clone();
        let server_config = config.clone();
        tokio::spawn(async move {
            if let Err(e) = server::run_server(server_config, server_state).await {
                log::error!("Server error: {}", e);
            }
        });
        log::info!("WebSocket server started on {}:{}", config.server.address, config.server.port);
    }

    run_simulation(state, config, args.no_checkpoint).await?;

    Ok(())
}

async fn run_simulation(
    state: Arc<RwLock<SimulationState>>,
    config: Config,
    no_checkpoint: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let tick_duration = Duration::from_millis(1000 / config.simulation.ticks_per_second);
    let mut tick_interval = interval(tick_duration);

    let checkpoint_interval = if !no_checkpoint && config.checkpoint.enabled {
        Some(Duration::from_secs(config.checkpoint.interval_seconds))
    } else {
        None
    };

    let mut last_checkpoint = Instant::now();
    let mut last_log = Instant::now();
    let log_interval = Duration::from_secs(10);

    loop {
        tick_interval.tick().await;

        {
            let mut sim_state = state.write().await;
            sim_state.tick(&config);

            if last_log.elapsed() >= log_interval {
                let metrics = sim_state.metrics();
                log::info!(
                    "Tick: {} | Population: {} | Avg Energy: {:.2} | Max Gen: {} | Food: {}",
                    metrics.tick,
                    metrics.population,
                    metrics.avg_energy,
                    metrics.max_generation,
                    metrics.total_food
                );
                last_log = Instant::now();

                if metrics.population == 0 {
                    log::warn!("All creatures have died! Simulation ended.");
                    break;
                }
            }
        }

        if let Some(checkpoint_dur) = checkpoint_interval {
            if last_checkpoint.elapsed() >= checkpoint_dur {
                let sim_state = state.read().await;
                match checkpoint::save_checkpoint(&sim_state, &config) {
                    Ok(path) => {
                        log::info!("Checkpoint saved: {}", path);
                    }
                    Err(e) => {
                        log::error!("Failed to save checkpoint: {}", e);
                    }
                }
                last_checkpoint = Instant::now();
            }
        }
    }

    Ok(())
}
