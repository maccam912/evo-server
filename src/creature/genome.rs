use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genome {
    pub genes: Vec<u8>,
    pub generation: u64,
}

impl Genome {
    pub fn random(size: usize) -> Self {
        let mut rng = rand::thread_rng();
        let genes = (0..size).map(|_| rng.gen()).collect();
        Self {
            genes,
            generation: 0,
        }
    }

    pub fn from_parent(parent: &Genome, mutation_rate: f64) -> Self {
        let mut rng = rand::thread_rng();
        let mut genes = parent.genes.clone();

        for gene in &mut genes {
            if rng.gen::<f64>() < mutation_rate {
                *gene = rng.gen();
            }
        }

        Self {
            genes,
            generation: parent.generation + 1,
        }
    }

    pub fn get_normalized(&self, index: usize) -> f64 {
        self.genes.get(index).map(|&g| g as f64 / 255.0).unwrap_or(0.0)
    }

    pub fn get_trait(&self, start: usize, count: usize) -> Vec<f64> {
        (start..start + count)
            .map(|i| self.get_normalized(i))
            .collect()
    }

    pub fn similarity(&self, other: &Genome) -> f64 {
        if self.genes.len() != other.genes.len() {
            return 0.0;
        }

        let matching: usize = self.genes
            .iter()
            .zip(&other.genes)
            .map(|(a, b)| {
                let diff = (*a as i32 - *b as i32).abs() as u32;
                if diff < 10 {
                    1
                } else {
                    0
                }
            })
            .sum();

        matching as f64 / self.genes.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_genome() {
        let genome = Genome::random(100);
        assert_eq!(genome.genes.len(), 100);
        assert_eq!(genome.generation, 0);
    }

    #[test]
    fn test_genome_from_parent() {
        let parent = Genome::random(100);
        let child = Genome::from_parent(&parent, 0.1);

        assert_eq!(child.genes.len(), parent.genes.len());
        assert_eq!(child.generation, parent.generation + 1);

        let differences = parent.genes
            .iter()
            .zip(&child.genes)
            .filter(|(a, b)| a != b)
            .count();

        assert!(differences > 0);
    }

    #[test]
    fn test_get_normalized() {
        let genome = Genome {
            genes: vec![0, 127, 255],
            generation: 0,
        };

        assert_eq!(genome.get_normalized(0), 0.0);
        assert!((genome.get_normalized(1) - 0.498).abs() < 0.01);
        assert_eq!(genome.get_normalized(2), 1.0);
    }

    #[test]
    fn test_get_trait() {
        let genome = Genome {
            genes: vec![0, 50, 100, 150, 200, 255],
            generation: 0,
        };

        let trait_values = genome.get_trait(1, 3);
        assert_eq!(trait_values.len(), 3);
        assert!(trait_values[0] > 0.0 && trait_values[0] < 1.0);
    }

    #[test]
    fn test_similarity() {
        let genome1 = Genome {
            genes: vec![100, 100, 100],
            generation: 0,
        };

        let genome2 = Genome {
            genes: vec![100, 105, 100],
            generation: 0,
        };

        let similarity = genome1.similarity(&genome2);
        assert!(similarity > 0.5);
    }
}
