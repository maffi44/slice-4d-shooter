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

// size of bloom effect in screen space
const BLOOM_RADIUS: f32 = 0.014;

@fragment
fn fs_main(inn: VertexOutput) -> @location(0) vec4<f32> {

    var uv: vec2<f32> = (inn.position.xy * vec2(0.5, -0.5)) + 0.5;

    let pixel_step = vec2(1.0) / vec2<f32>(textureDimensions(raymarched_view));

    let col = textureSample(raymarched_view, view_sampler, uv);

    var bloom = col.rgb * col.a;

    var j = 0.0;
    for (var i = 0.0; i <= BLOOM_RADIUS; i += pixel_step.y) {
        let offset = vec2<f32>(i, 0.0);
        let color = textureSample(raymarched_view, view_sampler, uv + offset);
        bloom += (color.rgb * color.a) * (1.0 - i/BLOOM_RADIUS);
        j += 1.0;
    }

    for (var i = 0.0; i >= -BLOOM_RADIUS; i -= pixel_step.y) {
        let offset = vec2<f32>(i, 0.0);
        let color = textureSample(raymarched_view, view_sampler, uv + offset);
        bloom += (color.rgb * color.a) * (1.0 - -i/BLOOM_RADIUS);
        j += 1.0;
    }

    for (var i = 0.0; i <= BLOOM_RADIUS; i += pixel_step.x) {
        let offset = vec2<f32>(0.0, i);
        let color = textureSample(raymarched_view, view_sampler, uv + offset);
        bloom += (color.rgb * color.a) * (1.0 - i/BLOOM_RADIUS);
        j += 1.0;
    }

    for (var i = 0.0; i >= -BLOOM_RADIUS; i -= pixel_step.x) {
        let offset = vec2<f32>(0.0, i);
        let color = textureSample(raymarched_view, view_sampler, uv + offset);
        bloom += (color.rgb * color.a) * (1.0 - -i/BLOOM_RADIUS);
        j += 1.0;
    }

    bloom /= j+1.0;

    return vec4(col.rgb + bloom*vec3(1.4), 1.0);
}