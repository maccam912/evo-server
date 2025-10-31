use rand::Rng;

pub fn mutate_byte(byte: u8, mutation_rate: f64) -> u8 {
    let mut rng = rand::thread_rng();

    if rng.gen::<f64>() < mutation_rate {
        rng.gen()
    } else {
        byte
    }
}

pub fn mutate_bytes(bytes: &[u8], mutation_rate: f64) -> Vec<u8> {
    bytes.iter().map(|&b| mutate_byte(b, mutation_rate)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutate_byte_no_mutation() {
        let original = 100u8;

        let mut same_count = 0;
        for _ in 0..100 {
            let mutated = mutate_byte(original, 0.0);
            if mutated == original {
                same_count += 1;
            }
        }

        assert_eq!(same_count, 100);
    }

    #[test]
    fn test_mutate_byte_with_mutation() {
        let original = 100u8;

        let mut different_count = 0;
        for _ in 0..1000 {
            let mutated = mutate_byte(original, 1.0);
            if mutated != original {
                different_count += 1;
            }
        }

        assert!(different_count > 900);
    }

    #[test]
    fn test_mutate_bytes() {
        let original = vec![50, 100, 150, 200, 250];
        let mutated = mutate_bytes(&original, 0.5);

        assert_eq!(mutated.len(), original.len());

        let differences = original
            .iter()
            .zip(&mutated)
            .filter(|(a, b)| a != b)
            .count();

        assert!(differences > 0);
    }
}
