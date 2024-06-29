// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
};

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

struct TimeUniform {
    time: f32,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> time: TimeUniform;

// Complex number utilities
fn complex_div(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    let denom = b.x * b.x + b.y * b.y;
    return vec2<f32>(
        (a.x * b.x + a.y * b.y) / denom,
        (a.y * b.x - a.x * b.y) / denom
    );
}

fn complex_add(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(a.x + b.x, a.y + b.y);
}

fn complex_sub(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(a.x - b.x, a.y - b.y);
}

fn complex_mult(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

// Function to generate hexagonal lattice points
fn hexagonal_lattice(index: i32) -> vec2<f32> {
    let sqrt3: f32 = 1.7320508075688772;  // sqrt(3)
    let row: f32 = f32(index / 11);
    let col: f32 = f32(index % 11);

    // Calculate the offset for alternating rows using fract
    let offset: f32 = (1.0 - fract(row / 2.0)) * 0.5;

    let x: f32 = col * sqrt3;
    let y: f32 = row * 1.5 + offset;

    return vec2<f32>(x, y);
}

// Weierstrass P-function
fn weierstrass_wp(z: vec2<f32>, omega1: vec2<f32>, omega2: vec2<f32>) -> vec2<f32> {
    var wp_value: vec2<f32> = vec2<f32>(0.0, 0.0);
    for (var i: i32 = 0; i < 121; i = i + 1) {
        let lam: vec2<f32> = hexagonal_lattice(i);
        let z_minus_lam: vec2<f32> = complex_sub(z, lam);
        let term1: vec2<f32> = complex_div(vec2<f32>(1.0, 0.0), complex_mult(z_minus_lam, z_minus_lam));
        let term2: vec2<f32> = complex_div(vec2<f32>(1.0, 0.0), complex_mult(lam, lam));
        wp_value = complex_add(wp_value, complex_sub(term1, term2));
    }

    if (z.x != 0.0 || z.y != 0.0) {
        let term: vec2<f32> = complex_div(vec2<f32>(1.0, 0.0), complex_mult(z, z));
        wp_value = complex_add(wp_value, term);
    }

    return wp_value;
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = camera.view_proj * vec4<f32>(input.position * 1.25, 1.0);
    output.world_pos = input.position;  // Assuming input.position represents world position
    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // var adjustedPosition: vec2<f32> = (in.world_pos.xy - vec2<f32>(0.5)) * vec2<f32>(camera.view_proj[0][0], 1.0) + vec2<f32>(0.5);
    // var z: vec2<f32> = adjustedPosition * 2.0 - vec2<f32>(1.0);  // Convert to range -1 to 1

    let pi: f32 = 3.141592653589793;
    let omega1: vec2<f32> = vec2<f32>(1.0, 0.0);
    let omega2: vec2<f32> = vec2<f32>(cos(2.0 * pi / 3.0), sin(2.0 * pi / 3.0));

    // Calculate the Weierstrass P-function at the given point
    let wp: vec2<f32> = weierstrass_wp(in.world_pos.xy, omega1, omega2);

    // Get magnitude of the complex value
    let magnitude: f32 = length(wp) * 0.01;

    // Calculate the phase (argument) of the complex value
    let phase: f32 = atan(wp.y / wp.x);

    // Map the phase to RGB values
    let color: vec3<f32> = vec3<f32>(
        magnitude - (phase * 4.0 * pi),
        magnitude - sin(phase * 4.0 * pi),
        magnitude - cos(phase * 4.0 * pi)
    );

    return vec4<f32>(color, 1.0);
}
