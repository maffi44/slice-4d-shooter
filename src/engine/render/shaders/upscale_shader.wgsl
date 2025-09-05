// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2025  Timofei Molokov

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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