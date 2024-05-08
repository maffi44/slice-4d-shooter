// Fragment shader

struct CameraUniform {
    cam_pos: vec4<f32>,
    cam_rot: mat4x4<f32>,
}


struct Shape {
    pos: vec4<f32>,
    size: vec4<f32>,
    material: i32,
    empty_bytes: vec2<f32>,
    roundness: f32,
}

struct PlayerForm {
    pos: vec4<f32>,
    empty_bytes: vec4<u32>,
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

struct IntersectedShapesMetadata {
    //normals
    st_cubes_start: u32,
    st_cubes_amount: u32,

    dyn_cubes_start: u32,
    dyn_cubes_amount: u32,


    st_spheres_start: u32,
    st_spheres_amount: u32,

    dyn_spheres_start: u32,
    dyn_spheres_amount: u32,


    st_inf_cubes_start: u32,
    st_inf_cubes_amount: u32,

    dyn_inf_cubes_start: u32,
    dyn_inf_cubes_amount: u32,


    st_sph_cubes_start: u32,
    st_sph_cubes_amount: u32,

    dyn_sph_cubes_start: u32,
    dyn_sph_cubes_amount: u32,


    //stickinesses
    st_s_cubes_start: u32,
    st_s_cubes_amount: u32,

    dyn_s_cubes_start: u32,
    dyn_s_cubes_amount: u32,


    st_s_spheres_start: u32,
    st_s_spheres_amount: u32,

    dyn_s_spheres_start: u32,
    dyn_s_spheres_amount: u32,


    st_s_inf_cubes_start: u32,
    st_s_inf_cubes_amount: u32,

    dyn_s_inf_cubes_start: u32,
    dyn_s_inf_cubes_amount: u32,


    st_s_sph_cubes_start: u32,
    st_s_sph_cubes_amount: u32,

    dyn_s_sph_cubes_start: u32,
    dyn_s_sph_cubes_amount: u32,


    //negatives
    st_neg_cubes_start: u32,
    st_neg_cubes_amount: u32,

    dyn_neg_cubes_start: u32,
    dyn_neg_cubes_amount: u32,


    st_neg_spheres_start: u32,
    st_neg_spheres_amount: u32,

    dyn_neg_spheres_start: u32,
    dyn_neg_spheres_amount: u32,


    st_neg_inf_cubes_start: u32,
    st_neg_inf_cubes_amount: u32,

    dyn_neg_inf_cubes_start: u32,
    dyn_neg_inf_cubes_amount: u32,


    st_neg_sph_cubes_start: u32,
    st_neg_sph_cubes_amount: u32,

    dyn_neg_sph_cubes_start: u32,
    dyn_neg_sph_cubes_amount: u32,

    //neg_stickinesses
    st_s_neg_cubes_start: u32,
    st_s_neg_cubes_amount: u32,

    dyn_s_neg_cubes_start: u32,
    dyn_s_neg_cubes_amount: u32,


    st_s_neg_spheres_start: u32,
    st_s_neg_spheres_amount: u32,

    dyn_s_neg_spheres_start: u32,
    dyn_s_neg_spheres_amount: u32,


    st_s_neg_inf_cubes_start: u32,
    st_s_neg_inf_cubes_amount: u32,

    dyn_s_neg_inf_cubes_start: u32,
    dyn_s_neg_inf_cubes_amount: u32,


    st_s_neg_sph_cubes_start: u32,
    st_s_neg_sph_cubes_amount: u32,

    dyn_s_neg_sph_cubes_start: u32,
    dyn_s_neg_sph_cubes_amount: u32,


    player_forms_start: u32,
    player_forms_amount: u32,
}

struct Intersections {
    ismd: IntersectedShapesMetadata,
    ish: array<u32, 64>,
    offset: f32,
    ray_w_rotated: bool,
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




struct OtherDynamicData {
    shapes_arrays_metadata: ShapesMetadata,
    spherical_areas_meatadata: SphericalAreasMetadata,
    camera_data: CameraUniform,
    empty_bytes1: vec3<u32>,
    beam_areas_amount: u32,
    player_forms_amount: u32,
    w_scaner_radius: f32,
    w_scaner_intesity: f32,
    death_screen_effect: f32,
    getting_damage_screen_effect: f32,
    stickiness: f32,
    screen_aspect: f32,
    time: f32,
}

struct Material {
    color: vec3<f32>,
}

struct OtherStaticData {
    shapes_arrays_metadata: ShapesMetadata,
    
    is_w_floor_exist: i32,
    w_floor: f32,
    // is_w_roof_exist: i32,
    // w_roof: f32,




    empty_bytes: vec2<f32>,
    materials: array<Material, 32>,

    players_mat1: i32,
    players_mat2: i32,
    w_cups_mat: i32,
    stickiness: f32,
}


@group(0) @binding(0) var<uniform> normal_shapes: array<Shape, 256>;
@group(0) @binding(1) var<uniform> negatives_shapes: array<Shape, 256>;
@group(0) @binding(2) var<uniform> stickiness_shapes: array<Shape, 256>;
@group(0) @binding(3) var<uniform> neg_stickiness_shapes: array<Shape, 256>;

@group(0) @binding(4) var<uniform> static_data: OtherStaticData;


@group(0) @binding(5) var<uniform> dyn_normal_shapes: array<Shape, 256>;
@group(0) @binding(6) var<uniform> dyn_negatives_shapes: array<Shape, 256>;
@group(0) @binding(7) var<uniform> dyn_stickiness_shapes: array<Shape, 256>;
@group(0) @binding(8) var<uniform> dyn_neg_stickiness_shapes: array<Shape, 256>;

@group(0) @binding(9) var<uniform> dynamic_data: OtherDynamicData;

@group(1) @binding(0) var<uniform> dyn_spherical_areas: array<SphericalArea, 256>;
@group(1) @binding(1) var<uniform> dyn_beam_areas: array<BeamArea, 64>;
@group(1) @binding(2) var<uniform> dyn_player_forms: array<PlayerForm, 16>;

const MAX_STEPS: i32 = 128;
const PI: f32 = 3.1415926535897;
const MIN_DIST: f32 = 0.01;
const MAX_DIST: f32 = 350.0;

const STICKINESS_EFFECT_COEF: f32 = 3.05;

fn rotate(angle: f32) -> mat2x2<f32> {
    //angle *= 0.017453;
    var c: f32 = cos(angle);
    var s: f32 = sin(angle);
    return mat2x2<f32>(c, -s, s, c);
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
    var d1: f32 = length(p.xy) - b.x;
    var d2: f32 = length(p.xz) - b.y;
    var d3: f32 = length(p.yz) - b.z;
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

fn smin_2(a: f32, b: f32, k: f32) -> f32
{
    let kk = k * (1.0/(1.0-sqrt(0.5)));
    let x: f32 = (b-a)/k;
    var g: f32 = 0.0;
    if (x > 1.0) {
        g = 0.0;
    } else {
        if (x < -1.0) {
            g = x;
        } else {
            g = sin(PI/4.0+asin(x*0.7071067))-1.0;
        }
    }
    return a + kk * g;
}

// fn smin(a: f32, b: f32, k: f32) -> f32
// {
//     let x = (b-a)/k;
//     let g = 0.5*(x-sqrt(x*x+0.25));
//     return a + k * g;
// }

fn smin( a: f32, b: f32, k: f32 ) -> f32
{
    let kk = k * 1.0/(1.0-sqrt(0.5));
    let h = max( kk-abs(a-b), 0.0 )/kk;
    return min(a,b) - kk*0.5*(1.0+h-sqrt(1.0-h*(h - 2.0)));
}

// fn get_color(start_pos: vec4<f32>, direction: vec4<f32>, distance: f32, ray_w_rotated: i32) -> vec3<f32> {
//     let point = start_pos + direction * distance;
    
//     var color = get_color_at_point(point, distance, ray_w_rotated);

//     // color += get_coloring_areas_color(point); 

//     return color;
// }

fn get_coloring_areas_color(p: vec4<f32>) -> vec3<f32> {
    var color = vec3<f32>(0.0);

    for (
        var i = dynamic_data.spherical_areas_meatadata.holegun_colorized_areas_start;
        i < dynamic_data.spherical_areas_meatadata.holegun_colorized_areas_amount + dynamic_data.spherical_areas_meatadata.holegun_colorized_areas_start;
        i++
    )
    {
        let d = -sd_sphere(p - dyn_spherical_areas[i].pos, dyn_spherical_areas[i].radius);

        color += dyn_spherical_areas[i].color * clamp(
            (d/dyn_spherical_areas[i].radius) * 10.0, 0.0, 1.0
        );
    }

    return color;
}

fn get_volume_areas_color(start_pos: vec4<f32>, direction: vec4<f32>, max_distance: f32) -> vec4<f32> {
    var color = vec4(0.0);

    for (
        var i = dynamic_data.spherical_areas_meatadata.explode_areas_start;
        i < dynamic_data.spherical_areas_meatadata.explode_areas_amount + dynamic_data.spherical_areas_meatadata.explode_areas_start;
        i++
    )
    {
        color += ray_march_individual_volume_sphere(
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
        color += ray_march_indicidual_volume_beam(
            dyn_beam_areas[i],
            start_pos,
            direction,
            max_distance
        );
    }

    color = clamp(color, vec4(0.0), vec4(1.0));

    return color;
}

fn ray_march_individual_volume_sphere(sphere: SphericalArea, start_pos: vec4<f32>, direction: vec4<f32>, max_distance: f32) -> vec4<f32> {
    var color = vec4(0.0);

    var total_dist = 0.0;

    var p = start_pos;

    var prev_d = MAX_DIST;

    for (var i = 0; i < MAX_STEPS; i++) {

        if total_dist > max_distance {
            break;
        }
        
        let d = sd_sphere(p - sphere.pos, sphere.radius);

        if d > prev_d {
            break;
        }

        prev_d = d;

        if d < MIN_DIST {

            let sphere_normal = get_sphere_normal(p, sphere.pos, sphere.radius);

            let color_coef = abs(dot(sphere_normal, direction));

            let color_rgb = mix(sphere.color, vec3(1.0), pow(color_coef, 40.5)) * pow(color_coef, 4.0) + vec3(0.05);

            color = vec4(color_rgb, pow(color_coef, 15.0));

            break;
        }
        total_dist += d;

        p += direction * d;
    }

    return color;
}

fn get_sphere_normal(p: vec4<f32>, sphere_pos: vec4<f32>, sphere_radius: f32) -> vec4<f32> {
    var h: vec3<f32> = vec3<f32>(0.001, -0.001, 0.0);
    
    var a: vec4<f32> = p + h.yxxz;
    var b: vec4<f32> = p + h.xyxz;
    var c: vec4<f32> = p + h.xxyz;
    var d: vec4<f32> = p + h.yyyz;
    var e: vec4<f32> = p + h.zzzx;
    var f: vec4<f32> = p + h.zzzy;

    var fa: f32 = sd_sphere(a - sphere_pos, sphere_radius);
    var fb: f32 = sd_sphere(b - sphere_pos, sphere_radius);
    var fc: f32 = sd_sphere(c - sphere_pos, sphere_radius);
    var fd: f32 = sd_sphere(d - sphere_pos, sphere_radius);
    var fe: f32 = sd_sphere(e - sphere_pos, sphere_radius);
    var ff: f32 = sd_sphere(f - sphere_pos, sphere_radius);

    return normalize(
        h.yxxz * fa +
        h.xyxz * fb +
        h.xxyz * fc +
        h.yyyz * fd +
        h.zzzx * fe +
        h.zzzy * ff
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


fn ray_march_indicidual_volume_beam(beam: BeamArea, start_pos: vec4<f32>, direction: vec4<f32>, max_distance: f32) -> vec4<f32> {
    var color = vec4(0.0);

    var total_dist = 0.0;

    var p = start_pos;

    var prev_d = MAX_DIST;

    for (var i = 0; i < MAX_STEPS; i++) {

        if total_dist > max_distance {
            break;
        }

        let d = sd_capsule(p, beam.pos1, beam.pos2, beam.radius);
        
        if d > prev_d {
            break;
        }

        prev_d = d;

        if d < MIN_DIST {
            let beam_normal = get_capsule_normal(p, beam.pos1, beam.pos2, beam.radius);

            let beam_dir = normalize(beam.pos1 - beam.pos2);

            let beam_perpendicular = normalize(direction - (dot(direction, beam_dir) * beam_dir));

            let color_coef = abs(dot(beam_normal, beam_perpendicular));

            let color_rgb = mix(beam.color, vec3(1.0), pow(color_coef, 40.5)) * pow(color_coef, 4.0);

            color = vec4(color_rgb, pow(color_coef, 15.0));

            break;
        }
        total_dist += d;

        p += direction * d;
    }

    return color;
}

// fn get_mat_at_point(p: vec4<f32>, distance: f32, ray_w_rotated: i32) -> vec3<f32> {

//     if distance > MAX_DIST {
//         return vec3(1.0);
//     }
    
//     var d = MAX_DIST;
//     var color = vec3(0.0, 0.0, 0.0);
//     var mat_counter = 0;

//     var mats: array<f32, 8>;

//     // static stickiness shapes
//     for (var i = static_data.shapes_arrays_metadata.s_cubes_start; i < static_data.shapes_arrays_metadata.s_cubes_amount + static_data.shapes_arrays_metadata.s_cubes_start; i++) {
//         let new_d = sd_box(p - stickiness_shapes[i].pos, stickiness_shapes[i].size) - stickiness_shapes[i].roundness;

//         if new_d < static_data.stickiness * STICKINESS_EFFECT_COEF {

//             if new_d < MAX_DIST || mat_counter == 0 {

//             }

//         }
        
//         let dd = smin(d, new_d, static_data.stickiness);

//         let coef = clamp((new_d - d) / (dd - d), 0.0, 1.0);

//         color = mix(color, stickiness_shapes[i].color, coef);

//         d = dd;
//     }
//     for (var i = static_data.shapes_arrays_metadata.s_spheres_start; i < static_data.shapes_arrays_metadata.s_spheres_amount + static_data.shapes_arrays_metadata.s_spheres_start; i++) {
//         let new_d = sd_sphere(p - stickiness_shapes[i].pos, stickiness_shapes[i].size.x) - stickiness_shapes[i].roundness;
        
//         let dd = smin(d, new_d, static_data.stickiness);

//         let coef = clamp((new_d - d) / (dd - d), 0.0, 1.0);

//         color = mix(color, stickiness_shapes[i].color, coef);

//         d = dd;
//     }
//     for (var i = static_data.shapes_arrays_metadata.s_sph_cubes_start; i < static_data.shapes_arrays_metadata.s_sph_cubes_amount + static_data.shapes_arrays_metadata.s_sph_cubes_start; i++) {
//         let new_d = sd_sph_box(p - stickiness_shapes[i].pos, stickiness_shapes[i].size) - stickiness_shapes[i].roundness;
        
//         let dd = smin(d, new_d, static_data.stickiness);

//         let coef = clamp((new_d - d) / (dd - d), 0.0, 1.0);

//         color = mix(color, stickiness_shapes[i].color, coef);

//         d = dd;
//     }
//     for (var i = static_data.shapes_arrays_metadata.s_inf_cubes_start; i < static_data.shapes_arrays_metadata.s_inf_cubes_amount + static_data.shapes_arrays_metadata.s_inf_cubes_start; i++) {
//         let new_d = sd_inf_box(p - stickiness_shapes[i].pos, stickiness_shapes[i].size.xyz) - stickiness_shapes[i].roundness;
        
//         let dd = smin(d, new_d, static_data.stickiness);

//         let coef = clamp((new_d - d) / (dd - d), 0.0, 1.0);

//         color = mix(color, stickiness_shapes[i].color, coef);

//         d = dd;
//     }

//     // dynamic stickiness shapes
//     for (var i = dynamic_data.shapes_arrays_metadata.s_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_cubes_amount + dynamic_data.shapes_arrays_metadata.s_cubes_start; i++) {
//         let new_d = sd_box(p - dyn_stickiness_shapes[i].pos, dyn_stickiness_shapes[i].size) - dyn_stickiness_shapes[i].roundness;
        
//         let dd = smin(d, new_d, static_data.stickiness);

//         let coef = clamp((new_d - d) / (dd - d), 0.0, 1.0);

//         color = mix(color, dyn_stickiness_shapes[i].color, coef);

//         d = dd;
//     }
//     for (var i = dynamic_data.shapes_arrays_metadata.s_spheres_start; i < dynamic_data.shapes_arrays_metadata.s_spheres_amount + dynamic_data.shapes_arrays_metadata.s_spheres_start; i++) {
//         let new_d = sd_sphere(p - dyn_stickiness_shapes[i].pos, dyn_stickiness_shapes[i].size.x) - dyn_stickiness_shapes[i].roundness;
        
//         let dd = smin(d, new_d, static_data.stickiness);

//         let coef = clamp((new_d - d) / (dd - d), 0.0, 1.0);

//         color = mix(color, dyn_stickiness_shapes[i].color, coef);

//         d = dd;
//     }
//     for (var i = dynamic_data.shapes_arrays_metadata.s_sph_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_sph_cubes_amount + dynamic_data.shapes_arrays_metadata.s_sph_cubes_start; i++) {
//         let new_d = sd_sph_box(p - dyn_stickiness_shapes[i].pos, dyn_stickiness_shapes[i].size) - dyn_stickiness_shapes[i].roundness;
        
//         let dd = smin(d, new_d, static_data.stickiness);

//         let coef = clamp((new_d - d) / (dd - d), 0.0, 1.0);

//         color = mix(color, dyn_stickiness_shapes[i].color, coef);

//         d = dd;
//     }
//     for (var i = dynamic_data.shapes_arrays_metadata.s_inf_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_inf_cubes_amount + dynamic_data.shapes_arrays_metadata.s_inf_cubes_start; i++) {
//         let new_d = sd_inf_box(p - dyn_stickiness_shapes[i].pos, dyn_stickiness_shapes[i].size.xyz) - dyn_stickiness_shapes[i].roundness;
        
//         let dd = smin(d, new_d, static_data.stickiness);

//         let coef = clamp((new_d - d) / (dd - d), 0.0, 1.0);

//         color = mix(color, dyn_stickiness_shapes[i].color, coef);

//         d = dd;
//     }

//     // static normal shapes
//     for (var i = static_data.shapes_arrays_metadata.cubes_start; i < static_data.shapes_arrays_metadata.cubes_amount + static_data.shapes_arrays_metadata.cubes_start; i++) {
//         let new_d = sd_box(p - normal_shapes[i].pos, normal_shapes[i].size) - normal_shapes[i].roundness;

//         if new_d < d {
//             color = normal_shapes[i].color;
//             d = new_d;
//         }
//     }
//     for (var i = static_data.shapes_arrays_metadata.spheres_start; i < static_data.shapes_arrays_metadata.spheres_amount + static_data.shapes_arrays_metadata.spheres_start; i++) {
//         let new_d = sd_sphere(p - normal_shapes[i].pos, normal_shapes[i].size.x) - normal_shapes[i].roundness;

//         if new_d < d {
//             color = normal_shapes[i].color;
//             d = new_d;
//         }
//     }
//     for (var i = static_data.shapes_arrays_metadata.sph_cubes_start; i < static_data.shapes_arrays_metadata.sph_cubes_amount + static_data.shapes_arrays_metadata.sph_cubes_start; i++) {
//         let new_d = sd_sph_box(p - normal_shapes[i].pos, normal_shapes[i].size) - normal_shapes[i].roundness;

//         if new_d < d {
//             color = normal_shapes[i].color;
//             d = new_d;
//         }
//     }
//     for (var i = static_data.shapes_arrays_metadata.inf_cubes_start; i < static_data.shapes_arrays_metadata.inf_cubes_amount + static_data.shapes_arrays_metadata.inf_cubes_start; i++) {
//         let new_d = sd_inf_box(p - normal_shapes[i].pos, normal_shapes[i].size.xyz) - normal_shapes[i].roundness;

//         if new_d < d {
//             color = normal_shapes[i].color;
//             d = new_d;
//         }
//     }

//     // dynamic normal shapes
//     for (var i = dynamic_data.shapes_arrays_metadata.cubes_start; i < dynamic_data.shapes_arrays_metadata.cubes_amount + dynamic_data.shapes_arrays_metadata.cubes_start; i++) {
//         let new_d = sd_box(p - dyn_normal_shapes[i].pos, dyn_normal_shapes[i].size) - dyn_normal_shapes[i].roundness;

//         if new_d < d {
//             color = dyn_normal_shapes[i].color;
//             d = new_d;
//         }
//     }
//     for (var i = dynamic_data.shapes_arrays_metadata.spheres_start; i < dynamic_data.shapes_arrays_metadata.spheres_amount + dynamic_data.shapes_arrays_metadata.spheres_start; i++) {
//         let new_d = sd_sphere(p - dyn_normal_shapes[i].pos, dyn_normal_shapes[i].size.x) - dyn_normal_shapes[i].roundness;

//         if new_d < d {
//             color = dyn_normal_shapes[i].color;
//             d = new_d;
//         }
//     }
//     for (var i = dynamic_data.shapes_arrays_metadata.sph_cubes_start; i < dynamic_data.shapes_arrays_metadata.sph_cubes_amount + dynamic_data.shapes_arrays_metadata.sph_cubes_start; i++) {
//         let new_d = sd_sph_box(p - dyn_normal_shapes[i].pos, dyn_normal_shapes[i].size) - dyn_normal_shapes[i].roundness;

//         if new_d < d {
//             color = dyn_normal_shapes[i].color;
//             d = new_d;
//         }
//     }
//     for (var i = dynamic_data.shapes_arrays_metadata.inf_cubes_start; i < dynamic_data.shapes_arrays_metadata.inf_cubes_amount + dynamic_data.shapes_arrays_metadata.inf_cubes_start; i++) {
//         let new_d = sd_inf_box(p - dyn_normal_shapes[i].pos, dyn_normal_shapes[i].size.xyz) - dyn_normal_shapes[i].roundness;

//         if new_d < d {
//             color = dyn_normal_shapes[i].color;
//             d = new_d;
//         }
//     }

//     if static_data.is_w_floor_exist == 1 {
//         if ray_w_rotated == 1 {
//             let new_d = p.w - static_data.w_floor + MIN_DIST;

//             if new_d < d {
//                 color = vec3(0.2,0.2,0.2);

//                 d = new_d;
//             }
//         }
//     }

//     if static_data.is_w_roof_exist == 1 {
//         if ray_w_rotated == 1 {
//             let new_d = static_data.w_roof - p.w - MIN_DIST;

//             if new_d < d {
//                 color = vec3(0.2,0.2,0.2);
//             }
//         }
//     }

//     if p.w > 0.0 {
//         let w_diff = clamp((1.0/p.w), 0.0, 1.0);

//         color = mix(vec3(0.41,0.21,0.0), color, w_diff);
//     } else if p.w < 0.0 {
//         let w_diff = clamp((-1.0/p.w), 0.0, 1.0);

//         color = mix(vec3(0.4,0.0,0.2), color, w_diff);
//     }

//     d = MIN_DIST + 0.003;

//     for (var i = 0u; i < dynamic_data.player_forms_amount; i++) {
//         var new_d = sd_sphere(p - dyn_player_forms[i].pos, dyn_player_forms[i].radius);
//         new_d = max(new_d, -sd_sphere(p - dyn_player_forms[i].pos, dyn_player_forms[i].radius * 0.86));
        
//         let rotated_p = dyn_player_forms[i].rotation * (p - dyn_player_forms[i].pos);
//         new_d = max(new_d, -sd_box(
//             rotated_p,
//             vec4(
//                 dyn_player_forms[i].radius * 0.18,
//                 dyn_player_forms[i].radius* 1.2,
//                 dyn_player_forms[i].radius* 1.2,
//                 dyn_player_forms[i].radius * 1.2
//             )));
        
//         new_d = max(
//             new_d,
//             -sd_sphere(
//                 rotated_p - vec4(0.0, 0.0, -dyn_player_forms[i].radius, 0.0),
//                 dyn_player_forms[i].radius * 0.53
//             )
//         );

//         if new_d < d {
//             d = new_d;

//             color = vec3(1.0);
//         }

//         new_d = sd_sphere(
//             p - dyn_player_forms[i].pos,
//             dyn_player_forms[i].radius * 0.6
//         );

//         new_d = max(
//             new_d,
//             -sd_sphere(
//                 rotated_p - vec4(0.0, 0.0, -dyn_player_forms[i].radius, 0.0)*0.6,
//                 dyn_player_forms[i].radius * 0.34
//             )
//         );

//         if new_d < d {
//             d = new_d;

//             color = dyn_player_forms[i].color;
//         }

//         new_d = sd_sphere(
//             rotated_p - dyn_player_forms[i].weapon_offset,
//             dyn_player_forms[i].radius * 0.286,
//         );

//         new_d = max(
//             new_d,
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

//         if new_d < d {
//             d = new_d;

//             color = vec3(1.0);
//         }

//         new_d = sd_capsule(
//             rotated_p,
//             dyn_player_forms[i].weapon_offset,
//             dyn_player_forms[i].weapon_offset -
//             vec4(
//                 0.0,
//                 0.0,
//                 dyn_player_forms[i].radius* 0.43,
//                 0.0
//             ),
//             dyn_player_forms[i].radius* 0.1
//         );

//         new_d = max(
//             new_d,
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
        

//         if new_d < d {
//             d = new_d;
//             color = dyn_player_forms[i].color;
//         }
//     }

    



//     return color;
// }



fn get_mat(
    cam_pos: vec4<f32>,
    ray_dir: vec4<f32>,
    dist: f32,
    in: ptr<function, Intersections>
) -> OutputMaterials {
    var output: OutputMaterials;

    if dist > MAX_DIST {
        output.materials_count = 1u;
        output.material_weights[0] = 1.0;
        output.materials[0] = -1;
        return output;
    }

    let p = cam_pos + ray_dir * dist;

    // intersected shapes metadata
    let ismda = (*in).ismd;
    output.materials_count = 0u;
    
    for (var j = ismda.player_forms_start; j < ismda.player_forms_amount + ismda.player_forms_start; j++) {
        
        let shape = dyn_player_forms[ (*in).ish[j] ];
        
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
            output.materials[0] = static_data.players_mat1;
            return output;
        }

        d = sd_sphere(
                p - shape.pos,
                shape.radius * 0.6
            );
        
        d = max(
            d,
            -sd_sphere(
                rotated_p - vec4(0.0, 0.0, -shape.radius, 0.0)*0.6,
                shape.radius * 0.34
            )
        );

        if d < MIN_DIST {
            output.materials_count = 1u;
            output.material_weights[0] = 1.0;
            output.materials[0] = static_data.players_mat2;
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
            output.materials[0] = static_data.players_mat1;
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
            output.materials[0] = static_data.players_mat2;
            return output;
        }
    }

    // static normal shapes
    for (var i =ismda.st_cubes_start; i < ismda.st_cubes_amount + ismda.st_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = normal_shapes[j];
        if sd_box(p - shape.pos, shape.size) - shape.roundness < MAX_DIST {
            output.materials_count = 1u;
            output.material_weights[0] = 1.0;
            output.materials[0] = shape.material;
            return output;
        }
    }
    for (var i = ismda.st_spheres_start; i < ismda.st_spheres_amount + ismda.st_spheres_start; i++) {
        let j = (*in).ish[i];
        let shape = normal_shapes[j];
        if sd_sphere(p - shape.pos, shape.size.x) - shape.roundness < MAX_DIST {
            output.materials_count = 1u;
            output.material_weights[0] = 1.0;
            output.materials[0] = shape.material;
            return output;
        }
    }
    for (var i = ismda.st_sph_cubes_start; i < ismda.st_sph_cubes_amount + ismda.st_sph_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = normal_shapes[j];
        if sd_sph_box(p - shape.pos, shape.size) - shape.roundness < MAX_DIST {
            output.materials_count = 1u;
            output.material_weights[0] = 1.0;
            output.materials[0] = shape.material;
            return output;
        }
    }
    for (var i = ismda.st_inf_cubes_start; i < ismda.st_inf_cubes_amount + ismda.st_inf_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = normal_shapes[j];
        if sd_inf_box(p - shape.pos, shape.size.xyz) - shape.roundness < MAX_DIST {
            output.materials_count = 1u;
            output.material_weights[0] = 1.0;
            output.materials[0] = shape.material;
            return output;
        }
    }

    // dynamic normal shapes
    for (var i = ismda.dyn_cubes_start; i < ismda.dyn_cubes_amount + ismda.dyn_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = dyn_normal_shapes[j];
        if sd_box(p - shape.pos, shape.size) - shape.roundness < MAX_DIST {
            output.materials_count = 1u;
            output.material_weights[0] = 1.0;
            output.materials[0] = shape.material;
            return output;
        }
    }
    for (var i = ismda.dyn_spheres_start; i < ismda.dyn_spheres_amount + ismda.dyn_spheres_start; i++) {
        let j = (*in).ish[i];
        let shape = dyn_normal_shapes[j];
        if sd_sphere(p - shape.pos, shape.size.x) - shape.roundness < MAX_DIST {
            output.materials_count = 1u;
            output.material_weights[0] = 1.0;
            output.materials[0] = shape.material;
            return output;
        }
    }
    for (var i = ismda.dyn_sph_cubes_start; i < ismda.dyn_sph_cubes_amount + ismda.dyn_sph_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = dyn_normal_shapes[j];
        if sd_sph_box(p - shape.pos, shape.size) - shape.roundness < MAX_DIST {
            output.materials_count = 1u;
            output.material_weights[0] = 1.0;
            output.materials[0] = shape.material;
            return output;
        }
    }
    for (var i = ismda.dyn_inf_cubes_start; i < ismda.dyn_inf_cubes_amount + ismda.dyn_inf_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = dyn_normal_shapes[j];
        if sd_inf_box(p - shape.pos, shape.size.xyz) - shape.roundness < MAX_DIST {
            output.materials_count = 1u;
            output.material_weights[0] = 1.0;
            output.materials[0] = shape.material;
            return output;
        }
    }

    // w_floor
    if static_data.is_w_floor_exist == 1 {
        if (*in).ray_w_rotated {
            if p.w - static_data.w_floor < MIN_DIST {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = static_data.w_cups_mat;
                return output;
            }
        }
    }

    
    var d = MAX_DIST * 2.0;
    // static stickiness shapes
    for (var i = ismda.st_s_cubes_start; i < ismda.st_s_cubes_amount + ismda.st_s_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = stickiness_shapes[j];
        let dd = sd_box(p - shape.pos, shape.size) - shape.roundness;
        
        if dd < MIN_DIST {
            output.materials_count = 1u;
            output.material_weights[0] = 1.0;
            output.materials[0] = shape.material;
            return output;
        }

        if dd < static_data.stickiness * STICKINESS_EFFECT_COEF {
            if output.materials_count == 0u {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = shape.material;
                d = dd;
            } else {
                let ddd = smin(d, dd, static_data.stickiness);
                let coef = clamp((dd - d) / (ddd - d), 0.0, 1.0);

                output.materials[output.materials_count] = shape.material;
                output.material_weights[output.materials_count] = coef;

                let mult = 1.0 - coef;

                for (var k = 0u; k < output.materials_count; k++) {
                    output.material_weights[k] *= mult;
                }

                output.materials_count += 1u;
                d = ddd;
            }
        }
    }
    for (var i = ismda.st_s_spheres_start; i < ismda.st_s_spheres_amount + ismda.st_s_spheres_start; i++) {
        let j = (*in).ish[i];
        let shape = stickiness_shapes[j];
        let dd = sd_sphere(p - shape.pos, shape.size.x) - shape.roundness;
        
        if dd < MIN_DIST {
            output.materials_count = 1u;
            output.material_weights[0] = 1.0;
            output.materials[0] = shape.material;
            return output;
        }

        if dd < static_data.stickiness * STICKINESS_EFFECT_COEF {
            if output.materials_count == 0u {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = shape.material;
                d = dd;
            } else {
                let ddd = smin(d, dd, static_data.stickiness);
                let coef = clamp((dd - d) / (ddd - d), 0.0, 1.0);

                output.materials[output.materials_count] = shape.material;
                output.material_weights[output.materials_count] = coef;

                let mult = 1.0 - coef;

                for (var k = 0u; k < output.materials_count; k++) {
                    output.material_weights[k] *= mult;
                }

                output.materials_count += 1u;
                d = ddd;
            }
        }
    }
    for (var i = ismda.st_s_sph_cubes_start; i < ismda.st_s_sph_cubes_amount + ismda.st_s_sph_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = stickiness_shapes[j];
        let dd = sd_sph_box(p - shape.pos, shape.size) - shape.roundness;
        
        if dd < MIN_DIST {
            output.materials_count = 1u;
            output.material_weights[0] = 1.0;
            output.materials[0] = shape.material;
            return output;
        }

        if dd < static_data.stickiness * STICKINESS_EFFECT_COEF {
            if output.materials_count == 0u {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = shape.material;
                d = dd;
            } else {
                let ddd = smin(d, dd, static_data.stickiness);
                let coef = clamp((dd - d) / (ddd - d), 0.0, 1.0);

                output.materials[output.materials_count] = shape.material;
                output.material_weights[output.materials_count] = coef;

                let mult = 1.0 - coef;

                for (var k = 0u; k < output.materials_count; k++) {
                    output.material_weights[k] *= mult;
                }

                output.materials_count += 1u;
                d = ddd;
            }
        }
    }
    for (var i = ismda.st_s_inf_cubes_start; i < ismda.st_s_inf_cubes_amount + ismda.st_s_inf_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = stickiness_shapes[j];
        let dd = sd_inf_box(p - shape.pos, shape.size.xyz) - shape.roundness;
        
        if dd < MIN_DIST {
            output.materials_count = 1u;
            output.material_weights[0] = 1.0;
            output.materials[0] = shape.material;
            return output;
        }

        if dd < static_data.stickiness * STICKINESS_EFFECT_COEF {
            if output.materials_count == 0u {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = shape.material;
                d = dd;
            } else {
                let ddd = smin(d, dd, static_data.stickiness);
                let coef = clamp((dd - d) / (ddd - d), 0.0, 1.0);

                output.materials[output.materials_count] = shape.material;
                output.material_weights[output.materials_count] = coef;

                let mult = 1.0 - coef;

                for (var k = 0u; k < output.materials_count; k++) {
                    output.material_weights[k] *= mult;
                }

                output.materials_count += 1u;
                d = ddd;
            }
        }
    }

    // dynamic stickiness
    for (var i = ismda.dyn_s_cubes_start; i < ismda.dyn_s_cubes_amount + ismda.dyn_s_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = dyn_stickiness_shapes[j];
        let dd = sd_box(p - shape.pos, shape.size) - shape.roundness;
        
        if dd < MIN_DIST {
            output.materials_count = 1u;
            output.material_weights[0] = 1.0;
            output.materials[0] = shape.material;
            return output;
        }

        if dd < static_data.stickiness * STICKINESS_EFFECT_COEF {
            if output.materials_count == 0u {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = shape.material;
                d = dd;
            } else {
                let ddd = smin(d, dd, static_data.stickiness);
                let coef = clamp((dd - d) / (ddd - d), 0.0, 1.0);

                output.materials[output.materials_count] = shape.material;
                output.material_weights[output.materials_count] = coef;

                let mult = 1.0 - coef;

                for (var k = 0u; k < output.materials_count; k++) {
                    output.material_weights[k] *= mult;
                }

                output.materials_count += 1u;
                d = ddd;
            }
        }
    }
    for (var i = ismda.dyn_s_spheres_start; i < ismda.dyn_s_spheres_amount + ismda.dyn_s_spheres_start; i++) {
        let j = (*in).ish[i];
        let shape = dyn_stickiness_shapes[j];
        let dd = sd_sphere(p - shape.pos, shape.size.x) - shape.roundness;
        
        if dd < MIN_DIST {
            output.materials_count = 1u;
            output.material_weights[0] = 1.0;
            output.materials[0] = shape.material;
            return output;
        }

        if dd < static_data.stickiness * STICKINESS_EFFECT_COEF {
            if output.materials_count == 0u {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = shape.material;
                d = dd;
            } else {
                let ddd = smin(d, dd, static_data.stickiness);
                let coef = clamp((dd - d) / (ddd - d), 0.0, 1.0);

                output.materials[output.materials_count] = shape.material;
                output.material_weights[output.materials_count] = coef;

                let mult = 1.0 - coef;

                for (var k = 0u; k < output.materials_count; k++) {
                    output.material_weights[k] *= mult;
                }

                output.materials_count += 1u;
                d = ddd;
            }
        }
    }
    for (var i = ismda.dyn_s_sph_cubes_start; i < ismda.dyn_s_sph_cubes_amount + ismda.dyn_s_sph_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = dyn_stickiness_shapes[j];
        let dd = sd_sph_box(p - shape.pos, shape.size) - shape.roundness;
        
        if dd < MIN_DIST {
            output.materials_count = 1u;
            output.material_weights[0] = 1.0;
            output.materials[0] = shape.material;
            return output;
        }

        if dd < static_data.stickiness * STICKINESS_EFFECT_COEF {
            if output.materials_count == 0u {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = shape.material;
                d = dd;
            } else {
                let ddd = smin(d, dd, static_data.stickiness);
                let coef = clamp((dd - d) / (ddd - d), 0.0, 1.0);

                output.materials[output.materials_count] = shape.material;
                output.material_weights[output.materials_count] = coef;

                let mult = 1.0 - coef;

                for (var k = 0u; k < output.materials_count; k++) {
                    output.material_weights[k] *= mult;
                }

                output.materials_count += 1u;
                d = ddd;
            }
        }
    }
    for (var i = ismda.dyn_s_inf_cubes_start; i < ismda.dyn_s_inf_cubes_amount + ismda.dyn_s_inf_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = dyn_stickiness_shapes[j];
        let dd = sd_inf_box(p - shape.pos, shape.size.xyz) - shape.roundness;
        
        if dd < MIN_DIST {
            output.materials_count = 1u;
            output.material_weights[0] = 1.0;
            output.materials[0] = shape.material;
            return output;
        }

        if dd < static_data.stickiness * STICKINESS_EFFECT_COEF {
            if output.materials_count == 0u {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = shape.material;
                d = dd;
            } else {
                let ddd = smin(d, dd, static_data.stickiness);
                let coef = clamp((dd - d) / (ddd - d), 0.0, 1.0);

                output.materials[output.materials_count] = shape.material;
                output.material_weights[output.materials_count] = coef;

                let mult = 1.0 - coef;

                for (var k = 0u; k < output.materials_count; k++) {
                    output.material_weights[k] *= mult;
                }

                output.materials_count += 1u;
                d = ddd;
            }
        }
    }
    
    if output.materials_count == 0u {
        output.materials_count = 1u;
        output.material_weights[0] = 1.0;
        output.materials[0] = -1;
    }

    return output;
}


fn map(p: vec4<f32>, in: ptr<function,Intersections>) -> f32 {
    var d = MAX_DIST;

    // intersected shapes metadata
    let ismda = (*in).ismd;

    // static stickiness shapes
    for (var i = ismda.st_s_cubes_start; i < ismda.st_s_cubes_amount + ismda.st_s_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = stickiness_shapes[j];
        d = smin(d, sd_box(p - shape.pos, shape.size) - shape.roundness, static_data.stickiness);
    }
    for (var i = ismda.st_s_spheres_start; i < ismda.st_s_spheres_amount + ismda.st_s_spheres_start; i++) {
        let j = (*in).ish[i];
        let shape = stickiness_shapes[j];
        d = smin(d, sd_sphere(p - shape.pos, shape.size.x) - shape.roundness, static_data.stickiness);
    }
    for (var i = ismda.st_s_sph_cubes_start; i < ismda.st_s_sph_cubes_amount + ismda.st_s_sph_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = stickiness_shapes[j];
        d = smin(d, sd_sph_box(p - shape.pos, shape.size) - shape.roundness, static_data.stickiness);
    }
    for (var i = ismda.st_s_inf_cubes_start; i < ismda.st_s_inf_cubes_amount + ismda.st_s_inf_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = stickiness_shapes[j];
        d = smin(d, sd_inf_box(p - shape.pos, shape.size.xyz) - shape.roundness, static_data.stickiness);
    }

    // dynamic stickiness
    for (var i = ismda.dyn_s_cubes_start; i < ismda.dyn_s_cubes_amount + ismda.dyn_s_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = dyn_stickiness_shapes[j];
        d = smin(d, sd_box(p - shape.pos, shape.size) - shape.roundness, static_data.stickiness);
    }
    for (var i = ismda.dyn_s_spheres_start; i < ismda.dyn_s_spheres_amount + ismda.dyn_s_spheres_start; i++) {
        let j = (*in).ish[i];
        let shape = dyn_stickiness_shapes[j];
        d = smin(d, sd_sphere(p - shape.pos, shape.size.x) - shape.roundness, static_data.stickiness);
    }
    for (var i = ismda.dyn_s_sph_cubes_start; i < ismda.dyn_s_sph_cubes_amount + ismda.dyn_s_sph_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = dyn_stickiness_shapes[j];
        d = smin(d, sd_sph_box(p - shape.pos, shape.size) - shape.roundness, static_data.stickiness);
    }
    for (var i = ismda.dyn_s_inf_cubes_start; i < ismda.dyn_s_inf_cubes_amount + ismda.dyn_s_inf_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = dyn_stickiness_shapes[j];
        d = smin(d, sd_inf_box(p - shape.pos, shape.size.xyz) - shape.roundness, static_data.stickiness);
    }


    // static normal shapes
    for (var i =ismda.st_cubes_start; i < ismda.st_cubes_amount + ismda.st_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = normal_shapes[j];
        d = min(d, sd_box(p - shape.pos, shape.size) - shape.roundness);
    }
    for (var i = ismda.st_spheres_start; i < ismda.st_spheres_amount + ismda.st_spheres_start; i++) {
        let j = (*in).ish[i];
        let shape = normal_shapes[j];
        d = min(d, sd_sphere(p - shape.pos, shape.size.x) - shape.roundness);
    }
    for (var i = ismda.st_sph_cubes_start; i < ismda.st_sph_cubes_amount + ismda.st_sph_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = normal_shapes[j];
        d = min(d, sd_sph_box(p - shape.pos, shape.size) - shape.roundness);
    }
    for (var i = ismda.st_inf_cubes_start; i < ismda.st_inf_cubes_amount + ismda.st_inf_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = normal_shapes[j];
        d = min(d, sd_inf_box(p - shape.pos, shape.size.xyz) - shape.roundness);
    }

    // dynamic normal shapes
    for (var i = ismda.dyn_cubes_start; i < ismda.dyn_cubes_amount + ismda.dyn_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = dyn_normal_shapes[j];
        d = min(d, sd_box(p - shape.pos, shape.size) - shape.roundness);
    }
    for (var i = ismda.dyn_spheres_start; i < ismda.dyn_spheres_amount + ismda.dyn_spheres_start; i++) {
        let j = (*in).ish[i];
        let shape = dyn_normal_shapes[j];
        d = min(d, sd_sphere(p - shape.pos, shape.size.x) - shape.roundness);
    }
    for (var i = ismda.dyn_sph_cubes_start; i < ismda.dyn_sph_cubes_amount + ismda.dyn_sph_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = dyn_normal_shapes[j];
        d = min(d, sd_sph_box(p - shape.pos, shape.size) - shape.roundness);
    }
    for (var i = ismda.dyn_inf_cubes_start; i < ismda.dyn_inf_cubes_amount + ismda.dyn_inf_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = dyn_normal_shapes[j];
        d = min(d, sd_inf_box(p - shape.pos, shape.size.xyz) - shape.roundness);
    }

    // static negative stickiness shapes
    var dd = MAX_DIST;

    for (var i = ismda.st_s_neg_cubes_start; i < ismda.st_s_neg_cubes_amount + ismda.st_s_neg_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = neg_stickiness_shapes[j];
        dd = smin(dd, sd_box(p - shape.pos, shape.size) - shape.roundness, static_data.stickiness);
    }
    for (var i = ismda.st_s_neg_spheres_start; i < ismda.st_s_neg_spheres_amount + ismda.st_s_neg_spheres_start; i++) {
        let j = (*in).ish[i];
        let shape = neg_stickiness_shapes[j];
        dd = smin(dd, sd_sphere(p - shape.pos, shape.size.x) - shape.roundness, static_data.stickiness);
    }
    for (var i = ismda.st_s_neg_sph_cubes_start; i < ismda.st_s_neg_sph_cubes_amount + ismda.st_s_neg_sph_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = neg_stickiness_shapes[j];
        dd = smin(dd, sd_sph_box(p - shape.pos, shape.size) - shape.roundness, static_data.stickiness);
    }
    for (var i = ismda.st_s_neg_inf_cubes_start; i < ismda.st_s_neg_inf_cubes_amount + ismda.st_s_neg_inf_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = neg_stickiness_shapes[j];
        dd = smin(dd, sd_inf_box(p - shape.pos, shape.size.xyz) - shape.roundness, static_data.stickiness);
    }
    d = max(d, -dd);

    // dynamic negative stickiness shapes
    var ddd = dd;

    for (var i = ismda.dyn_s_neg_cubes_start; i < ismda.dyn_s_neg_cubes_amount + ismda.dyn_s_neg_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = dyn_neg_stickiness_shapes[j];
        ddd = smin(ddd, sd_box(p - shape.pos, shape.size) - shape.roundness, static_data.stickiness);
    }
    for (var i = ismda.dyn_s_neg_spheres_start; i < ismda.dyn_s_neg_spheres_amount + ismda.dyn_s_neg_spheres_start; i++) {
        let j = (*in).ish[i];
        let shape = dyn_neg_stickiness_shapes[j];
        ddd = smin(ddd, sd_sphere(p - shape.pos, shape.size.x) - shape.roundness, static_data.stickiness);
    }
    for (var i = ismda.dyn_s_neg_sph_cubes_start; i < ismda.dyn_s_neg_sph_cubes_amount + ismda.dyn_s_neg_sph_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = dyn_neg_stickiness_shapes[j];
        ddd = smin(ddd, sd_sph_box(p - shape.pos, shape.size) - shape.roundness, static_data.stickiness);
    }
    for (var i = ismda.dyn_s_neg_inf_cubes_start; i < ismda.dyn_s_neg_inf_cubes_amount + ismda.dyn_s_neg_inf_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = dyn_neg_stickiness_shapes[j];
        ddd = smin(ddd, sd_inf_box(p - shape.pos, shape.size.xyz) - shape.roundness, static_data.stickiness);
    }
    d = max(d, -ddd);

    // static negative shapes
    for (var i = ismda.st_neg_cubes_start; i < ismda.st_neg_cubes_amount + ismda.st_neg_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = negatives_shapes[j];
        d = max(d, -(sd_box(p - shape.pos, shape.size) - shape.roundness));
    }
    for (var i = ismda.st_neg_spheres_start; i < ismda.st_neg_spheres_amount + ismda.st_neg_spheres_start; i++) {
        let j = (*in).ish[i];
        let shape = negatives_shapes[j];
        d = max(d, -(sd_sphere(p - shape.pos, shape.size.x) - shape.roundness));
    }
    for (var i = ismda.st_neg_sph_cubes_start; i < ismda.st_neg_sph_cubes_amount + ismda.st_neg_sph_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = negatives_shapes[j];
        d = max(d, -(sd_sph_box(p - shape.pos, shape.size) - shape.roundness));
    }
    for (var i = ismda.st_neg_inf_cubes_start; i < ismda.st_neg_inf_cubes_amount + ismda.st_neg_inf_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = negatives_shapes[j];
        d = max(d, -(sd_inf_box(p - shape.pos, shape.size.xyz) - shape.roundness));
    }

    // dynamic negative shapes
    for (var i = ismda.dyn_neg_cubes_start; i < ismda.dyn_neg_cubes_amount + ismda.dyn_neg_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = dyn_negatives_shapes[j];
        d = max(d, -(sd_box(p - shape.pos, shape.size) - shape.roundness));
    }
    for (var i = ismda.dyn_neg_spheres_start; i < ismda.dyn_neg_spheres_amount + ismda.dyn_neg_spheres_start; i++) {
        let j = (*in).ish[i];
        let shape = dyn_negatives_shapes[j];
        d = max(d, -(sd_sphere(p - shape.pos, shape.size.x) - shape.roundness));
    }
    for (var i = ismda.dyn_neg_sph_cubes_start; i < ismda.dyn_neg_sph_cubes_amount + ismda.dyn_neg_sph_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = dyn_negatives_shapes[j];
        d = max(d, -(sd_sph_box(p - shape.pos, shape.size) - shape.roundness));
    }
    for (var i = ismda.dyn_neg_inf_cubes_start; i < ismda.dyn_neg_inf_cubes_amount + ismda.dyn_neg_inf_cubes_start; i++) {
        let j = (*in).ish[i];
        let shape = dyn_negatives_shapes[j];
        d = max(d, -(sd_inf_box(p - shape.pos, shape.size.xyz) - shape.roundness));
    }

    var dddd = MAX_DIST;
    for (var j = ismda.player_forms_start; j < ismda.player_forms_amount + ismda.player_forms_start; j++) {
        
        let i = (*in).ish[j];
        let shape = dyn_player_forms[i];
        
        dddd = min(dddd, sd_sphere(p - shape.pos, shape.radius));
        dddd = max(dddd, -sd_sphere(p - shape.pos, shape.radius * 0.86));
        
        let rotated_p = shape.rotation * (p - shape.pos);
        dddd = max(dddd, -sd_box(
            rotated_p,
            vec4(
                shape.radius * 0.18,
                shape.radius* 1.2,
                shape.radius* 1.2,
                shape.radius * 1.2
            )));
        
        dddd = max(
            dddd,
            -sd_sphere(
                rotated_p - vec4(0.0, 0.0, -shape.radius, 0.0),
                shape.radius * 0.53
            )
        );

        dddd = min(
            dddd,
            sd_sphere(
                p - shape.pos,
                shape.radius * 0.6
            )
        );
        dddd = max(
            dddd,
            -sd_sphere(
                rotated_p - vec4(0.0, 0.0, -shape.radius, 0.0)*0.6,
                shape.radius * 0.34
            )
        );

        dddd = min(
            dddd,
            sd_sphere(
                rotated_p - shape.weapon_offset,
                shape.radius * 0.286,
            )
        );

        dddd = max(
            dddd,
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

        dddd = min(
            dddd,
            sd_capsule(
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
            )
        );

        dddd = max(
            dddd,
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
        

    }

    d = min(d, dddd);
    
    if static_data.is_w_floor_exist == 1 {
        if (*in).ray_w_rotated {
            d = min(d, p.w - static_data.w_floor);
        }
    }

    return d;
}

fn get_normal(p: vec4<f32>, in: ptr<function,Intersections>) -> vec4<f32> {
    var h: vec3<f32> = vec3<f32>(0.001, -0.001, 0.0);
    
    var a: vec4<f32> = p + h.yxxz;
    var b: vec4<f32> = p + h.xyxz;
    var c: vec4<f32> = p + h.xxyz;
    var d: vec4<f32> = p + h.yyyz;
    var e: vec4<f32> = p + h.zzzx;
    var f: vec4<f32> = p + h.zzzy;

    var fa: f32 = map(a, in);
    var fb: f32 = map(b, in);
    var fc: f32 = map(c, in);
    var fd: f32 = map(d, in);
    var fe: f32 = map(e, in);
    var ff: f32 = map(f, in);

    return normalize(
        h.yxxz * fa +
        h.xyxz * fb +
        h.xxyz * fc +
        h.yyyz * fd +
        h.zzzx * fe +
        h.zzzy * ff
    );
}

const MIN_STEP: f32 = 0.005;

fn ray_march(ray_origin_base: vec4<f32>, ray_direction: vec4<f32>, in: ptr<function,Intersections>) -> vec2<f32>  {
    
    if (*in).offset > MAX_DIST {
        return vec2(MAX_DIST, 0.0);
    }

    var total_distance: f32 = (*in).offset;
    
    var ray_origin = ray_origin_base + (ray_direction * (*in).offset);

    var i: i32 = 0;
    for (; i < MAX_STEPS; i++) {
        var d: f32  = map(ray_origin, in);
        total_distance += d;

        if (d < 0.) {
            // color.z = 1.;
            return vec2<f32>(total_distance, f32(i));
        }
        if (d < MIN_DIST) {
            // color.x = 1.;
            return vec2<f32>(total_distance, f32(i));
        }
        if (total_distance > MAX_DIST) {
            // color.y = 1.;
            return vec2<f32>(MAX_DIST, f32(i));
        }

        ray_origin += ray_direction * d;
    }
    return vec2<f32>(total_distance, f32(i));
}


fn add_w_scnner_color(pos: vec4<f32>, dist: f32, dir: vec4<f32>) -> vec3<f32> {
    var scanner_color = vec3(0.0);
    
    if dist > dynamic_data.w_scaner_radius {

        let y_coof = clamp(pow(1.0 - dir.y,3.0), 0.0, 1.0);

        scanner_color = vec3(0.4 * y_coof);
    }

    scanner_color += clamp(pow(1.0 - abs(dist - dynamic_data.w_scaner_radius), 5.0), 0.0, 1.0);

    scanner_color *= dynamic_data.w_scaner_intesity;

    for (var i = 0u; i < dynamic_data.player_forms_amount; i++) {

        let d = sd_sphere(pos - dyn_player_forms[i].pos, dyn_player_forms[i].radius);

        let visible = clamp((dynamic_data.w_scaner_radius - d) * 5.0, 0.0, 1.0);

        let vis_d = length(
            (
                (
                    pos + dir * min(
                        dynamic_data.w_scaner_radius,
                        length(pos.xyz - dyn_player_forms[i].pos.xyz)
                    )
                ) - dyn_player_forms[i].pos
            ).xyz
        ) - dyn_player_forms[i].radius;

        var red = pow(clamp((1.0 - abs(vis_d*10.0)), 0.0, 1.0), 2.0) * visible;
        red += pow((clamp(-vis_d * 2.5, 0.0, 1.0)), 2.0) * visible;
        red *= dynamic_data.w_scaner_intesity * 2.0;
        
        scanner_color.r += red;
    }
    
    return clamp(scanner_color, vec3(0.0), vec3(1.0));
}


fn cube_intersection( ro: vec4<f32>, rd: vec4<f32>, size: vec4<f32>) -> vec2<f32> {  // can precompute if traversing a set of aligned boxes
    let m = 1.0/rd;
    let n = m*ro;
    let k = abs(m)*size;
    let t1 = -n - k;
    let t2 = -n + k;
    let tN = max( max( max( t1.x, t1.y ), t1.z ), t1.w);
    let tF = min( min( min( t2.x, t2.y ), t2.z ), t2.w);
    if( tN>tF || tF<0.0) {
        return vec2(-1.0);
    } // no intersection
    return vec2( tN, tF );
}

fn inf_cube_intersection( ro: vec4<f32>, rd: vec4<f32>, size: vec3<f32>) -> vec2<f32> {  // can precompute if traversing a set of aligned boxes
    let m = 1.0/rd;
    let n = m*ro;
    let k = abs(m.xyz)*size;
    let t1 = -n.xyz - k.xyz;
    let t2 = -n.xyz + k.xyz;
    let tN = max( max( t1.x, t1.y ), t1.z );
    let tF = min( min( t2.x, t2.y ), t2.z );
    if( tN>tF || tF<0.0) {
        return vec2(-1.0);
    } // no intersection
    return vec2( tN, tF );
}

fn sph_intersection( ro: vec4<f32>, rd: vec4<f32>, ra: f32) -> vec2<f32> {  // can precompute if traversing a set of aligned boxes
    let b = dot( ro, rd );
    let c = dot( ro, ro ) - ra*ra;
    var h = b*b - c;
    if( h<0.0 ) {
        return vec2(-1.0);
    } // no intersection
    h = sqrt( h );
    return vec2( -b-h, -b+h );
}


fn find_intersections(ro: vec4<f32>, rd: vec4<f32>) -> Intersections {
    
    var in: Intersections;

    var offset: f32 = MAX_DIST * 2.0;
    var ish_index = 0u;
    // static stickiness shapes

    in.ismd.st_s_cubes_start = ish_index;

    for (var i = static_data.shapes_arrays_metadata.s_cubes_start; i < static_data.shapes_arrays_metadata.s_cubes_amount + static_data.shapes_arrays_metadata.s_cubes_start; i++) {
        let intr = cube_intersection(
            ro - stickiness_shapes[i].pos,
            rd,
            stickiness_shapes[i].size + stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            in.ismd.st_s_cubes_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;

            
            offset = min(offset, intr.x);
        }
    }

    in.ismd.st_s_spheres_start = ish_index;

    for (var i = static_data.shapes_arrays_metadata.s_spheres_start; i < static_data.shapes_arrays_metadata.s_spheres_amount + static_data.shapes_arrays_metadata.s_spheres_start; i++) {
        let intr = sph_intersection(
            ro - stickiness_shapes[i].pos,
            rd,
            stickiness_shapes[i].size.x + stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            in.ismd.st_s_spheres_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;

            
            offset = min(offset, intr.x);
        }
    }

    in.ismd.st_s_sph_cubes_start = ish_index;

    for (var i = static_data.shapes_arrays_metadata.s_sph_cubes_start; i < static_data.shapes_arrays_metadata.s_sph_cubes_amount + static_data.shapes_arrays_metadata.s_sph_cubes_start; i++) {
        let intr = cube_intersection(
            ro - stickiness_shapes[i].pos,
            rd,
            stickiness_shapes[i].size + stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            in.ismd.st_s_sph_cubes_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;

            
            offset = min(offset, intr.x);
        }
    }

    in.ismd.st_s_inf_cubes_start = ish_index;

    for (var i = static_data.shapes_arrays_metadata.s_inf_cubes_start; i < static_data.shapes_arrays_metadata.s_inf_cubes_amount + static_data.shapes_arrays_metadata.s_inf_cubes_start; i++) {
        let intr = inf_cube_intersection(
            ro - stickiness_shapes[i].pos,
            rd,
            stickiness_shapes[i].size.xyz + stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            in.ismd.st_s_inf_cubes_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;

            
            offset = min(offset, intr.x);
        }
    }
    

    // dynamic stickiness

    in.ismd.dyn_s_cubes_start = ish_index;

    for (var i = dynamic_data.shapes_arrays_metadata.s_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_cubes_amount + dynamic_data.shapes_arrays_metadata.s_cubes_start; i++) {
        let intr = cube_intersection(
            ro - dyn_stickiness_shapes[i].pos,
            rd,
            dyn_stickiness_shapes[i].size + dyn_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            in.ismd.dyn_s_cubes_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;

            
            offset = min(offset, intr.x);
        }
    }

    in.ismd.dyn_s_spheres_start = ish_index;

    for (var i = dynamic_data.shapes_arrays_metadata.s_spheres_start; i < dynamic_data.shapes_arrays_metadata.s_spheres_amount + dynamic_data.shapes_arrays_metadata.s_spheres_start; i++) {
        let intr = sph_intersection(
            ro - dyn_stickiness_shapes[i].pos,
            rd,
            dyn_stickiness_shapes[i].size.x + dyn_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            in.ismd.dyn_s_spheres_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;

            
            offset = min(offset, intr.x);
        }
    }

    in.ismd.dyn_s_sph_cubes_start = ish_index;

    for (var i = dynamic_data.shapes_arrays_metadata.s_sph_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_sph_cubes_amount + dynamic_data.shapes_arrays_metadata.s_sph_cubes_start; i++) {
        let intr = cube_intersection(
            ro - dyn_stickiness_shapes[i].pos,
            rd,
            dyn_stickiness_shapes[i].size + dyn_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            in.ismd.dyn_s_sph_cubes_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;

            
            offset = min(offset, intr.x);
        }
    }

    in.ismd.dyn_s_inf_cubes_start = ish_index;

    for (var i = dynamic_data.shapes_arrays_metadata.s_inf_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_inf_cubes_amount + dynamic_data.shapes_arrays_metadata.s_inf_cubes_start; i++) {
        let intr = inf_cube_intersection(
            ro - dyn_stickiness_shapes[i].pos,
            rd,
            dyn_stickiness_shapes[i].size.xyz + dyn_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            in.ismd.dyn_s_inf_cubes_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;

            if intr.x >= 0.0 {
                offset = min(offset, intr.x);
            }
        }
    }


    // static normal shapes
    in.ismd.st_cubes_start = ish_index;

    for (var i = static_data.shapes_arrays_metadata.cubes_start; i < static_data.shapes_arrays_metadata.cubes_amount + static_data.shapes_arrays_metadata.cubes_start; i++) {
        let intr = cube_intersection(
            ro - normal_shapes[i].pos,
            rd,
            normal_shapes[i].size + normal_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            in.ismd.st_cubes_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;

            
            offset = min(offset, intr.x);
        }
    }

    in.ismd.st_spheres_start = ish_index;

    for (var i = static_data.shapes_arrays_metadata.spheres_start; i < static_data.shapes_arrays_metadata.spheres_amount + static_data.shapes_arrays_metadata.spheres_start; i++) {
        let intr = sph_intersection(
            ro - normal_shapes[i].pos,
            rd,
            normal_shapes[i].size.x + normal_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            in.ismd.st_spheres_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;

            
            offset = min(offset, intr.x);
        }
    }

    in.ismd.st_sph_cubes_start = ish_index;

    for (var i = static_data.shapes_arrays_metadata.sph_cubes_start; i < static_data.shapes_arrays_metadata.sph_cubes_amount + static_data.shapes_arrays_metadata.sph_cubes_start; i++) {
        let intr = cube_intersection(
            ro - normal_shapes[i].pos,
            rd,
            normal_shapes[i].size + normal_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            in.ismd.st_sph_cubes_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;

            
            offset = min(offset, intr.x);
        }
    }

    in.ismd.st_inf_cubes_start = ish_index;

    for (var i = static_data.shapes_arrays_metadata.inf_cubes_start; i < static_data.shapes_arrays_metadata.inf_cubes_amount + static_data.shapes_arrays_metadata.inf_cubes_start; i++) {
        let intr = inf_cube_intersection(
            ro - normal_shapes[i].pos,
            rd,
            normal_shapes[i].size.xyz + normal_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            in.ismd.st_inf_cubes_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;

            
            offset = min(offset, intr.x);
        }
    }

    // dynamic normal shapes

    in.ismd.dyn_cubes_start = ish_index;

    for (var i = dynamic_data.shapes_arrays_metadata.cubes_start; i < dynamic_data.shapes_arrays_metadata.cubes_amount + dynamic_data.shapes_arrays_metadata.cubes_start; i++) {
        let intr = cube_intersection(
            ro - dyn_normal_shapes[i].pos,
            rd,
            dyn_normal_shapes[i].size + dyn_normal_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            in.ismd.dyn_cubes_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;

            
            offset = min(offset, intr.x);
        }
    }

    in.ismd.dyn_spheres_start = ish_index;

    for (var i = dynamic_data.shapes_arrays_metadata.spheres_start; i < dynamic_data.shapes_arrays_metadata.spheres_amount + dynamic_data.shapes_arrays_metadata.spheres_start; i++) {
        let intr = sph_intersection(
            ro - dyn_normal_shapes[i].pos,
            rd,
            dyn_normal_shapes[i].size.x + dyn_normal_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            in.ismd.dyn_spheres_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;

            
            offset = min(offset, intr.x);
        }
    }

    in.ismd.dyn_sph_cubes_start = ish_index;

    for (var i = dynamic_data.shapes_arrays_metadata.sph_cubes_start; i < dynamic_data.shapes_arrays_metadata.sph_cubes_amount + dynamic_data.shapes_arrays_metadata.sph_cubes_start; i++) {
        let intr = cube_intersection(
            ro - dyn_normal_shapes[i].pos,
            rd,
            dyn_normal_shapes[i].size + dyn_normal_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            in.ismd.dyn_sph_cubes_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;

            
            offset = min(offset, intr.x);
        }
    }

    in.ismd.dyn_inf_cubes_start = ish_index;

    for (var i = dynamic_data.shapes_arrays_metadata.inf_cubes_start; i < dynamic_data.shapes_arrays_metadata.inf_cubes_amount + dynamic_data.shapes_arrays_metadata.inf_cubes_start; i++) {
        let intr = inf_cube_intersection(
            ro - dyn_normal_shapes[i].pos,
            rd,
            dyn_normal_shapes[i].size.xyz + dyn_normal_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            in.ismd.dyn_inf_cubes_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;

            
            offset = min(offset, intr.x);
        }
    }

    // static negative stickiness shapes
    in.ismd.st_s_neg_cubes_start = ish_index;

    for (var i = static_data.shapes_arrays_metadata.s_neg_cubes_start; i < static_data.shapes_arrays_metadata.s_neg_cubes_amount + static_data.shapes_arrays_metadata.s_neg_cubes_start; i++) {
        let intr = cube_intersection(
            ro - neg_stickiness_shapes[i].pos,
            rd,
            neg_stickiness_shapes[i].size + neg_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            in.ismd.st_s_neg_cubes_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;
        }
    }

    in.ismd.st_s_neg_spheres_start = ish_index;

    for (var i = static_data.shapes_arrays_metadata.s_neg_spheres_start; i < static_data.shapes_arrays_metadata.s_neg_spheres_amount + static_data.shapes_arrays_metadata.s_neg_spheres_start; i++) {
        let intr = sph_intersection(
            ro - neg_stickiness_shapes[i].pos,
            rd,
            neg_stickiness_shapes[i].size.x + neg_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            in.ismd.st_s_neg_spheres_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;
        }
    }

    in.ismd.st_s_neg_sph_cubes_start = ish_index;

    for (var i = static_data.shapes_arrays_metadata.s_neg_sph_cubes_start; i < static_data.shapes_arrays_metadata.s_neg_sph_cubes_amount + static_data.shapes_arrays_metadata.s_neg_sph_cubes_start; i++) {
        let intr = cube_intersection(
            ro - neg_stickiness_shapes[i].pos,
            rd,
            neg_stickiness_shapes[i].size + neg_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            in.ismd.st_s_neg_sph_cubes_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;
        }
    }

    in.ismd.st_s_neg_inf_cubes_start = ish_index;

    for (var i = static_data.shapes_arrays_metadata.s_neg_inf_cubes_start; i < static_data.shapes_arrays_metadata.s_neg_inf_cubes_amount + static_data.shapes_arrays_metadata.s_neg_inf_cubes_start; i++) {
        let intr = inf_cube_intersection(
            ro - neg_stickiness_shapes[i].pos,
            rd,
            neg_stickiness_shapes[i].size.xyz + neg_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            in.ismd.st_s_neg_inf_cubes_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;
        }
    }

    // dynamic negative stickiness shapes
    in.ismd.dyn_s_neg_cubes_start = ish_index;

    for (var i = dynamic_data.shapes_arrays_metadata.s_neg_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_neg_cubes_amount + dynamic_data.shapes_arrays_metadata.s_neg_cubes_start; i++) {
        let intr = cube_intersection(
            ro - dyn_neg_stickiness_shapes[i].pos,
            rd,
            dyn_neg_stickiness_shapes[i].size + dyn_neg_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            in.ismd.dyn_s_neg_cubes_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;
        }
    }

    in.ismd.dyn_s_neg_spheres_start = ish_index;

    for (var i = dynamic_data.shapes_arrays_metadata.s_neg_spheres_start; i < dynamic_data.shapes_arrays_metadata.s_neg_spheres_amount + dynamic_data.shapes_arrays_metadata.s_neg_spheres_start; i++) {
        let intr = sph_intersection(
            ro - dyn_neg_stickiness_shapes[i].pos,
            rd,
            dyn_neg_stickiness_shapes[i].size.x + dyn_neg_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            in.ismd.dyn_s_neg_spheres_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;
        }
    }

    in.ismd.dyn_s_neg_sph_cubes_start = ish_index;

    for (var i = dynamic_data.shapes_arrays_metadata.s_neg_sph_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_neg_sph_cubes_amount + dynamic_data.shapes_arrays_metadata.s_neg_sph_cubes_start; i++) {
        let intr = cube_intersection(
            ro - dyn_neg_stickiness_shapes[i].pos,
            rd,
            dyn_neg_stickiness_shapes[i].size + dyn_neg_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            in.ismd.dyn_s_neg_sph_cubes_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;
        }
    }

    in.ismd.dyn_s_neg_inf_cubes_start = ish_index;

    for (var i = dynamic_data.shapes_arrays_metadata.s_neg_inf_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_neg_inf_cubes_amount + dynamic_data.shapes_arrays_metadata.s_neg_inf_cubes_start; i++) {
        let intr = inf_cube_intersection(
            ro - dyn_neg_stickiness_shapes[i].pos,
            rd,
            dyn_neg_stickiness_shapes[i].size.xyz + dyn_neg_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            in.ismd.dyn_s_neg_inf_cubes_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;
        }
    }

    // static negative shapes

    in.ismd.st_neg_cubes_start = ish_index;

    for (var i = static_data.shapes_arrays_metadata.neg_cubes_start; i < static_data.shapes_arrays_metadata.neg_cubes_amount + static_data.shapes_arrays_metadata.neg_cubes_start; i++) {
        let intr = cube_intersection(
            ro - negatives_shapes[i].pos,
            rd,
            negatives_shapes[i].size + negatives_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            in.ismd.st_neg_cubes_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;
        }
    }

    in.ismd.st_neg_spheres_start = ish_index;

    for (var i = static_data.shapes_arrays_metadata.neg_spheres_start; i < static_data.shapes_arrays_metadata.neg_spheres_amount + static_data.shapes_arrays_metadata.neg_spheres_start; i++) {
        let intr = sph_intersection(
            ro - negatives_shapes[i].pos,
            rd,
            negatives_shapes[i].size.x + negatives_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            in.ismd.st_neg_spheres_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;
        }
    }

    in.ismd.st_neg_sph_cubes_start = ish_index;

    for (var i = static_data.shapes_arrays_metadata.neg_sph_cubes_start; i < static_data.shapes_arrays_metadata.neg_sph_cubes_amount + static_data.shapes_arrays_metadata.neg_sph_cubes_start; i++) {
        let intr = cube_intersection(
            ro - negatives_shapes[i].pos,
            rd,
            negatives_shapes[i].size + negatives_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            in.ismd.st_neg_sph_cubes_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;
        }
    }

    in.ismd.st_neg_inf_cubes_start = ish_index;

    for (var i = static_data.shapes_arrays_metadata.neg_inf_cubes_start; i < static_data.shapes_arrays_metadata.neg_inf_cubes_amount + static_data.shapes_arrays_metadata.neg_inf_cubes_start; i++) {
        let intr = inf_cube_intersection(
            ro - negatives_shapes[i].pos,
            rd,
            negatives_shapes[i].size.xyz + negatives_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            in.ismd.st_neg_inf_cubes_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;
        }
    }

    // dynamic negative shapes

    in.ismd.dyn_neg_cubes_start = ish_index;

    for (var i = dynamic_data.shapes_arrays_metadata.neg_cubes_start; i < dynamic_data.shapes_arrays_metadata.neg_cubes_amount + dynamic_data.shapes_arrays_metadata.neg_cubes_start; i++) {
        let intr = cube_intersection(
            ro - dyn_negatives_shapes[i].pos,
            rd,
            dyn_negatives_shapes[i].size + dyn_negatives_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            in.ismd.dyn_neg_cubes_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;
        }
    }

    in.ismd.dyn_neg_spheres_start = ish_index;

    for (var i = dynamic_data.shapes_arrays_metadata.neg_spheres_start; i < dynamic_data.shapes_arrays_metadata.neg_spheres_amount + dynamic_data.shapes_arrays_metadata.neg_spheres_start; i++) {
        let intr = sph_intersection(
            ro - dyn_negatives_shapes[i].pos,
            rd,
            dyn_negatives_shapes[i].size.x + dyn_negatives_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            in.ismd.dyn_neg_spheres_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;
        }
    }

    in.ismd.dyn_neg_sph_cubes_start = ish_index;

    for (var i = dynamic_data.shapes_arrays_metadata.neg_sph_cubes_start; i < dynamic_data.shapes_arrays_metadata.neg_sph_cubes_amount + dynamic_data.shapes_arrays_metadata.neg_sph_cubes_start; i++) {
        let intr = cube_intersection(
            ro - dyn_negatives_shapes[i].pos,
            rd,
            dyn_negatives_shapes[i].size + dyn_negatives_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            in.ismd.dyn_neg_sph_cubes_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;
        }
    }

    in.ismd.dyn_neg_inf_cubes_start = ish_index;

    for (var i = dynamic_data.shapes_arrays_metadata.neg_inf_cubes_start; i < dynamic_data.shapes_arrays_metadata.neg_inf_cubes_amount + dynamic_data.shapes_arrays_metadata.neg_inf_cubes_start; i++) {
        let intr = inf_cube_intersection(
            ro - dyn_negatives_shapes[i].pos,
            rd,
            dyn_negatives_shapes[i].size.xyz + dyn_negatives_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            in.ismd.dyn_neg_inf_cubes_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;
        }
    }


    // player forms

    in.ismd.player_forms_start = ish_index;

    for (var i = 0u; i < dynamic_data.player_forms_amount; i++) {
        let intr = sph_intersection(
            ro - dyn_player_forms[i].pos,
            rd,
            dyn_player_forms[i].radius * 1.5
        );
        
        if intr.y > 0.0 {
            in.ismd.player_forms_amount += 1u;

            in.ish[ish_index] = i;
            
            ish_index += 1u;

            
            offset = min(offset, intr.x);
        }
    }


    in.ray_w_rotated = false;

    if rd.w < -0.0002{
        in.ray_w_rotated = true;
    }

    offset = clamp(offset, 0.0, MAX_DIST * 4.0);
    
    in.offset = offset;
    
    return in;
}


fn apply_material(
    pos: vec4<f32>,
    ray_dir: vec4<f32>,
    dist: f32,
    in: ptr<function, Intersections>,
    material: i32,
) -> vec3<f32> {
    
    if material < 0 {
        return vec3(0.7);
    }

    let diffuse = static_data.materials[material].color;

    let normal = get_normal(pos + ray_dir * dist, in);

    let dir_shade: f32 = dot(normal, normalize(vec4<f32>(1.0, 0.5, 0.3, 0.1)));

    // let shade = mix(0.32, 0.98, shade_coefficient);

    // var color = vec3(dist_and_depth.x / (MAX_DIST / 12.0));
    var color = diffuse * dir_shade;

    return color;
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


@fragment
fn fs_main(inn: VertexOutput) -> @location(0) vec4<f32> {

    var uv: vec2<f32> = inn.position.xy * 0.7;
    uv.x *= dynamic_data.screen_aspect;

    var ray_direction: vec4<f32> = normalize(vec4<f32>(uv, -1.0, 0.0));
    ray_direction *= dynamic_data.camera_data.cam_rot;

    var ray_w_rotated: i32 = 1;

    if ray_direction.w < 0.0002 && ray_direction.w > -0.0002{
        ray_w_rotated = 0;
    }

    let camera_position = dynamic_data.camera_data.cam_pos;

    var in = find_intersections(camera_position, ray_direction);
    
    let dist_and_depth: vec2<f32> = ray_march(camera_position, ray_direction, &in); 

    var mats = get_mat(camera_position, ray_direction, dist_and_depth.x, &in);

    var color = apply_material(camera_position, ray_direction, dist_and_depth.x, &in, mats.materials[0]);

    for (var i = 1u; i < mats.materials_count; i++) {

        let new_color = apply_material(camera_position, ray_direction, dist_and_depth.x, &in, mats.materials[i]);

        color = mix(color, new_color, mats.material_weights[i]);
    }



    // var color = clamp(normal.xyz, vec3(0.0), vec3(1.0));
    // color = mix(color, vec3(0.0), in.offset / (MAX_DIST / 12.0));



    // var color = get_color(camera_position, ray_direction, dist_and_depth.x, ray_w_rotated);

    // let coloring_color = get_coloring_areas_color(camera_position + ray_direction * dist_and_depth.x);

    // color += coloring_color * 0.4;

    // color *= shade * 1.2;

    // color += coloring_color * 0.4;

    // let volume_areas_color = get_volume_areas_color(camera_position, ray_direction, dist_and_depth.x); 

    // color = clamp(color, vec3(0.0), vec3(1.0));

    // color = mix(color, vec3<f32>(0.9, 1., 1.0), (dist_and_depth.x*0.4 / (MAX_DIST*0.4)));

    // color = mix(color, volume_areas_color.rgb, volume_areas_color.a);

    // if dynamic_data.w_scaner_radius > 0.0 {
    //     color += add_w_scnner_color(camera_position, dist_and_depth.x, ray_direction);
    // }


    //crosshair

    // var color = vec3(in.offset / (MAX_DIST / 4.0));
    // var color = vec3(f32(dist_and_depth.y) / 50.0);

    // color += normal_shapes[in.ish[0]].color;

    // var color = vec3(dist_and_depth.x / (MAX_DIST / 12.0));
    // color.r += dist_and_depth.y / 25.0;


    color = pow(color, vec3(0.4545));
    color += (0.007 - clamp(length(uv), 0.0, 0.007))*1000.0;
    return vec4<f32>(color, 1.0);
}