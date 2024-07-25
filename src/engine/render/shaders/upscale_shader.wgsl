@group(0) @binding(0) var raymarched_view: texture_2d<f32>;
@group(0) @binding(1) var view_sampler: sampler;


struct VertexInput {
    @location(0) @interpolate(perspective) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec3<f32>
};


@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.position = model.position;
    return out;
}

@fragment
fn fs_main(inn: VertexOutput) -> @location(0) vec4<f32> {

    var uv: vec2<f32> = (inn.position.xy * vec2(0.5, -0.5)) + 0.5;

    let col = textureSample(raymarched_view, view_sampler, uv);

    return col;
}