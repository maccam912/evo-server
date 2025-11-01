pub mod protocol;
pub mod state_stream;

use crate::config::Config;
use crate::simulation::SimulationState;
use axum::{
    extract::{ws::WebSocket, State as AxumState, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use futures_util::{SinkExt, StreamExt};
use protocol::{ClientMessage, ServerMessage};
use state_stream::StateStream;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tower_http::services::ServeDir;

#[derive(Clone)]
struct AppState {
    stream: StateStream,
    config: Config,
}

pub async fn run_server(
    config: Config,
    state: Arc<RwLock<SimulationState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{}:{}", config.server.address, config.server.port);
    let stream = StateStream::new(state.clone());

    let app_state = AppState {
        stream,
        config: config.clone(),
    };

    // Build the router
    let app = Router::new()
        // WebSocket endpoint
        .route("/ws", get(websocket_handler))
        // Restart endpoint
        .route("/api/restart", post(restart_handler))
        // Serve static files from the "static" directory
        .nest_service("/", ServeDir::new("static"))
        .with_state(app_state);

    log::info!("HTTP server with WebSocket listening on: {}", addr);
    log::info!("Static files served from: ./static/");
    log::info!("WebSocket endpoint: ws://{}/ws", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    AxumState(state): AxumState<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

async fn restart_handler() -> impl IntoResponse {
    log::warn!("Restart requested via API - shutting down process");

    // Spawn a task to exit the process after a short delay
    // This allows the HTTP response to be sent before the process exits
    tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        std::process::exit(0);
    });

    (StatusCode::OK, "Server restart initiated")
}

async fn handle_websocket(socket: WebSocket, app_state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut update_interval = interval(Duration::from_millis(
        1000 / app_state.config.server.update_rate_hz,
    ));
    let mut subscribed_creature_id: Option<u64> = None;

    loop {
        tokio::select! {
            _ = update_interval.tick() => {
                let state = app_state.stream.get_state().await;
                let metrics = state.metrics();
                let creatures = state.creatures_vec();

                let message = ServerMessage::update(metrics, &state.world, creatures);

                if let Ok(json) = serde_json::to_string(&message) {
                    if sender.send(axum::extract::ws::Message::Text(json)).await.is_err() {
                        log::info!("Client disconnected");
                        break;
                    }
                }

                // Send creature updates if subscribed
                if let Some(creature_id) = subscribed_creature_id {
                    if let Some(details) = get_creature_details(&state, creature_id, &app_state.config) {
                        let update_msg = ServerMessage::CreatureUpdate { details };
                        if let Ok(json) = serde_json::to_string(&update_msg) {
                            let _ = sender.send(axum::extract::ws::Message::Text(json)).await;
                        }
                    }
                }
            }

            Some(msg) = receiver.next() => {
                match msg {
                    Ok(axum::extract::ws::Message::Text(text)) => {
                        if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                            match client_msg {
                                ClientMessage::GetState => {
                                    let state = app_state.stream.get_state().await;
                                    let metrics = state.metrics();
                                    let creatures = state.creatures_vec();
                                    let message = ServerMessage::full_state(metrics, &state.world, creatures);

                                    if let Ok(json) = serde_json::to_string(&message) {
                                        let _ = sender.send(axum::extract::ws::Message::Text(json)).await;
                                    }
                                }
                                ClientMessage::GetRegion { .. } => {
                                    log::warn!("GetRegion not yet implemented");
                                }
                                ClientMessage::GetCreatureDetails { creature_id } => {
                                    let state = app_state.stream.get_state().await;
                                    if let Some(details) = get_creature_details(&state, creature_id, &app_state.config) {
                                        let message = ServerMessage::CreatureDetails(details);
                                        if let Ok(json) = serde_json::to_string(&message) {
                                            let _ = sender.send(axum::extract::ws::Message::Text(json)).await;
                                        }
                                    }
                                }
                                ClientMessage::SubscribeCreature { creature_id } => {
                                    subscribed_creature_id = creature_id;
                                }
                            }
                        }
                    }
                    Ok(axum::extract::ws::Message::Close(_)) => {
                        log::info!("Client requested close");
                        break;
                    }
                    Err(e) => {
                        log::error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    log::info!("WebSocket connection closed");
}

fn get_creature_details(
    state: &crate::simulation::SimulationState,
    creature_id: u64,
    config: &Config,
) -> Option<protocol::CreatureDetails> {
    let creature = state.creatures.get(&creature_id)?;

    let sensor_inputs = state.get_sensor_inputs(
        creature_id,
        creature.x,
        creature.y,
        creature.energy(),
        config,
    );

    let (network_outputs, network_probabilities) =
        creature.brain.get_outputs_and_probabilities(&sensor_inputs);

    Some(protocol::CreatureDetails {
        id: creature_id,
        genome: creature.genome.genes.clone(),
        sensor_inputs,
        network_outputs,
        network_probabilities,
    })
}
