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
    let new_displacement = displacement * atan(cos(time.time) * 2.5);
    let displaced_position = vec3<f32>(model.position.x, model.position.y + new_displacement, model.position.z);
    out.clip_position = camera.view_proj * vec4<f32>(displaced_position, 1.0);
    out.world_pos = displaced_position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let scale = 0.0025;
    let clip_pos_truncated = vec3<f32>(in.clip_position.x, in.clip_position.y, in.clip_position.z);
    let adjusted_pos = abs(in.world_pos * clip_pos_truncated * scale * (sin(time.time) * 0.25 + 0.75));

    let pattern1 = log(sin(dot(adjusted_pos, adjusted_pos) * 0.025));
    let pattern2 = log(cos(dot(adjusted_pos, adjusted_pos) * 0.025));
    let pattern3 = log(tan(dot(adjusted_pos, adjusted_pos) * 0.025));

    let color1 = 0.5 + 0.5 * pattern1;
    let color2 = 0.5 + 0.5 * pattern2;
    let color3 = 0.5 + 0.5 * pattern3;
    let height = in.world_pos.y;
    
    let final_color = vec4<f32>(1.0 - color1, 1.0 - color2, 1.0 - color3, 1.0);

    return final_color;
}