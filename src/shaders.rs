pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 460

            layout(location = 0) in vec2 position;
            layout(location = 1) in vec4 color;
            layout(location = 2) in vec4 star_data; // [kind, seed, a, b]
            layout(location = 3) in vec4 extra;     // [c, d, e, f]

            layout(push_constant) uniform PushConstants {
                float time;
            } push;

            layout(location = 0) out vec4 v_color;
            layout(location = 1) out float v_shape;     // 0.0 = round glow, 1.0 = streak
            layout(location = 2) out vec2 v_direction;   // streak orientation
            layout(location = 3) out float v_falloff;    // glow sharpness multiplier

            // Object kinds. Must match galaxy::kind in src/galaxy.rs.
            const int KIND_STAR      = 0;
            const int KIND_SHOOTING  = 1;
            const int KIND_GALAXY    = 2;
            const int KIND_NEBULA    = 3;
            const int KIND_COMET     = 4;
            const int KIND_SUPERNOVA = 5;
            const int KIND_PULSAR    = 6;
            const int KIND_BLACK_HOLE = 7;

            const float TAU = 6.28318530718;

            // Hash function for random distribution
            float hash(vec2 p) {
                vec3 p3 = fract(vec3(p.xyx) * vec3(443.897, 441.423, 437.195));
                p3 += dot(p3, p3.yzx + 19.19);
                return fract((p3.x + p3.y) * p3.z);
            }

            // Push the vertex off-screen so it isn't rasterized.
            void hide() {
                gl_Position = vec4(2.0, 2.0, 0.0, 1.0);
                gl_PointSize = 0.0;
                v_color = vec4(0.0);
            }

            void main() {
                int kind = int(star_data.x + 0.5);
                float seed = star_data.y;

                // Defaults (round, unsharpened glow); branches override as needed.
                v_shape = 0.0;
                v_direction = vec2(0.0);
                v_falloff = 1.0;

                if (kind == KIND_STAR) {
                    // === Normal twinkling star ===
                    gl_Position = vec4(position, 0.0, 1.0);

                    float h = hash(position + seed);
                    float twinkle = 1.0;
                    float speed = 0.3 + h * 2.7; // 0.3x to 3.0x speed

                    twinkle *= sin(push.time * speed * 2.0) * 0.5 + 0.5;
                    twinkle *= sin(push.time * speed * 3.0 + seed) * 0.25 + 0.75;
                    twinkle *= sin(push.time * speed * 5.0 + seed * 2.0) * 0.15 + 0.85;

                    float flash = pow(max(sin(push.time * (0.3 + h * 0.7) + seed), 0.0), 20.0);
                    twinkle += flash * 0.5;

                    float brightness = (color.r + color.g + color.b) / 3.0;
                    gl_PointSize = (1.0 + brightness * 1.5) * twinkle;
                    v_color = vec4(color.rgb * twinkle, color.a);
                }
                else if (kind == KIND_SHOOTING) {
                    // === Shooting star (head or trail segment) ===
                    vec2 dir = star_data.zw;
                    float trail_offset = extra.x; // 0.0 = head

                    float period = 4.0 + seed * 0.3;
                    float phase = mod(push.time + seed * 7.0, period);
                    float travel_time = 0.8;
                    float trail_spacing = 0.06;
                    float local_phase = phase - trail_offset * trail_spacing;

                    if (local_phase > 0.0 && local_phase < travel_time) {
                        float t = local_phase / travel_time;
                        vec2 offset = dir * t * 1.5;
                        gl_Position = vec4(position + offset, 0.0, 1.0);

                        float alpha = smoothstep(0.0, 0.1, t) * smoothstep(1.0, 0.4, t);
                        float trail_fade = 1.0 - trail_offset;
                        alpha *= trail_fade * trail_fade;

                        gl_PointSize = mix(4.0, 1.0, trail_offset) * smoothstep(0.0, 0.05, t);

                        vec3 col = mix(vec3(1.0), vec3(0.5, 0.6, 1.0), trail_offset);
                        v_color = vec4(col, alpha);
                        v_shape = 1.0;
                        v_direction = normalize(dir);
                    } else {
                        hide();
                    }
                }
                else if (kind == KIND_GALAXY) {
                    // === Distant galaxy (one point of many) ===
                    float r = star_data.z;
                    float angle0 = star_data.w;
                    float rot_speed = extra.x;
                    float bfac = extra.y;
                    float is_core = extra.z;
                    float scale = extra.w;

                    float a = angle0 + push.time * rot_speed;
                    vec2 off = scale * r * vec2(cos(a), sin(a));
                    gl_Position = vec4(position + off, 0.0, 1.0);

                    float tw = 0.8 + 0.2 * sin(push.time * (0.4 + seed * 0.05) + seed);
                    gl_PointSize = (mix(1.0, 2.4, bfac) + is_core * 1.2) * tw;
                    v_color = vec4(color.rgb * tw, color.a);
                    v_falloff = 1.6;
                }
                else if (kind == KIND_NEBULA) {
                    // === Nebula blob (large soft translucent cloud) ===
                    float size = star_data.z;
                    float pulse = star_data.w;
                    gl_Position = vec4(position, 0.0, 1.0);
                    float p = 0.85 + 0.15 * sin(push.time * pulse + seed);
                    gl_PointSize = min(size * p, 63.0);
                    v_color = color;
                    v_falloff = 0.28; // very broad, soft falloff
                }
                else if (kind == KIND_COMET) {
                    // === Comet (slow coma plus a tapering tail) ===
                    vec2 dir = star_data.zw;
                    float trail_offset = extra.x; // 0.0 = head/coma

                    float period = 16.0 + seed * 0.4;
                    float phase = mod(push.time + seed * 5.0, period);
                    float travel_time = 7.0;

                    if (phase >= travel_time) {
                        hide();
                    } else {
                        float t = phase / travel_time;
                        vec2 head = position + dir * (t * 2.6);
                        vec2 pos = head - dir * (trail_offset * 0.30);
                        gl_Position = vec4(pos, 0.0, 1.0);

                        float life = smoothstep(0.0, 0.05, t) * smoothstep(1.0, 0.85, t);
                        float headness = 1.0 - trail_offset;
                        float alpha = life * (0.12 + 0.88 * headness * headness);
                        gl_PointSize = min(mix(3.0, 14.0, headness), 63.0) * life;

                        vec3 col = mix(vec3(0.55, 0.75, 1.0), vec3(0.85, 1.0, 0.92), headness);
                        v_color = vec4(col, alpha);
                        v_falloff = mix(1.0, 0.45, headness);
                    }
                }
                else if (kind == KIND_SUPERNOVA) {
                    // === Supernova (flash + expanding shockwave) ===
                    float period = star_data.z;
                    float role = extra.x; // 0.0 = core, 1.0 = shockwave ring
                    float phase = mod(push.time + seed * 3.7, period);
                    float event = 3.5;

                    if (role < 0.5) {
                        gl_Position = vec4(position, 0.0, 1.0);
                        if (phase < event) {
                            float te = phase / event;
                            float flash = smoothstep(0.0, 0.04, te) * (1.0 - smoothstep(0.1, 1.0, te));
                            float bright = 0.4 + flash * 6.0;
                            gl_PointSize = min(mix(2.0, 26.0, flash), 63.0);
                            vec3 col = mix(vec3(0.7, 0.85, 1.0), vec3(1.0, 0.5, 0.3), te);
                            v_color = vec4(col * bright, 1.0);
                            v_falloff = mix(0.6, 1.2, te);
                        } else {
                            // Dim progenitor between events.
                            gl_PointSize = 1.6;
                            v_color = vec4(0.5, 0.6, 0.9, 0.8);
                        }
                    } else {
                        if (phase >= event) {
                            hide();
                        } else {
                            float te = phase / event;
                            float ang = star_data.w;
                            vec2 off = (te * 0.28) * vec2(cos(ang), sin(ang));
                            gl_Position = vec4(position + off, 0.0, 1.0);
                            float fade = 1.0 - te;
                            float alpha = smoothstep(0.0, 0.05, te) * fade * fade;
                            gl_PointSize = mix(1.0, 5.0, te);
                            vec3 col = mix(vec3(1.0, 0.95, 0.8), vec3(1.0, 0.4, 0.2), te);
                            v_color = vec4(col, alpha * 0.9);
                        }
                    }
                }
                else if (kind == KIND_PULSAR) {
                    // === Pulsar (sharp rhythmic double pulse) ===
                    float pp = star_data.z;
                    gl_Position = vec4(position, 0.0, 1.0);
                    float w = (mod(push.time, pp) / pp) * TAU;
                    float pulse = pow(max(sin(w), 0.0), 8.0)
                                + 0.6 * pow(max(sin(w + 0.6), 0.0), 16.0);
                    float bright = 0.5 + pulse * 4.0;
                    gl_PointSize = min(2.0 + pulse * 7.0, 63.0);
                    v_color = vec4(vec3(0.7, 0.82, 1.0) * bright, 1.0);
                    v_falloff = 1.3;
                }
                else if (kind == KIND_BLACK_HOLE) {
                    // === Black hole (dark disk + spinning accretion ring) ===
                    float size = star_data.z;
                    float spin_speed = star_data.w;
                    gl_Position = vec4(position, 0.0, 1.0);
                    gl_PointSize = min(size, 63.0);
                    float phase = push.time * spin_speed;
                    v_color = color;                              // accretion-disk tint
                    v_shape = 2.0;                                // black-hole fragment shape
                    v_direction = vec2(cos(phase), sin(phase));   // current spin phase
                }
                else {
                    hide();
                }
            }
        "
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r"
            #version 460

            layout(location = 0) in vec4 v_color;
            layout(location = 1) in float v_shape;
            layout(location = 2) in vec2 v_direction;
            layout(location = 3) in float v_falloff;

            layout(location = 0) out vec4 f_color;

            void main() {
                vec2 centerDist = 2.0 * gl_PointCoord - 1.0;

                if (v_shape < 0.5) {
                    // === Round glow (stars, galaxies, nebulae, supernovae, pulsars) ===
                    float dist = dot(centerDist, centerDist);

                    // centerDist is in [-1, 1]^2, so dist maxes at 2.0 in the
                    // corners; clip to the inscribed unit circle (dist > 1.0).
                    if (dist > 1.0) {
                        discard;
                    }

                    float f = v_falloff;
                    float innerGlow = exp(-dist * 8.0 * f);
                    float midGlow = exp(-dist * 4.0 * f) * 0.7;
                    float outerGlow = exp(-dist * 2.0 * f) * 0.3;
                    float glow = innerGlow + midGlow + outerGlow;

                    vec3 glowColor = mix(v_color.rgb, v_color.rgb * 1.5, innerGlow);
                    vec3 outerColor = mix(glowColor, vec3(0.6, 0.8, 1.0), outerGlow * 0.3);

                    f_color = vec4(outerColor * glow, v_color.a * glow);
                } else if (v_shape < 1.5) {
                    // === Elongated streak (shooting star) ===
                    vec2 dir = v_direction;
                    vec2 perp = vec2(-dir.y, dir.x);

                    float along = dot(centerDist, dir);
                    float across = dot(centerDist, perp);

                    float streakDist = along * along * 0.3 + across * across * 4.0;

                    if (streakDist > 2.0) {
                        discard;
                    }

                    float core = exp(-streakDist * 6.0);
                    float glow = exp(-streakDist * 2.0) * 0.5;
                    float intensity = core + glow;

                    vec3 streakColor = mix(vec3(0.6, 0.7, 1.0), vec3(1.0), core);

                    f_color = vec4(streakColor * intensity, v_color.a * intensity);
                } else {
                    // === Black hole: dark event-horizon disk + glowing photon ring ===
                    float dist = length(centerDist);
                    if (dist > 1.0) {
                        discard;
                    }

                    float ringPos = 0.55; // radius of the bright photon ring

                    // Doppler-style asymmetry: one side of the disk is brighter,
                    // and the bright side rotates with the encoded spin phase.
                    float ang = atan(centerDist.y, centerDist.x);
                    float spin = atan(v_direction.y, v_direction.x);
                    float doppler = 0.35 + 0.65 * (0.5 + 0.5 * cos(ang - spin));

                    if (dist < ringPos) {
                        // Event-horizon shadow: opaque (occludes the background)
                        // and pure black, brightening into the ring at its edge.
                        float d = ringPos - dist;
                        float innerEdge = exp(-(d * d) / 0.0016) * doppler;
                        vec3 col = mix(vec3(0.0), v_color.rgb, innerEdge);
                        f_color = vec4(col, 1.0);
                    } else {
                        // Outside the ring: bright photon ring + accretion halo,
                        // fading to transparent so the background shows through.
                        float d = dist - ringPos;
                        float ring = exp(-(d * d) / 0.0016);
                        float halo = exp(-(d * d) / 0.08) * 0.5;
                        float intensity = (ring + halo) * doppler;
                        vec3 col = mix(v_color.rgb, vec3(1.0), ring);
                        float alpha = clamp(ring + halo, 0.0, 1.0);
                        f_color = vec4(col * intensity, alpha);
                    }
                }
            }
        "
    }
}
