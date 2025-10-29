struct Uniform {
    view_projection: mat4x4<f32>,
    alpha: f32,
}

struct VertexInput {
    @location(0) position_old: vec2<f32>,
    @location(1) position_new: vec2<f32>,
    @location(2) rotation_old: vec2<f32>,
    @location(3) rotation_new: vec2<f32>,
    @location(4) origin_old: vec2<f32>,
    @location(5) origin_new: vec2<f32>,
    @location(6) color_old: u32,
    @location(7) color_new: u32,
    @location(8) flags_data: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) flags: u32,
    @location(2) barycentric: vec3<f32>,
}

const mask_flags: u32 = 0xffff;
const shift_data: u32 = 16u;

const flag_wireframe: u32 = 1u << 0u;

const wireframe_width: f32 = 1.0;
const wireframe_top: u32 = 1u;
const wireframe_left: u32 = 2u;
const wireframe_right: u32 = 3u;

@group(0) @binding(0)
var<uniform> uniform: Uniform;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let position = mix(in.position_old, in.position_new, uniform.alpha);
    let rotation = normalize(mix(in.rotation_old, in.rotation_new, uniform.alpha));
    let origin = mix(in.origin_old, in.origin_new, uniform.alpha);
    let color = mix(unpack(in.color_old), unpack(in.color_new), uniform.alpha);

    let offset = position - origin;
    let rotated = rotate(offset, vec2(-rotation.y, rotation.x));
    let world_position = rotated + origin;

    let flags = in.flags_data & mask_flags;
    let data = in.flags_data >> shift_data;

    out.clip_position = uniform.view_projection *  vec4(world_position, 0.0, 1.0);
    out.color = color;
    out.flags = flags;

    if has_flag(flags, flag_wireframe) {
        out.barycentric = barycentric(data);
    }
  
    return out;
}

@fragment
fn fs_main(out: VertexOutput) -> @location(0) vec4<f32> {
    if has_flag(out.flags, flag_wireframe) {
        // let edge_threshold = 0.01;
        // let is_edge =
        //        out.barycentric.x < edge_threshold
        //     || out.barycentric.y < edge_threshold
        //     || out.barycentric.z < edge_threshold;
        
        // return select(vec4(0.0, 0.0, 0.0, 0.0), out.color, is_edge);

        // let delta = fwidth(out.barycentric);
        // // let f = step(delta * wireframe_width, out.barycentric);
        // let width = wireframe_width * min(min(f.x, f.y), f.z);

        // return select(vec4(0.0, 0.0, 0.0, 0.0), out.color, c);
        
        // let delta = max(fwidth(out.barycentric), vec3(1e-4));
        // let min_bary = min(out.barycentric.x, min(out.barycentric.y, out.barycentric.z));
        // let width = min(delta.x, min(delta.y, delta.z));
        // let is_edge = min_bary < width;

        // return select(vec4(0.0, 0.0, 0.0, 0.0), out.color, is_edge);

        let edge_threshold = 0.0199;
        let edge_distance = min(out.barycentric.x, min(out.barycentric.y, out.barycentric.z));
        let saturated = max(edge_distance, edge_threshold);
        // let edge_factor = smoothstep(0.0, saturated, edge_distance);
        let edge_factor = step(saturated, edge_distance);
        // return select(vec4(0.0), out.color, edge_distance < edge_threshold);
        return mix(vec4(0.0), out.color, 1.0 - edge_factor);
    }
    
    return out.color;
}

fn rotate(position: vec2<f32>, rotation: vec2<f32>) -> vec2<f32> {
    return vec2(
        position.x * rotation.x - position.y * rotation.y,
        position.x * rotation.y + position.y * rotation.x
    );
}

fn unpack(c: u32) -> vec4<f32> {
    let r = f32((c >> 24) & 0xff);
    let g = f32((c >> 16) & 0xff);
    let b = f32((c >> 8) & 0xff);
    let a = f32((c >> 0) & 0xff);

    return vec4(r, g, b, a);
}

fn has_flag(flags: u32, flag: u32) -> bool {
    return (flags & flag) != 0u;
}

fn barycentric(data: u32) -> vec3<f32> {
    return vec3(
        select(0.0, 1.0, data == 1u),
        select(0.0, 1.0, data == 2u),
        select(0.0, 1.0, data == 3u),
    );
}
