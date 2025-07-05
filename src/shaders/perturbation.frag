#version 450 core
#extension GL_ARB_gpu_shader_fp64 : enable
#extension GL_ARB_gpu_shader_int64: enable

in vec2 uv;
out vec4 frag_color;

uniform vec2 center;
uniform int max_ref_iteration;
uniform int max_iterations;
uniform float bailout2;
uniform float pixel_step;
uniform vec2 dimensions;

uniform sampler2D orbit_texture;

dvec2 cmul(dvec2 a, dvec2 b) {
    return dvec2(
        a.x * b.x - a.y * b.y,
        a.x * b.y + a.y * b.x
    );
}

double cmodsq(dvec2 z) { return z.x * z.x + z.y * z.y; }
double cabs(dvec2 z) { return length(z); }

// Converts a pair of pixels into a single f64
double pixel_pair_to_double(vec4 pixel1, vec4 pixel2) {
    uvec4 unnorm_pixel1 = uvec4(pixel1 * 255.0);
    uvec4 unnorm_pixel2 = uvec4(pixel2 * 255.0);

    return uint64BitsToDouble(
         uint64_t(unnorm_pixel1.r) | 
        (uint64_t(unnorm_pixel1.g) << 8) | 
        (uint64_t(unnorm_pixel1.b) << 16) | 
        (uint64_t(unnorm_pixel1.a) << 24) | 
        (uint64_t(unnorm_pixel2.r) << 32) | 
        (uint64_t(unnorm_pixel2.g) << 40) | 
        (uint64_t(unnorm_pixel2.b) << 48) | 
        (uint64_t(unnorm_pixel2.a) << 56) 
    );
}

// Returns the orbit point at the given index
dvec2 get_orbit_point(int index) {
    ivec2 texCoord = ivec2(4 * index, 0);

    // Read the 4 pixels containing the pairs of 8 bytes
    vec4 pixel1 = texelFetch(orbit_texture, texCoord, 0);
    vec4 pixel2 = texelFetch(orbit_texture, texCoord + ivec2(1, 0), 0);
    vec4 pixel3 = texelFetch(orbit_texture, texCoord + ivec2(2, 0), 0);
    vec4 pixel4 = texelFetch(orbit_texture, texCoord + ivec2(3, 0), 0);

    // Combine bytes into f64s
    return dvec2(
        pixel_pair_to_double(pixel1, pixel2), // real part
        pixel_pair_to_double(pixel3, pixel4)  // imaginary part
    );
}

void main() {
    dvec2 dc = dvec2(
        (uv.x - 0.5) * dimensions.x * pixel_step,
        (uv.y - 0.5) * dimensions.y * pixel_step
    );

    // Uses this reference orbit method:
    // https://fractalforums.org/index.php?topic=4360.msg29835#msg29835
    precise dvec2 dz = dvec2(0.0, 0.0);
    int ref_iteration = 0;

    for (int i = 0; i < max_iterations; i++) {
        if (i > max_iterations) { break; }

        // mandelbrot perturbation | dz = 2.0 * dz * ref_z + dz * dz + dc
        precise dvec2 dz_new = cmul(dz, dz) + 2.0 * cmul(dz, get_orbit_point(ref_iteration)) + dc;
        dz = dz_new;
        ref_iteration++;

        // get this pixel's point | z = ref_z + dz
        precise dvec2 z = get_orbit_point(ref_iteration) + dz;

        // Point escaped
        if (cmodsq(z) > bailout2) {
            // TODO: colouring algorithm
            float t = float(i)/float(max_iterations);
            frag_color = vec4(t, t, t, 1.0);
            return;
        } 

        // Rebase |z + dz| < |dz| 
        if (cmodsq(z) < cmodsq(dz) || ref_iteration >= 5) {
            dz = z;
            ref_iteration = 0;
        }
    }

    frag_color = vec4(1.0, 1.0, 1.0, 1.0);  
}