// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;


struct VertexInput {
    @location(0) position: vec3<f32>,
}

// struct InstanceInput {
//     @location(5) model_matrix_0: vec4<f32>,
//     @location(6) model_matrix_1: vec4<f32>,
//     @location(7) model_matrix_2: vec4<f32>,
//     @location(8) model_matrix_3: vec4<f32>,
// }

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
}

struct TimeUniform {
    time: f32,
    _padding1: f32,
    _padding2: f32,
    _padding3: f32,
};

@group(1) @binding(0)
var<uniform> time: TimeUniform;

@vertex
fn vs_main(
    model: VertexInput,
    // instance: InstanceInput,
) -> VertexOutput {
    // let model_matr ix = mat4x4<f32>(
    //     instance.model_matrix_0,
    //     instance.model_matrix_1,
    //     instance.model_matrix_2,
    //     instance.model_matrix_3,
    // );
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position * 0.5, 1.0);
    out.world_pos = (vec4<f32>(model.position, 1.0)).xyz;  // Compute the world position.
    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let scale = 0.0003;
    let clip_pos_truncated = vec3<f32>(in.clip_position.x, in.clip_position.y, in.clip_position.z);
    let adjusted_pos = abs(in.world_pos * clip_pos_truncated * scale);

    let pattern1 = log(sin(dot(adjusted_pos, clip_pos_truncated) * 0.01));
    let pattern2 = log(cos(dot(adjusted_pos, clip_pos_truncated) * 0.01));
    let pattern3 = log(tan(dot(adjusted_pos, clip_pos_truncated) * 10.0));

    let color1 = mix(pattern1, pattern2, sin(time.time * clip_pos_truncated.x * clip_pos_truncated.y * 0.01));
    let color2 = mix(pattern2, pattern3, cos(time.time * clip_pos_truncated.y * clip_pos_truncated.y * 0.01));
    let color3 = mix(pattern3, pattern2, sin(time.time * clip_pos_truncated.z * clip_pos_truncated.y * 0.0001));
    
    let final_color = vec4<f32>(color1, color2, color3, 1.0);

    return final_color;
}

// Fragment shader
// @fragment
// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//     let scale = 0.0025; // Adjusted scale factor for the pattern
//     let clip_pos_truncated = vec3<f32>(in.clip_position.x, in.clip_position.y, in.clip_position.z);
//     let adjusted_pos = abs(in.world_pos * clip_pos_truncated * scale * (sin(time.time) * 0.25 + 0.75));


//     // Updated pattern calculation to create more variation
//     let pattern1 = log(sin(dot(adjusted_pos, adjusted_pos) * 0.1));
//     let pattern2 = log(cos(dot(adjusted_pos, adjusted_pos) * 0.1));
//     let pattern3 = log(tan(dot(adjusted_pos, adjusted_pos) * 0.1));

//     // Map the pattern to a color range
//     let color1 = 0.5 + 0.5 * pattern1 ;
//     let color2 = 0.5 + 0.5 * pattern2 ;
//     let color3 = 0.5 + 0.5 * pattern3 ;
    
//     let final_color = vec4<f32>(color2 - 0.5, 1.0 - color2, color3, 1.0);


//     return final_color;
// }