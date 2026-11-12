// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    var out: VertexOutput;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    out.world_pos = (model_matrix * vec4<f32>(model.position, 1.0)).xyz;  // Compute the world position.
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let scale = 0.0025; // Adjusted scale factor for the pattern
    let clip_pos_truncated = vec3<f32>(in.clip_position.x, in.clip_position.y, in.clip_position.z);
    let adjusted_pos = abs(in.world_pos * clip_pos_truncated * scale);


    // Updated pattern calculation to create more variation
    let pattern1 = log(sin(dot(adjusted_pos, adjusted_pos) * 0.025));
    let pattern2 = log(cos(dot(adjusted_pos, adjusted_pos) * 0.025));
    let pattern3 = log(tan(dot(adjusted_pos, adjusted_pos) * 0.025));

    // Map the pattern to a color range
    let color1 = 0.5 + 0.5 * pattern1;
    let color2 = 0.5 + 0.5 * pattern2;
    let color3 = 0.5 + 0.5 * pattern3;
    let final_color = vec4<f32>(1.0 - color1,1.0 -  color2,1.0 -  color3, 1.0); // Grayscale color

    return final_color;
}




// @fragment
// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//     var temp_pos = in.world_pos;
//     let mut pattern = 0.0;

//     for _ in 0..5 {
//         temp_pos = abs(temp_pos) / dot(temp_pos, temp_pos) - 1.5;
//         pattern += abs(dot(temp_pos, temp_pos));
//     }

//     let color = 0.5 + 0.5 * cos(pattern);
//     let final_color = vec4<f32>(color, color * 0.7, color * 0.5, 1.0);

//     return final_color;
// }