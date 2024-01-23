// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
};

struct TimeUniform {
    time: f32,
};

@group(1) @binding(0)
var<uniform> time: TimeUniform;


@vertex
fn vs_main(
    model: VertexInput,
    @location(1) displacement: f32, // Add displacement as an input attribute
) -> VertexOutput {
    var out: VertexOutput;

  // More complex displacement calculation for a natural, flowing effect
    let time_factor = time.time * 5.0; // Adjust time scaling for speed
    let sin_wave = sin(model.position.x * 1.0 + time_factor) * 0.5;
    let cos_wave = cos(model.position.z * 0.5 - time_factor) * 0.3;
    let combined_wave = sin_wave + cos_wave;

    let new_displacement = displacement * combined_wave;

    let displaced_position = vec3<f32>(model.position.x, model.position.y + new_displacement, model.position.z);
    out.clip_position = camera.view_proj * vec4<f32>(displaced_position, 1.0);
    out.world_pos = displaced_position;
    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let scale = 0.002;
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