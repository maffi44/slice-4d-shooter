// Fragment shader

struct CameraUniform {
    cam_pos: vec4<f32>,
    cam_rot: mat4x4<f32>,
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

// struct Intersections {
//     ismd: IntersectedShapesMetadata,
//     ish: array<u32, 16>,
//     offset: f32,
// }

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
    //all shapes bounding box sides
    bb_pos_side: vec4<f32>,
    bb_neg_side: vec4<f32>,
}

struct Material {
    color: vec4<f32>,
}

struct OtherStaticData {
    shapes_arrays_metadata: ShapesMetadata,
    
    is_w_floor_exist: i32,
    w_floor: f32,
    // is_w_roof_exist: i32,
    // w_roof: f32,

    players_mat1: i32,
    players_mat2: i32,
    w_cups_mat: i32,
    stickiness: f32,



    empty_byte1: u32,
    empty_byte2: u32,
    materials: array<Material, 32>,

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

const MAX_STEPS: i32 = 80;
const PI: f32 = 3.1415926535897;
const MIN_DIST: f32 = 0.01;
const MAX_DIST: f32 = 250.0;

const STICKINESS_EFFECT_COEF: f32 = 3.1415926535897;

var<private> w_plane_intersected: bool = false;
    
var<private> st_noramls_intersected: bool = false;
var<private> dyn_noramls_intersected: bool = false;
var<private> st_negative_intersected: bool = false;
var<private> dyn_negative_intersected: bool = false;
var<private> st_stickiness_intersected: bool = false;
var<private> dyn_stickiness_intersected: bool = false;
var<private> st_neg_stickiness_intersected: bool = false;
var<private> dyn_neg_stickiness_intersected: bool = false;
var<private> player_forms_intersected: bool = false;

// var<private> st_cubes_intersected: bool = false;
// var<private> dyn_cubes_intersected: bool = false;
// var<private> st_spheres_intersected: bool = false;
// var<private> dyn_spheres_intersected: bool = false;
// var<private> st_inf_cubes_intersected: bool = false;
// var<private> dyn_inf_cubes_intersected: bool = false;
// var<private> st_sph_cubes_intersected: bool = false;
// var<private> dyn_sph_cubes_intersected: bool = false;
// var<private> st_s_cubes_intersected: bool = false;
// var<private> dyn_s_cubes_intersected: bool = false;
// var<private> st_s_spheres_intersected: bool = false;
// var<private> dyn_s_spheres_intersected: bool = false;
// var<private> st_s_inf_cubes_intersected: bool = false;
// var<private> dyn_s_inf_cubes_intersected: bool = false;
// var<private> st_s_sph_cubes_intersected: bool = false;
// var<private> dyn_s_sph_cubes_intersected: bool = false;
// var<private> st_neg_cubes_intersected: bool = false;
// var<private> dyn_neg_cubes_intersected: bool = false;
// var<private> st_neg_spheres_intersected: bool = false;
// var<private> dyn_neg_spheres_intersected: bool = false;
// var<private> st_neg_inf_cubes_intersected: bool = false;
// var<private> dyn_neg_inf_cubes_intersected: bool = false;
// var<private> st_neg_sph_cubes_intersected: bool = false;
// var<private> dyn_neg_sph_cubes_intersected: bool = false;
// var<private> st_s_neg_cubes_intersected: bool = false;
// var<private> dyn_s_neg_cubes_intersected: bool = false;
// var<private> st_s_neg_spheres_intersected: bool = false;
// var<private> dyn_s_neg_spheres_intersected: bool = false;
// var<private> st_s_neg_inf_cubes_intersected: bool = false;
// var<private> dyn_s_neg_inf_cubes_intersected: bool = false;
// var<private> st_s_neg_sph_cubes_intersected: bool = false;
// var<private> dyn_s_neg_sph_cubes_intersected: bool = false;


fn rotate(angle: f32) -> mat2x2<f32> {
    //angle *= 0.017453;
    var c: f32 = cos(angle);
    var s: f32 = sin(angle);
    return mat2x2<f32>(c, -s, s, c);
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
        return vec2(-1.0); // no intersection
    }
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
        return vec2(-1.0); // no intersection
    }
    return vec2( tN, tF );
}

fn sph_intersection( ro: vec4<f32>, rd: vec4<f32>, ra: f32) -> vec2<f32> {  // can precompute if traversing a set of aligned boxes
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

// exponential
// fn smin(a: f32, b: f32, k: f32) -> f32
// {
//     let x = (b-a)/k;
//     let g = 0.5*(x-sqrt(x*x+0.25));
//     return a + k * g;
// }

// circular
fn smin( a: f32, b: f32, k: f32 ) -> f32
{
    let kk = k * 1.0/(1.0-sqrt(0.5));
    let h = max( kk-abs(a-b), 0.0 )/kk;
    return min(a,b) - kk*0.5*(1.0+h-sqrt(1.0-h*(h - 2.0)));
}

// fn get_color(start_pos: vec4<f32>, direction: vec4<f32>, distance: f32, w_plane_intersected: i32) -> vec3<f32> {
//     let point = start_pos + direction * distance;
    
//     var color = get_color_at_point(point, distance, w_plane_intersected);

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

fn get_volume_areas_color(start_pos: vec4<f32>, direction: vec4<f32>, max_distance: f32) -> vec3<f32> {
    var color = vec3(0.0);

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

    return color;
}

fn ray_march_individual_volume_sphere(sphere: SphericalArea, start_pos: vec4<f32>, direction: vec4<f32>, max_distance: f32) -> vec3<f32> {
    var color = vec3(0.0);

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

            color = mix(sphere.color, vec3(1.0), pow(color_coef, 40.5)) * pow(color_coef, 10.0) + vec3(0.05);

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


fn ray_march_indicidual_volume_beam(beam: BeamArea, start_pos: vec4<f32>, direction: vec4<f32>, max_distance: f32) -> vec3<f32> {
    var color = vec3(0.0);

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

            color = mix(beam.color, vec3(1.0), pow(color_coef, 80.5)) * pow(color_coef, 20.0);

            break;
        }
        total_dist += d;

        p += direction * d;
    }

    return color;
}


fn plane_w_intersect( ro: vec4<f32>, rd: vec4<f32>, h: f32 ) -> f32
{
    return (ro.w-h)/-rd.w;
}


fn find_intersections(ro: vec4<f32>, rd: vec4<f32>) -> f32 {
    
    var offset: f32 = MAX_DIST * 2.0;

    // static stickiness shapes
    for (var i = static_data.shapes_arrays_metadata.s_cubes_start; i < static_data.shapes_arrays_metadata.s_cubes_amount + static_data.shapes_arrays_metadata.s_cubes_start; i++) {
        let intr = cube_intersection(
            ro - stickiness_shapes[i].pos,
            rd,
            stickiness_shapes[i].size + stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            st_stickiness_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    for (var i = static_data.shapes_arrays_metadata.s_spheres_start; i < static_data.shapes_arrays_metadata.s_spheres_amount + static_data.shapes_arrays_metadata.s_spheres_start; i++) {
        let intr = sph_intersection(
            ro - stickiness_shapes[i].pos,
            rd,
            stickiness_shapes[i].size.x + stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            st_stickiness_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    for (var i = static_data.shapes_arrays_metadata.s_sph_cubes_start; i < static_data.shapes_arrays_metadata.s_sph_cubes_amount + static_data.shapes_arrays_metadata.s_sph_cubes_start; i++) {
        let intr = cube_intersection(
            ro - stickiness_shapes[i].pos,
            rd,
            stickiness_shapes[i].size + stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            st_stickiness_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    for (var i = static_data.shapes_arrays_metadata.s_inf_cubes_start; i < static_data.shapes_arrays_metadata.s_inf_cubes_amount + static_data.shapes_arrays_metadata.s_inf_cubes_start; i++) {
        let intr = inf_cube_intersection(
            ro - stickiness_shapes[i].pos,
            rd,
            stickiness_shapes[i].size.xyz + stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            st_stickiness_intersected = true;
            offset = min(offset, intr.x);
        }
    }
    

    // dynamic stickiness

    for (var i = dynamic_data.shapes_arrays_metadata.s_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_cubes_amount + dynamic_data.shapes_arrays_metadata.s_cubes_start; i++) {
        let intr = cube_intersection(
            ro - dyn_stickiness_shapes[i].pos,
            rd,
            dyn_stickiness_shapes[i].size + dyn_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            dyn_stickiness_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    for (var i = dynamic_data.shapes_arrays_metadata.s_spheres_start; i < dynamic_data.shapes_arrays_metadata.s_spheres_amount + dynamic_data.shapes_arrays_metadata.s_spheres_start; i++) {
        let intr = sph_intersection(
            ro - dyn_stickiness_shapes[i].pos,
            rd,
            dyn_stickiness_shapes[i].size.x + dyn_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            dyn_stickiness_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    for (var i = dynamic_data.shapes_arrays_metadata.s_sph_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_sph_cubes_amount + dynamic_data.shapes_arrays_metadata.s_sph_cubes_start; i++) {
        let intr = cube_intersection(
            ro - dyn_stickiness_shapes[i].pos,
            rd,
            dyn_stickiness_shapes[i].size + dyn_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            dyn_stickiness_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    for (var i = dynamic_data.shapes_arrays_metadata.s_inf_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_inf_cubes_amount + dynamic_data.shapes_arrays_metadata.s_inf_cubes_start; i++) {
        let intr = inf_cube_intersection(
            ro - dyn_stickiness_shapes[i].pos,
            rd,
            dyn_stickiness_shapes[i].size.xyz + dyn_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            dyn_stickiness_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    // static normal shapes

    for (var i = static_data.shapes_arrays_metadata.cubes_start; i < static_data.shapes_arrays_metadata.cubes_amount + static_data.shapes_arrays_metadata.cubes_start; i++) {
        let intr = cube_intersection(
            ro - normal_shapes[i].pos,
            rd,
            normal_shapes[i].size + normal_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            st_noramls_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    for (var i = static_data.shapes_arrays_metadata.spheres_start; i < static_data.shapes_arrays_metadata.spheres_amount + static_data.shapes_arrays_metadata.spheres_start; i++) {
        let intr = sph_intersection(
            ro - normal_shapes[i].pos,
            rd,
            normal_shapes[i].size.x + normal_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            st_noramls_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    for (var i = static_data.shapes_arrays_metadata.sph_cubes_start; i < static_data.shapes_arrays_metadata.sph_cubes_amount + static_data.shapes_arrays_metadata.sph_cubes_start; i++) {
        let intr = cube_intersection(
            ro - normal_shapes[i].pos,
            rd,
            normal_shapes[i].size + normal_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            st_noramls_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    for (var i = static_data.shapes_arrays_metadata.inf_cubes_start; i < static_data.shapes_arrays_metadata.inf_cubes_amount + static_data.shapes_arrays_metadata.inf_cubes_start; i++) {
        let intr = inf_cube_intersection(
            ro - normal_shapes[i].pos,
            rd,
            normal_shapes[i].size.xyz + normal_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            st_noramls_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    // dynamic normals 
    for (var i = dynamic_data.shapes_arrays_metadata.cubes_start; i < dynamic_data.shapes_arrays_metadata.cubes_amount + dynamic_data.shapes_arrays_metadata.cubes_start; i++) {
        let intr = cube_intersection(
            ro - dyn_normal_shapes[i].pos,
            rd,
            dyn_normal_shapes[i].size + dyn_normal_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            dyn_noramls_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    for (var i = dynamic_data.shapes_arrays_metadata.spheres_start; i < dynamic_data.shapes_arrays_metadata.spheres_amount + dynamic_data.shapes_arrays_metadata.spheres_start; i++) {
        let intr = sph_intersection(
            ro - dyn_normal_shapes[i].pos,
            rd,
            dyn_normal_shapes[i].size.x + dyn_normal_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            dyn_noramls_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    for (var i = dynamic_data.shapes_arrays_metadata.sph_cubes_start; i < dynamic_data.shapes_arrays_metadata.sph_cubes_amount + dynamic_data.shapes_arrays_metadata.sph_cubes_start; i++) {
        let intr = cube_intersection(
            ro - dyn_normal_shapes[i].pos,
            rd,
            dyn_normal_shapes[i].size + dyn_normal_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            dyn_noramls_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    for (var i = dynamic_data.shapes_arrays_metadata.inf_cubes_start; i < dynamic_data.shapes_arrays_metadata.inf_cubes_amount + dynamic_data.shapes_arrays_metadata.inf_cubes_start; i++) {
        let intr = inf_cube_intersection(
            ro - dyn_normal_shapes[i].pos,
            rd,
            dyn_normal_shapes[i].size.xyz + dyn_normal_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            dyn_noramls_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    // static negative stickiness shapes
    for (var i = static_data.shapes_arrays_metadata.s_neg_cubes_start; i < static_data.shapes_arrays_metadata.s_neg_cubes_amount + static_data.shapes_arrays_metadata.s_neg_cubes_start; i++) {
        let intr = cube_intersection(
            ro - neg_stickiness_shapes[i].pos,
            rd,
            neg_stickiness_shapes[i].size + neg_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            st_neg_stickiness_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    for (var i = static_data.shapes_arrays_metadata.s_neg_spheres_start; i < static_data.shapes_arrays_metadata.s_neg_spheres_amount + static_data.shapes_arrays_metadata.s_neg_spheres_start; i++) {
        let intr = sph_intersection(
            ro - neg_stickiness_shapes[i].pos,
            rd,
            neg_stickiness_shapes[i].size.x + neg_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            st_neg_stickiness_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    for (var i = static_data.shapes_arrays_metadata.s_neg_sph_cubes_start; i < static_data.shapes_arrays_metadata.s_neg_sph_cubes_amount + static_data.shapes_arrays_metadata.s_neg_sph_cubes_start; i++) {
        let intr = cube_intersection(
            ro - neg_stickiness_shapes[i].pos,
            rd,
            neg_stickiness_shapes[i].size + neg_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            st_neg_stickiness_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    for (var i = static_data.shapes_arrays_metadata.s_neg_inf_cubes_start; i < static_data.shapes_arrays_metadata.s_neg_inf_cubes_amount + static_data.shapes_arrays_metadata.s_neg_inf_cubes_start; i++) {
        let intr = inf_cube_intersection(
            ro - neg_stickiness_shapes[i].pos,
            rd,
            neg_stickiness_shapes[i].size.xyz + neg_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            st_neg_stickiness_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    // dynamic negative stickiness shapes
    for (var i = dynamic_data.shapes_arrays_metadata.s_neg_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_neg_cubes_amount + dynamic_data.shapes_arrays_metadata.s_neg_cubes_start; i++) {
        let intr = cube_intersection(
            ro - dyn_neg_stickiness_shapes[i].pos,
            rd,
            dyn_neg_stickiness_shapes[i].size + dyn_neg_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            dyn_neg_stickiness_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    for (var i = dynamic_data.shapes_arrays_metadata.s_neg_spheres_start; i < dynamic_data.shapes_arrays_metadata.s_neg_spheres_amount + dynamic_data.shapes_arrays_metadata.s_neg_spheres_start; i++) {
        let intr = sph_intersection(
            ro - dyn_neg_stickiness_shapes[i].pos,
            rd,
            dyn_neg_stickiness_shapes[i].size.x + dyn_neg_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            dyn_neg_stickiness_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    for (var i = dynamic_data.shapes_arrays_metadata.s_neg_sph_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_neg_sph_cubes_amount + dynamic_data.shapes_arrays_metadata.s_neg_sph_cubes_start; i++) {
        let intr = cube_intersection(
            ro - dyn_neg_stickiness_shapes[i].pos,
            rd,
            dyn_neg_stickiness_shapes[i].size + dyn_neg_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            dyn_neg_stickiness_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    for (var i = dynamic_data.shapes_arrays_metadata.s_neg_inf_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_neg_inf_cubes_amount + dynamic_data.shapes_arrays_metadata.s_neg_inf_cubes_start; i++) {
        let intr = inf_cube_intersection(
            ro - dyn_neg_stickiness_shapes[i].pos,
            rd,
            dyn_neg_stickiness_shapes[i].size.xyz + dyn_neg_stickiness_shapes[i].roundness +(static_data.stickiness * STICKINESS_EFFECT_COEF)
        );
        
        if intr.y > 0.0 {
            dyn_neg_stickiness_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    // static negative shapes
    for (var i = static_data.shapes_arrays_metadata.neg_cubes_start; i < static_data.shapes_arrays_metadata.neg_cubes_amount + static_data.shapes_arrays_metadata.neg_cubes_start; i++) {
        let intr = cube_intersection(
            ro - negatives_shapes[i].pos,
            rd,
            negatives_shapes[i].size + negatives_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            st_negative_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    for (var i = static_data.shapes_arrays_metadata.neg_spheres_start; i < static_data.shapes_arrays_metadata.neg_spheres_amount + static_data.shapes_arrays_metadata.neg_spheres_start; i++) {
        let intr = sph_intersection(
            ro - negatives_shapes[i].pos,
            rd,
            negatives_shapes[i].size.x + negatives_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            st_negative_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    for (var i = static_data.shapes_arrays_metadata.neg_sph_cubes_start; i < static_data.shapes_arrays_metadata.neg_sph_cubes_amount + static_data.shapes_arrays_metadata.neg_sph_cubes_start; i++) {
        let intr = cube_intersection(
            ro - negatives_shapes[i].pos,
            rd,
            negatives_shapes[i].size + negatives_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            st_negative_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    for (var i = static_data.shapes_arrays_metadata.neg_inf_cubes_start; i < static_data.shapes_arrays_metadata.neg_inf_cubes_amount + static_data.shapes_arrays_metadata.neg_inf_cubes_start; i++) {
        let intr = inf_cube_intersection(
            ro - negatives_shapes[i].pos,
            rd,
            negatives_shapes[i].size.xyz + negatives_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            st_negative_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    // dynamic negative shapes
    for (var i = dynamic_data.shapes_arrays_metadata.neg_cubes_start; i < dynamic_data.shapes_arrays_metadata.neg_cubes_amount + dynamic_data.shapes_arrays_metadata.neg_cubes_start; i++) {
        let intr = cube_intersection(
            ro - dyn_negatives_shapes[i].pos,
            rd,
            dyn_negatives_shapes[i].size + dyn_negatives_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            dyn_negative_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    for (var i = dynamic_data.shapes_arrays_metadata.neg_spheres_start; i < dynamic_data.shapes_arrays_metadata.neg_spheres_amount + dynamic_data.shapes_arrays_metadata.neg_spheres_start; i++) {
        let intr = sph_intersection(
            ro - dyn_negatives_shapes[i].pos,
            rd,
            dyn_negatives_shapes[i].size.x + dyn_negatives_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            dyn_negative_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    for (var i = dynamic_data.shapes_arrays_metadata.neg_sph_cubes_start; i < dynamic_data.shapes_arrays_metadata.neg_sph_cubes_amount + dynamic_data.shapes_arrays_metadata.neg_sph_cubes_start; i++) {
        let intr = cube_intersection(
            ro - dyn_negatives_shapes[i].pos,
            rd,
            dyn_negatives_shapes[i].size + dyn_negatives_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            dyn_negative_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    for (var i = dynamic_data.shapes_arrays_metadata.neg_inf_cubes_start; i < dynamic_data.shapes_arrays_metadata.neg_inf_cubes_amount + dynamic_data.shapes_arrays_metadata.neg_inf_cubes_start; i++) {
        let intr = inf_cube_intersection(
            ro - dyn_negatives_shapes[i].pos,
            rd,
            dyn_negatives_shapes[i].size.xyz + dyn_negatives_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            dyn_negative_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    // player forms
    for (var i = 0u; i < dynamic_data.player_forms_amount; i++) {
        let intr = sph_intersection(
            ro - dyn_player_forms[i].pos,
            rd,
            dyn_player_forms[i].radius * 1.7
        );
        
        if intr.y > 0.0 {
            player_forms_intersected = true;
            offset = min(offset, intr.x);
        }
    }

    w_plane_intersected = false;


    let w_offset = plane_w_intersect(ro, rd, static_data.w_floor);
    
    if w_offset < MAX_DIST*0.3333 && w_offset > 0.0 {
        w_plane_intersected = true;
        offset = min(offset, w_offset);
    }

    offset = clamp(offset, 0.0, MAX_DIST * 4.0);
    
    return offset;
}



fn not_opt_map(p: vec4<f32>) -> f32 {
    var d = MAX_DIST*2.0;

    // static stickiness shapes
    for (var i = static_data.shapes_arrays_metadata.s_cubes_start; i < static_data.shapes_arrays_metadata.s_cubes_amount + static_data.shapes_arrays_metadata.s_cubes_start; i++) {
        d = smin(d, sd_box(p - stickiness_shapes[i].pos, stickiness_shapes[i].size) - stickiness_shapes[i].roundness, static_data.stickiness);
    }
    for (var i = static_data.shapes_arrays_metadata.s_spheres_start; i < static_data.shapes_arrays_metadata.s_spheres_amount + static_data.shapes_arrays_metadata.s_spheres_start; i++) {
        d = smin(d, sd_sphere(p - stickiness_shapes[i].pos, stickiness_shapes[i].size.x) - stickiness_shapes[i].roundness, static_data.stickiness);
    }
    for (var i = static_data.shapes_arrays_metadata.s_sph_cubes_start; i < static_data.shapes_arrays_metadata.s_sph_cubes_amount + static_data.shapes_arrays_metadata.s_sph_cubes_start; i++) {
        d = smin(d, sd_sph_box(p - stickiness_shapes[i].pos, stickiness_shapes[i].size) - stickiness_shapes[i].roundness, static_data.stickiness);
    }
    for (var i = static_data.shapes_arrays_metadata.s_inf_cubes_start; i < static_data.shapes_arrays_metadata.s_inf_cubes_amount + static_data.shapes_arrays_metadata.s_inf_cubes_start; i++) {
        d = smin(d, sd_inf_box(p - stickiness_shapes[i].pos, stickiness_shapes[i].size.xyz) - stickiness_shapes[i].roundness, static_data.stickiness);
    }

    // dynamic stickiness
    for (var i = dynamic_data.shapes_arrays_metadata.s_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_cubes_amount + dynamic_data.shapes_arrays_metadata.s_cubes_start; i++) {
        d = smin(d, sd_box(p - dyn_stickiness_shapes[i].pos, dyn_stickiness_shapes[i].size) - dyn_stickiness_shapes[i].roundness, static_data.stickiness);
    }
    for (var i = dynamic_data.shapes_arrays_metadata.s_spheres_start; i < dynamic_data.shapes_arrays_metadata.s_spheres_amount + dynamic_data.shapes_arrays_metadata.s_spheres_start; i++) {
        d = smin(d, sd_sphere(p - dyn_stickiness_shapes[i].pos, dyn_stickiness_shapes[i].size.x) - dyn_stickiness_shapes[i].roundness, static_data.stickiness);
    }
    for (var i = dynamic_data.shapes_arrays_metadata.s_sph_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_sph_cubes_amount + dynamic_data.shapes_arrays_metadata.s_sph_cubes_start; i++) {
        d = smin(d, sd_sph_box(p - dyn_stickiness_shapes[i].pos, dyn_stickiness_shapes[i].size) - dyn_stickiness_shapes[i].roundness, static_data.stickiness);
    }
    for (var i = dynamic_data.shapes_arrays_metadata.s_inf_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_inf_cubes_amount + dynamic_data.shapes_arrays_metadata.s_inf_cubes_start; i++) {
        d = smin(d, sd_inf_box(p - dyn_stickiness_shapes[i].pos, dyn_stickiness_shapes[i].size.xyz) - dyn_stickiness_shapes[i].roundness, static_data.stickiness);
    }


    // static normal shapes
    for (var i = static_data.shapes_arrays_metadata.cubes_start; i < static_data.shapes_arrays_metadata.cubes_amount + static_data.shapes_arrays_metadata.cubes_start; i++) {
        d = min(d, sd_box(p - normal_shapes[i].pos, normal_shapes[i].size) - normal_shapes[i].roundness);
    }
    for (var i = static_data.shapes_arrays_metadata.spheres_start; i < static_data.shapes_arrays_metadata.spheres_amount + static_data.shapes_arrays_metadata.spheres_start; i++) {
        d = min(d, sd_sphere(p - normal_shapes[i].pos, normal_shapes[i].size.x) - normal_shapes[i].roundness);
    }
    for (var i = static_data.shapes_arrays_metadata.sph_cubes_start; i < static_data.shapes_arrays_metadata.sph_cubes_amount + static_data.shapes_arrays_metadata.sph_cubes_start; i++) {
        d = min(d, sd_sph_box(p - normal_shapes[i].pos, normal_shapes[i].size) - normal_shapes[i].roundness);
    }
    for (var i = static_data.shapes_arrays_metadata.inf_cubes_start; i < static_data.shapes_arrays_metadata.inf_cubes_amount + static_data.shapes_arrays_metadata.inf_cubes_start; i++) {
        d = min(d, sd_inf_box(p - normal_shapes[i].pos, normal_shapes[i].size.xyz) - normal_shapes[i].roundness);
    }

    // dynamic normal shapes
    for (var i = dynamic_data.shapes_arrays_metadata.cubes_start; i < dynamic_data.shapes_arrays_metadata.cubes_amount + dynamic_data.shapes_arrays_metadata.cubes_start; i++) {
        d = min(d, sd_box(p - dyn_normal_shapes[i].pos, dyn_normal_shapes[i].size) - dyn_normal_shapes[i].roundness);
    }
    for (var i = dynamic_data.shapes_arrays_metadata.spheres_start; i < dynamic_data.shapes_arrays_metadata.spheres_amount + dynamic_data.shapes_arrays_metadata.spheres_start; i++) {
        d = min(d, sd_sphere(p - dyn_normal_shapes[i].pos, dyn_normal_shapes[i].size.x) - dyn_normal_shapes[i].roundness);
    }
    for (var i = dynamic_data.shapes_arrays_metadata.sph_cubes_start; i < dynamic_data.shapes_arrays_metadata.sph_cubes_amount + dynamic_data.shapes_arrays_metadata.sph_cubes_start; i++) {
        d = min(d, sd_sph_box(p - dyn_normal_shapes[i].pos, dyn_normal_shapes[i].size) - dyn_normal_shapes[i].roundness);
    }
    for (var i = dynamic_data.shapes_arrays_metadata.inf_cubes_start; i < dynamic_data.shapes_arrays_metadata.inf_cubes_amount + dynamic_data.shapes_arrays_metadata.inf_cubes_start; i++) {
        d = min(d, sd_inf_box(p - dyn_normal_shapes[i].pos, dyn_normal_shapes[i].size.xyz) - dyn_normal_shapes[i].roundness);
    }

    // static negative stickiness shapes
    var dd = MAX_DIST;

    for (var i = static_data.shapes_arrays_metadata.s_neg_cubes_start; i < static_data.shapes_arrays_metadata.s_neg_cubes_amount + static_data.shapes_arrays_metadata.s_neg_cubes_start; i++) {
        dd = smin(dd, sd_box(p - neg_stickiness_shapes[i].pos, neg_stickiness_shapes[i].size) - neg_stickiness_shapes[i].roundness, static_data.stickiness);
    }
    for (var i = static_data.shapes_arrays_metadata.s_neg_spheres_start; i < static_data.shapes_arrays_metadata.s_neg_spheres_amount + static_data.shapes_arrays_metadata.s_neg_spheres_start; i++) {
        dd = smin(dd, sd_sphere(p - neg_stickiness_shapes[i].pos, neg_stickiness_shapes[i].size.x) - neg_stickiness_shapes[i].roundness, static_data.stickiness);
    }
    for (var i = static_data.shapes_arrays_metadata.s_neg_sph_cubes_start; i < static_data.shapes_arrays_metadata.s_neg_sph_cubes_amount + static_data.shapes_arrays_metadata.s_neg_sph_cubes_start; i++) {
        dd = smin(dd, sd_sph_box(p - neg_stickiness_shapes[i].pos, neg_stickiness_shapes[i].size) - neg_stickiness_shapes[i].roundness, static_data.stickiness);
    }
    for (var i = static_data.shapes_arrays_metadata.s_neg_inf_cubes_start; i < static_data.shapes_arrays_metadata.s_neg_inf_cubes_amount + static_data.shapes_arrays_metadata.s_neg_inf_cubes_start; i++) {
        dd = smin(dd, sd_inf_box(p - neg_stickiness_shapes[i].pos, neg_stickiness_shapes[i].size.xyz) - neg_stickiness_shapes[i].roundness, static_data.stickiness);
    }
    d = max(d, -dd);

    // dynamic negative stickiness shapes
    var ddd = dd;

    for (var i = dynamic_data.shapes_arrays_metadata.s_neg_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_neg_cubes_amount + dynamic_data.shapes_arrays_metadata.s_neg_cubes_start; i++) {
        ddd = smin(ddd, sd_box(p - dyn_neg_stickiness_shapes[i].pos, dyn_neg_stickiness_shapes[i].size) - dyn_neg_stickiness_shapes[i].roundness, static_data.stickiness);
    }
    for (var i = dynamic_data.shapes_arrays_metadata.s_neg_spheres_start; i < dynamic_data.shapes_arrays_metadata.s_neg_spheres_amount + dynamic_data.shapes_arrays_metadata.s_neg_spheres_start; i++) {
        ddd = smin(ddd, sd_sphere(p - dyn_neg_stickiness_shapes[i].pos, dyn_neg_stickiness_shapes[i].size.x) - dyn_neg_stickiness_shapes[i].roundness, static_data.stickiness);
    }
    for (var i = dynamic_data.shapes_arrays_metadata.s_neg_sph_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_neg_sph_cubes_amount + dynamic_data.shapes_arrays_metadata.s_neg_sph_cubes_start; i++) {
        ddd = smin(ddd, sd_sph_box(p - dyn_neg_stickiness_shapes[i].pos, dyn_neg_stickiness_shapes[i].size) - dyn_neg_stickiness_shapes[i].roundness, static_data.stickiness);
    }
    for (var i = dynamic_data.shapes_arrays_metadata.s_neg_inf_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_neg_inf_cubes_amount + dynamic_data.shapes_arrays_metadata.s_neg_inf_cubes_start; i++) {
        ddd = smin(ddd, sd_inf_box(p - dyn_neg_stickiness_shapes[i].pos, dyn_neg_stickiness_shapes[i].size.xyz) - dyn_neg_stickiness_shapes[i].roundness, static_data.stickiness);
    }
    d = max(d, -ddd);

    // static negative shapes
    for (var i = static_data.shapes_arrays_metadata.neg_cubes_start; i < static_data.shapes_arrays_metadata.neg_cubes_amount + static_data.shapes_arrays_metadata.neg_cubes_start; i++) {
        d = max(d, -(sd_box(p - negatives_shapes[i].pos, negatives_shapes[i].size) - negatives_shapes[i].roundness));
    }
    for (var i = static_data.shapes_arrays_metadata.neg_spheres_start; i < static_data.shapes_arrays_metadata.neg_spheres_amount + static_data.shapes_arrays_metadata.neg_spheres_start; i++) {
        d = max(d, -(sd_sphere(p - negatives_shapes[i].pos, negatives_shapes[i].size.x) - negatives_shapes[i].roundness));
    }
    for (var i = static_data.shapes_arrays_metadata.neg_sph_cubes_start; i < static_data.shapes_arrays_metadata.neg_sph_cubes_amount + static_data.shapes_arrays_metadata.neg_sph_cubes_start; i++) {
        d = max(d, -(sd_sph_box(p - negatives_shapes[i].pos, negatives_shapes[i].size) - negatives_shapes[i].roundness));
    }
    for (var i = static_data.shapes_arrays_metadata.neg_inf_cubes_start; i < static_data.shapes_arrays_metadata.neg_inf_cubes_amount + static_data.shapes_arrays_metadata.neg_inf_cubes_start; i++) {
        d = max(d, -(sd_inf_box(p - negatives_shapes[i].pos, negatives_shapes[i].size.xyz) - negatives_shapes[i].roundness));
    }

    // dynamic negative shapes
    for (var i = dynamic_data.shapes_arrays_metadata.neg_cubes_start; i < dynamic_data.shapes_arrays_metadata.neg_cubes_amount + dynamic_data.shapes_arrays_metadata.neg_cubes_start; i++) {
        d = max(d, -(sd_box(p - dyn_negatives_shapes[i].pos, dyn_negatives_shapes[i].size) - dyn_negatives_shapes[i].roundness));
    }
    for (var i = dynamic_data.shapes_arrays_metadata.neg_spheres_start; i < dynamic_data.shapes_arrays_metadata.neg_spheres_amount + dynamic_data.shapes_arrays_metadata.neg_spheres_start; i++) {
        d = max(d, -(sd_sphere(p - dyn_negatives_shapes[i].pos, dyn_negatives_shapes[i].size.x) - dyn_negatives_shapes[i].roundness));
    }
    for (var i = dynamic_data.shapes_arrays_metadata.neg_sph_cubes_start; i < dynamic_data.shapes_arrays_metadata.neg_sph_cubes_amount + dynamic_data.shapes_arrays_metadata.neg_sph_cubes_start; i++) {
        d = max(d, -(sd_sph_box(p - dyn_negatives_shapes[i].pos, dyn_negatives_shapes[i].size) - dyn_negatives_shapes[i].roundness));
    }
    for (var i = dynamic_data.shapes_arrays_metadata.neg_inf_cubes_start; i < dynamic_data.shapes_arrays_metadata.neg_inf_cubes_amount + dynamic_data.shapes_arrays_metadata.neg_inf_cubes_start; i++) {
        d = max(d, -(sd_inf_box(p - dyn_negatives_shapes[i].pos, dyn_negatives_shapes[i].size.xyz) - dyn_negatives_shapes[i].roundness));
    }

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
    
    if w_plane_intersected {
        d = min(d, p.w - static_data.w_floor);
    }

    // if static_data.is_w_roof_exist == 1 {
    //     if w_plane_intersected == 1 {
    //         d = min(d, static_data.w_roof - p.w);
    //     }
    // }

    return d;
}



fn map(p: vec4<f32>) -> f32 {
    var d = MAX_DIST*2.0;

    // static stickiness shapes
    if st_stickiness_intersected {
        for (var i = static_data.shapes_arrays_metadata.s_cubes_start; i < static_data.shapes_arrays_metadata.s_cubes_amount + static_data.shapes_arrays_metadata.s_cubes_start; i++) {
            d = smin(d, sd_box(p - stickiness_shapes[i].pos, stickiness_shapes[i].size) - stickiness_shapes[i].roundness, static_data.stickiness);
        }
        for (var i = static_data.shapes_arrays_metadata.s_spheres_start; i < static_data.shapes_arrays_metadata.s_spheres_amount + static_data.shapes_arrays_metadata.s_spheres_start; i++) {
            d = smin(d, sd_sphere(p - stickiness_shapes[i].pos, stickiness_shapes[i].size.x) - stickiness_shapes[i].roundness, static_data.stickiness);
        }
        for (var i = static_data.shapes_arrays_metadata.s_sph_cubes_start; i < static_data.shapes_arrays_metadata.s_sph_cubes_amount + static_data.shapes_arrays_metadata.s_sph_cubes_start; i++) {
            d = smin(d, sd_sph_box(p - stickiness_shapes[i].pos, stickiness_shapes[i].size) - stickiness_shapes[i].roundness, static_data.stickiness);
        }
        for (var i = static_data.shapes_arrays_metadata.s_inf_cubes_start; i < static_data.shapes_arrays_metadata.s_inf_cubes_amount + static_data.shapes_arrays_metadata.s_inf_cubes_start; i++) {
            d = smin(d, sd_inf_box(p - stickiness_shapes[i].pos, stickiness_shapes[i].size.xyz) - stickiness_shapes[i].roundness, static_data.stickiness);
        }
    }

    // dynamic stickiness
    if dyn_stickiness_intersected {
        for (var i = dynamic_data.shapes_arrays_metadata.s_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_cubes_amount + dynamic_data.shapes_arrays_metadata.s_cubes_start; i++) {
            d = smin(d, sd_box(p - dyn_stickiness_shapes[i].pos, dyn_stickiness_shapes[i].size) - dyn_stickiness_shapes[i].roundness, static_data.stickiness);
        }
        for (var i = dynamic_data.shapes_arrays_metadata.s_spheres_start; i < dynamic_data.shapes_arrays_metadata.s_spheres_amount + dynamic_data.shapes_arrays_metadata.s_spheres_start; i++) {
            d = smin(d, sd_sphere(p - dyn_stickiness_shapes[i].pos, dyn_stickiness_shapes[i].size.x) - dyn_stickiness_shapes[i].roundness, static_data.stickiness);
        }
        for (var i = dynamic_data.shapes_arrays_metadata.s_sph_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_sph_cubes_amount + dynamic_data.shapes_arrays_metadata.s_sph_cubes_start; i++) {
            d = smin(d, sd_sph_box(p - dyn_stickiness_shapes[i].pos, dyn_stickiness_shapes[i].size) - dyn_stickiness_shapes[i].roundness, static_data.stickiness);
        }
        for (var i = dynamic_data.shapes_arrays_metadata.s_inf_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_inf_cubes_amount + dynamic_data.shapes_arrays_metadata.s_inf_cubes_start; i++) {
            d = smin(d, sd_inf_box(p - dyn_stickiness_shapes[i].pos, dyn_stickiness_shapes[i].size.xyz) - dyn_stickiness_shapes[i].roundness, static_data.stickiness);
        }
    }


    // static normal shapes
    if st_noramls_intersected {
        for (var i = static_data.shapes_arrays_metadata.cubes_start; i < static_data.shapes_arrays_metadata.cubes_amount + static_data.shapes_arrays_metadata.cubes_start; i++) {
            d = min(d, sd_box(p - normal_shapes[i].pos, normal_shapes[i].size) - normal_shapes[i].roundness);
        }
        for (var i = static_data.shapes_arrays_metadata.spheres_start; i < static_data.shapes_arrays_metadata.spheres_amount + static_data.shapes_arrays_metadata.spheres_start; i++) {
            d = min(d, sd_sphere(p - normal_shapes[i].pos, normal_shapes[i].size.x) - normal_shapes[i].roundness);
        }
        for (var i = static_data.shapes_arrays_metadata.sph_cubes_start; i < static_data.shapes_arrays_metadata.sph_cubes_amount + static_data.shapes_arrays_metadata.sph_cubes_start; i++) {
            d = min(d, sd_sph_box(p - normal_shapes[i].pos, normal_shapes[i].size) - normal_shapes[i].roundness);
        }
        for (var i = static_data.shapes_arrays_metadata.inf_cubes_start; i < static_data.shapes_arrays_metadata.inf_cubes_amount + static_data.shapes_arrays_metadata.inf_cubes_start; i++) {
            d = min(d, sd_inf_box(p - normal_shapes[i].pos, normal_shapes[i].size.xyz) - normal_shapes[i].roundness);
        }
    }

    // dynamic normal shapes
    if dyn_noramls_intersected {
        for (var i = dynamic_data.shapes_arrays_metadata.cubes_start; i < dynamic_data.shapes_arrays_metadata.cubes_amount + dynamic_data.shapes_arrays_metadata.cubes_start; i++) {
            d = min(d, sd_box(p - dyn_normal_shapes[i].pos, dyn_normal_shapes[i].size) - dyn_normal_shapes[i].roundness);
        }
        for (var i = dynamic_data.shapes_arrays_metadata.spheres_start; i < dynamic_data.shapes_arrays_metadata.spheres_amount + dynamic_data.shapes_arrays_metadata.spheres_start; i++) {
            d = min(d, sd_sphere(p - dyn_normal_shapes[i].pos, dyn_normal_shapes[i].size.x) - dyn_normal_shapes[i].roundness);
        }
        for (var i = dynamic_data.shapes_arrays_metadata.sph_cubes_start; i < dynamic_data.shapes_arrays_metadata.sph_cubes_amount + dynamic_data.shapes_arrays_metadata.sph_cubes_start; i++) {
            d = min(d, sd_sph_box(p - dyn_normal_shapes[i].pos, dyn_normal_shapes[i].size) - dyn_normal_shapes[i].roundness);
        }
        for (var i = dynamic_data.shapes_arrays_metadata.inf_cubes_start; i < dynamic_data.shapes_arrays_metadata.inf_cubes_amount + dynamic_data.shapes_arrays_metadata.inf_cubes_start; i++) {
            d = min(d, sd_inf_box(p - dyn_normal_shapes[i].pos, dyn_normal_shapes[i].size.xyz) - dyn_normal_shapes[i].roundness);
        }
    }

    // static negative stickiness shapes
    var dd = MAX_DIST;
    if st_neg_stickiness_intersected {
        for (var i = static_data.shapes_arrays_metadata.s_neg_cubes_start; i < static_data.shapes_arrays_metadata.s_neg_cubes_amount + static_data.shapes_arrays_metadata.s_neg_cubes_start; i++) {
            dd = smin(dd, sd_box(p - neg_stickiness_shapes[i].pos, neg_stickiness_shapes[i].size) - neg_stickiness_shapes[i].roundness, static_data.stickiness);
        }
        for (var i = static_data.shapes_arrays_metadata.s_neg_spheres_start; i < static_data.shapes_arrays_metadata.s_neg_spheres_amount + static_data.shapes_arrays_metadata.s_neg_spheres_start; i++) {
            dd = smin(dd, sd_sphere(p - neg_stickiness_shapes[i].pos, neg_stickiness_shapes[i].size.x) - neg_stickiness_shapes[i].roundness, static_data.stickiness);
        }
        for (var i = static_data.shapes_arrays_metadata.s_neg_sph_cubes_start; i < static_data.shapes_arrays_metadata.s_neg_sph_cubes_amount + static_data.shapes_arrays_metadata.s_neg_sph_cubes_start; i++) {
            dd = smin(dd, sd_sph_box(p - neg_stickiness_shapes[i].pos, neg_stickiness_shapes[i].size) - neg_stickiness_shapes[i].roundness, static_data.stickiness);
        }
        for (var i = static_data.shapes_arrays_metadata.s_neg_inf_cubes_start; i < static_data.shapes_arrays_metadata.s_neg_inf_cubes_amount + static_data.shapes_arrays_metadata.s_neg_inf_cubes_start; i++) {
            dd = smin(dd, sd_inf_box(p - neg_stickiness_shapes[i].pos, neg_stickiness_shapes[i].size.xyz) - neg_stickiness_shapes[i].roundness, static_data.stickiness);
        }
        d = max(d, -dd);
    }

    // dynamic negative stickiness shapes
    if dyn_neg_stickiness_intersected {
        var ddd = dd;

        for (var i = dynamic_data.shapes_arrays_metadata.s_neg_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_neg_cubes_amount + dynamic_data.shapes_arrays_metadata.s_neg_cubes_start; i++) {
            ddd = smin(ddd, sd_box(p - dyn_neg_stickiness_shapes[i].pos, dyn_neg_stickiness_shapes[i].size) - dyn_neg_stickiness_shapes[i].roundness, static_data.stickiness);
        }
        for (var i = dynamic_data.shapes_arrays_metadata.s_neg_spheres_start; i < dynamic_data.shapes_arrays_metadata.s_neg_spheres_amount + dynamic_data.shapes_arrays_metadata.s_neg_spheres_start; i++) {
            ddd = smin(ddd, sd_sphere(p - dyn_neg_stickiness_shapes[i].pos, dyn_neg_stickiness_shapes[i].size.x) - dyn_neg_stickiness_shapes[i].roundness, static_data.stickiness);
        }
        for (var i = dynamic_data.shapes_arrays_metadata.s_neg_sph_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_neg_sph_cubes_amount + dynamic_data.shapes_arrays_metadata.s_neg_sph_cubes_start; i++) {
            ddd = smin(ddd, sd_sph_box(p - dyn_neg_stickiness_shapes[i].pos, dyn_neg_stickiness_shapes[i].size) - dyn_neg_stickiness_shapes[i].roundness, static_data.stickiness);
        }
        for (var i = dynamic_data.shapes_arrays_metadata.s_neg_inf_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_neg_inf_cubes_amount + dynamic_data.shapes_arrays_metadata.s_neg_inf_cubes_start; i++) {
            ddd = smin(ddd, sd_inf_box(p - dyn_neg_stickiness_shapes[i].pos, dyn_neg_stickiness_shapes[i].size.xyz) - dyn_neg_stickiness_shapes[i].roundness, static_data.stickiness);
        }
        d = max(d, -ddd);
    }

    // static negative shapes
    if st_negative_intersected {
        for (var i = static_data.shapes_arrays_metadata.neg_cubes_start; i < static_data.shapes_arrays_metadata.neg_cubes_amount + static_data.shapes_arrays_metadata.neg_cubes_start; i++) {
            d = max(d, -(sd_box(p - negatives_shapes[i].pos, negatives_shapes[i].size) - negatives_shapes[i].roundness));
        }
        for (var i = static_data.shapes_arrays_metadata.neg_spheres_start; i < static_data.shapes_arrays_metadata.neg_spheres_amount + static_data.shapes_arrays_metadata.neg_spheres_start; i++) {
            d = max(d, -(sd_sphere(p - negatives_shapes[i].pos, negatives_shapes[i].size.x) - negatives_shapes[i].roundness));
        }
        for (var i = static_data.shapes_arrays_metadata.neg_sph_cubes_start; i < static_data.shapes_arrays_metadata.neg_sph_cubes_amount + static_data.shapes_arrays_metadata.neg_sph_cubes_start; i++) {
            d = max(d, -(sd_sph_box(p - negatives_shapes[i].pos, negatives_shapes[i].size) - negatives_shapes[i].roundness));
        }
        for (var i = static_data.shapes_arrays_metadata.neg_inf_cubes_start; i < static_data.shapes_arrays_metadata.neg_inf_cubes_amount + static_data.shapes_arrays_metadata.neg_inf_cubes_start; i++) {
            d = max(d, -(sd_inf_box(p - negatives_shapes[i].pos, negatives_shapes[i].size.xyz) - negatives_shapes[i].roundness));
        }
    }

    // dynamic negative shapes
    if dyn_negative_intersected {
        for (var i = dynamic_data.shapes_arrays_metadata.neg_cubes_start; i < dynamic_data.shapes_arrays_metadata.neg_cubes_amount + dynamic_data.shapes_arrays_metadata.neg_cubes_start; i++) {
            d = max(d, -(sd_box(p - dyn_negatives_shapes[i].pos, dyn_negatives_shapes[i].size) - dyn_negatives_shapes[i].roundness));
        }
        for (var i = dynamic_data.shapes_arrays_metadata.neg_spheres_start; i < dynamic_data.shapes_arrays_metadata.neg_spheres_amount + dynamic_data.shapes_arrays_metadata.neg_spheres_start; i++) {
            d = max(d, -(sd_sphere(p - dyn_negatives_shapes[i].pos, dyn_negatives_shapes[i].size.x) - dyn_negatives_shapes[i].roundness));
        }
        for (var i = dynamic_data.shapes_arrays_metadata.neg_sph_cubes_start; i < dynamic_data.shapes_arrays_metadata.neg_sph_cubes_amount + dynamic_data.shapes_arrays_metadata.neg_sph_cubes_start; i++) {
            d = max(d, -(sd_sph_box(p - dyn_negatives_shapes[i].pos, dyn_negatives_shapes[i].size) - dyn_negatives_shapes[i].roundness));
        }
        for (var i = dynamic_data.shapes_arrays_metadata.neg_inf_cubes_start; i < dynamic_data.shapes_arrays_metadata.neg_inf_cubes_amount + dynamic_data.shapes_arrays_metadata.neg_inf_cubes_start; i++) {
            d = max(d, -(sd_inf_box(p - dyn_negatives_shapes[i].pos, dyn_negatives_shapes[i].size.xyz) - dyn_negatives_shapes[i].roundness));
        }
    }

    if player_forms_intersected {
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

    
    if w_plane_intersected {
        d = min(d, p.w - static_data.w_floor);
    }

    // if static_data.is_w_roof_exist == 1 {
    //     if w_plane_intersected == 1 {
    //         d = min(d, static_data.w_roof - p.w);
    //     }
    // }

    return d;
}


fn get_mats(
    cam_pos: vec4<f32>,
    ray_dir: vec4<f32>,
    dist: f32,
) -> OutputMaterials {
    var output: OutputMaterials;

    if dist > MAX_DIST-MIN_DIST {
        output.materials_count = 1u;
        output.material_weights[0] = 1.0;
        output.materials[0] = -1;
        return output;
    }

    let p = cam_pos + ray_dir * dist;

    // intersected shapes metadata
    output.materials_count = 0u;
    
    for (var i = 0u; i < dynamic_data.player_forms_amount; i++) {
        
        let shape = dyn_player_forms[i];
        
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
    if st_noramls_intersected {
        for (var i = static_data.shapes_arrays_metadata.cubes_start; i < static_data.shapes_arrays_metadata.cubes_amount + static_data.shapes_arrays_metadata.cubes_start; i++) {
            let shape = normal_shapes[i];
            if sd_box(p - shape.pos, shape.size) - shape.roundness < MIN_DIST*2.0 {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = shape.material;
                return output;
            }
        }
        for (var i = static_data.shapes_arrays_metadata.spheres_start; i < static_data.shapes_arrays_metadata.spheres_amount + static_data.shapes_arrays_metadata.spheres_start; i++) {
            let shape = normal_shapes[i];
            if sd_sphere(p - shape.pos, shape.size.x) - shape.roundness < MIN_DIST*2.0 {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = shape.material;
                return output;
            }
        }
        for (var i = static_data.shapes_arrays_metadata.sph_cubes_start; i < static_data.shapes_arrays_metadata.sph_cubes_amount + static_data.shapes_arrays_metadata.sph_cubes_start; i++) {
            let shape = normal_shapes[i];
            if sd_sph_box(p - shape.pos, shape.size) - shape.roundness < MIN_DIST*2.0 {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = shape.material;
                return output;
            }
        }
        for (var i = static_data.shapes_arrays_metadata.inf_cubes_start; i < static_data.shapes_arrays_metadata.inf_cubes_amount + static_data.shapes_arrays_metadata.inf_cubes_start; i++) {
            let shape = normal_shapes[i];
            if sd_inf_box(p - shape.pos, shape.size.xyz) - shape.roundness < MIN_DIST*2.0 {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = shape.material;
                return output;
            }
        }
    }

    // dynamic normal shapes
    if dyn_noramls_intersected {
        for (var i = dynamic_data.shapes_arrays_metadata.cubes_start; i < dynamic_data.shapes_arrays_metadata.cubes_amount + dynamic_data.shapes_arrays_metadata.cubes_start; i++) {
            let shape = dyn_normal_shapes[i];
            if sd_box(p - shape.pos, shape.size) - shape.roundness < MIN_DIST*2.0 {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = shape.material;
                return output;
            }
        }
        for (var i = dynamic_data.shapes_arrays_metadata.spheres_start; i < dynamic_data.shapes_arrays_metadata.spheres_amount + dynamic_data.shapes_arrays_metadata.spheres_start; i++) {
            let shape = dyn_normal_shapes[i];
            if sd_sphere(p - shape.pos, shape.size.x) - shape.roundness < MIN_DIST*2.0 {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = shape.material;
                return output;
            }
        }
        for (var i = dynamic_data.shapes_arrays_metadata.sph_cubes_start; i < dynamic_data.shapes_arrays_metadata.sph_cubes_amount + dynamic_data.shapes_arrays_metadata.sph_cubes_start; i++) {
            let shape = dyn_normal_shapes[i];
            if sd_sph_box(p - shape.pos, shape.size) - shape.roundness < MIN_DIST*2.0 {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = shape.material;
                return output;
            }
        }
        for (var i = dynamic_data.shapes_arrays_metadata.inf_cubes_start; i < dynamic_data.shapes_arrays_metadata.inf_cubes_amount + dynamic_data.shapes_arrays_metadata.inf_cubes_start; i++) {
            let shape = dyn_normal_shapes[i];
            if sd_inf_box(p - shape.pos, shape.size.xyz) - shape.roundness < MIN_DIST*2.0 {
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = shape.material;
                return output;
            }
        }
    }

    

    
    // static stickiness shapes
    var d = MAX_DIST * 2.0;
    if st_stickiness_intersected {
        for (var i = static_data.shapes_arrays_metadata.s_cubes_start; i < static_data.shapes_arrays_metadata.s_cubes_amount + static_data.shapes_arrays_metadata.s_cubes_start; i++) {
            let shape = stickiness_shapes[i];
            let dd = sd_box(p - shape.pos, shape.size) - shape.roundness;
            
            if dd < MIN_DIST*2.0 {
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
                    var coef = 0.0;
                    if d<dd {
                        coef = clamp(pow(d/dd,1.9) * 0.5, 0.0, 1.0);
                    } else {
                        coef = 1.0-clamp((pow(dd/d,1.9) * 0.5), 0.0, 1.0);
                    }
                    output.materials[output.materials_count] = shape.material;
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
        for (var i = static_data.shapes_arrays_metadata.s_spheres_start; i < static_data.shapes_arrays_metadata.s_spheres_amount + static_data.shapes_arrays_metadata.s_spheres_start; i++) {
            let shape = stickiness_shapes[i];
            let dd = sd_sphere(p - shape.pos, shape.size.x) - shape.roundness;
            
            if dd < MIN_DIST*2.0 {
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
                    var coef = 0.0;
                    if d<dd {
                        coef = clamp(pow(d/dd,1.9) * 0.5, 0.0, 1.0);
                    } else {
                        coef = 1.0-clamp((pow(dd/d,1.9) * 0.5), 0.0, 1.0);
                    }
                    output.materials[output.materials_count] = shape.material;
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
        for (var i = static_data.shapes_arrays_metadata.s_sph_cubes_start; i < static_data.shapes_arrays_metadata.s_sph_cubes_amount + static_data.shapes_arrays_metadata.s_sph_cubes_start; i++) {
            let shape = stickiness_shapes[i];
            let dd = sd_sph_box(p - shape.pos, shape.size) - shape.roundness;
            
            if dd < MIN_DIST*2.0 {
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
                    var coef = 0.0;
                    if d<dd {
                        coef = clamp(pow(d/dd,1.9) * 0.5, 0.0, 1.0);
                    } else {
                        coef = 1.0-clamp((pow(dd/d,1.9) * 0.5), 0.0, 1.0);
                    }
                    output.materials[output.materials_count] = shape.material;
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
        for (var i = static_data.shapes_arrays_metadata.s_inf_cubes_start; i < static_data.shapes_arrays_metadata.s_inf_cubes_amount + static_data.shapes_arrays_metadata.s_inf_cubes_start; i++) {
            let shape = stickiness_shapes[i];
            let dd = sd_inf_box(p - shape.pos, shape.size.xyz) - shape.roundness;
            
            if dd < MIN_DIST*2.0 {
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
                    var coef = 0.0;
                    if d<dd {
                        coef = clamp(pow(d/dd,1.9) * 0.5, 0.0, 1.0);
                    } else {
                        coef = 1.0-clamp((pow(dd/d,1.9) * 0.5), 0.0, 1.0);
                    }
                    output.materials[output.materials_count] = shape.material;
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

    // dynamic stickiness
    if dyn_stickiness_intersected {
        for (var i = dynamic_data.shapes_arrays_metadata.s_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_cubes_amount + dynamic_data.shapes_arrays_metadata.s_cubes_start; i++) {
            let shape = dyn_stickiness_shapes[i];
            let dd = sd_box(p - shape.pos, shape.size) - shape.roundness;
            
            if dd < MIN_DIST*2.0 {
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
                    var coef = 0.0;
                    if d<dd {
                        coef = clamp(pow(d/dd,1.9) * 0.5, 0.0, 1.0);
                    } else {
                        coef = 1.0-clamp((pow(dd/d,1.9) * 0.5), 0.0, 1.0);
                    }
                    output.materials[output.materials_count] = shape.material;
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
        for (var i = dynamic_data.shapes_arrays_metadata.s_spheres_start; i < dynamic_data.shapes_arrays_metadata.s_spheres_amount + dynamic_data.shapes_arrays_metadata.s_spheres_start; i++) {
            let shape = dyn_stickiness_shapes[i];
            let dd = sd_sphere(p - shape.pos, shape.size.x) - shape.roundness;
            
            if dd < MIN_DIST*2.0 {
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
                    var coef = 0.0;
                    if d<dd {
                        coef = clamp(pow(d/dd,1.9) * 0.5, 0.0, 1.0);
                    } else {
                        coef = 1.0-clamp((pow(dd/d,1.9) * 0.5), 0.0, 1.0);
                    }
                    output.materials[output.materials_count] = shape.material;
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
        for (var i = dynamic_data.shapes_arrays_metadata.s_sph_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_sph_cubes_amount + dynamic_data.shapes_arrays_metadata.s_sph_cubes_start; i++) {
            let shape = dyn_stickiness_shapes[i];
            let dd = sd_sph_box(p - shape.pos, shape.size) - shape.roundness;
            
            if dd < MIN_DIST*2.0 {
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
                    var coef = 0.0;
                    if d<dd {
                        coef = clamp(pow(d/dd,1.9) * 0.5, 0.0, 1.0);
                    } else {
                        coef = 1.0-clamp((pow(dd/d,1.9) * 0.5), 0.0, 1.0);
                    }
                    output.materials[output.materials_count] = shape.material;
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
        for (var i = dynamic_data.shapes_arrays_metadata.s_inf_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_inf_cubes_amount + dynamic_data.shapes_arrays_metadata.s_inf_cubes_start; i++) {
            let shape = dyn_stickiness_shapes[i];
            let dd = sd_inf_box(p - shape.pos, shape.size.xyz) - shape.roundness;
            
            if dd < MIN_DIST*2.0 {
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
                    var coef = 0.0;
                    if d<dd {
                        coef = clamp(pow(d/dd,1.9) * 0.5, 0.0, 1.0);
                    } else {
                        coef = 1.0-clamp((pow(dd/d,1.9) * 0.5), 0.0, 1.0);
                    }
                    output.materials[output.materials_count] = shape.material;
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

    // w plane
    if w_plane_intersected {
        if p.w - static_data.w_floor < MIN_DIST*2.0 {
            output.materials_count = 1u;
            output.material_weights[0] = 1.0;
            output.materials[0] = -2;
            return output;
        }
    }
    
    if output.materials_count == 0u {
        
        // for case steps in raymarch is maximum
        if w_plane_intersected {
            output.materials_count = 1u;
            output.material_weights[0] = 1.0;
            output.materials[0] = -2;
            return output;
        }
        output.materials_count = 1u;
        output.material_weights[0] = 1.0;
        output.materials[0] = -1;
    }

    return output;
}


fn get_normal(p: vec4<f32>) -> vec4<f32> {
    var h: vec3<f32> = vec3<f32>(0.001, -0.001, 0.0);
    
    var a: vec4<f32> = p + h.yxxz;
    var b: vec4<f32> = p + h.xyxz;
    var c: vec4<f32> = p + h.xxyz;
    var d: vec4<f32> = p + h.yyyz;
    var e: vec4<f32> = p + h.zzzx;
    var f: vec4<f32> = p + h.zzzy;

    var fa: f32 = map(a);
    var fb: f32 = map(b);
    var fc: f32 = map(c);
    var fd: f32 = map(d);
    var fe: f32 = map(e);
    var ff: f32 = map(f);

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

fn ray_march(ray_origin_base: vec4<f32>, ray_direction: vec4<f32>, offset: f32) -> vec2<f32>  {
    
    if offset > MAX_DIST {
        return vec2(MAX_DIST, 0.0);
    }

    var total_distance: f32 = offset;
    
    var ray_origin = ray_origin_base + ray_direction*offset;

    var i: i32 = 0;
    for (; i < MAX_STEPS; i++) {
        var d: f32  = map(ray_origin);
        total_distance += d;


        if (d < MIN_DIST) {
            // color.x = 1.;
            return vec2<f32>(total_distance, f32(i));
        }
        if (total_distance > MAX_DIST) {
            // color.y = 1.;
            return vec2<f32>(MAX_DIST*2.0, f32(i));
        }

        ray_origin += ray_direction * d;

        // if is_outside_of_bouding_box(ray_origin) {
        //     return vec2<f32>(MAX_DIST*2.0, f32(i));
        // }
    }
    return vec2<f32>(total_distance, f32(i));
}

fn is_outside_of_bouding_box(p: vec4<f32>) -> bool {
    if p.x > dynamic_data.bb_pos_side.x ||
       p.y > dynamic_data.bb_pos_side.y ||
       p.z > dynamic_data.bb_pos_side.z ||
       p.w > dynamic_data.bb_pos_side.w ||
       p.x < dynamic_data.bb_neg_side.x ||
       p.y < dynamic_data.bb_neg_side.y ||
       p.z < dynamic_data.bb_neg_side.z ||
       p.w < dynamic_data.bb_neg_side.w
    {
        return true;
    }
    return false;

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

fn get_soft_shadow( ro: vec4<f32>, rd: vec4<f32>) -> f32
{
    var res = 1.0;

    let tmax = 12.0;
    
    var t = 0.02;
    for( var i=0; i<50; i++ )
    {
		var h = not_opt_map( ro + rd*t);
        res = min(res, 16.0*h/t,);
        t += clamp( h, 0.05, 0.40 );
        if( res<0.005 || t>tmax ) {
            break;
        }
    }
    return clamp( res, 0.0, 1.0 );
}


fn get_shadow(ray_origin_base: vec4<f32>, ray_direction: vec4<f32>) -> f32 {

    let offset = find_intersections(ray_origin_base, ray_direction);
    
    if offset > MAX_DIST {
        return 1.0;
    }

    var total_distance: f32 = offset;
    
    var ray_origin = ray_origin_base + ray_direction*offset;

    var i: i32 = 0;
    for (; i < MAX_STEPS; i++) {
        var d: f32  = map(ray_origin);
        total_distance += d;

        if (d < 0.) {
            return 0.0;
        }
        if (d < MIN_DIST) {
            return 0.0;
        }
        if (total_distance > MAX_DIST) {
            return 1.0;
        }

        ray_origin += ray_direction * d;

        // if is_outside_of_bouding_box(ray_origin) {
        //     return 1.0;
        // }
    }
    return 1.0;
}

fn calc_ambient_occlusion( pos: vec4<f32>, nor: vec4<f32>) -> f32
{
	var occ = 0.0;
    var sca = 1.0;
    for(var i = 0; i<5; i++)
    {
        let h = 0.01 + 0.14*f32(i)/4.0;
        let d = not_opt_map(pos+h*nor);
        occ += (h-d)*sca;
        sca *= 0.95;
    }
    return clamp( 1.0 - 3.0*occ, 0.0, 1.0 );
}


fn hash( n: f32 ) -> f32
{
    return fract(sin(n)*43758.5453123);
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


const HORIZONT_COLOR: vec3<f32> = vec3(0.4, 0.8, 2.1);
// const HORIZONT_COLOR: vec3<f32> = vec3(2.1, 0.5, 0.1);
const SKY_COLOR: vec3<f32> = vec3(0.17, 0.14, 0.42)*0.4;
const FOG_COLOR: vec3<f32> = vec3(0.001, 0.01, 0.002);
const LINES_COLOR_1: vec3<f32> = vec3(0.2, 1.0, 3.5);
const LINES_COLOR_2: vec3<f32> = vec3(2.5, 0.2, 1.0);
const FRENEL_COLOR: vec3<f32> = vec3(0.3,0.3,0.7);
const BOUND_COLOR: vec3<f32> = vec3(0.2,0.8,0.2);
const W_PLANE_COLOR: vec3<f32> = vec3(0.2, 0.4, 1.1);

const SUN_COLOR_1: vec3<f32> = vec3(5.6, 1.8, 1.8)*0.9;
const SUN_DIR_1: vec4<f32> = vec4(1.0, 2.4, -2.3, 0.0);

const SUN_COLOR_2: vec3<f32> = vec3(1.2, 1.8, 2.6);
const SUN_DIR_2: vec4<f32> = vec4(0.5, 1.0, 0.4, 0.5);


fn apply_material(
    pos: vec4<f32>,
    ray_dir: vec4<f32>,
    dist: f32,
    material: i32,
) -> vec3<f32> {
    
    // sky
    if material == -2 {

        let normal = vec4(0.0, 0.0, 0.0, 1.0);

        let frenel = clamp(pow(1.0 + dot(normal, ray_dir), 5.0), 0.0, 1.0);

        var color = W_PLANE_COLOR*frenel;

        return color;
    }
    if material == -1 {
        var color =  2.9*SKY_COLOR*FOG_COLOR* mix(vec3(.1,0.2,0.55), vec3(0.1,1.2,2.4), sqrt(abs(ray_dir.y)+0.1));
        color = mix(HORIZONT_COLOR*0.12, color, sqrt(clamp(abs(ray_dir.y*2.0)+0.1,0.0,1.0)));
        // var color =  1.5*SKY_COLOR* mix(vec3(.3,0.0,0.05), SUN_COLOR_1*SKY_COLOR, sqrt(max(ray_dir.y, 0.001)));

        let sun = pow(clamp(dot(normalize(SUN_DIR_1),ray_dir), 0.0, 1.0), 5.0);
    	color += 0.4*vec3(.4,.2,0.67)*sun;
        color += SUN_COLOR_1*pow(sun, 20.0);

        let v = 1.0/( 2. * ( 1. + ray_dir.z ) );
        let xy = vec2(ray_dir.y * v, ray_dir.x * v);
        // ray_dir.z += time*.002;
        var s = noise(ray_dir.xz*134.0);
        s += noise(ray_dir.xz*270.);
        s += noise(ray_dir.xz*170.);
        s = pow(s,19.0) * 0.00000005 * abs(ray_dir.y);
        if (s > 0.0)
        {
            let backStars = vec3((1.0-sin(xy.x*20.0+13.0*ray_dir.x+xy.y*30.0))*.5*s,s, s); 
            color += backStars;
        }

        // color *= vec3
        //     (cos(ray_dir.y * 7.0 + dynamic_data.time) +
        //      sin(ray_dir.x * 7.0 + dynamic_data.time) +
        //      sin(ray_dir.z * 7.0 + dynamic_data.time),
        //      sin(ray_dir.y * 2.0 + dynamic_data.time) +
        //      cos(ray_dir.x * 2.0 + dynamic_data.time) +
        //      sin(ray_dir.z * 2.0 + dynamic_data.time),
        //      0.0,
        //      );
        
        color = clamp(color, vec3(0.0), vec3(1.0));

        // color = mix(color, FOG_COLOR, 1.0-ray_dir.y);




        color = clamp(color, vec3(0.0), vec3(1.0));

        return color;
    }

    let hited_pos = pos + ray_dir * dist;
    let normal = get_normal(hited_pos);
    let next_normal = get_normal(hited_pos+ray_dir*MIN_DIST*4.8);
    // let aocc = calc_ambient_occlusion(hited_pos, normal);

    let wireframe_fog = exp(-0.007*dist*dist);
    let wireframe_dif = clamp(1.0-abs(dot(normal, next_normal)),0.0,1.0);

    // sun light 1
    let sun_dir_1 = normalize(SUN_DIR_1);
    let sun_dif_1 = clamp(dot(normal, sun_dir_1),0.0,1.0);
    let sun_hal_1 = normalize(sun_dir_1-ray_dir);
    let sun_spe_1 = pow(clamp(dot(normal,sun_hal_1),0.0,1.0),20.0);
    let sun_shadow_1 = get_shadow(hited_pos + normal*MIN_DIST*2.0, sun_dir_1);

    // sun light 2
    // let sun_dir_2 = normalize(SUN_DIR_2);
    // let sun_dif_2 = clamp(dot(normal, sun_dir_2),0.0,1.0);
    // let sun_hal_2 = normalize(sun_dir_2-ray_dir);
    // let sun_spe_2 = pow(clamp(dot(normal,sun_hal_2),0.0,1.0),20.0);
    // let sun_shadow_2 = get_shadow(hited_pos + normal*MIN_DIST*2.0, sun_dir_2);
    // let bound_dif = clamp(0.5 + 0.5*-normal.y, 0.0, 1.0);

    // frenel reflection
    let reflection = reflect(ray_dir, normal);
    let frenel = smoothstep(0.0, 2.0,clamp(1.0 + dot(normal, ray_dir), 0.0, 1.0));

    // sky light    
    let sky_dif = clamp(0.5 + 0.5*normal.y,0.0,1.0);
    let sky_hal = normalize(vec4(0.0,1.0,0.0,0.0)-ray_dir);
    let sky_spe = pow(clamp(dot(normal,sky_hal),0.0,1.0),3.0);

    // let lines_dif = abs(0.5-fract((hited_pos.z+hited_pos.y+hited_pos.w)*0.1)) +
    //     abs(0.5-fract((hited_pos.z-hited_pos.y+hited_pos.w)*0.1)) +
    //     abs(0.5-fract((hited_pos.x+hited_pos.y+hited_pos.w)*0.1));
    // let lines_dif = 
    //     // (1.0-
    //     // clamp(
    //     //     // abs(
    //     //         0.5 + 0.5*0.33333*(
    //     //             sin(hited_pos.x*5.0) + 
    //     //             sin(hited_pos.z*5.0) +
    //     //             sin(hited_pos.y*5.0)
    //     //             // sin(hited_pos.w*50.0)
    //     //         ),
    //     //     // ),
    //     //     0.0,1.0
    //     // ));// +
    //     (1.0-
    //     clamp(
    //         abs(
    //             3.5*(
    //                 sin(hited_pos.x*2.0+4.0) +
    //                 sin(hited_pos.z*2.0+3.0) +
    //                 sin(hited_pos.y*2.0+2.0) +
    //                 sin(hited_pos.w*2.0+1.0)
    //             )
    //         ),
    //         0.0,1.0
    //     ));


    var light = vec3(0.0);


    light += SUN_COLOR_1  * sun_dif_1 * sun_shadow_1 * 0.72;// * aocc;
    light += SUN_COLOR_1  * sun_dif_1 * sun_spe_1 * sun_shadow_1 * 2.0;// * aocc;
    light += SKY_COLOR    * sky_dif   * 4.8 * clamp(sky_spe, 0.25, 1.0);// * 0.8;// * aocc;
    light += FRENEL_COLOR * frenel    * 0.8 * (0.6+0.4*sun_dif_1);// * aocc;
    light += LINES_COLOR_1* wireframe_dif*20.0 * (0.1+0.9*sun_dif_1*sun_shadow_1) * wireframe_fog; 
    
    // light += mix(LINES_COLOR_1, LINES_COLOR_2, clamp(hited_pos.w/10.0,0.0,1.0)) * 2.8*pow(lines_dif, 5.8);

    // light += SUN_COLOR_2 * sun_dif_2 * sun_shadow_2 * 0.6;// * aocc;
    // light += SUN_COLOR_2 * sun_dif_2 * sun_spe_2 * sun_shadow_2 * 4.0;// * aocc;
    
    // let diffuse = clamp(static_data.materials[material].color.xyz+vec3(10.0)*pow(lines_dif,25.0),vec3(0.0),vec3(1.0));
    let diffuse = static_data.materials[material].color.xyz + LINES_COLOR_1 * pow(wireframe_dif,2.5)*10.0*(0.1+0.9*wireframe_fog);
    var color = diffuse * light;

    color = clamp(color, vec3(0.0), vec3(1.0));

    let air_perspective = clamp(1.0-exp(-0.00002*dist*dist*dist),0.2,1.0);

    color = mix(color, SKY_COLOR, air_perspective);
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

    let camera_position = dynamic_data.camera_data.cam_pos;

    let offset = find_intersections(camera_position, ray_direction);

    let dist_and_depth: vec2<f32> = ray_march(camera_position, ray_direction, offset); 

    // var color = vec3(0.0);
    // color.b = dist_and_depth.x/MAX_DIST;
    // color.g = dist_and_depth.y/f32(MAX_STEPS);
    // color.r = offset/MAX_DIST;

    var mats = get_mats(camera_position, ray_direction, dist_and_depth.x);

    var color = apply_material(camera_position, ray_direction, dist_and_depth.x, mats.materials[0]);

    for (var i = 1u; i < mats.materials_count; i++) {

        let new_color = apply_material(camera_position, ray_direction, dist_and_depth.x, mats.materials[i]);

        color = mix(color, new_color, mats.material_weights[i]);
    }

    color += 0.145*get_coloring_areas_color(camera_position + ray_direction * dist_and_depth.x);
    color += 0.6*get_volume_areas_color(camera_position, ray_direction, dist_and_depth.x);

    color = pow(color, vec3(0.4545));
    color += (0.007 - clamp(length(uv), 0.0, 0.007))*1000.0;

    return vec4<f32>(color, 1.0);
}