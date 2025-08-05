use crate::vertex::MyVertex;
use rand::Rng;

pub fn generate_galaxy_vertices(num_stars: usize) -> Vec<MyVertex> {
    let mut rng = rand::rng();
    let mut vertices = Vec::with_capacity(num_stars);

    for _ in 0..num_stars {
        let angle = rng.random_range(0.0..std::f32::consts::PI * 2.0);
        let distance = rng.random_range(0.0..1.0);
        let spiral_angle = angle + distance * 4.0;

        let x = spiral_angle.cos() * distance;
        let y = spiral_angle.sin() * distance;

        // Star type distribution based on real stellar populations
        let star_type = match rng.random_range(0..100) {
            0..=1 => 0,   // 2% Blue giants (very rare, very bright)
            2..=15 => 1,  // 14% Blue-white stars
            16..=35 => 2, // 20% White stars
            36..=75 => 3, // 40% Yellow stars (like our Sun)
            _ => 4,       // 24% Orange-Red stars
        };

        // Colors based on stellar classification (OBAFGKM)
        let (r, g, b) = match star_type {
            0 => (
                // Blue giants (O type)
                rng.random_range(0.7..0.8), // Some red
                rng.random_range(0.8..0.9), // More green
                1.0,                        // Maximum blue
            ),
            1 => (
                // Blue-white stars (B type)
                rng.random_range(0.8..0.9), // More red
                rng.random_range(0.8..0.9), // More green
                rng.random_range(0.9..1.0), // Lots of blue
            ),
            2 => (
                // White stars (A type)
                rng.random_range(0.9..1.0), // Full red
                rng.random_range(0.9..1.0), // Full green
                rng.random_range(0.9..1.0), // Full blue
            ),
            3 => (
                // Yellow stars (G type, like our Sun)
                rng.random_range(0.9..1.0), // Full red
                rng.random_range(0.8..0.9), // Slightly less green
                rng.random_range(0.5..0.6), // Much less blue
            ),
            _ => (
                // Orange-Red stars (K-M type)
                rng.random_range(0.9..1.0), // Full red
                rng.random_range(0.5..0.7), // Much less green
                rng.random_range(0.1..0.3), // Very little blue
            ),
        };

        // Brightness based on distance from center and random variation
        let brightness = (1.0 - distance * 0.5) * rng.random_range(0.5..1.0);

        // Apply brightness and add slight color variation
        let color = [
            r * brightness * rng.random_range(0.95..1.0),
            g * brightness * rng.random_range(0.95..1.0),
            b * brightness * rng.random_range(0.95..1.0),
            0.9 + rng.random_range(-0.1..0.1), // Slight alpha variation
        ];

        vertices.push(MyVertex {
            position: [x, y],
            color,
        });
    }

    vertices
}
