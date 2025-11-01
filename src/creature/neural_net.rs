use super::genome::Genome;
use serde::{Deserialize, Serialize};
use rand::Rng;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralNetwork {
    input_size: usize,
    hidden_size: usize,
    output_size: usize,
    weights_ih: Vec<Vec<f64>>,
    weights_ho: Vec<Vec<f64>>,
}

impl NeuralNetwork {
    pub fn from_genome(genome: &Genome, input_size: usize, hidden_size: usize, output_size: usize) -> Self {
        let mut idx = 0;

        let mut weights_ih = vec![vec![0.0; input_size]; hidden_size];
        for h in 0..hidden_size {
            for i in 0..input_size {
                weights_ih[h][i] = genome.get_normalized(idx) * 2.0 - 1.0;
                idx += 1;
                if idx >= genome.genes.len() {
                    idx = 0;
                }
            }
        }

        let mut weights_ho = vec![vec![0.0; hidden_size]; output_size];
        for o in 0..output_size {
            for h in 0..hidden_size {
                weights_ho[o][h] = genome.get_normalized(idx) * 2.0 - 1.0;
                idx += 1;
                if idx >= genome.genes.len() {
                    idx = 0;
                }
            }
        }

        Self {
            input_size,
            hidden_size,
            output_size,
            weights_ih,
            weights_ho,
        }
    }

    pub fn forward(&self, inputs: &[f64]) -> Vec<f64> {
        assert_eq!(inputs.len(), self.input_size, "Input size mismatch");

        let hidden: Vec<f64> = self.weights_ih
            .iter()
            .map(|weights| {
                let sum: f64 = weights.iter().zip(inputs).map(|(w, i)| w * i).sum();
                Self::tanh(sum)
            })
            .collect();

        let output: Vec<f64> = self.weights_ho
            .iter()
            .map(|weights| {
                let sum: f64 = weights.iter().zip(&hidden).map(|(w, h)| w * h).sum();
                Self::tanh(sum)
            })
            .collect();

        output
    }

    fn tanh(x: f64) -> f64 {
        x.tanh()
    }

    pub fn decide_action(&self, inputs: &[f64]) -> Action {
        let outputs = self.forward(inputs);

        // Compute softmax probabilities
        let max_output = outputs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exp_outputs: Vec<f64> = outputs.iter().map(|&x| (x - max_output).exp()).collect();
        let sum_exp: f64 = exp_outputs.iter().sum();
        let probabilities: Vec<f64> = exp_outputs.iter().map(|&x| x / sum_exp).collect();

        // Sample action based on probabilities
        let mut rng = rand::thread_rng();
        let random_value: f64 = rng.gen(); // 0.0 to 1.0

        let mut cumulative = 0.0;
        for (i, &prob) in probabilities.iter().enumerate() {
            cumulative += prob;
            if random_value < cumulative {
                return match i {
                    0 => Action::MoveUp,
                    1 => Action::MoveDown,
                    2 => Action::MoveLeft,
                    3 => Action::MoveRight,
                    4 => Action::Attack,
                    5 => Action::Reproduce,
                    6 => Action::ShareEnergy,
                    7 => Action::SprintUp,
                    8 => Action::SprintDown,
                    9 => Action::SprintLeft,
                    10 => Action::SprintRight,
                    11 => Action::Rest,
                    _ => Action::Stay,
                };
            }
        }

        // Fallback (should never reach here)
        Action::Stay
    }

    /// Get the raw outputs and softmax probabilities for the given inputs
    pub fn get_outputs_and_probabilities(&self, inputs: &[f64]) -> (Vec<f64>, Vec<f64>) {
        let outputs = self.forward(inputs);

        // Compute softmax probabilities
        let max_output = outputs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let exp_outputs: Vec<f64> = outputs.iter().map(|&x| (x - max_output).exp()).collect();
        let sum_exp: f64 = exp_outputs.iter().sum();
        let probabilities: Vec<f64> = exp_outputs.iter().map(|&x| x / sum_exp).collect();

        (outputs, probabilities)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Stay,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Attack,
    Reproduce,
    ShareEnergy,
    SprintUp,
    SprintDown,
    SprintLeft,
    SprintRight,
    Rest,
}

impl Action {
    pub fn to_delta(&self) -> (i32, i32) {
        match self {
            Action::Stay => (0, 0),
            Action::MoveUp => (0, -1),
            Action::MoveDown => (0, 1),
            Action::MoveLeft => (-1, 0),
            Action::MoveRight => (1, 0),
            Action::SprintUp => (0, -1),
            Action::SprintDown => (0, 1),
            Action::SprintLeft => (-1, 0),
            Action::SprintRight => (1, 0),
            Action::Attack => (0, 0),
            Action::Reproduce => (0, 0),
            Action::ShareEnergy => (0, 0),
            Action::Rest => (0, 0),
        }
    }

    pub fn is_movement(&self) -> bool {
        matches!(self,
            Action::MoveUp | Action::MoveDown | Action::MoveLeft | Action::MoveRight |
            Action::SprintUp | Action::SprintDown | Action::SprintLeft | Action::SprintRight
        )
    }

    pub fn is_sprint(&self) -> bool {
        matches!(self,
            Action::SprintUp | Action::SprintDown | Action::SprintLeft | Action::SprintRight
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neural_network_creation() {
        let genome = Genome::random(100);
        let nn = NeuralNetwork::from_genome(&genome, 8, 6, 4);

        assert_eq!(nn.input_size, 8);
        assert_eq!(nn.hidden_size, 6);
        assert_eq!(nn.output_size, 4);
    }

    #[test]
    fn test_neural_network_forward() {
        let genome = Genome::random(100);
        let nn = NeuralNetwork::from_genome(&genome, 8, 6, 4);

        let inputs = vec![0.5, 0.3, 0.1, 0.9, 0.2, 0.7, 0.4, 0.6];
        let outputs = nn.forward(&inputs);

        assert_eq!(outputs.len(), 4);
        for output in &outputs {
            assert!(*output >= -1.0 && *output <= 1.0);
        }
    }

    #[test]
    fn test_decide_action() {
        let genome = Genome::random(100);
        let nn = NeuralNetwork::from_genome(&genome, 8, 6, 12);

        let inputs = vec![0.5, 0.3, 0.1, 0.9, 0.2, 0.7, 0.4, 0.6];
        let action = nn.decide_action(&inputs);

        assert!(matches!(
            action,
            Action::MoveUp | Action::MoveDown | Action::MoveLeft | Action::MoveRight |
            Action::Attack | Action::Reproduce | Action::ShareEnergy |
            Action::SprintUp | Action::SprintDown | Action::SprintLeft | Action::SprintRight |
            Action::Rest | Action::Stay
        ));
    }

    #[test]
    fn test_action_to_delta() {
        assert_eq!(Action::MoveUp.to_delta(), (0, -1));
        assert_eq!(Action::MoveDown.to_delta(), (0, 1));
        assert_eq!(Action::MoveLeft.to_delta(), (-1, 0));
        assert_eq!(Action::MoveRight.to_delta(), (1, 0));
        assert_eq!(Action::Stay.to_delta(), (0, 0));
    }
}
