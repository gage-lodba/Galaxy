pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 460

            layout(location = 0) in vec2 position;
            layout(location = 1) in vec4 color;
            layout(location = 2) in vec4 star_data; // [type, seed, dir_x, dir_y]

            layout(push_constant) uniform PushConstants {
                float time;
            } push;

            layout(location = 0) out vec4 v_color;
            layout(location = 1) out float v_star_type;
            layout(location = 2) out vec2 v_direction;

            // Hash function for random distribution
            float hash(vec2 p) {
                vec3 p3 = fract(vec3(p.xyx) * vec3(443.897, 441.423, 437.195));
                p3 += dot(p3, p3.yzx + 19.19);
                return fract((p3.x + p3.y) * p3.z);
            }

            void main() {
                float star_type = star_data.x;
                float seed = star_data.y;
                vec2 dir = star_data.zw;

                if (star_type < 0.5) {
                    // === Normal twinkling star ===
                    gl_Position = vec4(position, 0.0, 1.0);

                    float h = hash(position + seed);
                    float twinkle = 1.0;

                    // Per-star speed factor: some twinkle fast, others very slowly
                    float speed = 0.3 + h * 2.7; // Range 0.3x to 3.0x speed

                    // Layer multiple sine waves for natural twinkling
                    twinkle *= sin(push.time * speed * 2.0) * 0.5 + 0.5;
                    twinkle *= sin(push.time * speed * 3.0 + seed) * 0.25 + 0.75;
                    twinkle *= sin(push.time * speed * 5.0 + seed * 2.0) * 0.15 + 0.85;

                    // Random bright flashes: occasionally spike brightness
                    float flash = pow(max(sin(push.time * (0.3 + h * 0.7) + seed), 0.0), 20.0);
                    twinkle += flash * 0.5;

                    float brightness = (color.r + color.g + color.b) / 3.0;
                    gl_PointSize = (1.0 + brightness * 1.5) * twinkle;

                    v_color = vec4(color.rgb * twinkle, color.a);
                    v_star_type = 0.0;
                    v_direction = vec2(0.0);
                } else {
                    // === Shooting star (head or trail segment) ===
                    float trail_offset = star_type - 1.0; // 0.0 = head, >0 = trail

                    // Each shooting star has a cycle period based on its seed
                    float period = 4.0 + seed * 0.3; // 4-34 second cycle
                    float phase = mod(push.time + seed * 7.0, period);
                    float travel_time = 0.8; // How long the streak is visible
                    float trail_spacing = 0.06; // Time gap between trail segments

                    // Trail segments are delayed behind the head
                    float local_phase = phase - trail_offset * trail_spacing;

                    if (local_phase > 0.0 && local_phase < travel_time) {
                        float t = local_phase / travel_time;
                        float speed = 1.5;
                        vec2 offset = dir * t * speed;
                        gl_Position = vec4(position + offset, 0.0, 1.0);

                        // Head fades in then out; trail segments fade more
                        float alpha = smoothstep(0.0, 0.1, t) * smoothstep(1.0, 0.4, t);
                        // Trail segments are dimmer the further back they are
                        float trail_fade = 1.0 - trail_offset;
                        alpha *= trail_fade * trail_fade;

                        // Head is large, trail tapers off
                        float size = mix(4.0, 1.0, trail_offset);
                        gl_PointSize = size * smoothstep(0.0, 0.05, t);

                        // Color shifts from white (head) to blue-white (tail)
                        vec3 col = mix(vec3(1.0), vec3(0.5, 0.6, 1.0), trail_offset);
                        v_color = vec4(col, alpha);
                        v_star_type = 1.0;
                        v_direction = normalize(dir);
                    } else {
                        // Inactive: hide the vertex
                        gl_Position = vec4(2.0, 2.0, 0.0, 1.0);
                        gl_PointSize = 0.0;
                        v_color = vec4(0.0);
                        v_star_type = 1.0;
                        v_direction = vec2(0.0);
                    }
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
            layout(location = 1) in float v_star_type;
            layout(location = 2) in vec2 v_direction;

            layout(location = 0) out vec4 f_color;

            void main() {
                vec2 centerDist = 2.0 * gl_PointCoord - 1.0;

                if (v_star_type < 0.5) {
                    // === Normal star: circular glow ===
                    float dist = dot(centerDist, centerDist);

                    if (dist > 2.0) {
                        discard;
                    }

                    float innerGlow = exp(-dist * 8.0);
                    float midGlow = exp(-dist * 4.0) * 0.7;
                    float outerGlow = exp(-dist * 2.0) * 0.3;
                    float glow = innerGlow + midGlow + outerGlow;

                    vec3 glowColor = mix(v_color.rgb, v_color.rgb * 1.5, innerGlow);
                    vec3 outerColor = mix(glowColor, vec3(0.6, 0.8, 1.0), outerGlow * 0.3);

                    f_color = vec4(outerColor * glow, v_color.a * glow);
                } else {
                    // === Shooting star: elongated streak ===
                    // Rotate point coord into streak's local space
                    vec2 dir = v_direction;
                    vec2 perp = vec2(-dir.y, dir.x);

                    float along = dot(centerDist, dir);   // Along streak direction
                    float across = dot(centerDist, perp);  // Perpendicular

                    // Elongated shape: stretch along direction, narrow across
                    float streakDist = along * along * 0.3 + across * across * 4.0;

                    if (streakDist > 2.0) {
                        discard;
                    }

                    float core = exp(-streakDist * 6.0);
                    float glow = exp(-streakDist * 2.0) * 0.5;
                    float intensity = core + glow;

                    // Bright white core fading to blue-white
                    vec3 streakColor = mix(vec3(0.6, 0.7, 1.0), vec3(1.0), core);

                    f_color = vec4(streakColor * intensity, v_color.a * intensity);
                }
            }
        "
    }
}
