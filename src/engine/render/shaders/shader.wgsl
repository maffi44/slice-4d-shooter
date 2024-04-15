// Fragment shader

struct CameraUniform {
    cam_pos: vec4<f32>,
    cam_rot: mat4x4<f32>,
}


struct Shape {
    pos: vec4<f32>,
    size: vec4<f32>,
    color: vec3<f32>,
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

struct OtherDynamicData {
    shapes_arrays_metadata: ShapesMetadata,
    spherical_areas_meatadata: SphericalAreasMetadata,
    camera_data: CameraUniform,
    empty_bytes1: vec3<u32>,
    beam_areas_amount: u32,
    player_forms_amount: u32,
    // empty_bytes2: vec4<f32>,
    // explore_w_pos: f32,
    // explore_w_coef: f32,
    stickiness: f32,
    screen_aspect: f32,
    time: f32,
}

struct OtherStaticData {
    shapes_arrays_metadata: ShapesMetadata,
    
    is_w_floor_exist: i32,
    w_floor: f32,
    is_w_roof_exist: i32,
    w_roof: f32,

    empty_bytes: vec3<f32>,
    stickiness: f32
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

fn smin(a: f32, b: f32, k: f32) -> f32
{
    let x = (b-a)/k;
    let g = 0.5*(x-sqrt(x*x+0.25));
    return a + k * g;
}

fn get_color(start_pos: vec4<f32>, direction: vec4<f32>, distance: f32, ray_w_rotated: i32) -> vec3<f32> {
    let point = start_pos + direction * distance;
    
    var color = get_color_at_point(point, distance, ray_w_rotated);

    // color += get_coloring_areas_color(point); 

    return color;
}

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

            color = mix(sphere.color, vec3(1.0), pow(color_coef, 40.5)) * pow(color_coef, 4.0) + vec3(0.05);

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

            color = mix(beam.color, vec3(1.0), pow(color_coef, 40.5)) * pow(color_coef, 4.0);

            break;
        }
        total_dist += d;

        p += direction * d;
    }

    return color;
}

fn get_color_at_point(p: vec4<f32>, distance: f32, ray_w_rotated: i32) -> vec3<f32> {

    var d = MAX_DIST;
    var color = vec3(0.0, 0.0, 0.0);

    // static stickiness shapes
    for (var i = static_data.shapes_arrays_metadata.s_cubes_start; i < static_data.shapes_arrays_metadata.s_cubes_amount + static_data.shapes_arrays_metadata.s_cubes_start; i++) {
        let new_d = sd_box(p - stickiness_shapes[i].pos, stickiness_shapes[i].size) - stickiness_shapes[i].roundness;
        
        let dd = smin(d, new_d, static_data.stickiness);

        let coef = clamp((new_d - d) / (dd - d), 0.0, 1.0);

        color = mix(color, stickiness_shapes[i].color, coef);

        d = dd;
    }
    for (var i = static_data.shapes_arrays_metadata.s_spheres_start; i < static_data.shapes_arrays_metadata.s_spheres_amount + static_data.shapes_arrays_metadata.s_spheres_start; i++) {
        let new_d = sd_sphere(p - stickiness_shapes[i].pos, stickiness_shapes[i].size.x) - stickiness_shapes[i].roundness;
        
        let dd = smin(d, new_d, static_data.stickiness);

        let coef = clamp((new_d - d) / (dd - d), 0.0, 1.0);

        color = mix(color, stickiness_shapes[i].color, coef);

        d = dd;
    }
    for (var i = static_data.shapes_arrays_metadata.s_sph_cubes_start; i < static_data.shapes_arrays_metadata.s_sph_cubes_amount + static_data.shapes_arrays_metadata.s_sph_cubes_start; i++) {
        let new_d = sd_sph_box(p - stickiness_shapes[i].pos, stickiness_shapes[i].size) - stickiness_shapes[i].roundness;
        
        let dd = smin(d, new_d, static_data.stickiness);

        let coef = clamp((new_d - d) / (dd - d), 0.0, 1.0);

        color = mix(color, stickiness_shapes[i].color, coef);

        d = dd;
    }
    for (var i = static_data.shapes_arrays_metadata.s_inf_cubes_start; i < static_data.shapes_arrays_metadata.s_inf_cubes_amount + static_data.shapes_arrays_metadata.s_inf_cubes_start; i++) {
        let new_d = sd_inf_box(p - stickiness_shapes[i].pos, stickiness_shapes[i].size.xyz) - stickiness_shapes[i].roundness;
        
        let dd = smin(d, new_d, static_data.stickiness);

        let coef = clamp((new_d - d) / (dd - d), 0.0, 1.0);

        color = mix(color, stickiness_shapes[i].color, coef);

        d = dd;
    }

    // dynamic stickiness shapes
    for (var i = dynamic_data.shapes_arrays_metadata.s_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_cubes_amount + dynamic_data.shapes_arrays_metadata.s_cubes_start; i++) {
        let new_d = sd_box(p - dyn_stickiness_shapes[i].pos, dyn_stickiness_shapes[i].size) - dyn_stickiness_shapes[i].roundness;
        
        let dd = smin(d, new_d, static_data.stickiness);

        let coef = clamp((new_d - d) / (dd - d), 0.0, 1.0);

        color = mix(color, dyn_stickiness_shapes[i].color, coef);

        d = dd;
    }
    for (var i = dynamic_data.shapes_arrays_metadata.s_spheres_start; i < dynamic_data.shapes_arrays_metadata.s_spheres_amount + dynamic_data.shapes_arrays_metadata.s_spheres_start; i++) {
        let new_d = sd_sphere(p - dyn_stickiness_shapes[i].pos, dyn_stickiness_shapes[i].size.x) - dyn_stickiness_shapes[i].roundness;
        
        let dd = smin(d, new_d, static_data.stickiness);

        let coef = clamp((new_d - d) / (dd - d), 0.0, 1.0);

        color = mix(color, dyn_stickiness_shapes[i].color, coef);

        d = dd;
    }
    for (var i = dynamic_data.shapes_arrays_metadata.s_sph_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_sph_cubes_amount + dynamic_data.shapes_arrays_metadata.s_sph_cubes_start; i++) {
        let new_d = sd_sph_box(p - dyn_stickiness_shapes[i].pos, dyn_stickiness_shapes[i].size) - dyn_stickiness_shapes[i].roundness;
        
        let dd = smin(d, new_d, static_data.stickiness);

        let coef = clamp((new_d - d) / (dd - d), 0.0, 1.0);

        color = mix(color, dyn_stickiness_shapes[i].color, coef);

        d = dd;
    }
    for (var i = dynamic_data.shapes_arrays_metadata.s_inf_cubes_start; i < dynamic_data.shapes_arrays_metadata.s_inf_cubes_amount + dynamic_data.shapes_arrays_metadata.s_inf_cubes_start; i++) {
        let new_d = sd_inf_box(p - dyn_stickiness_shapes[i].pos, dyn_stickiness_shapes[i].size.xyz) - dyn_stickiness_shapes[i].roundness;
        
        let dd = smin(d, new_d, static_data.stickiness);

        let coef = clamp((new_d - d) / (dd - d), 0.0, 1.0);

        color = mix(color, dyn_stickiness_shapes[i].color, coef);

        d = dd;
    }

    // static normal shapes
    for (var i = static_data.shapes_arrays_metadata.cubes_start; i < static_data.shapes_arrays_metadata.cubes_amount + static_data.shapes_arrays_metadata.cubes_start; i++) {
        let new_d = sd_box(p - normal_shapes[i].pos, normal_shapes[i].size) - normal_shapes[i].roundness;

        if new_d < d {
            color = normal_shapes[i].color;
            d = new_d;
        }
    }
    for (var i = static_data.shapes_arrays_metadata.spheres_start; i < static_data.shapes_arrays_metadata.spheres_amount + static_data.shapes_arrays_metadata.spheres_start; i++) {
        let new_d = sd_sphere(p - normal_shapes[i].pos, normal_shapes[i].size.x) - normal_shapes[i].roundness;

        if new_d < d {
            color = normal_shapes[i].color;
            d = new_d;
        }
    }
    for (var i = static_data.shapes_arrays_metadata.sph_cubes_start; i < static_data.shapes_arrays_metadata.sph_cubes_amount + static_data.shapes_arrays_metadata.sph_cubes_start; i++) {
        let new_d = sd_sph_box(p - normal_shapes[i].pos, normal_shapes[i].size) - normal_shapes[i].roundness;

        if new_d < d {
            color = normal_shapes[i].color;
            d = new_d;
        }
    }
    for (var i = static_data.shapes_arrays_metadata.inf_cubes_start; i < static_data.shapes_arrays_metadata.inf_cubes_amount + static_data.shapes_arrays_metadata.inf_cubes_start; i++) {
        let new_d = sd_inf_box(p - normal_shapes[i].pos, normal_shapes[i].size.xyz) - normal_shapes[i].roundness;

        if new_d < d {
            color = normal_shapes[i].color;
            d = new_d;
        }
    }

    // dynamic normal shapes
    for (var i = dynamic_data.shapes_arrays_metadata.cubes_start; i < dynamic_data.shapes_arrays_metadata.cubes_amount + dynamic_data.shapes_arrays_metadata.cubes_start; i++) {
        let new_d = sd_box(p - dyn_normal_shapes[i].pos, dyn_normal_shapes[i].size) - dyn_normal_shapes[i].roundness;

        if new_d < d {
            color = dyn_normal_shapes[i].color;
            d = new_d;
        }
    }
    for (var i = dynamic_data.shapes_arrays_metadata.spheres_start; i < dynamic_data.shapes_arrays_metadata.spheres_amount + dynamic_data.shapes_arrays_metadata.spheres_start; i++) {
        let new_d = sd_sphere(p - dyn_normal_shapes[i].pos, dyn_normal_shapes[i].size.x) - dyn_normal_shapes[i].roundness;

        if new_d < d {
            color = dyn_normal_shapes[i].color;
            d = new_d;
        }
    }
    for (var i = dynamic_data.shapes_arrays_metadata.sph_cubes_start; i < dynamic_data.shapes_arrays_metadata.sph_cubes_amount + dynamic_data.shapes_arrays_metadata.sph_cubes_start; i++) {
        let new_d = sd_sph_box(p - dyn_normal_shapes[i].pos, dyn_normal_shapes[i].size) - dyn_normal_shapes[i].roundness;

        if new_d < d {
            color = dyn_normal_shapes[i].color;
            d = new_d;
        }
    }
    for (var i = dynamic_data.shapes_arrays_metadata.inf_cubes_start; i < dynamic_data.shapes_arrays_metadata.inf_cubes_amount + dynamic_data.shapes_arrays_metadata.inf_cubes_start; i++) {
        let new_d = sd_inf_box(p - dyn_normal_shapes[i].pos, dyn_normal_shapes[i].size.xyz) - dyn_normal_shapes[i].roundness;

        if new_d < d {
            color = dyn_normal_shapes[i].color;
            d = new_d;
        }
    }

    d = MIN_DIST + 0.003;

    for (var i = 0u; i < dynamic_data.player_forms_amount; i++) {
        var new_d = sd_sphere(p - dyn_player_forms[i].pos, dyn_player_forms[i].radius);
        new_d = max(new_d, -sd_sphere(p - dyn_player_forms[i].pos, dyn_player_forms[i].radius * 0.86));
        
        let rotated_p = dyn_player_forms[i].rotation * (p - dyn_player_forms[i].pos);
        new_d = max(new_d, -sd_box(
            rotated_p,
            vec4(
                dyn_player_forms[i].radius * 0.18,
                dyn_player_forms[i].radius* 1.2,
                dyn_player_forms[i].radius* 1.2,
                dyn_player_forms[i].radius * 1.2
            )));
        
        new_d = max(
            new_d,
            -sd_sphere(
                rotated_p - vec4(0.0, 0.0, -dyn_player_forms[i].radius, 0.0),
                dyn_player_forms[i].radius * 0.53
            )
        );

        if new_d < d {
            d = new_d;

            color = vec3(1.0);
        }

        new_d = sd_sphere(
            p - dyn_player_forms[i].pos,
            dyn_player_forms[i].radius * 0.6
        );

        new_d = max(
            new_d,
            -sd_sphere(
                rotated_p - vec4(0.0, 0.0, -dyn_player_forms[i].radius, 0.0)*0.6,
                dyn_player_forms[i].radius * 0.34
            )
        );

        if new_d < d {
            d = new_d;

            color = dyn_player_forms[i].color;
        }

        new_d = sd_sphere(
            rotated_p - dyn_player_forms[i].weapon_offset,
            dyn_player_forms[i].radius * 0.286,
        );

        new_d = max(
            new_d,
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

        if new_d < d {
            d = new_d;

            color = vec3(1.0);
        }

        new_d = sd_capsule(
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
        );

        new_d = max(
            new_d,
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
        

        if new_d < d {
            d = new_d;
            color = dyn_player_forms[i].color;
        }
    }

    if static_data.is_w_floor_exist == 1 {
        if ray_w_rotated == 1 {
            let new_d = p.w + static_data.w_floor;

            if new_d < d {
                color = vec3(0.2,0.2,0.2);

                d = new_d;
            }
        }
    }

    if static_data.is_w_roof_exist == 1 {
        if ray_w_rotated == 1 {
            let new_d = static_data.w_roof - p.w;

            if new_d < d {
                color = vec3(0.2,0.2,0.2);
            }
        }
    }

    if p.w > 0.0 {
        let w_diff = clamp((p.w / 6.0), 0.0, 1.0);

        color = mix(color, vec3(0.3,0.0,0.1), w_diff);
    }
    if p.w < 0.0 {
        let w_diff = clamp((1.0 / p.w), 0.0, 1.0);

        color = mix(vec3(0.1,0.0,0.5), color, w_diff);
    }

    return color;
}


fn map(p: vec4<f32>, ray_w_rotated: i32) -> f32 {
    var d = MAX_DIST;

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
    
    if static_data.is_w_floor_exist == 1 {
        if ray_w_rotated == 1 {
            d = min(d, p.w + static_data.w_floor);
        }
    }

    if static_data.is_w_roof_exist == 1 {
        if ray_w_rotated == 1 {
            d = min(d, static_data.w_roof - p.w);
        }
    }

    return d;
}

fn get_normal(p: vec4<f32>, ray_w_rotated: i32) -> vec4<f32> {
    var h: vec3<f32> = vec3<f32>(0.001, -0.001, 0.0);
    
    var a: vec4<f32> = p + h.yxxz;
    var b: vec4<f32> = p + h.xyxz;
    var c: vec4<f32> = p + h.xxyz;
    var d: vec4<f32> = p + h.yyyz;
    var e: vec4<f32> = p + h.zzzx;
    var f: vec4<f32> = p + h.zzzy;

    var fa: f32 = map(a, ray_w_rotated);
    var fb: f32 = map(b, ray_w_rotated);
    var fc: f32 = map(c, ray_w_rotated);
    var fd: f32 = map(d, ray_w_rotated);
    var fe: f32 = map(e, ray_w_rotated);
    var ff: f32 = map(f, ray_w_rotated);

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

fn ray_march(ray_origin_base: vec4<f32>, ray_direction: vec4<f32>, ray_w_rotated: i32) -> vec2<f32>  {
    // var color: vec3<f32> = vec3<f32>(0., 0., 0.);
    var total_distance: f32 = 0.;
    
    var ray_origin = ray_origin_base;

    var i: i32 = 0;
    for (; i < MAX_STEPS; i++) {
        var d: f32  = map(ray_origin, ray_w_rotated);
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
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    var uv: vec2<f32> = in.position.xy * 0.7;
    uv.x *= dynamic_data.screen_aspect;

    var ray_direction: vec4<f32> = normalize(vec4<f32>(uv, -1.0, 0.0));
    ray_direction *= dynamic_data.camera_data.cam_rot;

    var ray_w_rotated: i32 = 1;

    if ray_direction.w < 0.0002 && ray_direction.w > -0.0002{
        ray_w_rotated = 0;
    }

    let camera_position = dynamic_data.camera_data.cam_pos;

    let dist_and_depth: vec2<f32> = ray_march(camera_position, ray_direction, ray_w_rotated); 

    let normal: vec4<f32> = get_normal(dist_and_depth.x * ray_direction + camera_position, ray_w_rotated);

    let shade_coefficient: f32 = dot(normal, normalize(vec4<f32>(0.2, 1., 0.5, 0.1)));

    let shade = mix(0.32, 0.98, shade_coefficient);

    var color = get_color(camera_position, ray_direction, dist_and_depth.x, ray_w_rotated);

    let coloring_color = get_coloring_areas_color(camera_position + ray_direction * dist_and_depth.x);

    color += coloring_color * 0.4;

    color *= shade * 1.2;

    color += coloring_color * 0.4;

    color += get_volume_areas_color(camera_position, ray_direction, dist_and_depth.x);

    color = clamp(color, vec3(0.0), vec3(1.0));

    color = mix(color, vec3<f32>(0.9, 1., 1.0), (dist_and_depth.x*0.4 / (MAX_DIST*0.4)));

    // let point = camera_position + ray_direction * dist_and_depth.x;

    // let w_diff = clamp((1.0 / (point.w - camera_position.w)), 0.0, 1.0);

    // let new_color = mix(vec3(0.3,0.0,0.6), vec3(0.3,0.0,0.6), w_diff);

    // color = mix(vec3(0.5,0.0,0.1), color, w_diff);

    // if dynamic_data.explore_w_pos != 0.0 {

    //     let dist_and_depth_explore: vec2<f32> = ray_march(
    //         camera_position + vec4(0.0, 0.0, 0.0, dynamic_data.explore_w_pos), ray_direction
    //     );

    //     var explore_color = vec3(3.0, 0.1, 6.0);
    //     if dynamic_data.explore_w_pos < 0.0 {
    //         explore_color = vec3(0.1, 0.3, 16.0);
    //     }

    //     explore_color *= dynamic_data.explore_w_coef * dynamic_data.explore_w_coef;

    //     let e_normal: vec4<f32> = get_normal(
    //         dist_and_depth_explore.x * ray_direction +
    //         (camera_position + vec4(0.0, 0.0, 0.0, dynamic_data.explore_w_pos))
    //     );

    //     var e_shade_coefficient: f32 = dot(e_normal, normalize(vec4<f32>(0.2, 1., 0.5, 0.1)));

    //     e_shade_coefficient = clamp(e_shade_coefficient, 0.0, 0.6);

    //     explore_color *= e_shade_coefficient * 2.0;

    //     color = clamp(color, vec3(0.0), vec3(1.0));

    //     color += explore_color;
    //     // if dist_and_depth.x >= dist_and_depth_explore.x {
    //     // } else {
    //     //     color = mix(color, explore_color, 0.3);
    //     // }
    // }

    //crosshair
    color += (0.006 - clamp(length(uv), 0.0, 0.006))*200.0;

    return vec4<f32>(color, 1.0);
}