struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0)
var last_frame: texture_2d<f32>;

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
    let frag_coord = in.uv * vec2<f32>(textureDimensions(last_frame));
    let col = textureLoad(last_frame, vec2<u32>(frag_coord), 0);
    return vec4(col.rgb / col.a, 1.0);
}
