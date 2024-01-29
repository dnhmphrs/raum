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
    // @location(2) signal_value: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

struct TimeUniform {
    time: f32,
};

@group(1) @binding(0)
var<uniform> time: TimeUniform;

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;
    // Use instance.signal_value for the Y coordinate
    out.clip_position = camera.view_proj * vec4<f32>(vertex.position.x, vertex.position.y, vertex.position.z + instance.instance_position.z, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//     let speed = 5.0; // Control the speed of the rain
//     let repeat_y = 250.0; // The height at which the rain repeats

//     // Calculate the raindrop effect
//     let y_effect = fract(in.clip_position.y / repeat_y - time.time * speed);
// // Initialize opacity
//     var opacity: f32 = 0.0;

//     // Determine the opacity based on y_effect
//     if (y_effect < 0.5) {
//         opacity = 1.0;
//     } else {
//         opacity = 0.0;
//     }

    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}