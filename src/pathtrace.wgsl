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

// @group(0) @binding(0)
// var last_frame: texture_2d<f32>;
// @group(0) @binding(1)
// var last_frame_sampler: sampler;

@group(0) @binding(0)
var<uniform> view: View;

@group(1) @binding(0)
var sky_texture: texture_cube<f32>;
@group(1) @binding(1)
var sky_sampler: sampler;

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    let u = f32((in.vertex_index << 1u) & 2u);
    let v = f32(in.vertex_index & 2u);
    let uv = vec2<f32>(u, v);

    let position = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);

    return VertexOutput(position, uv);
}

fn inverse(m: mat4x4<f32>) -> mat4x4<f32> {
    let a00 = m[0][0];
    let a01 = m[0][1];
    let a02 = m[0][2];
    let a03 = m[0][3];
    let a10 = m[1][0];
    let a11 = m[1][1];
    let a12 = m[1][2];
    let a13 = m[1][3];
    let a20 = m[2][0];
    let a21 = m[2][1];
    let a22 = m[2][2];
    let a23 = m[2][3];
    let a30 = m[3][0];
    let a31 = m[3][1];
    let a32 = m[3][2];
    let a33 = m[3][3];

    let b00 = a00 * a11 - a01 * a10;
    let b01 = a00 * a12 - a02 * a10;
    let b02 = a00 * a13 - a03 * a10;
    let b03 = a01 * a12 - a02 * a11;
    let b04 = a01 * a13 - a03 * a11;
    let b05 = a02 * a13 - a03 * a12;
    let b06 = a20 * a31 - a21 * a30;
    let b07 = a20 * a32 - a22 * a30;
    let b08 = a20 * a33 - a23 * a30;
    let b09 = a21 * a32 - a22 * a31;
    let b10 = a21 * a33 - a23 * a31;
    let b11 = a22 * a33 - a23 * a32;

    let inv_det = 1.0 / (b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06);

    return mat4x4(
        (a11 * b11 - a12 * b10 + a13 * b09) * inv_det,
        (a02 * b10 - a01 * b11 - a03 * b09) * inv_det,
        (a31 * b05 - a32 * b04 + a33 * b03) * inv_det,
        (a22 * b04 - a21 * b05 - a23 * b03) * inv_det,
        (a12 * b08 - a10 * b11 - a13 * b07) * inv_det,
        (a00 * b11 - a02 * b08 + a03 * b07) * inv_det,
        (a32 * b02 - a30 * b05 - a33 * b01) * inv_det,
        (a20 * b05 - a22 * b02 + a23 * b01) * inv_det,
        (a10 * b10 - a11 * b08 + a13 * b06) * inv_det,
        (a01 * b08 - a00 * b10 - a03 * b06) * inv_det,
        (a30 * b04 - a31 * b02 + a33 * b00) * inv_det,
        (a21 * b02 - a20 * b04 - a23 * b00) * inv_det,
        (a11 * b07 - a10 * b09 - a12 * b06) * inv_det,
        (a00 * b09 - a01 * b07 + a02 * b06) * inv_det,
        (a31 * b01 - a30 * b03 - a32 * b00) * inv_det,
        (a20 * b03 - a21 * b01 + a22 * b00) * inv_det);
}

fn diag(a: vec4<f32>) -> mat4x4<f32> {
    return mat4x4(a.x, 0.0, 0.0, 0.0,
                0.0, a.y, 0.0, 0.0,
                0.0, 0.0, a.z, 0.0,
                0.0, 0.0, 0.0, a.w);
}

const cdist: f32 = 120.0;
const a: f32 = 0.99999;
const m: f32 = 1.0;
const Q: f32 = 0.9;
const eps: f32 = 0.005;
const dx: vec2<f32> = vec2<f32>(0.0, eps);

var<private> dt: f32 = 0.03;

fn sphere_intersect(ro: vec3<f32>, rd: vec3<f32>, sphere: vec4<f32>) -> f32 {
    let oc = ro - sphere.xyz;
    let b = dot(oc, rd);
    let c = dot(oc, oc) - sphere.w * sphere.w;
    let h = b * b - c;
    if (h < 0.0) {
        return 1e10;
    }

    return -b - sqrt(h);
}

// Kerr-Newman Metric
fn metric(x: vec4<f32>) -> mat4x4<f32> {
    let p = x.yzw;
    let rho = dot(p, p) - a * a;
    let r2 = 0.5 * (rho + sqrt(rho * rho + 4.0 * a * a * p.z * p.z));
    let r = sqrt(r2);
    let k = vec4(1.0, (r * p.x + a * p.y) / (r2 + a * a), (r * p.y - a * p.x) / (r2 + a * a), p.z / r);
    let f = smoothstep(cdist * 0.5, 0.0, r) * r2 * (2.0 * m * r - Q * Q) / (r2 * r2 + a * a * p.z * p.z);
    return f * mat4x4(k.x * k, k.y * k, k.z * k, k.w * k) + diag(vec4(-1.0, 1.0, 1.0, 1.0));
}

fn lagrangian(dxdt: vec4<f32>, x: vec4<f32>) -> f32 {
    let g = metric(x);
    return dot(g * dxdt, dxdt);
}

fn lagrangian_metric(dxdt: vec4<f32>, g: mat4x4<f32>) -> f32 {
    return dot(g * dxdt, dxdt);
}

fn null_momentum(v: vec3<f32>, x: vec3<f32>) -> vec4<f32> {
    return 2.0 * metric(vec4(0.0, x)) * vec4(1.0, v);
}

fn dxdt_from_momentum(p: vec4<f32>, x: vec4<f32>) -> vec4<f32> {
    return inverse(metric(x)) * p;
}

fn update_dt(P: vec4<f32>, x: vec4<f32>) -> bool {
    let p = x.yzw;
    let rho = dot(p, p) - a * a;
    let r2 = 0.5 * (rho + sqrt(rho * rho + 4.0 * a * a * p.z * p.z));
    let r = sqrt(r2);

    dt = mix(0.03, 10.0, pow(max(r - 1.0, 0.0) / cdist, 1.0));

    if (r < 1.0 && a <= 1.0 || length(P) > 45.0) {
        return true;
    }

    if (length(x.yzw) > cdist) {
        return true;
    }

    return false;
}

fn dhstep(s: mat2x4<f32>, dt: f32) -> mat2x4<f32> {
    let p = s[0];
    let x = s[1];

    let g = metric(x);
    let g_inv = inverse(g);
    let dxdt = g_inv * p;

    let dhdq = -(vec4(lagrangian(dxdt, x + dx.yxxx),
                      lagrangian(dxdt, x + dx.xyxx),
                      lagrangian(dxdt, x + dx.xxyx),
                      lagrangian(dxdt, x + dx.xxxy)) - lagrangian_metric(dxdt, g)) / eps;

    var dqp: mat2x4<f32>;

    dqp[0] = -dhdq * dt;
    dqp[1] = 2.0 * g_inv * p * dt;

    return dqp;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let frag_coord = in.uv * vec2<f32>(view.resolution.xy);
    // TODO: jitter frag coord
    let pos = (2.0 * frag_coord - vec2<f32>(view.resolution.xy)) / f32(view.resolution.y);

    let rd = normalize((view.camera * normalize(vec4(pos, view.focal_length, 1.0))).xyz);
    var ro = view.position;

    let t0 = sphere_intersect(ro, rd, vec4(0.0, 0.0, 0.0, cdist));
    if (t0 > 0.0 && t0 < 1e10) {
        ro += rd * t0;
    }

    var x = vec4(0.0, ro);
    var p = normalize(null_momentum(rd, x.yzw));

    let p0 = p.x;

    let steps = 128u;
    for (var i = 0u; i < steps; i++) {
        let dt1 = clamp(1.0 / length(p), 0.1, 4.0);
        var state = mat2x4(p, x);
        let dqp = dhstep(state, dt1 * dt);
        state += dqp;

        p = state[0];
        x = state[1];

        if (update_dt(p, x)) {
            break;
        }
    }

    let dxdt = dxdt_from_momentum(p, x);
    let out_dir = normalize(dxdt.yzw);

    let p1 = p.x;

    var col = vec3(0.0);

    if (length(x.yzw) > 3.0) {
        col = textureSample(sky_texture, sky_sampler, out_dir).rgb;
    }

    return vec4(col.xyz, 1.0);
}
