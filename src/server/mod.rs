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

    loop {
        tokio::select! {
            _ = update_interval.tick() => {
                let state = app_state.stream.get_state().await;
                let metrics = state.metrics();
                let creatures = state.creatures_vec();

                let message = ServerMessage::update(metrics, creatures);

                if let Ok(json) = serde_json::to_string(&message) {
                    if sender.send(axum::extract::ws::Message::Text(json)).await.is_err() {
                        log::info!("Client disconnected");
                        break;
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
