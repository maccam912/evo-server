use super::genome::Genome;
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::ser::SerializeStruct;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone)]
pub struct NeuralNetwork {
    input_size: usize,
    hidden_size: usize,
    output_size: usize,
    /// Row-major weights: hidden_size x input_size
    weights_ih: Vec<f64>,
    /// Row-major weights: output_size x hidden_size
    weights_ho: Vec<f64>,
    hidden: Vec<f64>,
    output: Vec<f64>,
    softmax: Vec<f64>,
    rng: StdRng,
}

#[derive(Serialize, Deserialize)]
struct NeuralNetworkData {
    input_size: usize,
    hidden_size: usize,
    output_size: usize,
    weights_ih: Vec<f64>,
    weights_ho: Vec<f64>,
}

impl NeuralNetwork {
    pub fn from_genome(
        genome: &Genome,
        input_size: usize,
        hidden_size: usize,
        output_size: usize,
    ) -> Self {
        let mut idx = 0;

        let mut weights_ih = vec![0.0; hidden_size * input_size];
        for h in 0..hidden_size {
            for i in 0..input_size {
                let value = genome.get_normalized(idx) * 2.0 - 1.0;
                weights_ih[h * input_size + i] = value;
                idx += 1;
                if idx >= genome.genes.len() {
                    idx = 0;
                }
            }
        }

        let mut weights_ho = vec![0.0; output_size * hidden_size];
        for o in 0..output_size {
            for h in 0..hidden_size {
                let value = genome.get_normalized(idx) * 2.0 - 1.0;
                weights_ho[o * hidden_size + h] = value;
                idx += 1;
                if idx >= genome.genes.len() {
                    idx = 0;
                }
            }
        }

        Self::from_weights(input_size, hidden_size, output_size, weights_ih, weights_ho)
    }

    fn from_weights(
        input_size: usize,
        hidden_size: usize,
        output_size: usize,
        weights_ih: Vec<f64>,
        weights_ho: Vec<f64>,
    ) -> Self {
        debug_assert_eq!(weights_ih.len(), hidden_size * input_size);
        debug_assert_eq!(weights_ho.len(), output_size * hidden_size);

        Self {
            input_size,
            hidden_size,
            output_size,
            weights_ih,
            weights_ho,
            hidden: vec![0.0; hidden_size],
            output: vec![0.0; output_size],
            softmax: vec![0.0; output_size],
            rng: Self::new_rng(),
        }
    }

    fn new_rng() -> StdRng {
        StdRng::from_rng(rand::thread_rng()).expect("Failed to initialize RNG")
    }

    pub fn forward<'a>(&'a mut self, inputs: &[f64]) -> &'a [f64] {
        assert_eq!(inputs.len(), self.input_size, "Input size mismatch");

        for h in 0..self.hidden_size {
            let mut sum = 0.0;
            let weight_row_start = h * self.input_size;
            for i in 0..self.input_size {
                sum += self.weights_ih[weight_row_start + i] * inputs[i];
            }
            self.hidden[h] = sum.tanh();
        }

        for o in 0..self.output_size {
            let mut sum = 0.0;
            let weight_row_start = o * self.hidden_size;
            for h in 0..self.hidden_size {
                sum += self.weights_ho[weight_row_start + h] * self.hidden[h];
            }
            self.output[o] = sum.tanh();
        }

        &self.output
    }

    fn compute_probabilities<'a>(&'a mut self) -> &'a [f64] {
        debug_assert_eq!(self.softmax.len(), self.output.len());
        let max_output = self
            .output
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);

        let mut sum = 0.0;
        for (slot, &value) in self.softmax.iter_mut().zip(self.output.iter()) {
            let exp = (value - max_output).exp();
            *slot = exp;
            sum += exp;
        }

        if sum > 0.0 {
            for slot in &mut self.softmax {
                *slot /= sum;
            }
        } else {
            let uniform = 1.0 / self.softmax.len().max(1) as f64;
            for slot in &mut self.softmax {
                *slot = uniform;
            }
        }

        &self.softmax
    }

    pub fn decide_action(&mut self, inputs: &[f64]) -> Action {
        self.forward(inputs);
        let random_value: f64 = self.rng.gen();
        let probabilities = self.compute_probabilities();

        let mut cumulative = 0.0;
        for (i, &prob) in probabilities.iter().enumerate() {
            cumulative += prob;
            if random_value < cumulative {
                return Action::from_index(i);
            }
        }

        Action::Stay
    }

    pub fn get_outputs_and_probabilities(&self, inputs: &[f64]) -> (Vec<f64>, Vec<f64>) {
        let mut temp = self.clone();
        temp.forward(inputs);
        let outputs = temp.output.clone();
        let probabilities = temp.compute_probabilities().to_vec();
        (outputs, probabilities)
    }
}

impl Serialize for NeuralNetwork {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("NeuralNetwork", 5)?;
        state.serialize_field("input_size", &self.input_size)?;
        state.serialize_field("hidden_size", &self.hidden_size)?;
        state.serialize_field("output_size", &self.output_size)?;
        state.serialize_field("weights_ih", &self.weights_ih)?;
        state.serialize_field("weights_ho", &self.weights_ho)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for NeuralNetwork {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let data = NeuralNetworkData::deserialize(deserializer)?;

        if data.weights_ih.len() != data.hidden_size * data.input_size {
            return Err(de::Error::custom(
                "weights_ih length does not match dimensions",
            ));
        }

        if data.weights_ho.len() != data.output_size * data.hidden_size {
            return Err(de::Error::custom(
                "weights_ho length does not match dimensions",
            ));
        }

        Ok(Self::from_weights(
            data.input_size,
            data.hidden_size,
            data.output_size,
            data.weights_ih,
            data.weights_ho,
        ))
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
    fn from_index(index: usize) -> Self {
        match index {
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
        }
    }

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
        matches!(
            self,
            Action::MoveUp
                | Action::MoveDown
                | Action::MoveLeft
                | Action::MoveRight
                | Action::SprintUp
                | Action::SprintDown
                | Action::SprintLeft
                | Action::SprintRight
        )
    }

    pub fn is_sprint(&self) -> bool {
        matches!(
            self,
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
        let mut nn = NeuralNetwork::from_genome(&genome, 8, 6, 4);

        let inputs = vec![0.5, 0.3, 0.1, 0.9, 0.2, 0.7, 0.4, 0.6];
        let outputs = nn.forward(&inputs).to_vec();

        assert_eq!(outputs.len(), 4);
        for output in &outputs {
            assert!(*output >= -1.0 && *output <= 1.0);
        }
    }

    #[test]
    fn test_decide_action() {
        let genome = Genome::random(100);
        let mut nn = NeuralNetwork::from_genome(&genome, 8, 6, 12);

        let inputs = vec![0.5, 0.3, 0.1, 0.9, 0.2, 0.7, 0.4, 0.6];
        let action = nn.decide_action(&inputs);

        assert!(matches!(
            action,
            Action::MoveUp
                | Action::MoveDown
                | Action::MoveLeft
                | Action::MoveRight
                | Action::Attack
                | Action::Reproduce
                | Action::ShareEnergy
                | Action::SprintUp
                | Action::SprintDown
                | Action::SprintLeft
                | Action::SprintRight
                | Action::Rest
                | Action::Stay
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
