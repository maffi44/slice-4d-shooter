// Fragment shader

struct CamerUniform {
    cam_pos: vec4<f32>,
    cam_rot: mat4x4<f32>,
    aspect: f32,
}

struct Shape {
    pos: vec4<f32>,
    size: vec4<f32>,
    color: vec3<f32>,
    roundness: f32,
}

struct NegShape {
    pos: vec4<f32>,
    size: vec4<f32>,
    empty_bytes: vec3<f32>,
    roundness: f32,
}

struct StickinessNegShape {
    pos: vec4<f32>,
    size: vec4<f32>,
    roundness: f32,
    stickiness: f32,
}

struct StickinessShape {
    pos: vec4<f32>,
    size: vec4<f32>,
    color: vec3<f32>,
    roundness: f32,
    stickiness: f32,
}

struct ShapesMetadata {
    cubes_amount: u32,
    s_cubes_amount: u32,
    neg_cubes_amount: u32,
    s_neg_cubes_amount: u32,

    spheres_amount: u32,
    s_spheres_amount: u32,
    neg_spheres_amount: u32,
    s_neg_spheres_amount: u32,

    inf_cubes_amount: u32,
    s_inf_cubes_amount: u32,
    neg_inf_cubes_amount: u32,
    s_neg_inf_cubes_amount: u32,

    sph_cubes_amount: u32,
    s_sph_cubes_amount: u32,
    neg_sph_cubes_amount: u32,
    s_neg_sph_cubes_amount: u32,
}

@group(0) @binding(0) var<uniform> camera_uni: CamerUniform;
@group(0) @binding(1) var<uniform> time: vec4<f32>;
@group(0) @binding(2) var<uniform> shapes_metadata: ShapesMetadata;

@group(0) @binding(3) var<uniform> cubes: array<Shape, 256>;
@group(0) @binding(4) var<uniform> s_cubes: array<StickinessShape, 256>;
@group(0) @binding(5) var<uniform> neg_cubes: array<NegShape, 256>;
@group(0) @binding(6) var<uniform> s_neg_cubes: array<StickinessNegShape, 256>;

@group(0) @binding(7) var<uniform> spheres: array<Shape, 256>;
@group(0) @binding(8) var<uniform> s_spheres: array<StickinessShape, 256>;
@group(0) @binding(9) var<uniform> neg_spheres: array<NegShape, 256>;
@group(0) @binding(10) var<uniform> s_neg_spheres: array<StickinessNegShape, 256>;

@group(1) @binding(0) var<uniform> inf_cubes: array<Shape, 256>;
@group(1) @binding(1) var<uniform> s_inf_cubes: array<StickinessShape, 256>;
@group(1) @binding(2) var<uniform> neg_inf_cubes: array<NegShape, 256>;
@group(1) @binding(3) var<uniform> s_neg_inf_cubes: array<StickinessNegShape, 256>;

@group(1) @binding(4) var<uniform> sph_cubes: array<Shape, 256>;
@group(1) @binding(5) var<uniform> s_sph_cubes: array<StickinessShape, 256>;
@group(1) @binding(6) var<uniform> neg_sph_cubes: array<NegShape, 256>;
@group(1) @binding(7) var<uniform> s_neg_sph_cubes: array<StickinessNegShape, 256>;

const MAX_STEPS: i32 = 200;
const PI: f32 = 3.1415926535897;
const MIN_DIST: f32 = 0.01;
const MAX_DIST: f32 = 700.0;    


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
    return (p.x+p.y+p.z+p.w-s)*0.57735027;
}

fn map(p: vec4<f32>) -> f32 {
    var d = MAX_DIST;

    // var p = pos;

    // var coof = (p.z + 10) / 20;

    // coof = fract(coof);

    // p.z = coof * 20 - 10;

    // coof = (p.x + 10) / 20;

    // coof = fract(coof);

    // p.x = coof * 20 - 10;

    
    
    //LAST ONE v


    for (var i = 0u; i < shapes_metadata.cubes_amount; i++) {
        d = min(d, sd_box(p - cubes[i].pos, cubes[i].size));
    }
    for (var i = 0u; i < shapes_metadata.spheres_amount; i++) {
        d = min(d, sd_sphere(p - spheres[i].pos, spheres[i].size.x));
    }
    for (var i = 0u; i < shapes_metadata.sph_cubes_amount; i++) {
        d = min(d, sd_sph_box(p - sph_cubes[i].pos, sph_cubes[i].size));
    }
    for (var i = 0u; i < shapes_metadata.inf_cubes_amount; i++) {
        d = min(d, sd_inf_box(p - inf_cubes[i].pos, inf_cubes[i].size.xyz));
    }

    for (var i = 0u; i < shapes_metadata.neg_cubes_amount; i++) {
        d = max(d, -sd_box(p - neg_cubes[i].pos, neg_cubes[i].size));
    }
    for (var i = 0u; i < shapes_metadata.neg_spheres_amount; i++) {
        d = max(d, -sd_sphere(p - neg_spheres[i].pos, neg_spheres[i].size.x));
    }
    for (var i = 0u; i < shapes_metadata.neg_sph_cubes_amount; i++) {
        d = max(d, -sd_sph_box(p - neg_sph_cubes[i].pos, neg_sph_cubes[i].size));
    }
    for (var i = 0u; i < shapes_metadata.neg_inf_cubes_amount; i++) {
        d = max(d, -sd_inf_box(p - neg_inf_cubes[i].pos, neg_inf_cubes[i].size.xyz));
    }





    // for (var i = 0u; i < shapes_array_metadata.cubes.amount; i++) {
    //     var index = i + shapes_array_metadata.cubes.first_index;
    //     d = min(d, sd_box(p - shapes[index].pos, shapes[index].size));
    // }
    // for (var i = 0u; i < shapes_array_metadata.cubes_inf_w.amount; i++) {
    //     var index = i + shapes_array_metadata.cubes_inf_w.first_index;
    //     d = min(d, sd_inf_box(p - shapes[index].pos, shapes[index].size.xyz));
    // }
    // for (var i = 0u; i < shapes_array_metadata.spheres.amount; i++) {
    //     var index = i + shapes_array_metadata.spheres.first_index;
    //     d = min(d, sd_sphere(p - shapes[index].pos, shapes[index].size.x));
    // }
    // for (var i = 0u; i < shapes_array_metadata.sph_cube.amount; i++) {
    //     var index = i + shapes_array_metadata.sph_cube.first_index;
    //     d = min(d, sd_sph_box(p - shapes[index].pos, shapes[index].size));
    // }

    // for (var i = 0u; i < shapes_array_metadata.neg_cubes.amount; i++) {
    //     var index = i + shapes_array_metadata.neg_cubes.first_index;
    //     d = max(d, -sd_box(p - shapes[index].pos, shapes[index].size));
    // }
    // for (var i = 0u; i < shapes_array_metadata.neg_cubes_inf_w.amount; i++) {
    //     var index = i + shapes_array_metadata.neg_cubes_inf_w.first_index;
    //     d = max(d, -sd_inf_box(p - shapes[index].pos, shapes[index].size.xyz));
    // }
    // for (var i = 0u; i < shapes_array_metadata.neg_spheres.amount; i++) {
    //     var index = i + shapes_array_metadata.neg_spheres.first_index;
    //     d = max(d, -sd_sphere(p - shapes[index].pos, shapes[index].size.x));
    // }
    // for (var i = 0u; i < shapes_array_metadata.neg_sph_cube.amount; i++) {
    //     var index = i + shapes_array_metadata.neg_sph_cube.first_index;
    //     d = max(d, -sd_sph_box(p - shapes[index].pos, shapes[index].size));
    // }

    return d;
}

fn get_normal(p: vec4<f32>) -> vec3<f32> {
    var e: vec3<f32> = vec3<f32>(0.001, -0.001, 0.0);
    var a: vec4<f32> = p + e.yxxz;
    var b: vec4<f32> = p + e.xyxz;
    var c: vec4<f32> = p + e.xxyz;
    var d: vec4<f32> = p + e.yyyz;

    var fa: f32 = map(a);
    var fb: f32 = map(b);
    var fc: f32 = map(c);
    var fd: f32 = map(d);

    return normalize(
        e.yxx * fa +
        e.xyx * fb +
        e.xxy * fc +
        e.yyy * fd
    );
}

const MIN_STEP: f32 = 0.005;

fn ray_march(ray_origin_base: vec4<f32>, ray_direction: vec4<f32> ) -> vec2<f32>  {
    // var color: vec3<f32> = vec3<f32>(0., 0., 0.);
    var total_distance: f32 = 0.;
    
    var ray_origin = ray_origin_base;

    var i: i32 = 0;
    for (; i < MAX_STEPS; i++) {
        var d: f32  = map(ray_origin);
        total_distance += d;

        if (d < 0.) {
            // color.z = 1.;
            return vec2<f32>(total_distance + d, f32(i));
        }
        if (d < MIN_DIST) {
            // color.x = 1.;
            return vec2<f32>(total_distance + d, f32(i));
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
    // var uv: vec2<f32> = in.clip_position.xy / vec2<f32>(800.0, 600.0) - 0.5;
    // var uv: vec2<f32>;
    // uv.x = in.position.y / 2.0;
    // uv.y = in.position.x / 2.0;
    var uv: vec2<f32> = in.position.xy * 0.7;
    uv.x *= camera_uni.aspect;

    var ray_direction: vec4<f32> = normalize(vec4<f32>(uv, -1.0, 0.0));
    ray_direction *= camera_uni.cam_rot;

    let camera_position = camera_uni.cam_pos;

    let cam_pos: vec4<f32> = vec4<f32>(camera_position);

    let dist_and_depth: vec2<f32> = ray_march(cam_pos, ray_direction); 

    let normal: vec3<f32> = get_normal(dist_and_depth.x * ray_direction + cam_pos);

    let shade_coefficient: f32 = dot(normal, normalize(vec3<f32>(0.2, 1., 0.5)));

    let shade = mix(0.31, 0.9, shade_coefficient);

    var color: vec3<f32> = vec3<f32>(shade * 1.33) + (dist_and_depth.x / MAX_DIST);

    color = mix(clamp(color, vec3(0.0), vec3(1.0)), vec3<f32>(0.9, 1., 1.0), dist_and_depth.x / MAX_DIST);

    return vec4<f32>(color, 1.0);
}