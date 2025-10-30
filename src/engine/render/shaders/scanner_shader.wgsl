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

// Fragment shader
struct CameraUniform {
    cam_pos: vec4<f32>,
    cam_zw_rot: mat4x4<f32>,
    cam_zy_rot: mat4x4<f32>,
    cam_zx_rot: mat4x4<f32>,
}

struct SphericalAreasMetadata {
    holegun_colorized_areas_start: u32,
    holegun_colorized_areas_amount: u32,
    explode_areas_start: u32,
    explode_areas_amount: u32,
}

struct ShapesMetadata {
    //normals
    cubes_start: u32,
    cubes_amount: u32,

    spheres_start: u32,
    spheres_amount: u32,

    inf_cubes_start: u32,
    inf_cubes_amount: u32,

    sph_cubes_start: u32,
    sph_cubes_amount: u32,

    //stickinesses
    s_cubes_start: u32,
    s_cubes_amount: u32,

    s_spheres_start: u32,
    s_spheres_amount: u32,

    s_inf_cubes_start: u32,
    s_inf_cubes_amount: u32,

    s_sph_cubes_start: u32,
    s_sph_cubes_amount: u32,

    //negatives
    neg_cubes_start: u32,
    neg_cubes_amount: u32,

    neg_spheres_start: u32,
    neg_spheres_amount: u32,

    neg_inf_cubes_start: u32,
    neg_inf_cubes_amount: u32,

    neg_sph_cubes_start: u32,
    neg_sph_cubes_amount: u32,

    //neg_stickinesses
    s_neg_cubes_start: u32,
    s_neg_cubes_amount: u32,

    s_neg_spheres_start: u32,
    s_neg_spheres_amount: u32,

    s_neg_inf_cubes_start: u32,
    s_neg_inf_cubes_amount: u32,

    s_neg_sph_cubes_start: u32,
    s_neg_sph_cubes_amount: u32,
}

struct RectTransformUniform {
    scale: vec2<f32>,
    translation: vec2<f32>,
    transparency: f32,
    empty_byte: f32,
    rotation_around_rect_center: f32,
    rotation_around_screen_center: f32,
}

struct PlayerProjection
{
    position: vec4<f32>,
    original_position: vec4<f32>,
    is_active_intensity: f32,
    radius: f32,
    zw_offset: f32,
    rel_zw_offset: f32,
    damage_intensity: f32,
    padding_byte1: f32,
    padding_byte2: f32,
    intensity: f32
}

struct OtherDynamicData {
    shapes_arrays_metadata: ShapesMetadata,
    spherical_areas_meatadata: SphericalAreasMetadata,
    camera_data: CameraUniform,
    waves_start: u32,
    waves_amount: u32,
    beam_areas_amount: u32,
    player_forms_amount: u32,

    player_projections: array<PlayerProjection, 16>,

    w_scanner_radius: f32,
    w_scanner_ring_intesity: f32,
    w_scanner_max_radius: f32,

    death_screen_effect: f32,

    // padding_byte_0: u32,
    splited_screen_in_2d_3d_example: f32,
    w_shift_coef: f32,
    w_shift_intensity: f32,

    getting_damage_screen_effect: f32,
    zx_player_rotation: f32,
    screen_aspect: f32,
    time: f32,
    shadows_enabled: i32,
    // padding_byte_1: i32,
    // padding_byte_2: i32,
    // padding_byte_3: i32,
    additional_data: vec4<f32>,
    additional_data_2: vec4<f32>,
}

struct PlayerForm {
    pos: vec4<f32>,
    empty_bytes: vec4<u32>,
    color: vec3<f32>,
    radius: f32,
    rotation: mat4x4<f32>,
    weapon_offset: vec4<f32>,
}

struct ScannerData {
    empty_byte0: u32,
    empty_byte1: u32,
    empty_byte2: u32,
    orientation: u32,

    // hits: array<vec4<f32>, 16>,
}

struct Shape {
    pos: vec4<f32>,
    size: vec4<f32>,
    material: i32,
    empty_bytes1: u32,
    empty_bytes2: u32,
    roundness: f32,
}

const MAX_DIST: f32 = 150.0;


@group(0) @binding(0) var<uniform> rect_transform: RectTransformUniform;
@group(0) @binding(1) var<uniform> dynamic_data: OtherDynamicData;
@group(0) @binding(2) var<uniform> dyn_player_forms: array<PlayerForm, 16>;
@group(0) @binding(3) var<uniform> scanner_data: ScannerData;

@group(0) @binding(4) var<uniform> dyn_normal_shapes: array<Shape, 256>;
@group(0) @binding(5) var<uniform> dyn_negatives_shapes: array<Shape, 256>;
@group(0) @binding(6) var<uniform> dyn_stickiness_shapes: array<Shape, 256>;
@group(0) @binding(7) var<uniform> dyn_neg_stickiness_shapes: array<Shape, 256>;



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

fn sd_sphere(p: vec4<f32>, radius: f32) -> f32 {
    return length(p) - radius;
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

const MAX_SCANNER_RADIUS: f32 = 21.0; 

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    let uv_pos = (in.uv-vec2(0.5))*2.0;

    let dist_to_cntr = length(uv_pos);

    var col = vec4(1.0, 1.0, 1.0, 0.0);

    let sc_ring_radius = dynamic_data.w_scanner_radius / MAX_SCANNER_RADIUS;

    let ring_a = pow(max(1.0-abs(dist_to_cntr-sc_ring_radius),0.0), 13.0)*dynamic_data.w_scanner_ring_intesity;

    col.a += ring_a;

    var en_a = 0.0;

    let rot_xz_mat_4d = rotate_in_xz_4d(-dynamic_data.zx_player_rotation);
    let rot_xz_mat_2d = rotate_in_2d(dynamic_data.zx_player_rotation);

    for (var i = 0u; i < 16u; i++) {
        
        if dynamic_data.player_projections[i].radius > 0.0
        {
            var en_pos  = (dynamic_data.camera_data.cam_pos - dynamic_data.player_projections[i].original_position) / MAX_SCANNER_RADIUS;

            if scanner_data.orientation == 0 {

                let en_pos_2d = rot_xz_mat_2d * (en_pos.xz*vec2(-1.0,-1.0)); 
                let e = clamp((0.09 - length(uv_pos-en_pos_2d)) * 100.0, 0.0, 1.0);

                let l = clamp((0.13 - sd_line(uv_pos, en_pos_2d, vec2(0.0))) * 12.0, 0.0, 1.0) *
                    clamp(dynamic_data.player_projections[i].damage_intensity*2.0,0.0,1.0);

                en_a += (e+l) * clamp(dynamic_data.player_projections[i].intensity*2.0,0.0,1.0);

            } else {
                en_pos *= vec4(-1.0, 1.0, -1.0, 1.0);
                
                en_pos *= rot_xz_mat_4d;

                let en_pos_2d = en_pos.zw;

                let e = clamp((0.09 - length(uv_pos-en_pos_2d)) * 100.0, 0.0, 1.0);

                let l = clamp((0.13 - sd_line(uv_pos, en_pos_2d, vec2(0.0))) * 12.0, 0.0, 1.0) *
                    clamp(dynamic_data.player_projections[i].damage_intensity*2.0,0.0,1.0);

                en_a += (e+l)* clamp(dynamic_data.player_projections[i].intensity*2.0,0.0,1.0);
            }
        }
    }

    col.a += en_a;
    col.g -= en_a;
    col.b -= en_a;
    
    col.a *= rect_transform.transparency;

    return col;
}


fn sd_line(p: vec2<f32>, a: vec2<f32> , b: vec2<f32>) -> f32
{
    let pa = p-a;
    let ba = b-a;
    let h = clamp( dot(pa,ba)/dot(ba,ba), 0.0, 1.0 );
    return length( pa - ba*h );
}


fn rotate_in_2d(a: f32) -> mat2x2<f32>
{
    let c_1: vec2<f32> = vec2<f32>(cos(a), sin(a));
    let c_2: vec2<f32> = vec2<f32>(-sin(a), cos(a));
    let matrix = mat2x2<f32>(c_1, c_2); 
    return matrix; 
}


fn rotate_in_xz_4d(a: f32) -> mat4x4<f32>
{
    let c_1: vec4<f32> = vec4<f32>(cos(a), sin(a), 0., 0.);
    let c_2: vec4<f32> = vec4<f32>(0., 1., 0., 0.);
    let c_3: vec4<f32> = vec4<f32>(-sin(a), 0., cos(a), 0.);
    let c_4: vec4<f32> = vec4<f32>(0., 0., 0., 1.);
    let matrix = mat4x4<f32>(c_1, c_2, c_3, c_4); 
    return matrix; 
}
