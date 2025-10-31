use crate::creature::Creature;
use crate::stats::SimulationMetrics;
use crate::world::World;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "update")]
    Update {
        metrics: SimulationMetrics,
        creatures: Vec<CreatureSnapshot>,
    },
    #[serde(rename = "world_region")]
    WorldRegion {
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        cells: Vec<u8>,
    },
    #[serde(rename = "full_state")]
    FullState {
        metrics: SimulationMetrics,
        world_width: usize,
        world_height: usize,
        creatures: Vec<CreatureSnapshot>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureSnapshot {
    pub id: u64,
    pub x: usize,
    pub y: usize,
    pub energy: f64,
    pub generation: u64,
}

impl From<&Creature> for CreatureSnapshot {
    fn from(creature: &Creature) -> Self {
        Self {
            id: creature.id,
            x: creature.x,
            y: creature.y,
            energy: creature.energy(),
            generation: creature.genome.generation,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "get_state")]
    GetState,
    #[serde(rename = "get_region")]
    GetRegion { x: usize, y: usize, width: usize, height: usize },
}

impl ServerMessage {
    pub fn update(metrics: SimulationMetrics, creatures: Vec<Creature>) -> Self {
        let snapshots = creatures.iter().map(CreatureSnapshot::from).collect();
        ServerMessage::Update {
            metrics,
            creatures: snapshots,
        }
    }

    pub fn full_state(metrics: SimulationMetrics, world: &World, creatures: Vec<Creature>) -> Self {
        let snapshots = creatures.iter().map(CreatureSnapshot::from).collect();
        ServerMessage::FullState {
            metrics,
            world_width: world.width(),
            world_height: world.height(),
            creatures: snapshots,
        }
    }
}
