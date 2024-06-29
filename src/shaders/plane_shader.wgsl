// Vertex shader (assuming minimal transformation, just passing position through)
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

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = camera.view_proj * vec4<f32>(input.position * 1.0, 1.0); // Adjust scaling if necessary
    output.world_pos = input.position;  // Assuming input.position represents world position
    return output;
}

// Fragment shader
struct FragmentOutput {
    @location(0) color: vec4<f32>,
};

// Function to map fragment position to the fundamental domain
fn map_to_fundamental_domain(position: vec2<f32>, omega1: vec2<f32>, omega2: vec2<f32>) -> vec2<f32> {
    let denom: f32 = omega1.x * omega2.y - omega1.y * omega2.x;
    let u: f32 = (position.x * omega2.y - position.y * omega2.x) / denom;
    let v: f32 = (-position.x * omega1.y + position.y * omega1.x) / denom;
    return vec2<f32>(u, v);
}

// Function to calculate distance from a point to the center of its fundamental domain
fn distance_to_center(position: vec2<f32>, omega1: vec2<f32>, omega2: vec2<f32>) -> f32 {
    // Calculate mapped position in the fundamental domain
    let mapped_pos: vec2<f32> = map_to_fundamental_domain(position, omega1, omega2);
    
    // Calculate the center of the fundamental domain
    let center: vec2<f32> = fract(mapped_pos) - 0.5; // Center relative to the current fundamental domain
    
    // Calculate distance to the center
    return length(center);
}

// Function to calculate distance to the inner circle
fn distance_to_inner_circle(position: vec2<f32>, omega1: vec2<f32>, omega2: vec2<f32>, inner_radius: f32) -> f32 {
    // Calculate mapped position in the fundamental domain
    let mapped_pos: vec2<f32> = map_to_fundamental_domain(position, omega1, omega2);
    
    // Calculate the center of the fundamental domain
    let center: vec2<f32> = fract(mapped_pos) - 0.5; // Center relative to the current fundamental domain
    
    // Calculate distance to the center
    let dist_to_center: f32 = length(center);
    
    // Calculate distance to the inner circle
    let dist_to_inner_circle: f32 = abs(inner_radius - dist_to_center);
    
    return dist_to_inner_circle;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    // Define period vectors (lattice vectors)
    let omega1: vec2<f32> = vec2<f32>(1.0, 0.0); // Adjust these vectors as needed
    let omega2: vec2<f32> = vec2<f32>(0.5, 0.86602540378); // sqrt(3)/2
    
    // Parameters for circles
    let outer_radius: f32 = 0.3; // Radius of the outer circle
    let inner_radius: f32 = 0.1; // Radius of the inner circle
    
    // Calculate distances to the center and inner circle
    let dist_to_center: f32 = distance_to_center(in.world_pos.xz, omega1, omega2);
    let dist_to_inner_circle: f32 = distance_to_inner_circle(in.world_pos.xz, omega1, omega2, inner_radius);
    
    // Smoothstep to create smooth transitions between different color regions
    let threshold_outer: f32 = 0.25; // Adjust this threshold for the outer circle
    let threshold_inner: f32 = 0.15; // Adjust this threshold for the inner circle
    
    let fade_outer: f32 = smoothstep(outer_radius - threshold_outer, outer_radius, dist_to_center);
    let fade_inner: f32 = smoothstep(inner_radius - threshold_inner, inner_radius, dist_to_inner_circle);
    
    // Assign colors based on the fading effect
    let color_background: vec3<f32> = vec3<f32>(0.0, 0.0, 0.5); // Yellow for the inner circle
    let color_outer_circle: vec3<f32> = vec3<f32>(0.0, 1.0, 0.0); // Green for the outer circle
    let color_inner_circle: vec3<f32> = vec3<f32>(1.0, 1.0, 0.0); // Blue for the background
    
    // Mix colors based on the fade distances
    let color_outer: vec3<f32> = mix(color_outer_circle, color_background, fade_outer);
    let final_color: vec3<f32> = mix(color_inner_circle, color_outer, fade_inner);
    
    // Output color
    var frag_output: FragmentOutput;
    frag_output.color = vec4<f32>(final_color, 1.0);
    return frag_output;
}


// // Fragment shader
// struct FragmentOutput {
//     @location(0) color: vec4<f32>,
// };

// // Function to map fragment position to the fundamental domain
// fn map_to_fundamental_domain(position: vec2<f32>, omega1: vec2<f32>, omega2: vec2<f32>) -> vec2<f32> {
//     let denom: f32 = omega1.x * omega2.y - omega1.y * omega2.x;
//     let u: f32 = (position.x * omega2.y - position.y * omega2.x) / denom;
//     let v: f32 = (-position.x * omega1.y + position.y * omega1.x) / denom;
//     return vec2<f32>(u, v);
// }

// // Function to calculate distance from a point to the center of its fundamental domain
// fn distance_to_center(position: vec2<f32>, omega1: vec2<f32>, omega2: vec2<f32>) -> f32 {
//     // Calculate mapped position in the fundamental domain
//     let mapped_pos: vec2<f32> = map_to_fundamental_domain(position, omega1 , omega2);
    
//     // Calculate the center of the fundamental domain
//     let center: vec2<f32> = fract(mapped_pos) - 0.5; // Center relative to the current fundamental domain
    
//     // Calculate distance to the center
//     return length(center);
// }

// @fragment
// fn fs_main(in: VertexOutput) -> FragmentOutput {
//     // Define period vectors (lattice vectors)
//     let omega1: vec2<f32> = vec2<f32>(1.0, 0.0); // Adjust these vectors as needed
//     let omega2: vec2<f32> = vec2<f32>(0.5, 0.86602540378); // sqrt(3)/2

//     // Calculate distance to the center of the current fundamental domain
//     let dist_to_center: f32 = distance_to_center(in.world_pos.xz, omega1, omega2);

//     // Smoothstep to create a smooth circular gradient
//     let threshold: f32 = 0.5; // Adjust this threshold to control the fading effect range
//     let fade: f32 = smoothstep(0.0, threshold, dist_to_center);

//     // Assign color based on the fading effect
//     let color_near_lattice: vec3<f32> = vec3<f32>(0.0, 1.0, 0.0); // Red color for points near lattice
//     let color_far_from_lattice: vec3<f32> = vec3<f32>(0.0, 0.0, 1.0); // Green color for points far from lattice

//     let color: vec3<f32> = mix(color_near_lattice, color_far_from_lattice, fade);

//     // Output color
//     var frag_output: FragmentOutput;
//     frag_output.color = vec4<f32>(color, 1.0);
//     return frag_output;
// }