use allocation_counter::measure;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use evo_server::creature::{genome::Genome, neural_net::NeuralNetwork};

fn decide_action_allocations(c: &mut Criterion) {
    let genome = Genome::random(256);
    let mut network = NeuralNetwork::from_genome(&genome, 8, 6, 12);
    let inputs = vec![0.5; 8];

    c.bench_function("decide_action_allocation_free", |b| {
        b.iter(|| {
            let info = measure(|| {
                let action = network.decide_action(black_box(&inputs));
                black_box(action);
            });
            assert_eq!(info.count_total, 0, "decide_action should not allocate");
        });
    });
}

criterion_group!(benches, decide_action_allocations);
criterion_main!(benches);
