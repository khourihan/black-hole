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

// Optimized inverse of symmetric matrix
fn inverse(m: mat4x4<f32>) -> mat4x4<f32> {
    let n11 = m[0][0];
    let n12 = m[1][0];
    let n13 = m[2][0];
    let n14 = m[3][0];
	let n22 = m[1][1];
    let n23 = m[2][1];
    let n24 = m[3][1];
	let n33 = m[2][2];
    let n34 = m[3][2];
	let n44 = m[3][3];

	let t11 = 2.0 * n23 * n34 * n24 - n24 * n33 * n24 - n22 * n34 * n34 - n23 * n23 * n44 + n22 * n33 * n44;
	let t12 = n14 * n33 * n24 - n13 * n34 * n24 - n14 * n23 * n34 + n12 * n34 * n34 + n13 * n23 * n44 - n12 * n33 * n44;
	let t13 = n13 * n24 * n24 - n14 * n23 * n24 + n14 * n22 * n34 - n12 * n24 * n34 - n13 * n22 * n44 + n12 * n23 * n44;
	let t14 = n14 * n23 * n23 - n13 * n24 * n23 - n14 * n22 * n33 + n12 * n24 * n33 + n13 * n22 * n34 - n12 * n23 * n34;

	let det = n11 * t11 + n12 * t12 + n13 * t13 + n14 * t14;
	let idet = 1.0f / det;

	var ret: mat4x4<f32>;

	ret[0][0] = t11 * idet;
	ret[0][1] = (n24 * n33 * n14 - n23 * n34 * n14 - n24 * n13 * n34 + n12 * n34 * n34 + n23 * n13 * n44 - n12 * n33 * n44) * idet;
	ret[0][2] = (n22 * n34 * n14 - n24 * n23 * n14 + n24 * n13 * n24 - n12 * n34 * n24 - n22 * n13 * n44 + n12 * n23 * n44) * idet;
	ret[0][3] = (n23 * n23 * n14 - n22 * n33 * n14 - n23 * n13 * n24 + n12 * n33 * n24 + n22 * n13 * n34 - n12 * n23 * n34) * idet;

	ret[1][0] = ret[0][1];
	ret[1][1] = (2.0 * n13 * n34 * n14 - n14 * n33 * n14 - n11 * n34 * n34 - n13 * n13 * n44 + n11 * n33 * n44) * idet;
	ret[1][2] = (n14 * n23 * n14 - n12 * n34 * n14 - n14 * n13 * n24 + n11 * n34 * n24 + n12 * n13 * n44 - n11 * n23 * n44) * idet;
	ret[1][3] = (n12 * n33 * n14 - n13 * n23 * n14 + n13 * n13 * n24 - n11 * n33 * n24 - n12 * n13 * n34 + n11 * n23 * n34) * idet;

	ret[2][0] = ret[0][2];
	ret[2][1] = ret[1][2];
    ret[2][2] = (2.0 * n12 * n24 * n14 - n14 * n22 * n14 - n11 * n24 * n24 - n12 * n12 * n44 + n11 * n22 * n44) * idet;
	ret[2][3] = (n13 * n22 * n14 - n12 * n23 * n14 - n13 * n12 * n24 + n11 * n23 * n24 + n12 * n12 * n34 - n11 * n22 * n34) * idet;

	ret[3][0] = ret[0][3];
	ret[3][1] = ret[1][3];
	ret[3][2] = ret[2][3];
	ret[3][3] = (2.0 * n12 * n23 * n13 - n13 * n22 * n13 - n11 * n23 * n23 - n12 * n12 * n33 + n11 * n22 * n33) * idet;

	return ret;
}

fn diag(a: vec4<f32>) -> mat4x4<f32> {
    return mat4x4(a.x, 0.0, 0.0, 0.0,
                0.0, a.y, 0.0, 0.0,
                0.0, 0.0, a.z, 0.0,
                0.0, 0.0, 0.0, a.w);
}

const a: f32 = 0.99;
const m: f32 = 1.0;
const Q: f32 = 0.0;
const eps: f32 = 0.01;
const timestep: f32 = 0.05;
const steps: u32 = 4096u;

fn r_from_coords(x: vec4<f32>) -> f32 {
    let p = x.yzw;
    let rho = dot(p, p) - a * a;
    let r2 = 0.5 * (rho + sqrt(rho * rho + 4.0 * a * a * p.z * p.z));
    return sqrt(r2);
}

// Kerr-Newman Metric
fn metric(x: vec4<f32>) -> mat4x4<f32> {
    let p = x.yzw;
    let r = r_from_coords(x);
    let r2 = r * r;
    let k = vec4(1.0, (r * p.x + a * p.y) / (r2 + a * a), (r * p.y - a * p.x) / (r2 + a * a), p.z / r);
    let f = r2 * (2.0 * m * r - Q * Q) / (r2 * r2 + a * a * p.z * p.z);
    return f * mat4x4(k.x * k, k.y * k, k.z * k, k.w * k) + diag(vec4(-1.0, 1.0, 1.0, 1.0));
}

fn hamiltonian(x: vec4<f32>, p: vec4<f32>) -> f32 {
    let g_inv = inverse(metric(x));
    return 0.5 * dot(g_inv * p, p);
}

fn hamiltonian_gradient(x: vec4<f32>, p: vec4<f32>) -> vec4<f32> {
    return (vec4(hamiltonian(x + vec4(eps, 0.0, 0.0, 0.0), p),
                 hamiltonian(x + vec4(0.0, eps, 0.0, 0.0), p),
                 hamiltonian(x + vec4(0.0, 0.0, eps, 0.0), p),
                 hamiltonian(x + vec4(0.0, 0.0, 0.0, eps), p)) - hamiltonian(x, p)) / eps;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let frag_coord = in.uv * vec2<f32>(view.resolution.xy);
    // TODO: jitter frag coord
    let pos = (2.0 * frag_coord - vec2<f32>(view.resolution.xy)) / f32(view.resolution.y);

    let rd = normalize((view.camera * normalize(vec4(pos, view.focal_length, 1.0))).xyz);
    let ro = view.position;

    let time = 0.0;
    var x = vec4(time, ro);
    var p = metric(x) * vec4(1.0, rd);
    var captured = false;

    for (var i = 0u; i < steps; i++) {
        p -= timestep * hamiltonian_gradient(x, p);
        x += timestep * inverse(metric(x)) * p;

        let r = r_from_coords(x);
        captured = r < 1.0 + sqrt(1.0 - a * a);
        if (captured) {
            break;
        }
    }

    let dxdt = inverse(metric(x)) * p;
    let out_dir = normalize(dxdt.yzw);
    let out_pos = x.yzw;
    let out_time = x.x;

    let col = textureSample(sky_texture, sky_sampler, out_dir) * f32(!captured);

    return vec4(col.xyz, 1.0);
}
