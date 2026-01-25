struct Uniform {
    view_projection: mat4x4<f32>,
}

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) rotation: vec2<f32>,
    @location(2) origin: vec2<f32>,
    @location(3) color: u32,
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

    let offset = in.position - in.origin;
    let rotated = rotate(offset, in.rotation);
    let world_position = rotated + in.origin;

    out.clip_position = uniform.view_projection *  vec4(world_position, 0.0, 1.0);
    out.color = unpack(in.color);

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
