use crate::simulation::SimulationState;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct StateStream {
    state: Arc<RwLock<SimulationState>>,
}

impl StateStream {
    pub fn new(state: Arc<RwLock<SimulationState>>) -> Self {
        Self { state }
    }

    pub async fn get_state(&self) -> SimulationState {
        self.state.read().await.clone()
    }
}
