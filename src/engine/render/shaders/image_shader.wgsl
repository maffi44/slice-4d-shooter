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

struct RectTransformUniform {
    scale: vec2<f32>,
    translation: vec2<f32>,
    transparency: f32,
    empty_byte: f32,
    rotation_around_rect_center: f32,
    rotation_around_screen_center: f32,
}

@group(0) @binding(0) var<uniform> rect_transform: RectTransformUniform;
@group(0) @binding(1) var texture: texture_2d<f32>;
@group(0) @binding(2) var tex_sampler: sampler;

struct VertexInput {
    @location(0) @interpolate(perspective) position: vec3<f32>,
    @location(1) @interpolate(perspective) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>
};

fn rotation_mat(angle: f32) -> mat2x2<f32> {
    var c: f32 = cos(angle);
    var s: f32 = sin(angle);

    return mat2x2<f32>(c, -s, s, c);
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {

    var coords = model.position;

    let c = vec2(coords.x, coords.y) * rotation_mat(rect_transform.rotation_around_screen_center);

    coords.x = c.x;
    coords.y = c.y;
    
    coords *= vec3(rect_transform.scale, 0.0);

    coords += vec3(rect_transform.translation, 0.0);


    var out: VertexOutput;
    out.clip_position = vec4<f32>(coords, 1.0);
    out.uv = model.uv;
    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    var col = textureSample(texture, tex_sampler, in.uv);
    col.a *= rect_transform.transparency;
    return col;
}