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
    frame_count: u32,
    _padding: f32,
};

@group(0) @binding(0)
var last_frame: texture_2d<f32>;

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

const PI: f32 = 3.1415927;
const TAU: f32 = 6.28318531;

fn rotate2(v: vec2<f32>, t: f32) -> vec2<f32> {
    let s = sin(t);
    let c = cos(t);
    return vec2(v.x * c - v.y * s, v.x * s + v.y * c);
}

var<private> rng_state: u32 = 0u;

fn triple32(v: u32) -> u32 {
    var x = v;
    x ^= x >> 17u;
    x *= 0xED5AD4BBu;
    x ^= x >> 11u;
    x *= 0xAC4C1B51u;
    x ^= x >> 15u;
    x *= 0x31848BABu;
    x ^= x >> 14u;
    return x;
}

fn rand() -> f32 {
    rng_state = triple32(rng_state);
    return f32(rng_state) / f32(0xFFFFFFFFu);
}

fn rand2() -> vec2<f32> {
    return vec2<f32>(rand(), rand());
}

fn udir3() -> vec3<f32> {
    let z = rand2();
    let r = vec2(TAU * z.x, acos(2.0 * z.y - 1.0));
    let s = sin(r);
    let c = cos(r);
    return vec3(c.x * s.y, s.x * s.y, c.y);
}

fn hash44(p4: vec4<f32>) -> vec4<f32> {
    var p = fract(p4 * vec4(0.1031, 0.1030, 0.0973, 0.1099));
    p += dot(p, p.wzxy + 33.33);
    return fract((p.xxyz + p.yzzw) * p.zywx);
}

fn noise(p: vec3<f32>, octave: u32) -> f32 {
    let f = fract(p);
    let i = floor(p);

    let s = smoothstep(vec3(0.0), vec3(1.0), f);

    let t0 = hash44(vec4(i + vec3(0.0, 0.0, 0.0), f32(octave))).x;
    let t1 = hash44(vec4(i + vec3(1.0, 0.0, 0.0), f32(octave))).x;
    let t2 = hash44(vec4(i + vec3(0.0, 1.0, 0.0), f32(octave))).x;
    let t3 = hash44(vec4(i + vec3(1.0, 1.0, 0.0), f32(octave))).x;
    let t4 = hash44(vec4(i + vec3(0.0, 0.0, 1.0), f32(octave))).x;
    let t5 = hash44(vec4(i + vec3(1.0, 0.0, 1.0), f32(octave))).x;
    let t6 = hash44(vec4(i + vec3(0.0, 1.0, 1.0), f32(octave))).x;
    let t7 = hash44(vec4(i + vec3(1.0, 1.0, 1.0), f32(octave))).x;

    return mix(
        mix(mix(t0, t1, s.x), mix(t2, t3, s.x), s.y),
        mix(mix(t4, t5, s.x), mix(t6, t7, s.x), f.y),
        s.z
    );
}

fn fbm(p: vec3<f32>, iter: u32) -> f32 {
    var v = 0.0;
    var acc = 0.0;
    var att = 0.5;
    var scale = 1.0;

    for (var i = 0u; i < iter; i++) {
        v += att * noise(scale * p, iter);
        acc += att;
        att *= 0.5;
        scale *= 2.5;
    }

    var res = v;

    if (acc != 0.0) {
        res = v / acc;
    }

    return res;
}

const XYZ2sRGB: mat3x3<f32> = mat3x3<f32>(
    3.240,  -1.537, -0.499,
    -0.969, 1.876,   0.042,
    0.056,  -0.204,  1.057
);

fn xyz2rgb(xyz: vec3<f32>) -> vec3<f32> {
    return xyz * XYZ2sRGB;
}

fn rgb2xyz(rgb: vec3<f32>) -> vec3<f32> {
    return rgb * transpose(XYZ2sRGB);
}

// Computes the XYZ color of an ideal black-body radiator, given its temperature in Kelvin.
fn blackbody(t: f32) -> vec3<f32> {
    let u = (0.860117757 + 1.54118254E-4 * t + 1.28641212E-7 * t * t) / (1.0 + 8.42420235E-4 * t + 7.08145163E-7 * t * t);
    let v = (0.317398726 + 4.22806245E-5 * t + 4.20481691E-8 * t * t) / (1.0 - 2.89741816E-5 * t + 1.61456053E-7 * t * t);

    let xyy = vec2(3.0 * u, 2.0 * v) / (2.0 * u - 8.0 * v + 4.0);
    return vec3(xyy.x / xyy.y, 1.0, (1.0 - xyy.x - xyy.y) / xyy.y);
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
const a: f32 = 0.3;
const m: f32 = 1.0;
const Q: f32 = 0.2;
const eps: f32 = 0.005;
const dx: vec2<f32> = vec2<f32>(0.0, eps);
const max_bounces: u32 = 4;

const disc_radius: f32 = 10.0;
const disc_height: f32 = 0.8;
const disc_falloff: vec2<f32> = vec2<f32>(0.1, 0.5); // radial, vertical
const disc_emission_falloff: vec2<f32> = vec2<f32>(0.06, 0.6); // radial, vertical
const disc_temperature_scale: f32 = 4000.0;
const disc_temperature_offset: f32 = 2000.0;
const disc_radial_scale: f32 = 8.0;

const dt_min: f32 = 0.03;
const dt_max: f32 = 1.0;

const steps: u32 = 256u;

var<private> dt: f32 = dt_min;

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

struct SampleVolumeOut {
    c: vec3<f32>,
    e: vec3<f32>,
    v: f32,
};

fn sample_volume(p: vec3<f32>, redshift: f32) -> SampleVolumeOut {
    var out: SampleVolumeOut;

    out.c = vec3(0.3, 0.2, 0.1);
    out.e = vec3(0.0);
    out.v = 0.0;

    // Reject if not hit disc
    if (dot(p.xy, p.xy) > disc_radius * disc_radius || p.z * p.z > disc_height * disc_height) {
        return out;
    };

    let n0 = fbm(disc_radial_scale * vec3(rotate2(p.xy, (8.0 * p.z) + (disc_radial_scale * length(p.xy))), p.z).xyz, 8u);

    let d_falloff = length(disc_falloff.xxy * p);
    let e_falloff = length(disc_emission_falloff.xxy * p);

    // Sample the color temperature of the accretion disc (with some random jitter) and normalize
    let t = rand();
    out.e = xyz2rgb(blackbody((disc_temperature_scale * t * t) + disc_temperature_offset));
    out.e = clamp(out.e / max(max(max(out.e.r, out.e.g), out.e.b), 0.01), vec3(0.0), vec3(1.0));

    // Account for density and emission falloff near edges of disc
    out.e *= 128.0 * max(n0 - e_falloff, 0.0) / (dot(0.5 * p, 0.5 * p) + 0.05);
    out.v = 128.0 * max(n0 - d_falloff, 0.0);

    return out;
}

// Kerr-Newman metric in Kerr-Schild coordinates for spinning charged black hole rotating around the Z-axis.
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

fn update_dt(p: vec4<f32>, x: vec4<f32>) -> bool {
    let pos = x.yzw;
    let rho = dot(pos, pos) - a * a;
    let r2 = 0.5 * (rho + sqrt(rho * rho + 4.0 * a * a * pos.z * pos.z));
    let r = sqrt(r2);

    dt = mix(dt_min, dt_max, pow(max(r - 1.0, 0.0) / cdist, 1.0));

    if (r < 1.0 && a <= 1.0 || length(p) > 45.0) {
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
    rng_state = view.frame_count * view.resolution.x * view.resolution.y + u32(frag_coord.y) * view.resolution.x + u32(frag_coord.x);
    let pos = (2.0 * (frag_coord + rand2() - 0.5) - vec2<f32>(view.resolution.xy)) / f32(view.resolution.y);

    let rd = normalize((view.camera * normalize(vec4(pos, view.focal_length, 1.0))).xyz);
    var ro = view.position;

    let t0 = sphere_intersect(ro, rd, vec4(0.0, 0.0, 0.0, cdist));
    if (t0 > 0.0 && t0 < 1e10) {
        ro += rd * t0;
    }

    var x = vec4(0.0, ro);
    var p = normalize(null_momentum(rd, x.yzw));

    let p0 = p.x;

    var discard_sample = false;
    var bounces = 0u;

    var r = vec3(0.0);
    var att = vec3(1.0);

    for (var i = 0u; i < steps; i++) {
        if (bounces > max_bounces) {
            discard_sample = true;
            break;
        }

        let d = sample_volume(x.yzw, p0 / p.x);
        r += att * d.e * dt;

        if (d.v > 0.0) {
            let absorb = exp(-1.0 * dt * d.v);

            if (absorb < rand()) {
                let v = length(p.yzw) * reflect(normalize(p.yzw), udir3());
                p = vec4(p.x, v.x, v.y, v.z);
                att *= d.c;
                bounces += 1u;
            }
        }

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

    if (length(x.yzw) > 3.0 && !discard_sample) {
        // r += att * textureSample(sky_texture, sky_sampler, out_dir).rgb;
        col = r;
    }

    var old_col = vec4(0.0);

    if (view.frame_count != 0u) {
        let load_coord = vec2(in.uv.x, 1.0 - in.uv.y) * vec2<f32>(view.resolution.xy);
        old_col = textureLoad(last_frame, vec2<u32>(load_coord), 0);
    }

    if (discard_sample) {
        return old_col;
    } else {
        return vec4(col, 1.0) + old_col;
    }
}
