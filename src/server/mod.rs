pub mod protocol;
pub mod state_stream;

use crate::config::Config;
use crate::simulation::SimulationState;
use futures_util::{SinkExt, StreamExt};
use protocol::{ClientMessage, ServerMessage};
use state_stream::StateStream;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tokio_tungstenite::tungstenite::Message;

pub async fn run_server(
    config: Config,
    state: Arc<RwLock<SimulationState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("{}:{}", config.server.address, config.server.port);
    let listener = TcpListener::bind(&addr).await?;
    log::info!("WebSocket server listening on: {}", addr);

    let stream = StateStream::new(state.clone());

    while let Ok((tcp_stream, peer_addr)) = listener.accept().await {
        log::info!("New client connected: {}", peer_addr);
        let stream = stream.clone();
        let config = config.clone();

        tokio::spawn(handle_client(tcp_stream, peer_addr, stream, config));
    }

    Ok(())
}

async fn handle_client(
    tcp_stream: TcpStream,
    peer_addr: SocketAddr,
    stream: StateStream,
    config: Config,
) {
    let ws_stream = match tokio_tungstenite::accept_async(tcp_stream).await {
        Ok(ws) => ws,
        Err(e) => {
            log::error!("WebSocket handshake failed for {}: {}", peer_addr, e);
            return;
        }
    };

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    let mut update_interval = interval(Duration::from_millis(1000 / config.server.update_rate_hz));

    loop {
        tokio::select! {
            _ = update_interval.tick() => {
                let state = stream.get_state().await;
                let metrics = state.metrics();
                let creatures = state.creatures_vec();

                let message = ServerMessage::update(metrics, creatures);

                if let Ok(json) = serde_json::to_string(&message) {
                    if ws_sender.send(Message::Text(json)).await.is_err() {
                        log::info!("Client {} disconnected", peer_addr);
                        break;
                    }
                }
            }

            Some(msg) = ws_receiver.next() => {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                            match client_msg {
                                ClientMessage::GetState => {
                                    let state = stream.get_state().await;
                                    let metrics = state.metrics();
                                    let creatures = state.creatures_vec();
                                    let message = ServerMessage::full_state(metrics, &state.world, creatures);

                                    if let Ok(json) = serde_json::to_string(&message) {
                                        let _ = ws_sender.send(Message::Text(json)).await;
                                    }
                                }
                                ClientMessage::GetRegion { .. } => {
                                    log::warn!("GetRegion not yet implemented");
                                }
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        log::info!("Client {} requested close", peer_addr);
                        break;
                    }
                    Err(e) => {
                        log::error!("WebSocket error for {}: {}", peer_addr, e);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    log::info!("Client {} connection closed", peer_addr);
}
