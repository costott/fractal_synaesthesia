#version 330 core

in vec2 uv;
out vec4 frag_color;

uniform vec2 center;
uniform int max_ref_iteration;
uniform int max_iterations;
uniform float bailout2;
uniform float pixel_step;
uniform vec2 dimensions;

uniform sampler2D orbit_texture;

vec2 mandelbrot(vec2 z, vec2 c) {
    return vec2(
        (z.x + z.y) * (z.x - z.y) + c.x,
        2.0 * z.x * z.y + c.y
    );
}

void main() {
    float aspect = dimensions.x / dimensions.y;

    // Convert uv to centered coords between -1 and +1
    vec2 centered_uv = (uv - 0.5) * vec2(2.0 * aspect, 2.0);

    vec2 dc = centered_uv * pixel_step;
    vec2 c = center + dc;

    vec2 z = vec2(0.0, 0.0);

    for (int i = 0; i < max_iterations; i++) {
        z = mandelbrot(z, c);

        // Point escaped
        if (dot(z, z) > bailout2) {
            // TODO: colouring algorithm
            vec3 colour = vec3(float(i)/float(max_iterations), float(i)/float(max_iterations), float(i)/float(max_iterations));
            frag_color = vec4(colour, 1.0);
            return;
        } 
    }

    frag_color = vec4(1.0, 1.0, 1.0, 1.0);  
}