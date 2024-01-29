struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct InstanceInput {
    @location(1) instance_position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

struct TimeUniform {
    time: f32,
};

@group(1) @binding(0)
var<uniform> time: TimeUniform;

@group(2) @binding(0)
var<storage, read> signalData: array<f32>;

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    // Calculate the index into the signal data buffer
    let signal_index: u32 = u32(instance.instance_position.z);
    let vertex_index: u32 = u32(vertex.position.x);
    let time_index: u32 = u32(time.time * 2500.0);
    let data_index = signal_index * 1000000u + vertex_index + time_index;

    // Fetch the signal value
    let signal_value = signalData[data_index];

    // Use signal_value for the Y coordinate
    out.clip_position = camera.view_proj * vec4<f32>(vertex.position.x * 0.02 - 5.0, signal_value, vertex.position.z + instance.instance_position.z - 4.0, 1.0);
    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}