use std::f32::consts::TAU;

use crate::vertex::StarVertex;
use rand::Rng;

/// Object-kind tags stored in `StarVertex::star_data[0]`. These must match the
/// `KIND_*` constants in the vertex shader (`src/shaders.rs`).
pub mod kind {
    pub const STAR: f32 = 0.0;
    pub const SHOOTING: f32 = 1.0;
    pub const GALAXY: f32 = 2.0;
    pub const NEBULA: f32 = 3.0;
    pub const COMET: f32 = 4.0;
    pub const SUPERNOVA: f32 = 5.0;
    pub const PULSAR: f32 = 6.0;
    pub const BLACK_HOLE: f32 = 7.0;
}

/// Trailing point sprites behind each shooting star.
const TRAIL_SEGMENTS: usize = 20;
/// Point sprites that make up a single distant galaxy.
const STARS_PER_GALAXY: usize = 600;
/// Overlapping soft blobs that make up one nebula cloud.
const NEBULA_BLOBS: usize = 7;
/// Tail point sprites behind a comet's coma.
const COMET_TRAIL: usize = 24;
/// Points forming a supernova's expanding shockwave ring.
const SHOCKWAVE_SEGMENTS: usize = 48;

/// How many of each kind of object to place in the scene.
pub struct SceneConfig {
    pub stars: usize,
    pub shooting_stars: usize,
    pub galaxies: usize,
    pub nebulae: usize,
    pub comets: usize,
    pub supernovae: usize,
    pub pulsars: usize,
    pub black_holes: usize,
}

/// Builds the full set of vertices for the scene. Objects are emitted roughly
/// back-to-front (nebulae and galaxies first, bright transient events last) so
/// alpha blending layers them sensibly.
pub fn generate_scene_vertices(config: &SceneConfig) -> Vec<StarVertex> {
    let mut rng = rand::rng();

    let capacity = config.stars
        + config.shooting_stars * (1 + TRAIL_SEGMENTS)
        + config.galaxies * STARS_PER_GALAXY
        + config.nebulae * NEBULA_BLOBS
        + config.comets * (1 + COMET_TRAIL)
        + config.supernovae * (1 + SHOCKWAVE_SEGMENTS)
        + config.pulsars
        + config.black_holes;
    let mut vertices = Vec::with_capacity(capacity);

    for _ in 0..config.nebulae {
        generate_nebula(&mut rng, &mut vertices);
    }
    for _ in 0..config.galaxies {
        generate_galaxy(&mut rng, &mut vertices);
    }
    generate_background_stars(&mut rng, &mut vertices, config.stars);
    // Black holes are drawn after the background so their dark disks occlude the
    // stars, galaxies and nebulae behind them, producing a visible "dark spot".
    for _ in 0..config.black_holes {
        generate_black_hole(&mut rng, &mut vertices);
    }
    for _ in 0..config.supernovae {
        generate_supernova(&mut rng, &mut vertices);
    }
    for _ in 0..config.pulsars {
        generate_pulsar(&mut rng, &mut vertices);
    }
    for _ in 0..config.comets {
        generate_comet(&mut rng, &mut vertices);
    }
    generate_shooting_stars(&mut rng, &mut vertices, config.shooting_stars);

    vertices
}

/// Background stars: 30% gather into random clusters, 70% follow a loose spiral.
fn generate_background_stars(rng: &mut impl Rng, vertices: &mut Vec<StarVertex>, num_stars: usize) {
    let num_clusters = rng.random_range(5..15);
    let mut clusters: Vec<(f32, f32, f32)> = Vec::with_capacity(num_clusters);
    for _ in 0..num_clusters {
        let cx = rng.random_range(-0.8..0.8);
        let cy = rng.random_range(-0.8..0.8);
        let radius = rng.random_range(0.05..0.25);
        clusters.push((cx, cy, radius));
    }

    let cluster_star_count = (num_stars as f32 * 0.3) as usize;
    let spiral_star_count = num_stars - cluster_star_count;

    // Clustered stars (Box-Muller-like Gaussian spread around a cluster center).
    for _ in 0..cluster_star_count {
        let cluster = &clusters[rng.random_range(0..clusters.len())];
        let r1: f32 = rng.random_range(0.0001..1.0);
        let r2: f32 = rng.random_range(0.0..TAU);
        let spread = (-2.0 * r1.ln()).sqrt() * cluster.2 * 0.5;
        let x = (cluster.0 + spread * r2.cos()).clamp(-1.0, 1.0);
        let y = (cluster.1 + spread * r2.sin()).clamp(-1.0, 1.0);
        let distance = (x * x + y * y).sqrt().min(1.0);
        vertices.push(generate_star(rng, x, y, distance));
    }

    // Spiral-distributed stars.
    for _ in 0..spiral_star_count {
        let angle = rng.random_range(0.0..TAU);
        let distance = rng.random_range(0.0..1.0);
        let spiral_angle = angle + distance * 4.0;
        let x = spiral_angle.cos() * distance;
        let y = spiral_angle.sin() * distance;
        vertices.push(generate_star(rng, x, y, distance));
    }
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
        star_data: [kind::STAR, seed, 0.0, 0.0],
        extra: [0.0; 4],
    }
}

/// A distant galaxy: a dense bright core surrounded by points arranged into
/// spiral arms (or a smooth elliptical blob) that slowly rotate as a rigid body.
fn generate_galaxy(rng: &mut impl Rng, vertices: &mut Vec<StarVertex>) {
    let cx = rng.random_range(-0.7..0.7);
    let cy = rng.random_range(-0.7..0.7);
    let scale = rng.random_range(0.08..0.18);
    let rot_speed = if rng.random_bool(0.5) { 1.0 } else { -1.0 } * rng.random_range(0.03..0.12);
    let is_spiral = rng.random_bool(0.7);
    let arms: usize = rng.random_range(2..=4);
    let twist = rng.random_range(2.0..3.5);

    // (core_rgb, arm_rgb) palettes.
    let palettes = [
        ([1.0, 0.95, 0.85], [0.55, 0.7, 1.0]),  // warm core, blue arms
        ([1.0, 0.92, 0.72], [1.0, 0.78, 0.55]), // golden elliptical
        ([1.0, 0.88, 0.95], [0.85, 0.55, 1.0]), // pink / violet
    ];
    let (core_rgb, arm_rgb) = palettes[rng.random_range(0..palettes.len())];

    for _ in 0..STARS_PER_GALAXY {
        // r^2 concentrates points toward the core.
        let u: f32 = rng.random_range(0.0..1.0);
        let r = u * u;
        let angle = if is_spiral {
            let arm = rng.random_range(0..arms) as f32;
            arm / arms as f32 * TAU + r * twist + rng.random_range(-0.3..0.3)
        } else {
            rng.random_range(0.0..TAU)
        };

        let is_core = r < 0.18;
        let base = if is_core { core_rgb } else { arm_rgb };
        let bfac = 1.0 - r * 0.7; // brighter toward the center
        let j = rng.random_range(0.85..1.0);
        let color = [
            base[0] * bfac * j,
            base[1] * bfac * j,
            base[2] * bfac * j,
            0.9,
        ];
        let seed = rng.random_range(0.0..100.0);

        vertices.push(StarVertex {
            position: [cx, cy],
            color,
            star_data: [kind::GALAXY, seed, r, angle],
            extra: [rot_speed, bfac, if is_core { 1.0 } else { 0.0 }, scale],
        });
    }
}

/// A nebula: a handful of large, very translucent, slowly pulsing colored blobs
/// that overlap into a soft cloud.
fn generate_nebula(rng: &mut impl Rng, vertices: &mut Vec<StarVertex>) {
    let cx = rng.random_range(-0.8..0.8);
    let cy = rng.random_range(-0.8..0.8);
    let base_size = rng.random_range(36.0..56.0);

    let palettes = [
        [1.0, 0.4, 0.55], // rose / emission
        [0.4, 0.6, 1.0],  // blue reflection
        [0.5, 1.0, 0.7],  // teal
        [0.8, 0.5, 1.0],  // violet
    ];
    let base = palettes[rng.random_range(0..palettes.len())];

    for _ in 0..NEBULA_BLOBS {
        let ox = rng.random_range(-0.09..0.09);
        let oy = rng.random_range(-0.09..0.09);
        let size = base_size * rng.random_range(0.55..1.0);
        let pulse = rng.random_range(0.1..0.4);
        let alpha = rng.random_range(0.04..0.10);
        let j = rng.random_range(0.7..1.0);
        let seed = rng.random_range(0.0..100.0);

        vertices.push(StarVertex {
            position: [cx + ox, cy + oy],
            color: [base[0] * j, base[1] * j, base[2] * j, alpha],
            star_data: [kind::NEBULA, seed, size, pulse],
            extra: [0.0; 4],
        });
    }
}

/// A comet: a bright soft coma with a tapering tail that drifts slowly across
/// the sky on a long cycle (distinct from the fast, brief shooting stars).
fn generate_comet(rng: &mut impl Rng, vertices: &mut Vec<StarVertex>) {
    let x = rng.random_range(-1.0..1.0);
    let y = rng.random_range(-1.0..1.0);
    let angle = rng.random_range(0.0..TAU);
    let dir_x = angle.cos();
    let dir_y = angle.sin();
    let seed = rng.random_range(0.0..100.0);

    // Head / coma (trail_offset = 0.0).
    vertices.push(StarVertex {
        position: [x, y],
        color: [1.0, 1.0, 1.0, 1.0],
        star_data: [kind::COMET, seed, dir_x, dir_y],
        extra: [0.0; 4],
    });

    for i in 1..=COMET_TRAIL {
        let trail_offset = i as f32 / COMET_TRAIL as f32;
        vertices.push(StarVertex {
            position: [x, y],
            color: [1.0, 1.0, 1.0, 1.0],
            star_data: [kind::COMET, seed, dir_x, dir_y],
            extra: [trail_offset, 0.0, 0.0, 0.0],
        });
    }
}

/// A supernova: a dim progenitor that periodically flashes brilliantly and
/// throws off an expanding, fading shockwave ring before settling again.
fn generate_supernova(rng: &mut impl Rng, vertices: &mut Vec<StarVertex>) {
    let x = rng.random_range(-0.85..0.85);
    let y = rng.random_range(-0.85..0.85);
    let seed = rng.random_range(0.0..100.0);
    let period = rng.random_range(14.0..40.0);

    // Core / progenitor (extra[0] = 0.0 → core role).
    vertices.push(StarVertex {
        position: [x, y],
        color: [0.8, 0.85, 1.0, 1.0],
        star_data: [kind::SUPERNOVA, seed, period, 0.0],
        extra: [0.0; 4],
    });

    // Shockwave ring points (extra[0] = 1.0 → ring role).
    for i in 0..SHOCKWAVE_SEGMENTS {
        let angle = i as f32 / SHOCKWAVE_SEGMENTS as f32 * TAU;
        vertices.push(StarVertex {
            position: [x, y],
            color: [1.0, 0.9, 0.7, 1.0],
            star_data: [kind::SUPERNOVA, seed, period, angle],
            extra: [1.0, 0.0, 0.0, 0.0],
        });
    }
}

/// A pulsar: a steady, dense bluish point that emits sharp rhythmic pulses.
fn generate_pulsar(rng: &mut impl Rng, vertices: &mut Vec<StarVertex>) {
    let x = rng.random_range(-0.9..0.9);
    let y = rng.random_range(-0.9..0.9);
    let seed = rng.random_range(0.0..100.0);
    let pulse_period = rng.random_range(0.4..1.6);

    vertices.push(StarVertex {
        position: [x, y],
        color: [0.75, 0.85, 1.0, 1.0],
        star_data: [kind::PULSAR, seed, pulse_period, 0.0],
        extra: [0.0; 4],
    });
}

/// A black hole: a single sprite whose opaque dark disk occludes the background
/// (the "dark spot"), ringed by a glowing accretion disk / photon ring with a
/// slowly rotating, Doppler-brightened side. `star_data` carries the sprite size
/// and spin speed; `color` is the disk tint.
fn generate_black_hole(rng: &mut impl Rng, vertices: &mut Vec<StarVertex>) {
    let x = rng.random_range(-0.8..0.8);
    let y = rng.random_range(-0.8..0.8);
    let seed = rng.random_range(0.0..100.0);
    let size = rng.random_range(46.0..62.0); // pixels (capped at 63 in the shader)
    let spin = if rng.random_bool(0.5) { 1.0 } else { -1.0 } * rng.random_range(0.4..1.0);

    // Warm accretion-disk tints (orange / amber / gold).
    let palettes = [[1.0, 0.55, 0.2], [1.0, 0.7, 0.35], [1.0, 0.45, 0.55]];
    let tint = palettes[rng.random_range(0..palettes.len())];

    vertices.push(StarVertex {
        position: [x, y],
        color: [tint[0], tint[1], tint[2], 1.0],
        star_data: [kind::BLACK_HOLE, seed, size, spin],
        extra: [0.0; 4],
    });
}

/// Shooting stars: a bright head plus a short delayed trail that streaks across
/// the sky on a per-star cycle.
fn generate_shooting_stars(rng: &mut impl Rng, vertices: &mut Vec<StarVertex>, count: usize) {
    for _ in 0..count {
        let x = rng.random_range(-1.0..1.0);
        let y = rng.random_range(-1.0..1.0);
        let angle = rng.random_range(0.0..TAU);
        let dir_x = angle.cos();
        let dir_y = angle.sin();
        let seed = rng.random_range(0.0..100.0);

        // Head (trail_offset = 0.0).
        vertices.push(StarVertex {
            position: [x, y],
            color: [1.0, 1.0, 1.0, 1.0],
            star_data: [kind::SHOOTING, seed, dir_x, dir_y],
            extra: [0.0; 4],
        });

        for i in 1..=TRAIL_SEGMENTS {
            let trail_offset = i as f32 / TRAIL_SEGMENTS as f32;
            vertices.push(StarVertex {
                position: [x, y],
                color: [1.0, 1.0, 1.0, 1.0],
                star_data: [kind::SHOOTING, seed, dir_x, dir_y],
                extra: [trail_offset, 0.0, 0.0, 0.0],
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> SceneConfig {
        SceneConfig {
            stars: 2000,
            shooting_stars: 10,
            galaxies: 3,
            nebulae: 4,
            comets: 5,
            supernovae: 2,
            pulsars: 6,
            black_holes: 2,
        }
    }

    #[test]
    fn vertex_count_matches_inputs() {
        let c = test_config();
        let expected = c.stars
            + c.shooting_stars * (1 + TRAIL_SEGMENTS)
            + c.galaxies * STARS_PER_GALAXY
            + c.nebulae * NEBULA_BLOBS
            + c.comets * (1 + COMET_TRAIL)
            + c.supernovae * (1 + SHOCKWAVE_SEGMENTS)
            + c.pulsars
            + c.black_holes;
        assert_eq!(generate_scene_vertices(&c).len(), expected);
    }

    #[test]
    fn positions_stay_within_clip_space() {
        let vertices = generate_scene_vertices(&test_config());
        for v in &vertices {
            assert!(
                (-1.0..=1.0).contains(&v.position[0]),
                "x out of range: {}",
                v.position[0]
            );
            assert!(
                (-1.0..=1.0).contains(&v.position[1]),
                "y out of range: {}",
                v.position[1]
            );
        }
    }

    #[test]
    fn every_vertex_has_a_known_kind() {
        let known = [
            kind::STAR,
            kind::SHOOTING,
            kind::GALAXY,
            kind::NEBULA,
            kind::COMET,
            kind::SUPERNOVA,
            kind::PULSAR,
            kind::BLACK_HOLE,
        ];
        for v in &generate_scene_vertices(&test_config()) {
            assert!(
                known.contains(&v.star_data[0]),
                "unexpected kind tag: {}",
                v.star_data[0]
            );
        }
    }

    #[test]
    fn object_counts_match_per_kind() {
        let c = test_config();
        let v = generate_scene_vertices(&c);
        let count = |k: f32| v.iter().filter(|x| x.star_data[0] == k).count();

        assert_eq!(count(kind::STAR), c.stars);
        assert_eq!(
            count(kind::SHOOTING),
            c.shooting_stars * (1 + TRAIL_SEGMENTS)
        );
        assert_eq!(count(kind::GALAXY), c.galaxies * STARS_PER_GALAXY);
        assert_eq!(count(kind::NEBULA), c.nebulae * NEBULA_BLOBS);
        assert_eq!(count(kind::COMET), c.comets * (1 + COMET_TRAIL));
        assert_eq!(
            count(kind::SUPERNOVA),
            c.supernovae * (1 + SHOCKWAVE_SEGMENTS)
        );
        assert_eq!(count(kind::PULSAR), c.pulsars);
        assert_eq!(count(kind::BLACK_HOLE), c.black_holes);
    }
}
