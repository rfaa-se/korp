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
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

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

    out.clip_position = uniform.view_projection *  vec4(world_position, 0.0, 1.0);
    out.color = color;

    return out;
}

@fragment
fn fs_main(out: VertexOutput) -> @location(0) vec4<f32> {
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
