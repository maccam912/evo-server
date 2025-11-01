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
        food: Vec<FoodSnapshot>,
    },
    #[serde(rename = "creature_details")]
    CreatureDetails(CreatureDetails),
    #[serde(rename = "creature_update")]
    CreatureUpdate {
        details: CreatureDetails,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodSnapshot {
    pub x: usize,
    pub y: usize,
    pub amount: u32,
    pub is_meat: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureDetails {
    pub id: u64,
    pub genome: Vec<u8>,
    pub sensor_inputs: Vec<f64>,
    pub network_outputs: Vec<f64>,
    pub network_probabilities: Vec<f64>,
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
    #[serde(rename = "get_creature_details")]
    GetCreatureDetails { creature_id: u64 },
    #[serde(rename = "subscribe_creature")]
    SubscribeCreature { creature_id: Option<u64> },
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

        // Collect food snapshots from the world
        let mut food = Vec::new();
        for y in 0..world.height() {
            for x in 0..world.width() {
                if let Some(cell) = world.get(x, y) {
                    if cell.is_food() {
                        let amount = cell.food_amount();
                        if amount > 0 {
                            food.push(FoodSnapshot {
                                x,
                                y,
                                amount,
                                is_meat: cell.is_meat(),
                            });
                        }
                    }
                }
            }
        }

        ServerMessage::FullState {
            metrics,
            world_width: world.width(),
            world_height: world.height(),
            creatures: snapshots,
            food,
        }
    }
}
