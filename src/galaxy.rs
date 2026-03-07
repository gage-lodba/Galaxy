use crate::vertex::StarVertex;
use rand::Rng;

const TRAIL_SEGMENTS: usize = 20;

pub fn generate_galaxy_vertices(num_stars: usize, num_shooting_stars: usize) -> Vec<StarVertex> {
    let mut rng = rand::rng();
    let mut vertices = Vec::with_capacity(num_stars + num_shooting_stars * (1 + TRAIL_SEGMENTS));

    // Generate random cluster centers
    let num_clusters = rng.random_range(5..15);
    let mut clusters: Vec<(f32, f32, f32)> = Vec::with_capacity(num_clusters);
    for _ in 0..num_clusters {
        let cx = rng.random_range(-0.8..0.8);
        let cy = rng.random_range(-0.8..0.8);
        let radius = rng.random_range(0.05..0.25);
        clusters.push((cx, cy, radius));
    }

    // 30% of stars go into clusters, 70% use spiral distribution
    let cluster_star_count = (num_stars as f32 * 0.3) as usize;
    let spiral_star_count = num_stars - cluster_star_count;

    // Generate clustered stars
    for _ in 0..cluster_star_count {
        let cluster = &clusters[rng.random_range(0..clusters.len())];
        // Gaussian-ish distribution around cluster center using Box-Muller-like approach
        let r1: f32 = rng.random_range(0.0001..1.0);
        let r2: f32 = rng.random_range(0.0..std::f32::consts::PI * 2.0);
        let spread = (-2.0 * r1.ln()).sqrt() * cluster.2 * 0.5;
        let x = (cluster.0 + spread * r2.cos()).clamp(-1.0, 1.0);
        let y = (cluster.1 + spread * r2.sin()).clamp(-1.0, 1.0);

        let distance = (x * x + y * y).sqrt().min(1.0);

        vertices.push(generate_star(&mut rng, x, y, distance));
    }

    // Generate spiral stars
    for _ in 0..spiral_star_count {
        let angle = rng.random_range(0.0..std::f32::consts::PI * 2.0);
        let distance = rng.random_range(0.0..1.0);
        let spiral_angle = angle + distance * 4.0;

        let x = spiral_angle.cos() * distance;
        let y = spiral_angle.sin() * distance;

        vertices.push(generate_star(&mut rng, x, y, distance));
    }

    // Generate shooting stars with trails
    for _ in 0..num_shooting_stars {
        // Random starting position (spread across the sky)
        let x = rng.random_range(-1.0..1.0);
        let y = rng.random_range(-1.0..1.0);

        // Random direction (normalized)
        let angle = rng.random_range(0.0..std::f32::consts::PI * 2.0);
        let dir_x = angle.cos();
        let dir_y = angle.sin();

        // Phase offset so they don't all fire at once
        let seed = rng.random_range(0.0..100.0);

        // Head vertex (trail_offset = 0.0)
        vertices.push(StarVertex {
            position: [x, y],
            color: [1.0, 1.0, 1.0, 1.0],
            star_data: [1.0, seed, dir_x, dir_y],
        });

        // Trail segment vertices
        for i in 1..=TRAIL_SEGMENTS {
            let trail_offset = i as f32 / TRAIL_SEGMENTS as f32;
            vertices.push(StarVertex {
                position: [x, y],
                color: [1.0, 1.0, 1.0, 1.0],
                // Encode trail offset in star_data.x: 1.0 = head, >1.0 = trail
                star_data: [1.0 + trail_offset, seed, dir_x, dir_y],
            });
        }
    }

    vertices
}

fn generate_star(rng: &mut impl Rng, x: f32, y: f32, distance: f32) -> StarVertex {
    let star_type = match rng.random_range(0..100) {
        0..=1 => 0,   // 2% Blue giants
        2..=15 => 1,  // 14% Blue-white stars
        16..=35 => 2, // 20% White stars
        36..=75 => 3, // 40% Yellow stars
        _ => 4,       // 24% Orange-Red stars
    };

    let (r, g, b) = match star_type {
        0 => (rng.random_range(0.7..0.8), rng.random_range(0.8..0.9), 1.0),
        1 => (
            rng.random_range(0.8..0.9),
            rng.random_range(0.8..0.9),
            rng.random_range(0.9..1.0),
        ),
        2 => (
            rng.random_range(0.9..1.0),
            rng.random_range(0.9..1.0),
            rng.random_range(0.9..1.0),
        ),
        3 => (
            rng.random_range(0.9..1.0),
            rng.random_range(0.8..0.9),
            rng.random_range(0.5..0.6),
        ),
        _ => (
            rng.random_range(0.9..1.0),
            rng.random_range(0.5..0.7),
            rng.random_range(0.1..0.3),
        ),
    };

    let brightness = (1.0 - distance * 0.5) * rng.random_range(0.5..1.0);

    let color = [
        r * brightness * rng.random_range(0.95..1.0),
        g * brightness * rng.random_range(0.95..1.0),
        b * brightness * rng.random_range(0.95..1.0),
        0.9 + rng.random_range(-0.1..0.1),
    ];

    let seed = rng.random_range(0.0..100.0);

    StarVertex {
        position: [x, y],
        color,
        star_data: [0.0, seed, 0.0, 0.0],
    }
}
