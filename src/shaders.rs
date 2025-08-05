pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 460

            layout(location = 0) in vec2 position;
            layout(location = 1) in vec4 color;
            
            layout(push_constant) uniform PushConstants {
                float time;
            } push;
            
            layout(location = 0) out vec4 v_color;

            // Better hash function for more random distribution
            float hash(vec2 p) {
                vec3 p3 = fract(vec3(p.xyx) * vec3(443.897, 441.423, 437.195));
                p3 += dot(p3, p3.yzx + 19.19);
                return fract((p3.x + p3.y) * p3.z);
            }

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
                
                // Generate multiple frequencies of twinkle
                float h = hash(position);  // Unique per star
                float twinkle = 1.0;
                
                // Layer multiple sine waves for more natural twinkling
                twinkle *= sin(push.time * (1.0 + h) * 2.0) * 0.5 + 0.5;
                twinkle *= sin(push.time * (2.0 + h) * 3.0) * 0.25 + 0.75;
                twinkle *= sin(push.time * (3.0 + h) * 5.0) * 0.15 + 0.85;
                
                // Increase base point size to allow for glow
                float brightness = (color.r + color.g + color.b) / 3.0;
                gl_PointSize = (2.0 + brightness * 3.0) * twinkle;
                
                v_color = vec4(color.rgb * twinkle, color.a);
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
            layout(location = 0) out vec4 f_color;

            void main() {
                // Calculate distance from center of point
                vec2 centerDist = 2.0 * gl_PointCoord - 1.0;
                float dist = dot(centerDist, centerDist);
                
                // Discard pixels far outside the glow radius
                if (dist > 2.0) {
                    discard;
                }
                
                // Create multiple layers of glow
                float innerGlow = exp(-dist * 8.0);        // Bright core
                float midGlow = exp(-dist * 4.0) * 0.7;    // Medium glow
                float outerGlow = exp(-dist * 2.0) * 0.3;  // Soft outer glow
                
                // Combine the glow layers
                float glow = innerGlow + midGlow + outerGlow;
                
                // Create color variation for the glow
                vec3 glowColor = mix(v_color.rgb, v_color.rgb * 1.5, innerGlow);
                
                // Add slight color shift towards blue in the outer glow
                vec3 outerColor = mix(glowColor, vec3(0.6, 0.8, 1.0), outerGlow * 0.3);
                
                // Final color combining all effects
                f_color = vec4(outerColor * glow, v_color.a * glow);
            }
        "
    }
}
