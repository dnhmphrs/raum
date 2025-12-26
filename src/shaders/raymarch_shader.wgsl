struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct TimeUniform {
    time: f32,
    _padding1: f32,
    _padding2: f32,
    _padding3: f32,
};

@group(1) @binding(0)
var<uniform> time: TimeUniform;

struct RaymarchUniforms {
    resolution: vec2<f32>,
    _padding: vec2<f32>,
};

@group(2) @binding(0)
var<uniform> uniforms: RaymarchUniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.position * 0.5 + 0.5;
    return out;
}

// ----- Noise and glitch helpers -----

fn hash(p: vec2<f32>) -> f32 {
    let p2 = fract(p * vec2<f32>(443.897, 441.423));
    let p3 = p2 + dot(p2, p2.yx + 19.19);
    return fract((p3.x + p3.y) * p3.x);
}

fn hash3(p: vec3<f32>) -> f32 {
    let p2 = fract(p * vec3<f32>(443.897, 441.423, 437.195));
    let p3 = p2 + dot(p2, p2.yzx + 19.19);
    return fract((p3.x + p3.y + p3.z) * p3.x);
}

// ----- Raymarching helpers -----

const MAX_STEPS: i32 = 80;
const MAX_DIST: f32 = 50.0;
const SURF_DIST: f32 = 0.002;

fn rotateY(angle: f32) -> mat3x3<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return mat3x3<f32>(
        vec3<f32>(c, 0.0, s),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(-s, 0.0, c)
    );
}

fn rotateX(angle: f32) -> mat3x3<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return mat3x3<f32>(
        vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(0.0, c, -s),
        vec3<f32>(0.0, s, c)
    );
}

fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}

fn sdSphere(p: vec3<f32>, r: f32) -> f32 {
    return length(p) - r;
}

fn sdBox(p: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}

fn sdOctahedron(p: vec3<f32>, s: f32) -> f32 {
    let p_abs = abs(p);
    return (p_abs.x + p_abs.y + p_abs.z - s) * 0.57735027;
}

// Distorted, glitchy scene - BIGGER shapes
fn sceneSDF(p: vec3<f32>) -> f32 {
    let t = time.time * 3.0;
    
    // Glitchy rotation
    let glitch_rot = step(0.9, sin(t * 7.0)) * 0.5;
    let rot = rotateY(t * 0.4 + glitch_rot) * rotateX(t * 0.3);
    var p_rot = rot * p;
    
    // Add noise displacement for fuzzy edges
    let noise_freq = 3.0;
    let noise_amp = 0.2 + 0.15 * sin(t * 2.0);
    let noise_offset = hash3(floor(p_rot * noise_freq + t)) * noise_amp;
    p_rot += noise_offset;
    
    // Central morphing shape - MUCH BIGGER
    let morph = sin(t * 2.0) * 0.5 + 0.5;
    let sphere = sdSphere(p_rot, 2.0);
    let box_shape = sdBox(p_rot, vec3<f32>(1.5 + 0.2 * sin(t)));
    let octa = sdOctahedron(p_rot, 2.5);
    
    var core = mix(sphere, box_shape, morph);
    core = mix(core, octa, sin(t * 0.7) * 0.5 + 0.5);
    
    // Orbiting fragments - BIGGER and closer
    var fragments = MAX_DIST;
    for (var i = 0; i < 12; i++) {
        let angle = f32(i) * 0.5236 + t * (0.5 + 0.3 * sin(f32(i)));
        let radius = 3.5 + 0.5 * sin(t * 2.0 + f32(i));
        let orbit_pos = vec3<f32>(
            cos(angle) * radius,
            sin(t * 1.5 + f32(i) * 0.7) * 1.2,
            sin(angle) * radius
        );
        
        // Glitchy size - BIGGER fragments
        let frag_size = 0.35 + 0.2 * step(0.8, hash(vec2<f32>(f32(i), floor(t * 8.0))));
        let frag = sdBox(p_rot - orbit_pos, vec3<f32>(frag_size));
        fragments = min(fragments, frag);
    }
    
    return smin(core, fragments, 0.4);
}

fn calcNormal(p: vec3<f32>) -> vec3<f32> {
    let e = vec2<f32>(0.001, 0.0);
    return normalize(vec3<f32>(
        sceneSDF(p + e.xyy) - sceneSDF(p - e.xyy),
        sceneSDF(p + e.yxy) - sceneSDF(p - e.yxy),
        sceneSDF(p + e.yyx) - sceneSDF(p - e.yyx)
    ));
}

fn raymarch(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var d = 0.0;
    for (var i = 0; i < MAX_STEPS; i++) {
        let p = ro + rd * d;
        let ds = sceneSDF(p);
        d += ds * 0.8;
        if ds < SURF_DIST || d > MAX_DIST {
            break;
        }
    }
    return d;
}

// ASCII-like dithering pattern
fn asciiPattern(uv: vec2<f32>, brightness: f32) -> f32 {
    let cell_size = 6.0;
    let cell = floor(uv * uniforms.resolution / cell_size);
    let cell_uv = fract(uv * uniforms.resolution / cell_size);
    
    let pattern_hash = hash(cell + floor(time.time * 0.5));
    
    if brightness > 0.8 {
        return 1.0;
    } else if brightness > 0.6 {
        let cross_h = step(0.4, cell_uv.y) * step(cell_uv.y, 0.6);
        let cross_v = step(0.4, cell_uv.x) * step(cell_uv.x, 0.6);
        return max(cross_h, cross_v);
    } else if brightness > 0.4 {
        let dist = length(cell_uv - 0.5);
        return step(dist, 0.25);
    } else if brightness > 0.2 {
        let dist = length(cell_uv - 0.5);
        return step(dist, 0.15) * step(0.5, pattern_hash);
    } else {
        return step(0.9, pattern_hash) * step(length(cell_uv - 0.5), 0.1);
    }
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = time.time;
    
    var uv = in.uv;
    
    // Aspect ratio correction
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    var screen_uv = uv * 2.0 - 1.0;
    screen_uv.x *= aspect;
    
    // Camera - CLOSER to scene, wider FOV feel
    let cam_jitter = hash(vec2<f32>(floor(t * 15.0), 0.0)) * 0.015;
    let cam_time = t * 0.6;
    let cam_dist = 6.0;  // Closer camera
    let ro = vec3<f32>(
        sin(cam_time) * cam_dist + cam_jitter,
        1.0 + sin(cam_time * 0.7) * 0.8,
        cos(cam_time) * cam_dist
    );
    
    // Look at origin with slight shake
    let shake = (hash(vec2<f32>(t * 20.0, 1.0)) - 0.5) * 0.008;
    let look_at = vec3<f32>(shake, shake, 0.0);
    let forward = normalize(look_at - ro);
    let right = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), forward));
    let up = cross(forward, right);
    
    // Wider FOV by scaling screen_uv
    let fov_scale = 1.3;
    let rd_jitter = (hash(vec2<f32>(screen_uv.x * 100.0, t * 30.0)) - 0.5) * 0.002;
    let rd = normalize(forward + (screen_uv.x * fov_scale + rd_jitter) * right + screen_uv.y * fov_scale * up);
    
    // Raymarch
    let d = raymarch(ro, rd);
    
    // Base silver/grey gradient background
    var brightness = 0.0;
    
    if d < MAX_DIST {
        let p = ro + rd * d;
        let n = calcNormal(p);
        
        // Harsh directional light
        let light_dir = normalize(vec3<f32>(1.0, 1.0, 0.5));
        let diff = max(dot(n, light_dir), 0.0);
        
        // Strong specular for metallic look
        let view_dir = normalize(ro - p);
        let ref_dir = reflect(-light_dir, n);
        let spec = pow(max(dot(view_dir, ref_dir), 0.0), 64.0);
        
        // Fresnel rim
        let fresnel = pow(1.0 - max(dot(view_dir, n), 0.0), 2.0);
        
        // Combine for silver metallic look
        brightness = diff * 0.5 + spec * 0.8 + fresnel * 0.3;
        
        // Add noise grain to brightness
        let grain = (hash(vec2<f32>(in.uv.x * 500.0 + t, in.uv.y * 500.0)) - 0.5) * 0.15;
        brightness += grain;
        
        // Edge detection for that ASCII outline feel
        let edge_strength = 1.0 - smoothstep(0.0, 0.02, abs(sceneSDF(p)));
        brightness = mix(brightness, 1.0, edge_strength * 0.5);
    }
    
    // Apply ASCII dithering
    let ascii = asciiPattern(in.uv, brightness);
    var final_brightness = ascii * brightness;
    
    // Occasional full-screen flash
    let flash = step(0.995, hash(vec2<f32>(floor(t * 4.0), 0.0))) * 0.2;
    final_brightness += flash;
    
    // Silver/grey tint with slight blue in shadows
    var col = vec3<f32>(final_brightness);
    col = mix(col, col * vec3<f32>(0.9, 0.95, 1.0), 1.0 - final_brightness);
    
    // Subtle vignette
    let vignette = 1.0 - length(in.uv - 0.5) * 0.5;
    col *= vignette;
    
    // Slight chromatic aberration
    let ca_offset = 0.0015;
    let r = asciiPattern(in.uv + vec2<f32>(ca_offset, 0.0), brightness);
    let b = asciiPattern(in.uv - vec2<f32>(ca_offset, 0.0), brightness);
    col.r = mix(col.r, r * brightness, 0.25);
    col.b = mix(col.b, b * brightness, 0.25);
    
    return vec4<f32>(col, 1.0);
}