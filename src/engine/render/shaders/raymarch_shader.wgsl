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
    
fn rotate(angle: f32) -> mat2x2<f32> {
    var c: f32 = cos(angle);
    var s: f32 = sin(angle);
    return mat2x2<f32>(c, -s, s, c);
}

// This code is taken and adapted from the Inigo Quilez's website on
// https://iquilezles.org
// thank you Inigo! :)
// V----------------------------------------------------------------------------------V 
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
// ^----------------------------------------------------------------------------------^ 

fn get_coloring_areas_color(p: vec4<f32>, dist: f32) -> vec3<f32> {
    var color = vec3<f32>(0.0);

    for (
        var i = dynamic_data.spherical_areas_meatadata.holegun_colorized_areas_start;
        i < dynamic_data.spherical_areas_meatadata.holegun_colorized_areas_amount + dynamic_data.spherical_areas_meatadata.holegun_colorized_areas_start;
        i++
    )
    {
        let d = -sd_sphere(p - dyn_spherical_areas[i].pos, dyn_spherical_areas[i].radius);

        let air_perspective = clamp(1.0 - ((dist)/50.0),0.14,1.0);

        color += (dyn_spherical_areas[i].color * clamp(
            (d/dyn_spherical_areas[i].radius) * 10.0, 0.0, 1.0
        )) * air_perspective;
    }

    return color;
}


fn get_volume_areas_color(start_pos: vec4<f32>, direction: vec4<f32>, max_distance: f32) -> vec4<f32> {
    var color = vec3(0.0);

    var ray_march_individual_wave_sphere_color = vec4(0.0);

    for (
        var i = dynamic_data.spherical_areas_meatadata.explode_areas_start;
        i < dynamic_data.spherical_areas_meatadata.explode_areas_amount + dynamic_data.spherical_areas_meatadata.explode_areas_start;
        i++
    )
    {
        color += get_individual_volume_sphere_color(
            dyn_spherical_areas[i],
            start_pos,
            direction, 
            max_distance
        );
    }

    for (
        var i = 0u;
        i < dynamic_data.beam_areas_amount;
        i++
    )
    {
        color += get_indicidual_volume_beam_color(
            dyn_beam_areas[i],
            start_pos,
            direction,
            max_distance
        );
    }
    for (
        var i = dynamic_data.waves_start;
        i < dynamic_data.waves_amount + dynamic_data.waves_start;
        i++
    )
    {
        ray_march_individual_wave_sphere_color += ray_march_individual_wave_sphere(
            dyn_spherical_areas[i],
            start_pos,
            direction, 
            max_distance
        );
    }

    let output_color = vec4(
        color.r + ray_march_individual_wave_sphere_color.r,
        color.g + ray_march_individual_wave_sphere_color.g,
        color.b + ray_march_individual_wave_sphere_color.b,
        ray_march_individual_wave_sphere_color.a
    );

    return output_color;
}


fn get_individual_volume_sphere_color(sphere: SphericalArea, start_pos: vec4<f32>, direction: vec4<f32>, max_distance: f32) -> vec3<f32> {
    var color = vec3(0.0);

    let intr = sph_intersection(
        start_pos - sphere.pos,
        direction,
        sphere.radius
    );

    if intr.x > 0.0 {

        if intr.x < max_distance
        {
            let sphere_normal = get_sphere_normal(start_pos+direction*intr.y, sphere.pos, sphere.radius);

            let color_coef = abs(dot(sphere_normal, direction));

            let air_perspective = clamp(1.0 - ((intr.x)/50.0),0.4,1.0);

            color = mix(sphere.color, vec3(1.0), pow(color_coef, 40.5)) * pow(color_coef, 10.0) * air_perspective;
        }
    }
    else
    {
        if intr.y > 0.0
        {
            if dot(sphere.pos - start_pos, direction) > 0
            {
                let sphere_normal = get_sphere_normal(start_pos+direction*intr.y, sphere.pos, sphere.radius);

                let color_coef = abs(dot(sphere_normal, direction));

                color = mix(sphere.color, vec3(1.0), pow(color_coef, 40.5)) * pow(color_coef, 10.0) + vec3(0.00);
            }
            else
            {
                let sphere_normal = get_sphere_normal(start_pos+direction*-intr.y, sphere.pos, sphere.radius);

                let color_coef = abs(dot(sphere_normal, direction));

                color = (mix(sphere.color, vec3(1.0), pow(color_coef, 40.5)) * pow(color_coef, 10.0) + vec3(0.00));
            }
        } 
    }

    return color;
}


fn ray_march_individual_wave_sphere(sphere: SphericalArea, start_pos: vec4<f32>, direction: vec4<f32>, max_distance: f32) -> vec4<f32> {
    var color = vec3(0.0);

    var output_color = vec4(0.0);

    var luminosity = 0.0;

    var total_dist = 0.0;

    var p = start_pos;

    var prev_d = MAX_DIST;

    let intr = sph_intersection(
        start_pos - sphere.pos,
        direction,
        sphere.radius
    );
        
    if intr.y > 0.0 {

        if intr.y < max_distance
        {
            let sphere_normal = get_sphere_normal(p+direction*intr.y, sphere.pos, sphere.radius);

            var color_coef = clamp(1.0 - abs(dot(sphere_normal, direction)), 0.0, 1.0);

            color_coef = clamp(pow(color_coef, 3.0)*1.0, 0.0, 1.0);
                
            color = mix(vec3(0.0), sphere.color, color_coef);
            
            color += (vec3(0.5)*pow(color_coef,4.0));

            luminosity = pow(color_coef,5.0)*4.0;

            if intr.x > 0.0
            {
                let air_perspective = clamp(1.0-(intr.x/50.0),0.01,1.0);

                color *= 2.0*air_perspective;
                luminosity *= 2.0*air_perspective;

            }
        }
    }

    let dist_to_wave = sd_sphere((start_pos + direction*max_distance) - sphere.pos, sphere.radius);
        
    let edge_intensity = clamp(pow(max(1.0 - abs(dist_to_wave),0.0), 5.0), 0.0, 1.0);
    
    color += sphere.color*edge_intensity;
    
    color += vec3(0.5 * max(max(sphere.color.r, sphere.color.g), sphere.color.b)*pow(edge_intensity,4.0));

    luminosity += edge_intensity;

    let target_max_v = max(max(sphere.color.r, sphere.color.g), sphere.color.b);

    color = clamp(color, vec3(0.0), vec3(target_max_v));
    luminosity = clamp(luminosity, 0.0, 1.0);
    
    output_color.r = color.r;
    output_color.g = color.g;
    output_color.b = color.b;
    output_color.a = luminosity;

    return output_color;
}


fn get_sphere_normal(p: vec4<f32>, sphere_pos: vec4<f32>, sphere_radius: f32) -> vec4<f32> {
    return normalize(
        p - sphere_pos
    );
}


fn get_capsule_normal(p: vec4<f32>, beam_pos1: vec4<f32>, beam_pos2: vec4<f32>, beam_radius: f32) -> vec4<f32> {
    var h: vec3<f32> = vec3<f32>(0.001, -0.001, 0.0);
    
    var a: vec4<f32> = p + h.yxxz;
    var b: vec4<f32> = p + h.xyxz;
    var c: vec4<f32> = p + h.xxyz;
    var d: vec4<f32> = p + h.yyyz;
    var e: vec4<f32> = p + h.zzzx;
    var f: vec4<f32> = p + h.zzzy;

    var fa: f32 = sd_capsule(a, beam_pos1, beam_pos2, beam_radius);
    var fb: f32 = sd_capsule(b, beam_pos1, beam_pos2, beam_radius);
    var fc: f32 = sd_capsule(c, beam_pos1, beam_pos2, beam_radius);
    var fd: f32 = sd_capsule(d, beam_pos1, beam_pos2, beam_radius);
    var fe: f32 = sd_capsule(e, beam_pos1, beam_pos2, beam_radius);
    var ff: f32 = sd_capsule(f, beam_pos1, beam_pos2, beam_radius);

    return normalize(
        h.yxxz * fa +
        h.xyxz * fb +
        h.xxyz * fc +
        h.yyyz * fd +
        h.zzzx * fe +
        h.zzzy * ff
    );
}


fn capsule_intersection(ro: vec4<f32>, rd: vec4<f32>, pa: vec4<f32>, pb: vec4<f32>, ra: f32) -> f32
{
    let  ba = pb - pa;
    let  oa = ro - pa;
    let baba = dot(ba,ba);
    let bard = dot(ba,rd);
    let baoa = dot(ba,oa);
    let rdoa = dot(rd,oa);
    let oaoa = dot(oa,oa);
    let a = baba      - bard*bard;
    var b = baba*rdoa - baoa*bard;
    var c = baba*oaoa - baoa*baoa - ra*ra*baba;
    var h = b*b - a*c;
    if(h >= 0.0)
    {
        let t = (-b-sqrt(h))/a;
        let y = baoa + t*bard;
        
        if (y>0.0 && y<baba) {return t;}
        
        var oc: vec4<f32> = oa;
        if (y > 0.0)
        {
            oc = ro-pb;
        }

        b = dot(rd,oc);
        c = dot(oc,oc) - ra*ra;
        h = b*b - c;
        if (h>0.0) {return -b - sqrt(h);}
    }
    return -1.0;
}


fn get_indicidual_volume_beam_color(beam: BeamArea, start_pos: vec4<f32>, direction: vec4<f32>, max_distance: f32) -> vec3<f32> {

    var color = vec3(0.0);

    let d = capsule_intersection(start_pos, direction, beam.pos1, beam.pos2, beam.radius);

    if d > 0.0 && d < max_distance {
        let p = start_pos + direction*d;
        
        let beam_normal = get_capsule_normal(p, beam.pos1, beam.pos2, beam.radius);

        let beam_dir = normalize(beam.pos1 - beam.pos2);

        let beam_perpendicular = normalize(direction - (dot(direction, beam_dir) * beam_dir));

        let color_coef = abs(dot(beam_normal, beam_perpendicular));

        color = mix(beam.color, vec3(1.0), pow(color_coef, 80.5)) * pow(color_coef, 20.0);
    }

    return color;
}


fn plane_w_intersect( ro: vec4<f32>, rd: vec4<f32>, h: f32 ) -> f32
{
    return (ro.w-h)/-rd.w;
}

// fn tri_intersect( ro: vec4<f32>, rd: vec4<f32>, v0: vec4<f32>, v1: vec4<f32>, v2: vec4<f32> ) -> f32
// {
//     let v1v0 = v1 - v0;
//     let v2v0 = v2 - v0;
//     let rov0 = ro - v0;
//     let n = cross( v1v0, v2v0 );
//     let q = cross( rov0, rd );
//     let d = 1.0/dot( rd, n );
//     let u = d*dot( -q, v2v0 );
//     let v = d*dot(  q, v1v0 );
//     var t = d*dot( -n, rov0 );
//     if ( u<0.0 || v<0.0 || (u+v)>1.0 )
//     {
//         t = -1.0;
//     }
//     return t;
// }

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

    for (var i = 0u; i < dynamic_data.player_forms_amount; i++) {
        let intr = sph_intersection(
            ro - dyn_player_forms[i].pos,
            rd,
            dyn_player_forms[i].radius * 1.65
        );
        
        if intr.y > 0.0 {
            intr_players = true;
            store_intersection_entrance_and_exit_for_unbreakables(intr);
        }
    }

    //###find_intersections###
    combine_interscted_entrances_and_exites_for_all_intrs();
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
            d = smax(d, -(sd_sphere(p - dyn_neg_stickiness_shapes[i].pos, dyn_neg_stickiness_shapes[i].size.x) - dyn_neg_stickiness_shapes[i].roundness), static_data.stickiness);
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

    //###map###

    if intr_players
    {
        var dddd = MAX_DIST;
        for (var i = 0u; i < dynamic_data.player_forms_amount; i++) {
            dddd = min(dddd, sd_sphere(p - dyn_player_forms[i].pos, dyn_player_forms[i].radius));
            dddd = max(dddd, -sd_sphere(p - dyn_player_forms[i].pos, dyn_player_forms[i].radius * 0.86));
            
            let rotated_p = dyn_player_forms[i].rotation * (p - dyn_player_forms[i].pos);
            dddd = max(dddd, -sd_box(
                rotated_p,
                vec4(
                    dyn_player_forms[i].radius * 0.18,
                    dyn_player_forms[i].radius* 1.2,
                    dyn_player_forms[i].radius* 1.2,
                    dyn_player_forms[i].radius * 1.2
                )));
            
            dddd = max(
                dddd,
                -sd_sphere(
                    rotated_p - vec4(0.0, 0.0, -dyn_player_forms[i].radius, 0.0),
                    dyn_player_forms[i].radius * 0.53
                )
            );
    
            dddd = min(
                dddd,
                sd_sphere(
                    p - dyn_player_forms[i].pos,
                    dyn_player_forms[i].radius * 0.6
                )
            );
            dddd = max(
                dddd,
                -sd_sphere(
                    rotated_p - vec4(0.0, 0.0, -dyn_player_forms[i].radius, 0.0)*0.6,
                    dyn_player_forms[i].radius * 0.34
                )
            );
    
            dddd = min(
                dddd,
                sd_sphere(
                    rotated_p - dyn_player_forms[i].weapon_offset,
                    dyn_player_forms[i].radius * 0.286,
                )
            );
    
            dddd = max(
                dddd,
                -sd_capsule(
                    rotated_p,
                    dyn_player_forms[i].weapon_offset,
                    dyn_player_forms[i].weapon_offset -
                    vec4(
                        0.0,
                        0.0,
                        dyn_player_forms[i].radius* 0.49,
                        0.0
                    ),
                    dyn_player_forms[i].radius* 0.18
                )
            );
    
            dddd = min(
                dddd,
                sd_capsule(
                    rotated_p,
                    dyn_player_forms[i].weapon_offset,
                    dyn_player_forms[i].weapon_offset -
                    vec4(
                        0.0,
                        0.0,
                        dyn_player_forms[i].radius* 0.43,
                        0.0
                    ),
                    dyn_player_forms[i].radius* 0.1
                )
            );
    
            dddd = max(
                dddd,
                -sd_capsule(
                    rotated_p,
                    dyn_player_forms[i].weapon_offset,
                    dyn_player_forms[i].weapon_offset -
                    vec4(
                        0.0,
                        0.0,
                        dyn_player_forms[i].radius* 0.65,
                        0.0
                    ),
                    dyn_player_forms[i].radius* 0.052
                )
            );
        }
        d = min(d, dddd);
    }

    return d;
}

fn get_mats_simple(
    cam_pos: vec4<f32>,
    ray_dir: vec4<f32>,
    dist: f32,
) -> OutputMaterials {
    var output: OutputMaterials;
    
    if dist > MAX_DIST-MIN_DIST {

        output.materials_count = 1u;
        output.material_weights[0] = 1.0;
        output.materials[0] = -2;
        return output;
    }

    output.materials_count = 1u;
    output.material_weights[0] = 1.0;
    output.materials[0] = 3;
    return output;
}

fn get_mats(
    cam_pos: vec4<f32>,
    ray_dir: vec4<f32>,
    dist: f32,
) -> OutputMaterials {

    //###get_mats###
    
    var output: OutputMaterials;

    if dist > MAX_DIST-MIN_DIST {

        output.materials_count = 1u;
        output.material_weights[0] = 1.0;
        output.materials[0] = -2;
        return output;
    }

    let p = cam_pos + ray_dir * dist;

    output.materials_count = 0u;
    
    for (var i = 0u; i < dynamic_data.player_forms_amount; i++) {
        
        let shape = dyn_player_forms[i];

        if sd_sphere(p - shape.pos, shape.radius*1.7) < 0.0
        {
            var d = sd_sphere(p - shape.pos, shape.radius);
            d = max(d, -sd_sphere(p - shape.pos, shape.radius * 0.86));
            
            let rotated_p = shape.rotation * (p - shape.pos);
            d = max(d, -sd_box(
                rotated_p,
                vec4(
                    shape.radius * 0.18,
                    shape.radius* 1.2,
                    shape.radius* 1.2,
                    shape.radius * 1.2
                )));
            
            d = max(
                d,
                -sd_sphere(
                    rotated_p - vec4(0.0, 0.0, -shape.radius, 0.0),
                    shape.radius * 0.53
                )
            );
    
            if d < MIN_DIST {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                if shape.is_red.x == 1
                {
                    output.materials[0] = static_data.red_players_mat1;
                } else {
                    output.materials[0] = static_data.blue_players_mat1;
                }
                return output;
            }
    
            d = sd_sphere(
                    p - shape.pos,
                    shape.radius * 0.6
                );
    
            let dd = sd_sphere(
                    rotated_p - vec4(0.0, 0.0, -shape.radius/2.0, 0.0)*0.6,
                    shape.radius * 0.24
                );
            
            d = max(
                d,
                -sd_sphere(
                    rotated_p - vec4(0.0, 0.0, -shape.radius, 0.0)*0.6,
                    shape.radius * 0.34
                )
            );
    
            if d < MIN_DIST {
                if dd < 0.0 {
                    output.materials_count = 2u;
                    output.material_weights[0] = 0.26;
                    if shape.is_red.x == 1
                    {
                        output.materials[0] = -3;
                    } else {
                        output.materials[0] = -4;
                    }
                    output.material_weights[1] = 0.74;
                    if shape.is_red.x == 1
                    {
                        output.materials[1] = static_data.red_players_mat2;
                    } else {
                        output.materials[1] = static_data.blue_players_mat2;
                    }
                    return output;
                }
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                if shape.is_red.x == 1
                {
                    output.materials[0] = static_data.red_players_mat2;
                } else {
                    output.materials[0] = static_data.blue_players_mat2;
                }
                return output;
            }
    
            d = sd_sphere(
                    rotated_p - shape.weapon_offset,
                    shape.radius * 0.286,
                );
    
            d = max(
                d,
                -sd_capsule(
                    rotated_p,
                    shape.weapon_offset,
                    shape.weapon_offset -
                    vec4(
                        0.0,
                        0.0,
                        shape.radius* 0.49,
                        0.0
                    ),
                    shape.radius* 0.18
                )
            );
    
            if d < MIN_DIST {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                if shape.is_red.x == 1
                {
                    output.materials[0] = static_data.red_players_mat1;
                } else {
                    output.materials[0] = static_data.blue_players_mat1;
                }
                return output;
            }
    
            d = sd_capsule(
                    rotated_p,
                    shape.weapon_offset,
                    shape.weapon_offset -
                    vec4(
                        0.0,
                        0.0,
                        shape.radius* 0.43,
                        0.0
                    ),
                    shape.radius* 0.1
                );
    
            d = max(
                d,
                -sd_capsule(
                    rotated_p,
                    shape.weapon_offset,
                    shape.weapon_offset -
                    vec4(
                        0.0,
                        0.0,
                        shape.radius* 0.65,
                        0.0
                    ),
                    shape.radius* 0.052
                )
            );
    
            if d < MIN_DIST {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                if shape.is_red.x == 1
                {
                    output.materials[0] = static_data.red_players_mat2;
                } else {
                    output.materials[0] = static_data.blue_players_mat2;
                }
                return output;
            }
        }
    }

    var d = MAX_DIST * 2.0;
    output.materials_count = 0u;

    for (var i = 0u; i < dynamic_data.shapes_arrays_metadata.sph_cubes_amount + dynamic_data.shapes_arrays_metadata.sph_cubes_start; i++) {
        if (i < dynamic_data.shapes_arrays_metadata.spheres_start) {
            let dd = sd_box(p - dyn_normal_shapes[i].pos, dyn_normal_shapes[i].size) - dyn_normal_shapes[i].roundness;

            if  dd < MIN_DIST*2.0 {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = dyn_normal_shapes[i].material;
                return output;
            }

            if dd < d {
                d = dd;
                output.materials[0] = dyn_normal_shapes[i].material;
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
            }
        } else if (i < dynamic_data.shapes_arrays_metadata.sph_cubes_start) {
            let dd = sd_sphere(p - dyn_normal_shapes[i].pos, dyn_normal_shapes[i].size.x) - dyn_normal_shapes[i].roundness;

            if  dd < MIN_DIST*2.0 {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = dyn_normal_shapes[i].material;
                return output;
            }

            if dd < d {
                d = dd;
                output.materials[0] = dyn_normal_shapes[i].material;
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
            }
        } else {
            let dd = sd_sph_box(p - dyn_normal_shapes[i].pos, dyn_normal_shapes[i].size) - dyn_normal_shapes[i].roundness;

            if  dd < MIN_DIST*2.0 {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = dyn_normal_shapes[i].material;
                return output;
            }

            if dd < d {
                d = dd;
                output.materials[0] = dyn_normal_shapes[i].material;
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
            }
        }
    }

    if d > static_data.stickiness * STICKINESS_EFFECT_COEF
    {
        output.materials_count = 0u;
    }

    for (var i = 0u; i < dynamic_data.shapes_arrays_metadata.s_sph_cubes_amount + dynamic_data.shapes_arrays_metadata.s_sph_cubes_start; i++) {
        if (i < dynamic_data.shapes_arrays_metadata.s_spheres_start) {
            let dd = sd_box(p - dyn_stickiness_shapes[i].pos, dyn_stickiness_shapes[i].size) - dyn_stickiness_shapes[i].roundness;
        
            if dd < MIN_DIST*2.0 {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = dyn_stickiness_shapes[i].material;
                return output;
            }

            if dd < static_data.stickiness * STICKINESS_EFFECT_COEF {

                if output.materials_count == 0u
                {
                    output.materials_count = 1u;
                    output.material_weights[0] = 1.0;
                    output.materials[0] = dyn_stickiness_shapes[i].material;
                    d = dd;
                }
                else
                {
                    var coef = 0.0;
                    if d<dd {
                        coef = clamp(pow(max(d/dd,0.0),1.9) * 0.5, 0.0, 1.0);
                    } else {
                        coef = 1.0-clamp((pow(max(dd/d,0.0),1.9) * 0.5), 0.0, 1.0);
                    }
                    output.materials[output.materials_count] = dyn_stickiness_shapes[i].material;
                    output.material_weights[output.materials_count] = coef;
    
                    let mult = 1.0 - coef;
    
                    for (var k = 0u; k < output.materials_count; k++) {
                        output.material_weights[k] *= mult;
                    }
    
                    output.materials_count += 1u;
                    
                    d = min(d,dd);
                }
            }

        } else if (i < dynamic_data.shapes_arrays_metadata.s_sph_cubes_start) {
            let dd = sd_sphere(p - dyn_stickiness_shapes[i].pos, dyn_stickiness_shapes[i].size.x) - dyn_stickiness_shapes[i].roundness;
        
            if dd < MIN_DIST*2.0 {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = dyn_stickiness_shapes[i].material;
                return output;
            }

            if dd < static_data.stickiness * STICKINESS_EFFECT_COEF {

                if output.materials_count == 0u
                {
                    output.materials_count = 1u;
                    output.material_weights[0] = 1.0;
                    output.materials[0] = dyn_stickiness_shapes[i].material;
                    d = dd;
                }
                else
                {
                    var coef = 0.0;
                    if d<dd {
                        coef = clamp(pow(max(d/dd,0.0),1.9) * 0.5, 0.0, 1.0);
                    } else {
                        coef = 1.0-clamp((pow(max(dd/d,0.0),1.9) * 0.5), 0.0, 1.0);
                    }
                    output.materials[output.materials_count] = dyn_stickiness_shapes[i].material;
                    output.material_weights[output.materials_count] = coef;
    
                    let mult = 1.0 - coef;
    
                    for (var k = 0u; k < output.materials_count; k++) {
                        output.material_weights[k] *= mult;
                    }
    
                    output.materials_count += 1u;
                    
                    d = min(d,dd);
                }
            }

        } else {
            let dd = sd_sph_box(p - dyn_stickiness_shapes[i].pos, dyn_stickiness_shapes[i].size) - dyn_stickiness_shapes[i].roundness;
            
            if dd < MIN_DIST*2.0 {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = dyn_stickiness_shapes[i].material;
                return output;
            }

            if dd < static_data.stickiness * STICKINESS_EFFECT_COEF {

                if output.materials_count == 0u
                {
                    output.materials_count = 1u;
                    output.material_weights[0] = 1.0;
                    output.materials[0] = dyn_stickiness_shapes[i].material;
                    d = dd;
                }
                else
                {
                    var coef = 0.0;
                    if d<dd {
                        coef = clamp(pow(max(d/dd,0.0),1.9) * 0.5, 0.0, 1.0);
                    } else {
                        coef = 1.0-clamp((pow(max(dd/d,0.0),1.9) * 0.5), 0.0, 1.0);
                    }
                    output.materials[output.materials_count] = dyn_stickiness_shapes[i].material;
                    output.material_weights[output.materials_count] = coef;
    
                    let mult = 1.0 - coef;
    
                    for (var k = 0u; k < output.materials_count; k++) {
                        output.material_weights[k] *= mult;
                    }
    
                    output.materials_count += 1u;
                    
                    d = min(d,dd);
                }
            }
        }
    }

    for (var i = 0u; i < dynamic_data.shapes_arrays_metadata.unbreakable_sph_cubes_amount + dynamic_data.shapes_arrays_metadata.unbreakable_sph_cubes_start; i++) {
        if (i < dynamic_data.shapes_arrays_metadata.unbreakable_spheres_start) {
            let dd = sd_box(p - dyn_undestroyable_normal_shapes[i].pos, dyn_undestroyable_normal_shapes[i].size) - dyn_undestroyable_normal_shapes[i].roundness;

            if  dd < MIN_DIST*2.0 {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = dyn_undestroyable_normal_shapes[i].material;
                return output;
            }

            if dd < d {
                d = dd;
                output.materials[0] = dyn_undestroyable_normal_shapes[i].material;
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
            }
        } else if (i < dynamic_data.shapes_arrays_metadata.unbreakable_sph_cubes_start) {
            let dd = sd_sphere(p - dyn_undestroyable_normal_shapes[i].pos, dyn_undestroyable_normal_shapes[i].size.x) - dyn_undestroyable_normal_shapes[i].roundness;

            if  dd < MIN_DIST*2.0 {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = dyn_undestroyable_normal_shapes[i].material;
                return output;
            }

            if dd < d {
                d = dd;
                output.materials[0] = dyn_undestroyable_normal_shapes[i].material;
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
            }
        } else {
            let dd = sd_sph_box(p - dyn_undestroyable_normal_shapes[i].pos, dyn_undestroyable_normal_shapes[i].size) - dyn_undestroyable_normal_shapes[i].roundness;

            if  dd < MIN_DIST*2.0 {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = dyn_undestroyable_normal_shapes[i].material;
                return output;
            }

            if dd < d {
                d = dd;
                output.materials[0] = dyn_undestroyable_normal_shapes[i].material;
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
            }
        }
    }

    if d > static_data.stickiness * STICKINESS_EFFECT_COEF
    {
        output.materials_count = 0u;
    }

    for (var i = dynamic_data.shapes_arrays_metadata.unbreakable_s_spheres_start; i < dynamic_data.shapes_arrays_metadata.unbreakable_s_spheres_amount + dynamic_data.shapes_arrays_metadata.unbreakable_s_spheres_start; i++) {
        let dd = sd_sphere(p - dyn_undestroyable_stickiness_shapes[i].pos, dyn_undestroyable_stickiness_shapes[i].size.x) - dyn_undestroyable_stickiness_shapes[i].roundness;
    
        if dd < MIN_DIST*2.0 {
            output.materials_count = 1u;
            output.material_weights[0] = 1.0;
            output.materials[0] = dyn_undestroyable_stickiness_shapes[i].material;
            return output;
        }

        if dd < static_data.stickiness * STICKINESS_EFFECT_COEF {

            if output.materials_count == 0u
            {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = dyn_undestroyable_stickiness_shapes[i].material;
                d = dd;
            }
            else
            {
                var coef = 0.0;
                if d<dd {
                    coef = clamp(pow(max(d/dd,0.0),1.9) * 0.5, 0.0, 1.0);
                } else {
                    coef = 1.0-clamp((pow(max(dd/d,0.0),1.9) * 0.5), 0.0, 1.0);
                }
                output.materials[output.materials_count] = dyn_undestroyable_stickiness_shapes[i].material;
                output.material_weights[output.materials_count] = coef;

                let mult = 1.0 - coef;

                for (var k = 0u; k < output.materials_count; k++) {
                    output.material_weights[k] *= mult;
                }

                output.materials_count += 1u;
                
                d = min(d,dd);
            }
        }
    }
    
    return output;
    //###get_mats###
}


fn get_normal_prev(p: vec4<f32>, intrs_players: bool) -> vec4<f32> {
    var h: vec3<f32> = vec3<f32>(0.001, -0.001, 0.0);
    
    var a: vec4<f32> = p + h.yxxz;
    var b: vec4<f32> = p + h.xyxz;
    var c: vec4<f32> = p + h.xxyz;
    var d: vec4<f32> = p + h.yyyz;
    var e: vec4<f32> = p + h.zzzx;
    var f: vec4<f32> = p + h.zzzy;

    var fa: f32 = map(a, intrs_players);
    var fb: f32 = map(b, intrs_players);
    var fc: f32 = map(c, intrs_players);
    var fd: f32 = map(d, intrs_players);
    var fe: f32 = map(e, intrs_players);
    var ff: f32 = map(f, intrs_players);

    return normalize(
        h.yxxz * fa +
        h.xyxz * fb +
        h.xxyz * fc +
        h.yyyz * fd +
        h.zzzx * fe +
        h.zzzy * ff
    );
}


// for preventing inline func map
fn get_normal(p: vec4<f32>, intrs_players: bool) -> vec4<f32> {
    var h: vec3<f32> = vec3<f32>(MIN_DIST, -MIN_DIST, 0.0);
    
    let a: vec4<f32> = h.yxxz;
    let b: vec4<f32> = h.xyxz;
    let c: vec4<f32> = h.xxyz;
    let d: vec4<f32> = h.yyyz;
    let e: vec4<f32> = h.zzzx;
    let f: vec4<f32> = h.zzzy;

    // after making this const array I catched segmentation fault on
    // (ubuntu 22.04, nvidia RTX 3070 mobile, driver 565.77, ryzen 9)
    // segmentation fault probably occurred during the compilation of naga

    // let arr = array<vec4<f32>, 6>(
    //     vec4(-MIN_DIST, MIN_DIST, MIN_DIST, 0.0),
    //     vec4(MIN_DIST, -MIN_DIST, MIN_DIST, 0.0),
    //     vec4(MIN_DIST, MIN_DIST, -MIN_DIST, 0.0),
    //     vec4(-MIN_DIST, -MIN_DIST, -MIN_DIST, 0.0),
    //     vec4(0.0, 0.0, 0.0, MIN_DIST),
    //     vec4(0.0, 0.0, 0.0, -MIN_DIST),
    // );

    var n = vec4(0.0);

    for( var i=(min(i32(dynamic_data.time), 0)); i<6; i += 1 )
    {
        //let nn = arr[i];

        var nn = vec4(0.0);
         
        if i == 0
        {
            nn = a;
        }
        else if i == 1
        {
            nn = b;
        }
        else if i == 2
        {
            nn = c;
        }
        else if i == 3
        {
            nn = d;
        }
        else if i == 4
        {
            nn = e;
        }
        else
        {
            nn = f;
        }

        n += nn*map(p+nn, intrs_players);
    }

    return normalize(n);
}

const MIN_STEP: f32 = 0.005;

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


fn w_scanner_ring_color(pos: vec4<f32>, dist: f32, ray_dir: vec4<f32>) -> vec4<f32> {
    var scanner_color = vec4(1.0,1.0,1.0,0.0);
    
    if dynamic_data.w_scanner_ring_intesity > 0.0 {

        if dist > dynamic_data.w_scanner_radius {

            let view_dir = vec4(0.0, 0.0, -1.0, 0.0)*dynamic_data.camera_data.cam_zy_rot*dynamic_data.camera_data.cam_zx_rot*dynamic_data.camera_data.cam_zw_rot;

            let y_coof = clamp(pow(max(1.0-dot(ray_dir, view_dir),0.0)*3.0,2.4), 0.0, 1.0);
            let y_coof2 = clamp(pow(max(1.0-ray_dir.y,0.0) , 6.0), 0.0, 1.0);

            scanner_color.a = 0.13 * (y_coof+y_coof2);
            scanner_color.a += y_coof2*0.12;

            scanner_color.a *= clamp((33.0 - dynamic_data.w_scanner_radius)/33.0, 0.0, 0.9);
        }

        let edge_intensity = clamp(pow(max(1.0 - abs(dist - dynamic_data.w_scanner_radius),0.0), 5.0), 0.0, 1.0);
        
        scanner_color.a += edge_intensity;

        scanner_color.a = clamp(scanner_color.a, 0.0, 1.0);

        scanner_color.a *= dynamic_data.w_scanner_ring_intesity;

    }

    return scanner_color;
}


fn w_scanner_enemies_color(pos: vec4<f32>, dist: f32, ray_dir: vec4<f32>) -> vec4<f32> {
    var scanner_color = vec4(1.0,0.0,0.0,0.0);

    var closest_intr = vec2(999.0, -999.0);
    
    for (var i = 0u; i < 16u; i++) {

        if dynamic_data.player_projections[i].radius > 0.0
        {
            let current_intr = sph_intersection(
                pos - dynamic_data.player_projections[i].position,
                ray_dir,
                dynamic_data.player_projections[i].radius
            );
    
            if current_intr.x > 0.0
            {
                if current_intr.x < closest_intr.x
                {
                    closest_intr = vec2(current_intr.x, f32(i));
                } 
            }
        }
    }

    if closest_intr.y > -1.0
    {
        let i = u32(closest_intr.y);

        let n = get_sphere_normal(
            pos+ray_dir*closest_intr.x,
            dynamic_data.player_projections[i].position,
            dynamic_data.player_projections[i].radius
        );

        let vis_d = dot(ray_dir,n);

        var red = pow(clamp((1.0 - abs(vis_d*1.1)), 0.0, 1.0), 2.0);
        
        let rot_coef = abs(sin(dynamic_data.player_projections[i].zw_offset));
        
        red += pow((clamp(-vis_d * 1.3, 0.0, 1.0)), mix(25.0, 9.0, rot_coef)) * rot_coef;
        
        scanner_color.a += red * (dynamic_data.player_projections[i].intensity*0.3 + dynamic_data.player_projections[i].is_active_intensity*1.0);
        scanner_color.a *= dynamic_data.player_projections[i].intensity;
    }

    scanner_color.a = clamp(scanner_color.a, 0.0, 1.0);

    return scanner_color;
}


fn noise( x: vec2<f32> ) -> f32
{
    let p = floor(x);
    var f = fract(x);

    f = f*f*(3.0 - 2.0*f);

    let n = p.x + p.y*57.0;

    let res = mix(mix( hash(n+  0.0), hash(n+  1.0),f.x),
              mix( hash(n+ 57.0), hash(n+ 58.0),f.x),f.y);

    return res;
}


fn hash( n: f32 ) -> f32
{
    return fract(sin(n)*7813.74365523);
}


fn hash2d( n: vec2<f32> ) -> f32
{
    return fract(sin(
        dot(n, vec2(1370.834, 236.623))
    )*8818.5453123);
}


fn noise2( x: vec4<f32> ) -> f32
{
    let p = floor(x);
    var f = fract(x);

    f = f*f*(3.0 - 2.0*f);

    let res = mix(
        mix(
            hash2d(p.xy),
            hash2d(p.xy + vec2(1.0,0.0)),
            f.x
        ),
        mix(
            hash2d(p.xy + vec2(0.0,1.0)),
            hash2d(p.xy + vec2(1.0,1.0)),
            f.x
        ),
        f.y
    );

    return res;
}


fn get_sky_color(ray_dir: vec4<f32>, shadow: f32) -> vec3<f32> {
    let sun = pow(clamp(dot(normalize(static_data.sun_direction),ray_dir), 0.0, 1.0), 10.0);

    var color = static_data.sun_color*pow(sun, 40.0)*shadow;

    color += pow(textureSample(sky_box, sky_box_sampler, normalize(ray_dir.xyz)).xyz, vec3(2.1));

    return color;
}


fn get_color_and_light_from_mats(
    pos: vec4<f32>,
    ray_dir: vec4<f32>,
    dist: f32,
    mats: OutputMaterials
) -> vec4<f32> {
    var lightness = 0.0;
    
    if mats.materials[0] == -2 {
        var color = get_sky_color(ray_dir, 1.0);
        
        color = clamp(color, vec3(0.0), vec3(1.0));

        return vec4(color, lightness);
    }

    if mats.materials[0] == -3 {
        var color = static_data.red_base_color*0.5;
        
        let hited_pos = pos + ray_dir * dist;
        let normal = get_normal(hited_pos, intr_players);
        let c = pow(abs(dot(normal, ray_dir)),9.0);

        color = mix(vec3(0.5),color, c);

        return vec4(color, 20.0);
    }

    if mats.materials[0] == -4 {
        var color = static_data.blue_base_color*0.5;
        
        let hited_pos = pos + ray_dir * dist;
        let normal = get_normal(hited_pos, intr_players);
        let c = pow(abs(dot(normal, ray_dir)),9.0);

        color = mix(vec3(0.5),color, c);

        return vec4(color, 20.0);
    }

    var base_diffuse = static_data.materials[mats.materials[0]].color.xyz;
    var roughness = static_data.materials[mats.materials[0]].color.w;

    for (var i = 1u; i < mats.materials_count; i++) {
        let new_roughness = static_data.materials[mats.materials[i]].color.w;
        roughness = mix(roughness, new_roughness, mats.material_weights[i]);

        let new_base_diffuse = static_data.materials[mats.materials[i]].color.xyz;
        base_diffuse = mix(base_diffuse, new_base_diffuse, mats.material_weights[i]);
    }

    let hited_pos = pos + ray_dir * dist;
    let normal = get_normal(hited_pos, intr_players);
    
    var lines_size = 5.8;

    if mats.materials[0] == static_data.blue_players_mat1 ||
        mats.materials[0] == static_data.blue_players_mat2 ||
        mats.materials[0] == static_data.red_players_mat1 ||
        mats.materials[0] == static_data.red_players_mat2
    {
        lines_size = 1.8;
    }

    let next_normal = get_normal(hited_pos+ray_dir*MIN_DIST*lines_size, intr_players);

    let wireframe_fog = exp(-0.007*dist*dist);
    let wireframe_dif = pow(clamp(1.0-abs(dot(normal, next_normal)),0.0,1.0),1.3);

    // sun light 1
    let sun_dir_1 = normalize(static_data.sun_direction);
    let sun_dif_1 = clamp(dot(normal, sun_dir_1),0.0,1.0);
    let sun_hal_1 = normalize(sun_dir_1-ray_dir);
    let sun_spe_1 = pow(clamp(dot(normal,sun_hal_1),0.0,1.0),45.0+(1.0-roughness)*40.0);
    
    var sun_shadow_1 = 1.0;
    if dynamic_data.shadows_enabled == 1
    {
        sun_shadow_1 = get_shadow(hited_pos+(normal*MIN_DIST*1.6), sun_dir_1);
    }

    let base_coef = clamp((hited_pos.z - static_data.blue_base_position.z) / (static_data.red_base_position.z - static_data.blue_base_position.z), 0.0, 1.0);

    var neon_wireframe_color = mix(
        static_data.blue_base_color,
        static_data.red_base_color,
        base_coef
    );

    if mats.materials[0] == static_data.blue_players_mat1 || mats.materials[0] == static_data.blue_players_mat2 {
        neon_wireframe_color = static_data.blue_base_color * 0.8;
    } else {
        if mats.materials[0] == static_data.red_players_mat1 || mats.materials[0] == static_data.red_players_mat2 {
            neon_wireframe_color = static_data.red_base_color * 0.8;
        }
    }

    var ref_dir = reflect(ray_dir, normal);

    ref_dir = normalize(
        ref_dir +
        vec4(
            hash(ref_dir.x) - 0.5,
            hash(ref_dir.y) - 0.5,
            hash(ref_dir.z) - 0.5,
            hash(ref_dir.w) - 0.5,
        ) * max((roughness*0.08)-0.15,0.0)
    ); 
    let frenel = smoothstep(0.0, 2.0,clamp(1.0 + dot(normal, ray_dir), 0.0, 1.0));

    // sky light    
    let sky_dif = clamp(0.5 + 0.5*normal.y,0.0,1.0);
    let sky_hal = normalize(vec4(0.0,1.0,0.0,0.0)-ray_dir);
    let sky_spe = pow(clamp(dot(normal,sky_hal),0.0,1.0),5.0);

    var light = vec3(0.0);
    light += static_data.sun_color  * sun_dif_1 * sun_shadow_1 * 0.13;// * aocc;
    light += static_data.sun_color  * sun_dif_1 * sun_spe_1 * sun_shadow_1 * 3.0;// * aocc;
    light += static_data.sky_color    * sky_dif   * 0.3 * clamp(sky_spe, 0.25, 1.0);// * 0.8;// * aocc;
    light += static_data.frenel_color * frenel    * 0.3 * (0.6+0.4*sun_dif_1);// * aocc;
    light += neon_wireframe_color * wireframe_dif*40.0 * (0.08+0.5*sun_dif_1*sun_shadow_1) * (wireframe_fog*0.5+0.5);

    lightness = wireframe_dif*30.0;

    if mats.materials[0] != static_data.blue_players_mat1 && mats.materials[0] != static_data.blue_players_mat2 &&
        mats.materials[0] != static_data.red_players_mat1 && mats.materials[0] != static_data.red_players_mat2
    {
        let inverted_base_diffuse = vec3(base_diffuse.b, base_diffuse.g, base_diffuse.r);

        let w_height_coef = clamp((hited_pos.w - 0.3) / 7.5, 0.0, 1.0);

        base_diffuse = mix(
            mix(base_diffuse, inverted_base_diffuse, base_coef),
            mix(
                inverted_base_diffuse*0.5*vec3(2.2,4.9,1.2),
                base_diffuse*0.5*vec3(1.2,4.9,2.2),
                base_coef),
            pow(w_height_coef, 0.69)
        );
    }

    let diffuse = base_diffuse + neon_wireframe_color * pow(wireframe_dif,2.5)*20.0*(0.1+0.9*wireframe_fog);
    
    let ref_col = get_sky_color(ref_dir, sun_shadow_1);

    var color = diffuse * mix(ref_col, light, clamp(roughness, 0.0, 1.0));

    color = clamp(color, vec3(0.0), vec3(1.0));

    let air_perspective = clamp(1.0-exp(-0.000006*dist*dist*dist),0.2,1.0);

    color = mix(color, static_data.sky_color, air_perspective);
    return vec4(color, lightness);
}


fn get_shadow(init_position: vec4<f32>, ray_direction: vec4<f32>) -> f32 {

    intr_neg_size = 0u;
    intr_normal_size = 0u;
    intr_unbreakables_size = 0u;
    intr_players = false;

    
    find_intersections(init_position, ray_direction);

    let dist_and_depth: vec2<f32> = ray_march(init_position, ray_direction, 11.0);

    let shadow_coef = pow((min(max(dist_and_depth.x,0.0), 11.0))/11.0, 1.6);

    return clamp(shadow_coef, 0.0, 1.0);
}


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


fn tv_hash(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 19.19);
    return fract((p3.x + p3.y) * p3.z);
}

fn tv_noise(uv: vec2<f32>, time: f32) -> f32 {
    let scale = 10.0;
    let p = uv * scale + vec2<f32>(time);
    return tv_hash(p);
}

// modified code from https://www.shadertoy.com/view/stGXzy
// V-------------------------------------------------------V
fn rand_vec(p: vec2<f32>) -> vec2<f32> {
    var r = fract(sin(dot(p, vec2(12.345, 741.85)))*4563.12);
    r *= 2.0*PI;
    return vec2(sin(r), cos(r));
}

fn fn_mod(x: vec2<f32>, y: vec2<f32>) -> vec2<f32> {
    return x-y*floor(x/y);
}

// Seamless tiled perlin noise
fn perlin(p: vec2<f32>, t: vec2<f32>) -> f32 {
    let f = fract(p);
    let s = smoothstep(vec2(0.0), vec2(1.0), f);
    let i = floor(p);

    // Apply mod() to vertex position to make it tileable
    let a = dot(rand_vec(fn_mod(i,t)), f);
    let b = dot(rand_vec(fn_mod(i+vec2(1.0,0.0),t)), f-vec2(1.0,0.0));
    let c = dot(rand_vec(fn_mod(i+vec2(0.0,1.0),t)), f-vec2(0.0,1.0));
    let d = dot(rand_vec(fn_mod(i+vec2(1.0,1.0),t)), f-vec2(1.0,1.0));
    return mix(mix(a, b, s.x), mix(c, d, s.x), s.y);
}

// Seamless tiled fractal noise
fn fbm(pp: vec2<f32>, tt: vec2<f32>) -> f32 {
    var a = 0.5;
    var r = 0.0;
    var p = pp;
    var t = tt;
    for (var i = 0; i < 7; i++) {
        r += a*perlin(p, t);
        a *= 0.5;
        p *= 2.0;
        t *= 2.0;
    }
    return r;
}

fn w_shift_effect(uv: vec2<f32>, shift_coef: f32, intensity: f32) -> f32
{
    let cuv = vec2((atan(uv.x / uv.y)+PI)/(2.0*PI), 0.005/length(uv)+0.03*shift_coef+111.0);

    var v = clamp(pow(length(uv),23.0),0.0,1.0);

    return clamp((pow(max(0.9+0.5*fbm(29.0*cuv, vec2(20)),0.0),40.0)),0.0,1.0)*intensity*v;
}
//^---------------------------------------------------------^





@fragment
fn fs_main(inn: VertexOutput) -> @location(0) vec4<f32> {

    var uv: vec2<f32> = inn.position.xy * 0.7;
    uv.x *= dynamic_data.screen_aspect;

    var ray_direction: vec4<f32> = normalize(vec4<f32>(uv, -1.0, 0.0));

    ray_direction *= dynamic_data.camera_data.cam_zw_rot;
    ray_direction *= dynamic_data.camera_data.cam_zy_rot;
    ray_direction *= dynamic_data.camera_data.cam_zx_rot;

    let camera_position = dynamic_data.camera_data.cam_pos;

    intr_neg_size = 0u;
    intr_normal_size = 0u;
    intr_unbreakables_size = 0u;
    intr_players = false;

    find_intersections(camera_position, ray_direction);

    let dist_and_depth: vec2<f32> = ray_march(camera_position, ray_direction, MAX_DIST); 

    var mats = get_mats(camera_position, ray_direction, dist_and_depth.x);
    var color_and_light = get_color_and_light_from_mats(camera_position, ray_direction, dist_and_depth.x, mats);

    var color = color_and_light.rgb;

    var lightness = color_and_light.a;

    if mats.materials[0] != static_data.blue_players_mat1 && mats.materials[0] != static_data.blue_players_mat2 && mats.materials[0] != static_data.red_players_mat1 && mats.materials[0] != static_data.red_players_mat2 {
        color += 0.145*get_coloring_areas_color(camera_position + ray_direction * dist_and_depth.x, dist_and_depth.x);
    }

    let color_areas = 0.6*get_volume_areas_color(camera_position, ray_direction, dist_and_depth.x);

    color += color_areas.rgb;
    lightness += color_areas.a;

    let sc_r_c = w_scanner_ring_color(camera_position, dist_and_depth.x, ray_direction);
    let sc_e_c = w_scanner_enemies_color(camera_position, dist_and_depth.x, ray_direction);
    color = mix(color, sc_r_c.rgb, sc_r_c.a*0.3);
    color = mix(color, sc_e_c.rgb, sc_e_c.a*0.55);

    // color correction
    color = pow(max(color,vec3(0.0)), vec3(0.4645));

    // for debug
    // color = mix(color, vec3(1.0, 0.0, 0.0), dist_and_depth.y / f32(MAX_STEPS));

    let tv_noise = tv_noise(uv*100.0, dynamic_data.time);
    
    // making damage effect
    let q = (inn.position.xy+vec2(1.0))/2.0;
    
    // making vignetting effect
    let v = 0.2+pow(max(30.0*q.x*q.y*(1.0-q.x)*(1.0-q.y),0.0),0.32 );
    color *= v;

    color += 0.36*w_shift_effect(
        uv,
        dynamic_data.w_shift_coef,
        dynamic_data.w_shift_intensity,
    );

    let hurt_coef = max(
        clamp(0.01+pow(max(30.0*q.x*q.y*(1.0-q.x)*(1.0-q.y),0.0),0.2),0.0,1.0),
        (1.0-clamp(dynamic_data.getting_damage_screen_effect,0.0,1.0))
    );


    color -= (1.0-hurt_coef)*0.2;

    color += (tv_noise- 0.5)*1.5*(0.92-hurt_coef)*dynamic_data.getting_damage_screen_effect;

    // add w rotation effect
    let zw_dir = dynamic_data.camera_data.cam_zw_rot * vec4(0.0, 0.0, -1.0, 0.0);

    let ring_r = 0.29;
    let line_width = 0.004;
    let n_uv = normalize(uv);

    // draw segment of circle to show angle of rotation along w axis
    let rot_c = clamp(clamp(line_width-abs(length(uv)-(ring_r-line_width)),0.0,1.0)*100.0,0.0,1.0);

    if zw_dir.w > 0.0
    {
        if n_uv.x < 0.0 && n_uv.y < 0.0
        {
            if -n_uv.y < zw_dir.w
            {
                color = mix(color, vec3(2.0, 0.0, 0.0), rot_c);
            } 
        }
    }
    else
    {
        if n_uv.x > 0.0 && n_uv.y > 0.0
        {
            if -n_uv.y > zw_dir.w
            {
                color = mix(color, vec3(2.0, 0.0, 0.0), rot_c);
            }
        }
    }

    // making death effect
    let death_eff_col = max(
        clamp(0.4+pow(max(10.0*q.x*q.y*(1.0-q.x)*(1.0-q.y),0.0),0.4),0.0,1.0),
        (1.0-clamp(dynamic_data.death_screen_effect,0.0,1.0))
    );
    color *= death_eff_col;

    //making monochrome effect for death effect
    var bw_col = clamp(color, vec3(color.r), vec3(100.0));
    bw_col = clamp(bw_col, vec3(color.g), vec3(100.0));
    bw_col = clamp(bw_col, vec3(color.b), vec3(100.0));
    bw_col += (tv_noise - 0.5)*(1.0-death_eff_col*0.5)*0.3;

    color = mix(
        color,
        bw_col*(bw_col*1.4),
        clamp(dynamic_data.death_screen_effect, 0.0, 1.0)
    );

    return vec4<f32>(color, lightness);
}