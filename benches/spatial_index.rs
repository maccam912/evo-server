use criterion::{black_box, criterion_group, criterion_main, Criterion};
use evo_server::config::Config;
use evo_server::creature::genome::Genome;
use evo_server::creature::Creature;
use evo_server::simulation::SimulationState;

fn populate_simulation(population: usize, width: usize, height: usize) -> SimulationState {
    let mut config = Config::default();
    config.world.width = width;
    config.world.height = height;
    config.creature.initial_population = 0;

    let mut sim = SimulationState::new(&config);
    let genome_template = Genome {
        genes: vec![0; config.evolution.genome_size],
        generation: 0,
    };

    for id in 0..population {
        let x = id % width;
        let y = id / width;
        if y >= height {
            break;
        }

        let mut genome = genome_template.clone();
        genome.generation = (id % 6) as u64;

        let creature = Creature::new(
            id as u64,
            x,
            y,
            genome,
            config.creature.initial_energy,
            config.creature.max_energy,
            (
                config.evolution.neural_net_inputs,
                config.evolution.neural_net_hidden,
                config.evolution.neural_net_outputs,
            ),
        );
        sim.add_creature_to_position(creature.id, creature.x, creature.y);
        sim.creatures.insert(creature.id, creature);
    }

    sim
}

fn naive_count(sim: &SimulationState, x: usize, y: usize, radius: usize) -> usize {
    let x_min = x.saturating_sub(radius);
    let x_max = x.saturating_add(radius).min(sim.world.width() - 1);
    let y_min = y.saturating_sub(radius);
    let y_max = y.saturating_add(radius).min(sim.world.height() - 1);

    sim.creatures
        .values()
        .filter(|c| c.x >= x_min && c.x <= x_max && c.y >= y_min && c.y <= y_max)
        .count()
}

fn naive_find_nearest(
    sim: &SimulationState,
    self_id: u64,
    x: usize,
    y: usize,
) -> Option<(f64, u64)> {
    sim.creatures
        .iter()
        .filter(|(&id, _)| id != self_id)
        .map(|(&id, c)| {
            let dx = (c.x as f64 - x as f64).abs();
            let dy = (c.y as f64 - y as f64).abs();
            let dist = (dx * dx + dy * dy).sqrt();
            (dist, id)
        })
        .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
}

fn bench_spatial_index_counts(c: &mut Criterion) {
    let width = 128;
    let height = 128;
    let population = width * height / 2;
    let sim = populate_simulation(population, width, height);

    let sample_points = vec![(0, 0), (10, 10), (64, 64), (120, 120)];
    let radius = 10;

    c.bench_function("spatial_index_count_nearby", |b| {
        b.iter(|| {
            for (x, y) in &sample_points {
                black_box(sim.count_nearby_creatures(*x, *y, radius));
            }
        });
    });

    c.bench_function("hashmap_count_nearby", |b| {
        b.iter(|| {
            for (x, y) in &sample_points {
                black_box(naive_count(&sim, *x, *y, radius));
            }
        });
    });
}

fn bench_spatial_index_find_nearest(c: &mut Criterion) {
    let width = 128;
    let height = 128;
    let population = width * height / 2;
    let sim = populate_simulation(population, width, height);

    let subject = sim.creatures.keys().next().copied().unwrap_or(0);
    let (x, y) = sim
        .creatures
        .get(&subject)
        .map(|c| (c.x, c.y))
        .unwrap_or((0, 0));

    c.bench_function("spatial_index_find_nearest", |b| {
        b.iter(|| black_box(sim.find_nearest_creature(subject, x, y)));
    });

    c.bench_function("hashmap_find_nearest", |b| {
        b.iter(|| black_box(naive_find_nearest(&sim, subject, x, y)));
    });
}

criterion_group!(
    spatial_index_benches,
    bench_spatial_index_counts,
    bench_spatial_index_find_nearest
);
criterion_main!(spatial_index_benches);
