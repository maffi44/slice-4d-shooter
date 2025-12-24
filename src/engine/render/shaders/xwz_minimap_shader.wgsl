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

struct CameraUniform {
    cam_pos: vec4<f32>,
    cam_zw_rot: mat4x4<f32>,
    cam_zy_rot: mat4x4<f32>,
    cam_zx_rot: mat4x4<f32>,
}

struct Shape {
    pos: vec4<f32>,
    size: vec4<f32>,
    material: i32,
    empty_bytes1: u32,
    empty_bytes2: u32,
    roundness: f32,
}

struct PlayerForm {
    pos: vec4<f32>,
    is_red: vec4<u32>,
    color: vec3<f32>,
    radius: f32,
    rotation: mat4x4<f32>,
    weapon_offset: vec4<f32>,
}

struct ShapesMetadata {
    //normals
    cubes_start: u32,
    cubes_amount: u32,

    spheres_start: u32,
    spheres_amount: u32,

    // inf_cubes_start: u32,
    // inf_cubes_amount: u32,

    sph_cubes_start: u32,
    sph_cubes_amount: u32,

    //stickinesses
    s_cubes_start: u32,
    s_cubes_amount: u32,

    s_spheres_start: u32,
    s_spheres_amount: u32,

    // s_inf_cubes_start: u32,
    // s_inf_cubes_amount: u32,

    s_sph_cubes_start: u32,
    s_sph_cubes_amount: u32,

    //negatives
    neg_cubes_start: u32,
    neg_cubes_amount: u32,

    neg_spheres_start: u32,
    neg_spheres_amount: u32,

    // neg_inf_cubes_start: u32,
    // neg_inf_cubes_amount: u32,

    neg_sph_cubes_start: u32,
    neg_sph_cubes_amount: u32,

    //neg_stickinesses
    s_neg_cubes_start: u32,
    s_neg_cubes_amount: u32,

    s_neg_spheres_start: u32,
    s_neg_spheres_amount: u32,

    // s_neg_inf_cubes_start: u32,
    // s_neg_inf_cubes_amount: u32,

    s_neg_sph_cubes_start: u32,
    s_neg_sph_cubes_amount: u32,

    //unbreakable normals
    unbreakable_cubes_start: u32,
    unbreakable_cubes_amount: u32,

    unbreakable_spheres_start: u32,
    unbreakable_spheres_amount: u32,

    // unbreakable_inf_cubes_start: u32,
    // unbreakable_inf_cubes_amount: u32,

    unbreakable_sph_cubes_start: u32,
    unbreakable_sph_cubes_amount: u32,

    //unbreakable stickinesses
    unbreakable_s_cubes_start: u32,
    unbreakable_s_cubes_amount: u32,

    unbreakable_s_spheres_start: u32,
    unbreakable_s_spheres_amount: u32,

    // unbreakable_s_inf_cubes_start: u32,
    // unbreakable_s_inf_cubes_amount: u32,

    unbreakable_s_sph_cubes_start: u32,
    unbreakable_s_sph_cubes_amount: u32,
}


struct SphericalAreasMetadata {
    holegun_colorized_areas_start: u32,
    holegun_colorized_areas_amount: u32,
    explode_areas_start: u32,
    explode_areas_amount: u32,
}

struct SphericalArea {
    pos: vec4<f32>,
    color: vec3<f32>,
    radius: f32,
}

struct BeamArea {
    pos1: vec4<f32>,
    pos2: vec4<f32>,
    color: vec3<f32>,
    radius: f32,
}

struct OutputMaterials {
    materials_count: u32,
    empty_bytes: vec3<f32>,
    materials: array<i32, 16>,
    material_weights: array<f32, 16>,
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

struct Material {
    color: vec4<f32>,
}

struct OtherStaticData {
    is_w_floor_exist: i32,
    w_floor: f32,
    blue_players_mat1: i32,
    blue_players_mat2: i32,

    red_players_mat1: i32,
    red_players_mat2: i32,


    w_cups_mat: i32,
    stickiness: f32,

    red_base_position: vec4<f32>,
    blue_base_position: vec4<f32>,
    materials: array<Material, 32>,

    red_base_color: vec3<f32>,
    blue_base_color: vec3<f32>,

    sky_color: vec3<f32>,
    sun_color: vec3<f32>,
    fog_color: vec3<f32>,
    frenel_color: vec3<f32>,
    neon_wireframe_color: vec3<f32>,
    sun_direction: vec4<f32>,
}

@group(0) @binding(0) var<uniform> static_data: OtherStaticData;

@group(0) @binding(1) var<uniform> dyn_normal_shapes: array<Shape, 256>;
@group(0) @binding(2) var<uniform> dyn_negatives_shapes: array<Shape, 256>;
@group(0) @binding(3) var<uniform> dyn_stickiness_shapes: array<Shape, 256>;
@group(0) @binding(4) var<uniform> dyn_neg_stickiness_shapes: array<Shape, 256>;
@group(0) @binding(5) var<uniform> dyn_undestroyable_normal_shapes: array<Shape, 256>;
@group(0) @binding(6) var<uniform> dyn_undestroyable_stickiness_shapes: array<Shape, 256>;



@group(0) @binding(7) var<uniform> dynamic_data: OtherDynamicData;

@group(1) @binding(0) var<uniform> dyn_spherical_areas: array<SphericalArea, 256>;
@group(1) @binding(1) var<uniform> dyn_beam_areas: array<BeamArea, 256>;
@group(1) @binding(2) var<uniform> dyn_player_forms: array<PlayerForm, 16>;

@group(1) @binding(3) var sky_box_sampler: sampler;
@group(1) @binding(4) var sky_box: texture_cube<f32>;

const MAX_STEPS: i32 = 120;
const PI: f32 = 3.1415926535897;
const MIN_DIST: f32 = 0.012;
const MAX_DIST: f32 = 150.0;

const STICKINESS_EFFECT_COEF: f32 = 3.1415926535897;

struct VertexInput {
    @location(0) @interpolate(perspective) position: vec3<f32>,
    @location(1) @interpolate(perspective) rel_position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) rel_position: vec3<f32>
};

fn cube_intersection( ro: vec4<f32>, rd: vec4<f32>, size: vec4<f32>) -> vec2<f32> {

    let m = 1.0/rd;
    let n = m*ro;
    let k = abs(m)*size;
    let t1 = -n - k;
    let t2 = -n + k;
    let tN = max( max( max( t1.x, t1.y ), t1.z ), t1.w);
    let tF = min( min( min( t2.x, t2.y ), t2.z ), t2.w);
    if( tN>tF || tF<0.0) {
        return vec2(-1.0); // no intersection
    }
    return vec2( tN, tF );
}

fn inf_cube_intersection( ro: vec4<f32>, rd: vec4<f32>, size: vec3<f32>) -> vec2<f32> {
    let m = 1.0/rd;
    let n = m*ro;
    let k = abs(m.xyz)*size;
    let t1 = -n.xyz - k.xyz;
    let t2 = -n.xyz + k.xyz;
    let tN = max( max( t1.x, t1.y ), t1.z );
    let tF = min( min( t2.x, t2.y ), t2.z );
    if( tN>tF || tF<0.0) {
        return vec2(-1.0); // no intersection
    }
    return vec2( tN, tF );
}


fn tri_intersect_3d( ro: vec3<f32>, rd: vec3<f32>, v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32> ) -> f32
{
    let v1v0 = v1 - v0;
    let v2v0 = v2 - v0;
    let rov0 = ro - v0;
    let n = cross( v1v0, v2v0 );
    let q = cross( rov0, rd );
    let d = 1.0/dot( rd, n );
    let u = d*dot( -q, v2v0 );
    let v = d*dot(  q, v1v0 );
    var t = d*dot( -n, rov0 );
    if ( u<0.0 || v<0.0 || (u+v)>1.0 )
    {
        t = MAX_DIST*2.0;
    }
    return t;
}

fn sph_intersection( ro: vec4<f32>, rd: vec4<f32>, ra: f32) -> vec2<f32> {
    let b = dot( ro, rd );
    let c = dot( ro, ro ) - ra*ra;
    var h = b*b - c;
    if( h<0.0 ) {
        return vec2(-1.0); // no intersection
    }
    h = sqrt( h );
    return vec2( -b-h, -b+h );
}


fn sd_sphere(p: vec4<f32>, radius: f32) -> f32 {
    return length(p) - radius;
}


fn sd_inf_sphere(p: vec4<f32>, radius: f32) -> f32 {
    return length(p.xyz) - radius;
}


fn sd_inf_box(p: vec4<f32>, b: vec3<f32>) -> f32 {
    var d: vec3<f32> = abs(p.xyz) - b;
    return min(max(d.x, max(d.y, d.z)),0.0) + length(max(d,vec3<f32>(0.0)));
}


fn sd_box(p: vec4<f32>, b: vec4<f32>) -> f32 {
    var d: vec4<f32> = abs(p) - b;
    return min(max(d.x,max(d.y,max(d.z, d.w))),0.0) + length(max(d,vec4<f32>(0.0)));
}


fn sd_sph_inf_box(p: vec4<f32>, b: vec4<f32>) -> f32 {
    var d1: f32 = length(p.wx) - b.x;
    var d2: f32 = length(p.wy) - b.y;
    var d: vec2<f32> = abs(p.xy) - b.xy;
    return max(d1,max(d2,min(max(d.x,d.y),0.0) + length(max(d,vec2<f32>(0.0)))));
}


fn sd_sph_box(p: vec4<f32>, b: vec4<f32>) -> f32 {
    var d1: f32 = length(p.xy) - b.z;
    var d2: f32 = length(p.xz) - b.y;
    var d3: f32 = length(p.yz) - b.x;
    var d4: f32 = length(p.wx) - b.w;
    var d5: f32 = length(p.wy) - b.w;
    var d6: f32 = length(p.wz) - b.w;
    return max(d6,max(d5,max(d4,max(d1,max(d2, d3)))));
}


fn sd_box_sph(p: vec4<f32>, b: vec4<f32>) -> f32 {
    var ds: f32 = length(p) - b.w;
    var d: vec4<f32> = abs(p) - b;
    return max(ds, (min(max(d.x,max(d.y,max(d.z, d.w))),0.0) + length(max(d,vec4<f32>(0.0)))));
}


fn sd_solid_angle(p: vec4<f32>, c: vec2<f32>, ra: f32) -> f32 {
    var q: vec2<f32> = vec2<f32>( length(p.xz), p.y );
    var l: f32 = length(q) - ra;
    var m: f32 = length(q - c*clamp(dot(q,c),0.0,ra) );
    return max(l,m*sign(c.y*q.x-c.x*q.y));
}


fn sd_octahedron(point: vec4<f32>, s: f32) -> f32 {
    var p = abs(point);
    return (p.x+p.y+p.z+p.w-s)*0.57725627;
}


fn sd_capsule(p: vec4<f32>, a: vec4<f32>, b: vec4<f32>, r: f32) -> f32
{
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa,ba)/dot(ba,ba), 0.0, 1.0);
    return length(pa - ba*h) - r;
}


fn smin( a: f32, b: f32, k: f32 ) -> f32
{
    let kk = k * 1.0/(1.0-sqrt(0.5));
    let h = max( kk-abs(a-b), 0.0 )/kk;
    return min(a,b) - kk*0.5*(1.0+h-sqrt(1.0-h*(h - 2.0)));
}


fn smax( a: f32, b: f32, k: f32 ) -> f32
{
    let kk = k * 1.0/(1.0-sqrt(0.5));
    let h = max( kk-abs(a-b), 0.0 )/kk;
    return max(a,b) + kk*0.5*(1.0+h-sqrt(1.0-h*(h - 2.0)));
}

var<private> intr_normal: array<vec2<f32>, 32>;
var<private> intr_normal_size: u32 = 0u;
var<private> intr_neg: array<vec2<f32>, 32>;
var<private> intr_neg_size: u32 = 0u;
var<private> intr_unbreakables: array<vec2<f32>, 32>;
var<private> intr_unbreakables_size: u32 = 0u;
var<private> intr_players: bool = false;

fn store_intersection_entrance_and_exit_for_neg(intr: vec2<f32>)
{
    store_value_in_array_in_order_of_first_elem_for_neg(intr);
}


fn store_intersection_entrance_and_exit(intr: vec2<f32>)
{
    store_value_in_array_in_order_of_first_elem_for_normal(intr);
}


fn store_intersection_entrance_and_exit_for_unbreakables(intr: vec2<f32>)
{
    store_value_in_array_in_order_of_first_elem_for_normal(intr);
    store_value_in_array_in_order_of_first_elem_for_unbreakables(intr);
}


fn combine_interscted_entrances_and_exites_for_all_intrs()
{
    combine_interscted_entrances_and_exites_for_unbreakables();
    combine_interscted_entrances_and_exites_for_normal();
    combine_interscted_entrances_and_exites_for_neg();
}


fn combine_interscted_entrances_and_exites_for_normal() {
    var i = intr_normal_size;

    if i > 1u
    {
        while i > 1u
        {
            i -= 1u;

            if intr_normal[i].x < intr_normal[i-1].y
            {
                if intr_normal[i-1].y < intr_normal[i].y
                {
                    intr_normal[i-1].y = intr_normal[i].y;
                }

                let size = intr_normal_size - 1u;

                while i < size
                {
                    intr_normal[i] = intr_normal[i+1u];
                    i += 1u;
                }

                intr_normal_size -= 1u;
            }
        }
    }
}


fn combine_interscted_entrances_and_exites_for_neg() {
    var i = intr_neg_size;

    if i > 1u
    {
        while i > 1u
        {
            i -= 1u;

            if intr_neg[i].x < intr_neg[i-1].y
            {
                if intr_neg[i-1].y < intr_neg[i].y
                {
                    intr_neg[i-1].y = intr_neg[i].y;
                }

                let size = intr_neg_size - 1u;

                while i < size
                {
                    intr_neg[i] = intr_neg[i+1u];
                    i += 1u;
                }

                intr_neg_size -= 1u;
            }
        }
    }
}


fn combine_interscted_entrances_and_exites_for_unbreakables() {
    var i = intr_unbreakables_size;

    if i > 1u
    {
        while i > 1u
        {
            i -= 1u;

            if intr_unbreakables[i].x < intr_unbreakables[i-1].y
            {
                if intr_unbreakables[i-1].y < intr_unbreakables[i].y
                {
                    intr_unbreakables[i-1].y = intr_unbreakables[i].y;
                }

                let size = intr_unbreakables_size - 1u;

                while i < size
                {
                    intr_unbreakables[i] = intr_unbreakables[i+1u];
                    i += 1u;
                }

                intr_unbreakables_size -= 1u;
            }
        }
    }
}


fn store_value_in_array_in_order_of_first_elem_for_normal(
    val: vec2<f32>
) {
    var i = intr_normal_size;

    if i > 0
    {
        while intr_normal[i-1].x > val.x
        {
            i -= 1;

            if i == 0 {break;}
        }

        var j = intr_normal_size;
    
        while j > i
        {
            intr_normal[j] = intr_normal[j-1];
            j -= 1;
        }
    }

    intr_normal[i] = val;

    intr_normal_size += 1u;
}


fn store_value_in_array_in_order_of_first_elem_for_neg(
    val: vec2<f32>
) {
    var i = intr_neg_size;

    if i > 0
    {
        while intr_neg[i-1].x > val.x
        {
            i -= 1;

            if i == 0 {break;}
        }

        var j = intr_neg_size;
    
        while j > i
        {
            intr_neg[j] = intr_neg[j-1];
            j -= 1;
        }
    }

    intr_neg[i] = val;

    intr_neg_size += 1u;
}


fn store_value_in_array_in_order_of_first_elem_for_unbreakables(
    val: vec2<f32>
) {
    var i = intr_unbreakables_size;

    if i > 0
    {
        while intr_unbreakables[i-1].x > val.x
        {
            i -= 1;

            if i == 0 {break;}
        }

        var j = intr_unbreakables_size;
    
        while j > i
        {
            intr_unbreakables[j] = intr_unbreakables[j-1];
            j -= 1;
        }
    }

    intr_unbreakables[i] = val;

    intr_unbreakables_size += 1u;
}


fn find_intersections(ro: vec4<f32>, rdd: vec4<f32>) {
    //###find_intersections###
    var rd = rdd;

    if rd.x == 0 {
        rd.x += 0.000001; 
    }
    if rd.y == 0 {
        rd.y += 0.000001; 
    }
    if rd.z == 0 {
        rd.z += 0.000001; 
    }
    if rd.w == 0 {
        rd.w += 0.000001; 
    }

    for (var i = 0u; i < dynamic_data.shapes_arrays_metadata.s_sph_cubes_amount + dynamic_data.shapes_arrays_metadata.s_sph_cubes_start; i++) {
        if (i < dynamic_data.shapes_arrays_metadata.s_spheres_start) {
            let intr = cube_intersection(
                ro - dyn_stickiness_shapes[i].pos,
                rd,
                dyn_stickiness_shapes[i].size + dyn_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
            );
            
            if intr.y > 0.0 {

                store_intersection_entrance_and_exit(intr);
            }
        } else if (i < dynamic_data.shapes_arrays_metadata.s_sph_cubes_start) {
            let intr = sph_intersection(
                ro - dyn_stickiness_shapes[i].pos,
                rd,
                dyn_stickiness_shapes[i].size.x + dyn_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
            );
            
            if intr.y > 0.0 {
                store_intersection_entrance_and_exit(intr);
            }
        } else {
            let s = dyn_stickiness_shapes[i].size;

            let size = vec4(
                min(min(s.y, s.z),s.w),    
                min(min(s.x, s.z),s.w),    
                min(min(s.y, s.x),s.w),
                s.w
            );
            
            let intr = cube_intersection(
                ro - dyn_stickiness_shapes[i].pos,
                rd,
                size + dyn_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
            );
            
            if intr.y > 0.0 {
                store_intersection_entrance_and_exit(intr);
            }
        }
    }

    for (var i = 0u; i < dynamic_data.shapes_arrays_metadata.sph_cubes_amount + dynamic_data.shapes_arrays_metadata.sph_cubes_start; i++) {
        if (i < dynamic_data.shapes_arrays_metadata.spheres_start) {
            let intr = cube_intersection(
                ro - dyn_normal_shapes[i].pos,
                rd,
                dyn_normal_shapes[i].size + dyn_normal_shapes[i].roundness
            );
            
            if intr.y > 0.0 {
                store_intersection_entrance_and_exit(intr);
            }
        } else if (i < dynamic_data.shapes_arrays_metadata.sph_cubes_start) {
            let intr = sph_intersection(
                ro - dyn_normal_shapes[i].pos,
                rd,
                dyn_normal_shapes[i].size.x + dyn_normal_shapes[i].roundness
            );
            
            if intr.y > 0.0 {
                store_intersection_entrance_and_exit(intr);
            }
        } else {
            let s = dyn_normal_shapes[i].size;

            let size = vec4(
                min(min(s.y, s.z),s.w),    
                min(min(s.x, s.z),s.w),    
                min(min(s.y, s.x),s.w),
                s.w
            );
            
            let intr = cube_intersection(
                ro - dyn_normal_shapes[i].pos,
                rd,
                size + dyn_normal_shapes[i].roundness
            );
            
            if intr.y > 0.0 {
                store_intersection_entrance_and_exit(intr);
            }
        }
    }

    


    for (var i = 0u; i < dynamic_data.shapes_arrays_metadata.neg_sph_cubes_amount + dynamic_data.shapes_arrays_metadata.neg_sph_cubes_start; i++) {
        if (i < dynamic_data.shapes_arrays_metadata.neg_spheres_start) {
            
            let r = dyn_negatives_shapes[i].roundness;

            let intr = cube_intersection(
                ro - dyn_negatives_shapes[i].pos,
                rd,
                dyn_negatives_shapes[i].size + r*0.707106781*0.80,
            );

            if intr.y > 0.0 {
                store_intersection_entrance_and_exit_for_neg(intr);
            }
        } else if (i < dynamic_data.shapes_arrays_metadata.neg_sph_cubes_start) {
            let intr = sph_intersection(
                ro - dyn_negatives_shapes[i].pos,
                rd,
                dyn_negatives_shapes[i].size.x + dyn_negatives_shapes[i].roundness
            );
            
            if intr.y > 0.0 {
                store_intersection_entrance_and_exit_for_neg(intr);
            }
        }
        // else {
            // let s = dyn_negatives_shapes[i].size;

            // let size = vec4(
            //     min(min(s.y, s.z),s.w),    
            //     min(min(s.x, s.z),s.w),    
            //     min(min(s.y, s.x),s.w),
            //     s.w
            // );
            
            // let intr = cube_intersection(
            //     ro - dyn_negatives_shapes[i].pos,
            //     rd,
            //     size + dyn_negatives_shapes[i].roundness
            // );
            
            // if intr.y > 0.0 {
            //     store_intersection_entrance_and_exit_for_neg(intr);
            // }
        // }
    }

    for (var i = 0u; i < dynamic_data.shapes_arrays_metadata.s_neg_sph_cubes_amount + dynamic_data.shapes_arrays_metadata.s_neg_sph_cubes_start; i++) {
        if (i < dynamic_data.shapes_arrays_metadata.s_neg_spheres_start) {

            let r = dyn_neg_stickiness_shapes[i].roundness;

            let intr = cube_intersection(
                ro - dyn_neg_stickiness_shapes[i].pos,
                rd,
                dyn_neg_stickiness_shapes[i].size + r*0.707106781*0.80,
            );

            
            if intr.y > 0.0 {
                store_intersection_entrance_and_exit_for_neg(intr);
            }
        } else if (i < dynamic_data.shapes_arrays_metadata.s_neg_sph_cubes_start) {
            let intr = sph_intersection(
                ro - dyn_neg_stickiness_shapes[i].pos,
                rd,
                dyn_neg_stickiness_shapes[i].size.x + dyn_neg_stickiness_shapes[i].roundness
            );
            
            if intr.y > 0.0 {
                store_intersection_entrance_and_exit_for_neg(intr);
            }
        }
        // else {
            // let s = dyn_neg_stickiness_shapes[i].size;

            // let size = vec4(
            //     min(min(s.y, s.z),s.w),    
            //     min(min(s.x, s.z),s.w),    
            //     min(min(s.y, s.x),s.w),
            //     s.w
            // );
            
            // let intr = cube_intersection(
            //     ro - dyn_neg_stickiness_shapes[i].pos,
            //     rd,
            //     size + dyn_neg_stickiness_shapes[i].roundness
            // );
            
            // if intr.y > 0.0 {
            //     store_intersection_entrance_and_exit_for_neg(intr);
            // }
        // }
    }

    for (var i = 0u; i < dynamic_data.shapes_arrays_metadata.unbreakable_s_sph_cubes_amount + dynamic_data.shapes_arrays_metadata.unbreakable_s_sph_cubes_start; i++) {
        if (i < dynamic_data.shapes_arrays_metadata.unbreakable_s_spheres_start) {
            let intr = cube_intersection(
                ro - dyn_undestroyable_stickiness_shapes[i].pos,
                rd,
                dyn_undestroyable_stickiness_shapes[i].size + dyn_undestroyable_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
            );
            
            if intr.y > 0.0 {

                store_intersection_entrance_and_exit_for_unbreakables(intr);
            }
        } else if (i < dynamic_data.shapes_arrays_metadata.unbreakable_s_sph_cubes_start) {
            let intr = sph_intersection(
                ro - dyn_undestroyable_stickiness_shapes[i].pos,
                rd,
                dyn_undestroyable_stickiness_shapes[i].size.x + dyn_undestroyable_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
            );
            
            if intr.y > 0.0 {
                store_intersection_entrance_and_exit_for_unbreakables(intr);
            }
        } else {
            let s = dyn_undestroyable_stickiness_shapes[i].size;

            let size = vec4(
                min(min(s.y, s.z),s.w),    
                min(min(s.x, s.z),s.w),    
                min(min(s.y, s.x),s.w),
                s.w
            );
            
            let intr = cube_intersection(
                ro - dyn_undestroyable_stickiness_shapes[i].pos,
                rd,
                size + dyn_undestroyable_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
            );
            
            if intr.y > 0.0 {
                store_intersection_entrance_and_exit_for_unbreakables(intr);
            }
        }
    }

    for (var i = 0u; i < dynamic_data.shapes_arrays_metadata.unbreakable_sph_cubes_amount + dynamic_data.shapes_arrays_metadata.unbreakable_sph_cubes_start; i++) {
        if (i < dynamic_data.shapes_arrays_metadata.unbreakable_spheres_start) {
            let intr = cube_intersection(
                ro - dyn_undestroyable_normal_shapes[i].pos,
                rd,
                dyn_undestroyable_normal_shapes[i].size + dyn_undestroyable_normal_shapes[i].roundness
            );
            
            if intr.y > 0.0 {
                store_intersection_entrance_and_exit_for_unbreakables(intr);
            }
        } else if (i < dynamic_data.shapes_arrays_metadata.unbreakable_sph_cubes_start) {
            let intr = sph_intersection(
                ro - dyn_undestroyable_normal_shapes[i].pos,
                rd,
                dyn_undestroyable_normal_shapes[i].size.x + dyn_undestroyable_normal_shapes[i].roundness
            );
            
            if intr.y > 0.0 {
                store_intersection_entrance_and_exit_for_unbreakables(intr);
            }
        } else {
            let s = dyn_undestroyable_normal_shapes[i].size;

            let size = vec4(
                min(min(s.y, s.z),s.w),    
                min(min(s.x, s.z),s.w),    
                min(min(s.y, s.x),s.w),
                s.w
            );
            
            let intr = cube_intersection(
                ro - dyn_undestroyable_normal_shapes[i].pos,
                rd,
                size + dyn_undestroyable_normal_shapes[i].roundness
            );
            
            if intr.y > 0.0 {
                store_intersection_entrance_and_exit_for_unbreakables(intr);
            }
        }
    }

    let intr = sph_intersection(
        ro - dynamic_data.camera_data.cam_pos,
        rd,
        0.4
    );
    
    if intr.y > 0.0 {
        store_intersection_entrance_and_exit_for_unbreakables(intr);
    }

    // for (var i = 0u; i < dynamic_data.player_forms_amount; i++) {
    //     let intr = sph_intersection(
    //         ro - dyn_player_forms[i].pos,
    //         rd,
    //         dyn_player_forms[i].radius * 1.65
    //     );
        
    //     if intr.y > 0.0 {
    //         intr_players = true;
    //         store_intersection_entrance_and_exit_for_unbreakables(intr);
    //     }
    // }

    combine_interscted_entrances_and_exites_for_all_intrs();
    //###find_intersections###
}


fn map(p: vec4<f32>, intr_players: bool) -> f32 {
    //###map###
    var d = MAX_DIST*2.0;

    for (var i = 0u; i < dynamic_data.shapes_arrays_metadata.sph_cubes_amount + dynamic_data.shapes_arrays_metadata.sph_cubes_start; i++) {
        if (i < dynamic_data.shapes_arrays_metadata.spheres_start) {
            d = min(d, sd_box(p - dyn_normal_shapes[i].pos, dyn_normal_shapes[i].size) - dyn_normal_shapes[i].roundness);
        } else if (i < dynamic_data.shapes_arrays_metadata.sph_cubes_start) {
            d = min(d, sd_sphere(p - dyn_normal_shapes[i].pos, dyn_normal_shapes[i].size.x) - dyn_normal_shapes[i].roundness);
        } else {
            d = min(d, sd_sph_box(p - dyn_normal_shapes[i].pos, dyn_normal_shapes[i].size) - dyn_normal_shapes[i].roundness);
        }
    }

    for (var i = 0u; i < dynamic_data.shapes_arrays_metadata.s_sph_cubes_amount + dynamic_data.shapes_arrays_metadata.s_sph_cubes_start; i++) {
        if (i < dynamic_data.shapes_arrays_metadata.s_spheres_start) {
            d = smin(d, sd_box(p - dyn_stickiness_shapes[i].pos, dyn_stickiness_shapes[i].size) - dyn_stickiness_shapes[i].roundness, static_data.stickiness);
        } else if (i < dynamic_data.shapes_arrays_metadata.s_sph_cubes_start) {
            d = smin(d, sd_sphere(p - dyn_stickiness_shapes[i].pos, dyn_stickiness_shapes[i].size.x) - dyn_stickiness_shapes[i].roundness, static_data.stickiness);
        } else {
            d = smin(d, sd_sph_box(p - dyn_stickiness_shapes[i].pos, dyn_stickiness_shapes[i].size) - dyn_stickiness_shapes[i].roundness, static_data.stickiness);
        }
    }

    for (var i = 0u; i < dynamic_data.shapes_arrays_metadata.neg_sph_cubes_amount + dynamic_data.shapes_arrays_metadata.neg_sph_cubes_start; i++) {
        if (i < dynamic_data.shapes_arrays_metadata.neg_spheres_start) {
            d = max(d, -(sd_box(p - dyn_negatives_shapes[i].pos, dyn_negatives_shapes[i].size) - dyn_negatives_shapes[i].roundness));
        } else if (i < dynamic_data.shapes_arrays_metadata.neg_sph_cubes_start) {
            d = max(d, -(sd_sphere(p - dyn_negatives_shapes[i].pos, dyn_negatives_shapes[i].size.x) - dyn_negatives_shapes[i].roundness));
        } else {
            d = max(d, -(sd_sph_box(p - dyn_negatives_shapes[i].pos, dyn_negatives_shapes[i].size) - dyn_negatives_shapes[i].roundness));
        }
    }

    for (var i = 0u; i < dynamic_data.shapes_arrays_metadata.s_neg_sph_cubes_amount + dynamic_data.shapes_arrays_metadata.s_neg_sph_cubes_start; i++) {
        if (i < dynamic_data.shapes_arrays_metadata.s_neg_spheres_start) {
            d = smax(d, -(sd_box(p - dyn_neg_stickiness_shapes[i].pos, dyn_neg_stickiness_shapes[i].size) - dyn_neg_stickiness_shapes[i].roundness), static_data.stickiness);
        } else if (i < dynamic_data.shapes_arrays_metadata.s_neg_sph_cubes_start) {
            d = smax(d, -(sd_box(p - dyn_neg_stickiness_shapes[i].pos, dyn_neg_stickiness_shapes[i].size) - dyn_neg_stickiness_shapes[i].roundness), static_data.stickiness);
        } else {
            d = smax(d, -(sd_sph_box(p - dyn_neg_stickiness_shapes[i].pos, dyn_neg_stickiness_shapes[i].size) - dyn_neg_stickiness_shapes[i].roundness), static_data.stickiness);
        }
    }

    for (var i = 0u; i < dynamic_data.shapes_arrays_metadata.unbreakable_sph_cubes_amount + dynamic_data.shapes_arrays_metadata.unbreakable_sph_cubes_start; i++) {
        if (i < dynamic_data.shapes_arrays_metadata.unbreakable_spheres_start) {
            d = min(d, sd_box(p - dyn_undestroyable_normal_shapes[i].pos, dyn_undestroyable_normal_shapes[i].size) - dyn_undestroyable_normal_shapes[i].roundness);
        } else if (i < dynamic_data.shapes_arrays_metadata.unbreakable_sph_cubes_start) {
            d = min(d, sd_sphere(p - dyn_undestroyable_normal_shapes[i].pos, dyn_undestroyable_normal_shapes[i].size.x) - dyn_undestroyable_normal_shapes[i].roundness);
        } else {
            d = min(d, sd_sph_box(p - dyn_undestroyable_normal_shapes[i].pos, dyn_undestroyable_normal_shapes[i].size) - dyn_undestroyable_normal_shapes[i].roundness);
        }
    }

    for (var i = 0u; i < dynamic_data.shapes_arrays_metadata.unbreakable_s_sph_cubes_amount + dynamic_data.shapes_arrays_metadata.unbreakable_s_sph_cubes_start; i++) {
        if (i < dynamic_data.shapes_arrays_metadata.unbreakable_s_spheres_start) {
            d = smin(d, sd_box(p - dyn_undestroyable_stickiness_shapes[i].pos, dyn_undestroyable_stickiness_shapes[i].size) - dyn_undestroyable_stickiness_shapes[i].roundness, static_data.stickiness);
        } else if (i < dynamic_data.shapes_arrays_metadata.unbreakable_s_sph_cubes_start) {
            d = smin(d, sd_sphere(p - dyn_undestroyable_stickiness_shapes[i].pos, dyn_undestroyable_stickiness_shapes[i].size.x) - dyn_undestroyable_stickiness_shapes[i].roundness, static_data.stickiness);
        } else {
            d = smin(d, sd_sph_box(p - dyn_undestroyable_stickiness_shapes[i].pos, dyn_undestroyable_stickiness_shapes[i].size) - dyn_undestroyable_stickiness_shapes[i].roundness, static_data.stickiness);
        }
    }

    d = min(d, sd_sphere(p - dynamic_data.camera_data.cam_pos, 0.4));

    //###map###

    // if intr_players
    // {
    //     var dddd = MAX_DIST;
    //     for (var i = 0u; i < dynamic_data.player_forms_amount; i++) {
    //         dddd = min(dddd, sd_sphere(p - dyn_player_forms[i].pos, dyn_player_forms[i].radius));
    //         dddd = max(dddd, -sd_sphere(p - dyn_player_forms[i].pos, dyn_player_forms[i].radius * 0.86));
            
    //         let rotated_p = dyn_player_forms[i].rotation * (p - dyn_player_forms[i].pos);
    //         dddd = max(dddd, -sd_box(
    //             rotated_p,
    //             vec4(
    //                 dyn_player_forms[i].radius * 0.18,
    //                 dyn_player_forms[i].radius* 1.2,
    //                 dyn_player_forms[i].radius* 1.2,
    //                 dyn_player_forms[i].radius * 1.2
    //             )));
            
    //         dddd = max(
    //             dddd,
    //             -sd_sphere(
    //                 rotated_p - vec4(0.0, 0.0, -dyn_player_forms[i].radius, 0.0),
    //                 dyn_player_forms[i].radius * 0.53
    //             )
    //         );
    
    //         dddd = min(
    //             dddd,
    //             sd_sphere(
    //                 p - dyn_player_forms[i].pos,
    //                 dyn_player_forms[i].radius * 0.6
    //             )
    //         );
    //         dddd = max(
    //             dddd,
    //             -sd_sphere(
    //                 rotated_p - vec4(0.0, 0.0, -dyn_player_forms[i].radius, 0.0)*0.6,
    //                 dyn_player_forms[i].radius * 0.34
    //             )
    //         );
    
    //         dddd = min(
    //             dddd,
    //             sd_sphere(
    //                 rotated_p - dyn_player_forms[i].weapon_offset,
    //                 dyn_player_forms[i].radius * 0.286,
    //             )
    //         );
    
    //         dddd = max(
    //             dddd,
    //             -sd_capsule(
    //                 rotated_p,
    //                 dyn_player_forms[i].weapon_offset,
    //                 dyn_player_forms[i].weapon_offset -
    //                 vec4(
    //                     0.0,
    //                     0.0,
    //                     dyn_player_forms[i].radius* 0.49,
    //                     0.0
    //                 ),
    //                 dyn_player_forms[i].radius* 0.18
    //             )
    //         );
    
    //         dddd = min(
    //             dddd,
    //             sd_capsule(
    //                 rotated_p,
    //                 dyn_player_forms[i].weapon_offset,
    //                 dyn_player_forms[i].weapon_offset -
    //                 vec4(
    //                     0.0,
    //                     0.0,
    //                     dyn_player_forms[i].radius* 0.43,
    //                     0.0
    //                 ),
    //                 dyn_player_forms[i].radius* 0.1
    //             )
    //         );
    
    //         dddd = max(
    //             dddd,
    //             -sd_capsule(
    //                 rotated_p,
    //                 dyn_player_forms[i].weapon_offset,
    //                 dyn_player_forms[i].weapon_offset -
    //                 vec4(
    //                     0.0,
    //                     0.0,
    //                     dyn_player_forms[i].radius* 0.65,
    //                     0.0
    //                 ),
    //                 dyn_player_forms[i].radius* 0.052
    //             )
    //         );
    //     }
    //     d = min(d, dddd);
    // }

    return d;
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.rel_position = model.rel_position;
    return out;
}


fn rotate_zw(angle: f32) -> mat4x4<f32> {
    var c: f32 = cos(angle);
    var s: f32 = sin(angle);

    return mat4x4<f32>(
        vec4(1.0, 0.0, 0.0, 0.0),
        vec4(0.0, 1.0, 0.0, 0.0),
        vec4(0.0, 0.0, c, -s),
        vec4(0.0, 0.0, s, c)
    );
}

fn rotate_xz(angle: f32) -> mat4x4<f32> {
    var c: f32 = cos(angle);
    var s: f32 = sin(angle);

    return mat4x4<f32>(
        vec4(c, 0.0, -s, 0.0),
        vec4(0.0, 1.0, 0.0, 0.0),
        vec4(s, 0.0, c, 0.0),
        vec4(0.0, 0.0, 0.0, 1.0)
    );
}


fn ray_march_skip_first_obstacle(
    ray_origin: vec4<f32>,
    ray_direction: vec4<f32>,
    max_dist: f32,
) -> vec2<f32>  {
    
    if intr_normal_size == 0u {
        return vec2(MAX_DIST*2.0, 0.0);
    }
    
    var closest_normal_intrs_index = 0u;
    var closest_normal_intrs = intr_normal[closest_normal_intrs_index];

    var total_distance: f32 = max(closest_normal_intrs.x, 0.0);

    
    var closest_neg_intrs_index = 0u;
    var closest_neg_intrs = vec2(MAX_DIST*2.0);
    if intr_neg_size > 0u
    {
        closest_neg_intrs = intr_neg[0u];
    }

    var closest_unbreakables_intrs_index = 0u;
    var closest_unbreakables_intrs = vec2(MAX_DIST*2.0);
    if intr_unbreakables_size > 0u
    {
        closest_unbreakables_intrs = intr_unbreakables[0u];
    }

    if closest_normal_intrs.x < 0.0
    {
        
        let intr = min(
            min(closest_normal_intrs.y, closest_neg_intrs.y),
            closest_unbreakables_intrs.x
        );

        total_distance = max(intr, 0.0);
    }

    var i: i32 = 0;
    for (; i < MAX_STEPS; i++)
    {
        while total_distance < max_dist
        {
            // cheking if ray is out of area of positive (not negative) objects
            // in this case go to next closest positve object or finish ray marching 
            // if it was last area of positive objects
            while total_distance > closest_normal_intrs.y
            {
                closest_normal_intrs_index += 1u;
    
                if closest_normal_intrs_index < intr_normal_size
                {
                    closest_normal_intrs = intr_normal[closest_normal_intrs_index];
    
                    total_distance = max(total_distance, closest_normal_intrs.x);
                }
                else
                {
                    return vec2(MAX_DIST*2.0, f32(i));
                }
            }

            // finding closet area of unbreakable objects
            while total_distance > closest_unbreakables_intrs.y
            {
                closest_unbreakables_intrs_index += 1u;
    
                if closest_unbreakables_intrs_index < intr_unbreakables_size
                {
                    closest_unbreakables_intrs = intr_unbreakables[closest_unbreakables_intrs_index];
                }
                else
                {
                    closest_unbreakables_intrs = vec2(MAX_DIST*2.0);
                }
            }

            // finding closet area of negative objects
            while total_distance > closest_neg_intrs.y
            {
                closest_neg_intrs_index += 1u;

                if closest_neg_intrs_index < intr_neg_size
                {
                    closest_neg_intrs = intr_neg[closest_neg_intrs_index];
                }
                else
                {
                    closest_neg_intrs = vec2(MAX_DIST*2.0);
                }
            }
            
            
            // cheking if ray is entered in area of negative objects
            // skip whole nagtive area if ray is not collided
            // by area of unbreakable objects. 
            // if ray is not entered nagtive area - it's means that ray is inside 
            // area of positive objects
            if total_distance > closest_neg_intrs.x && total_distance < closest_unbreakables_intrs.x
            {
                if closest_unbreakables_intrs.x < closest_neg_intrs.y
                {
                    total_distance = closest_unbreakables_intrs.x;

                    break;
                }
                else
                {
                    total_distance = closest_neg_intrs.y;

                    closest_neg_intrs_index += 1u;

                    if closest_neg_intrs_index < intr_neg_size
                    {
                        closest_neg_intrs = intr_neg[closest_neg_intrs_index];
                    }
                    else
                    {
                        closest_neg_intrs = vec2(MAX_DIST*2.0);
                    }

                    continue;
                }
            }
            else
            {
                break;
            }
        }
        
        if total_distance > max_dist
        {
            return vec2<f32>(total_distance, f32(i));
        }

        var d: f32  = map(ray_origin + ray_direction * total_distance, intr_players);
        total_distance += d;

        
        if (d < MIN_DIST) {

            return vec2<f32>(total_distance, f32(i));
        }
    }
    return vec2<f32>(total_distance, f32(i));
}

fn ray_march(
    ray_origin: vec4<f32>,
    ray_direction: vec4<f32>,
    max_dist: f32,
) -> vec2<f32>  {
    
    if intr_normal_size == 0u {
        return vec2(MAX_DIST*2.0, 0.0);
    }
    
    var closest_normal_intrs_index = 0u;
    var closest_normal_intrs = intr_normal[closest_normal_intrs_index];

    var total_distance: f32 = max(closest_normal_intrs.x, 0.0);

    
    var closest_neg_intrs_index = 0u;
    var closest_neg_intrs = vec2(MAX_DIST*2.0);
    if intr_neg_size > 0u
    {
        closest_neg_intrs = intr_neg[0u];
    }

    var closest_unbreakables_intrs_index = 0u;
    var closest_unbreakables_intrs = vec2(MAX_DIST*2.0);
    if intr_unbreakables_size > 0u
    {
        closest_unbreakables_intrs = intr_unbreakables[0u];
    }

    var i: i32 = 0;
    for (; i < MAX_STEPS; i++)
    {
        while total_distance < max_dist
        {
            // cheking if ray is out of area of positive (not negative) objects
            // in this case go to next closest positve object or finish ray marching 
            // if it was last area of positive objects
            while total_distance > closest_normal_intrs.y
            {
                closest_normal_intrs_index += 1u;
    
                if closest_normal_intrs_index < intr_normal_size
                {
                    closest_normal_intrs = intr_normal[closest_normal_intrs_index];
    
                    total_distance = max(total_distance, closest_normal_intrs.x);
                }
                else
                {
                    return vec2(MAX_DIST*2.0, f32(i));
                }
            }

            // finding closet area of unbreakable objects
            while total_distance > closest_unbreakables_intrs.y
            {
                closest_unbreakables_intrs_index += 1u;
    
                if closest_unbreakables_intrs_index < intr_unbreakables_size
                {
                    closest_unbreakables_intrs = intr_unbreakables[closest_unbreakables_intrs_index];
                }
                else
                {
                    closest_unbreakables_intrs = vec2(MAX_DIST*2.0);
                }
            }

            // finding closet area of negative objects
            while total_distance > closest_neg_intrs.y
            {
                closest_neg_intrs_index += 1u;

                if closest_neg_intrs_index < intr_neg_size
                {
                    closest_neg_intrs = intr_neg[closest_neg_intrs_index];
                }
                else
                {
                    closest_neg_intrs = vec2(MAX_DIST*2.0);
                }
            }
            
            
            // cheking if ray is entered in area of negative objects
            // skip whole nagtive area if ray is not collided
            // by area of unbreakable objects. 
            // if ray is not entered nagtive area - it's means that ray is inside 
            // area of positive objects
            if total_distance > closest_neg_intrs.x && total_distance < closest_unbreakables_intrs.x
            {
                if closest_unbreakables_intrs.x < closest_neg_intrs.y
                {
                    total_distance = closest_unbreakables_intrs.x;

                    break;
                }
                else
                {
                    total_distance = closest_neg_intrs.y;

                    closest_neg_intrs_index += 1u;

                    if closest_neg_intrs_index < intr_neg_size
                    {
                        closest_neg_intrs = intr_neg[closest_neg_intrs_index];
                    }
                    else
                    {
                        closest_neg_intrs = vec2(MAX_DIST*2.0);
                    }

                    continue;
                }
            }
            else
            {
                break;
            }
        }
        
        if total_distance > max_dist
        {
            return vec2<f32>(total_distance, f32(i));
        }

        var d: f32  = map(ray_origin + ray_direction * total_distance, intr_players);
        total_distance += d;

        
        if (d < MIN_DIST) {

            return vec2<f32>(total_distance, f32(i));
        }
    }
    return vec2<f32>(total_distance, f32(i));
}


@fragment
fn fs_main(inn: VertexOutput) -> @location(0) vec4<f32> {

    var pos = inn.rel_position.xy;
    
    pos.x *= dynamic_data.screen_aspect;

    if length(pos.xy) > 1.0
    {
        return vec4(0.0);
    }

    let uv: vec2<f32> = pos * 0.7;

    var ray_direction: vec4<f32> = normalize(vec4<f32>(0.0, 0.0, -1.0, 0.0));
    
    var screen_offset = vec4<f32>(uv.x, 0.0, 0.0, uv.y)*12.0;

    var main_camera_offest = vec4(0.0, 0.0, 10.0, 0.0);

    let zw_rot = 0.3;
    let xz_rot = 1.1;

    ray_direction *= rotate_zw(zw_rot);
    ray_direction *= rotate_xz(xz_rot);

    screen_offset *= rotate_zw(zw_rot);
    screen_offset *= rotate_xz(xz_rot);

    main_camera_offest *= rotate_zw(zw_rot);
    main_camera_offest *= rotate_xz(xz_rot);

    let camera_position = dynamic_data.camera_data.cam_pos + main_camera_offest + screen_offset;
    
    intr_neg_size = 0u;
    intr_normal_size = 0u;
    intr_unbreakables_size = 0u;
    intr_players = false;

    find_intersections(camera_position, ray_direction);

    let dist_and_depth: vec2<f32> = ray_march_skip_first_obstacle(camera_position, ray_direction, MAX_DIST);

    var view_tri_left_vert= vec4(-40.0*dynamic_data.screen_aspect, 0.0, -40.0, 0.0);
    var view_tri_right_vert= vec4(40.0*dynamic_data.screen_aspect, 0.0, -40.0, 0.0);

    view_tri_left_vert *= dynamic_data.camera_data.cam_zw_rot;
    view_tri_right_vert *= dynamic_data.camera_data.cam_zw_rot;

    let camp_pos_3d = vec3(camera_position.x, camera_position.w, camera_position.z);
    let ray_dir_3d = vec3(ray_direction.x, ray_direction.w, ray_direction.z);
    let v0 = vec3(dynamic_data.camera_data.cam_pos.x, dynamic_data.camera_data.cam_pos.w, dynamic_data.camera_data.cam_pos.z);
    let v1 = v0 + vec3(view_tri_left_vert.x, view_tri_left_vert.w, view_tri_left_vert.z);
    let v2 = v0 + vec3(view_tri_right_vert.x, view_tri_right_vert.w, view_tri_right_vert.z);

    var tri_dist = tri_intersect_3d( camp_pos_3d, ray_dir_3d, v0, v1, v2 );

    if tri_dist < MAX_DIST
    {
        intr_neg_size = 0u;
        intr_normal_size = 0u;
        intr_unbreakables_size = 0u;
        intr_players = false;
        
        let ro = camera_position+ray_direction*tri_dist;
        let rd =  normalize(dynamic_data.camera_data.cam_pos - ro);
        
        find_intersections(ro, rd);
    
        let dist_to_player = distance(ro, dynamic_data.camera_data.cam_pos);
        let dist_to_map = ray_march(ro, rd, MAX_DIST).x;
    
        if dist_to_player-(0.4+MIN_DIST) > dist_to_map
        {
            tri_dist = MAX_DIST*2.0;
        }
    }


    return vec4(1.0 - (min(dist_and_depth.x, tri_dist) / (MAX_DIST*0.2)));
    // return vec4(vec3(1.0), 0.1);
    // return vec4(1.0, 0.0, 0.0, 1.0);
}