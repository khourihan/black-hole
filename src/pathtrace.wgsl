struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct View {
    camera: mat4x4<f32>,
    position: vec3<f32>,
    focal_length: f32,
    resolution: vec2<u32>,
    _padding: vec2<f32>,
};

@group(0) @binding(0)
var last_frame: texture_2d<f32>;
@group(0) @binding(1)
var last_frame_sampler: sampler;

@group(1) @binding(0)
var<uniform> view: View;

@group(2) @binding(0)
var sky_texture: texture_cube<f32>;
@group(2) @binding(1)
var sky_sampler: sampler;

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    let u = f32((in.vertex_index << 1u) & 2u);
    let v = f32(in.vertex_index & 2u);
    let uv = vec2<f32>(u, v);

    let position = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);

    return VertexOutput(position, uv);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let frag_coord = in.uv * vec2<f32>(view.resolution.xy);
    // TODO: jitter frag coord
    let p = (2.0 * frag_coord - vec2<f32>(view.resolution.xy)) / f32(view.resolution.y);

    let rd = normalize((view.camera * normalize(vec4(p, view.focal_length, 1.0))).xyz);
    let ro = view.position;

    let col = textureSample(sky_texture, sky_sampler, rd);

    return vec4(col.xyz, 1.0);
}
