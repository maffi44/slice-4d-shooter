// Fragment shader

struct CamerUniform {
    cam_pos: vec4<f32>,
    cam_rot: mat4x4<f32>,
    aspect: f32,
}

@group(0) @binding(0) var<uniform> camera_uni: CamerUniform;
@group(0) @binding(1) var<uniform> time: f32;
// @group(0) @binding(3) var<uniform> resolution: vec3<f32>;
// @group(0) @binding(5) var<uniform> iTimeDelta: f32;
// @group(0) @binding(6) var<uniform> iFrame: i32;
// @group(0) @binding(7) var<uniform> iFrameRate: f32;
// @group(0) @binding(8) var<uniform> iMouse: vec4<f32>;

// uniform f32 aspect;
// uniform vec3<f32> camera_position;
// uniform mat3 rotation_matrix;
// uniform vec3<f32> iResolution;
// uniform f32 iTime;
// uniform f32 iTimeDelta;
// uniform i32 iFrame;
// uniform f32 iFrameRate;
// uniform vec4<f32> iMouse;

const MAX_STEPS: i32 = 70;
const PI: f32 = 3.1415926535897;
const MIN_DIST: f32 = 0.003;
const MAX_DIST: f32 = 40.0;
// #define MAX_STEPS 70
// #define PI 3.1415926535897
// #define MIN_DIST 0.003
// #define MAX_DIST 40.0
// in vec2<f32> fragCoord;
// out vec4<f32> fragColor;
// #define BO

fn rotate(angle: f32) -> mat2x2<f32> {
    //angle *= 0.017453;
    var c: f32 = cos(angle);
    var s: f32 = sin(angle);
    return mat2x2<f32>(c, -s, s, c);
}
// mat2 rotate(f32 angle) {
//     //angle *= 0.017453;
//     f32 c = cos(angle);
//     f32 s = sin(angle);
//     return mat2(c, -s, s, c);
// }

fn sd_sphere(p: vec4<f32>, radius: f32) -> f32 {
    return length(p) - radius;
}
// f32 sd_sphere(vec4<f32> p, f32 radius) {
//     return length(p) - radius;
// }

fn sd_inf_sphere(p: vec4<f32>, radius: f32) -> f32 {
    return length(p.xyz) - radius;
}
// f32 sd_inf_sphere(vec4<f32> p, f32 radius) {
//     return length(p.xyz) - radius;
// }

fn sd_inf_box(p: vec4<f32>, b: vec3<f32>) -> f32 {
    var d: vec3<f32> = abs(p.xyz) - b;
    return min(max(d.x, max(d.y, d.z)),0.0) + length(max(d,vec3<f32>(0.0)));
}
// f32 sd_inf_box(vec4<f32> p, vec3<f32> b) {
//     vec3<f32> d = abs(p.xyz) - b;
//     return min(max(d.x, max(d.y, d.z)),0.0) + length(max(d,0.0));
// }

fn sd_box(p: vec4<f32>, b: vec4<f32>) -> f32 {
    var d: vec4<f32> = abs(p) - b;
    return min(max(d.x,max(d.y,max(d.z, d.w))),0.0) + length(max(d,vec4<f32>(0.0)));
}
// f32 sd_box(vec4<f32> p, vec4<f32> b) {
//     vec4<f32> d = abs(p) - b;
//     return min(max(d.x,max(d.y,max(d.z, d.w))),0.0) + length(max(d,0.0));
// }

fn sd_sph_inf_box(p: vec4<f32>, b: vec4<f32>) -> f32 {
    var d1: f32 = length(p.wx) - b.x;
    var d2: f32 = length(p.wy) - b.y;
    var d: vec2<f32> = abs(p.xy) - b.xy;
    return max(d1,max(d2,min(max(d.x,d.y),0.0) + length(max(d,vec2<f32>(0.0)))));
}
// f32 sd_sph_inf_box(vec4<f32> p, vec4<f32> b) {
//     f32 d1 = length(p.wx) - b.x;
//     f32 d2 = length(p.wy) - b.y;
//     // f32 d3 = length(p.wz) - b.z;
//     vec2<f32> d = abs(p.xy) - b.xy;
//     return max(d1,max(d2,min(max(d.x,d.y),0.0) + length(max(d,0.0))));
//     // return max(d1,max(d2, d3));
// }

fn sd_sph_box(p: vec4<f32>, b: vec4<f32>) -> f32 {
    var d1: f32 = length(p.xy) - b.x;
    var d2: f32 = length(p.xz) - b.y;
    var d3: f32 = length(p.yz) - b.z;
    var d4: f32 = length(p.wx) - b.w;
    var d5: f32 = length(p.wy) - b.w;
    var d6: f32 = length(p.wz) - b.w;
    return max(d6,max(d5,max(d4,max(d1,max(d2, d3)))));
}
// f32 sd_sph_box(vec4<f32> p, vec4<f32> b) {
//     f32 d1 = length(p.xy) - b.x;
//     f32 d2 = length(p.xz) - b.y;
//     f32 d3 = length(p.yz) - b.z;
//     f32 d4 = length(p.wx) - b.w;
//     f32 d5 = length(p.wy) - b.w;
//     f32 d6 = length(p.wz) - b.w;
//     return max(d6,max(d5,max(d4,max(d1,max(d2, d3)))));
// }

fn sd_box_sph(p: vec4<f32>, b: vec4<f32>) -> f32 {
    var ds: f32 = length(p) - b.w;
    var d: vec4<f32> = abs(p) - b;
    return max(ds, (min(max(d.x,max(d.y,max(d.z, d.w))),0.0) + length(max(d,vec4<f32>(0.0)))));
}
// f32 sd_box_sph(vec4<f32> p, vec4<f32> b) {
//     f32 ds = length(p) - b.w;
//     vec4<f32> d = abs(p) - b;
//     return max(ds, (min(max(d.x,max(d.y,max(d.z, d.w))),0.0) + length(max(d,0.0))));
// }

fn sd_solid_angle(p: vec4<f32>, c: vec2<f32>, ra: f32) -> f32 {
    var q: vec2<f32> = vec2<f32>( length(p.xz), p.y );
    var l: f32 = length(q) - ra;
    var m: f32 = length(q - c*clamp(dot(q,c),0.0,ra) );
    return max(l,m*sign(c.y*q.x-c.x*q.y));
}
// f32 sdSolidAngle(vec4<f32> p, vec2<f32> c, f32 ra)
// {
//   vec2<f32> q = vec2<f32>( length(p.xz), p.y );
//   f32 l = length(q) - ra;
//   f32 m = length(q - c*clamp(dot(q,c),0.0,ra) );
//   return max(l,m*sign(c.y*q.x-c.x*q.y));
// }

fn sd_octahedron(point: vec4<f32>, s: f32) -> f32 {
    var p = abs(point);
    return (p.x+p.y+p.z+p.w-s)*0.57735027;
}
// f32 sdOctahedron( vec4<f32> p, f32 s)
// {
//   p = abs(p);
//   return (p.x+p.y+p.z+p.w-s)*0.57735027;
// }

fn map(p: vec4<f32>) -> f32 {
    var d: f32 = sd_inf_box(p - vec4<f32>(0.0, 0.0, 0.0, 0.0), vec3<f32>(3., 0.1, 3.));
    // f32 d = sd_sphere(p - vec4<f32>(0, 1, 0, 2), 1.0);
    // = sd_inf_sphere(p - vec4<f32>(-3, 0, 4, 0.5), 1.0);
    // vec4<f32> pr = p - vec4<f32>(2, 2, 2, 3);
    var pr: vec4<f32> = p - vec4<f32>(2., 2., 2., 2.);
    
    // var xw = pr.xw;
    // xw *= rotate(time);
    // pr.xw = xw;

    // pr.xw *= rotate(time);
    // pr.yw *= rotate(time);
    // pr.zw *= rotate(time);
    d = min(sd_box(p - vec4<f32>(1., 1., 2., 1.), vec4<f32>(1., 0.2, 1., 0.5)), d);
    // d = min(sd_box(pr, vec4<f32>(1, 1, 1, 1)), d);
    d = min(sd_inf_box(p - vec4<f32>(-2., 2., -2., 0.), vec3<f32>(2., 2., 1.)), d);
    // d = min(sd_sphere(p - vec4<f32>(1, 1, 1, 0), 1.2), d);
    d = max(-sd_sph_inf_box(p - vec4<f32>(-2., 2., -2., 1.5), vec4<f32>(1., 1., 1., 1.)), d);
    // d = min(sd_sph_box(p - vec4<f32>(3, 1.5, 3, 2), vec4<f32>(1)), d);
    // d = min(sd_box_sph(p - vec4<f32>(3, 1.5, -3, 2), vec4<f32>(0.5, 0.5, 0.5, 1)), d);
    return d;
}
// f32 map(vec4<f32> p) {
//     f32 d = sd_inf_box(p - vec4<f32>(0, 0, 0, 0), vec3<f32>(3, 0.1, 3));
//     // f32 d = sd_sphere(p - vec4<f32>(0, 1, 0, 2), 1.0);
//     // = sd_inf_sphere(p - vec4<f32>(-3, 0, 4, 0.5), 1.0);
//     // vec4<f32> pr = p - vec4<f32>(2, 2, 2, 3);
//     vec4<f32> pr = p - vec4<f32>(2, 2., 2, 2);
//     pr.xw *= rotate(iTime);
//     pr.yw *= rotate(iTime);
//     pr.zw *= rotate(iTime);
//     d = min(sd_box(p - vec4<f32>(1, 1., 2, 1), vec4<f32>(1, 0.2, 1, 0.5)), d);
//     // d = min(sd_box(pr, vec4<f32>(1, 1, 1, 1)), d);
//     d = min(sd_inf_box(p - vec4<f32>(-2, 2, -2, 0), vec3<f32>(2, 2, 1)), d);
//     // d = min(sd_sphere(p - vec4<f32>(1, 1, 1, 0), 1.2), d);
//     d = max(-sd_sph_inf_box(p - vec4<f32>(-2, 2, -2, 1.5), vec4<f32>(1, 1, 1, 1)), d);
//     // d = min(sd_sph_box(p - vec4<f32>(3, 1.5, 3, 2), vec4<f32>(1)), d);
//     // d = min(sd_box_sph(p - vec4<f32>(3, 1.5, -3, 2), vec4<f32>(0.5, 0.5, 0.5, 1)), d);
//     return d;
// }

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
// vec3<f32> get_normal(vec4<f32> p) {
//     vec3<f32> e = vec3<f32>(0.001, -0.001, 0.0);
//     vec4<f32> a = p + e.yxxz;
//     vec4<f32> b = p + e.xyxz;
//     vec4<f32> c = p + e.xxyz;
//     vec4<f32> d = p + e.yyyz;

//     f32 fa = map(a);
//     f32 fb = map(b);
//     f32 fc = map(c);
//     f32 fd = map(d);

//     return normalize(
//         e.yxx * fa +
//         e.xyx * fb +
//         e.xxy * fc +
//         e.yyy * fd
//     );
// }

fn ray_march(ray_origin_base: vec4<f32>, ray_direction: vec4<f32> ) -> vec2<f32>  {
    var color: vec3<f32> = vec3<f32>(0., 0., 0.);
    var total_distance: f32 = 0.;
    
    var ray_origin = ray_origin_base;

    var i: i32 = 0;
    for (; i < MAX_STEPS; i++) {
        var d: f32  = map(ray_origin);
        total_distance += d;

        if (d < 0.) {
            color.z = 1.;
            return vec2<f32>(total_distance, f32(i));
        }
        if (d < MIN_DIST) {
            color.x = 1.;
            return vec2<f32>(total_distance, f32(i));
        }
        if (total_distance > MAX_DIST) {
            color.y = 1.;
            return vec2<f32>(total_distance, f32(i));
        }

        ray_origin += ray_direction * d;
    }
    //color.z = 1.;
    return vec2<f32>(total_distance, f32(i));
}
// vec2<f32> ray_march(vec4<f32> ray_origin, vec4<f32> ray_direction) {
//     vec3<f32> color = vec3<f32>(0, 0, 0);
//     f32 total_distance = 0.;

//     i32 i = 0;
//     for (; i < MAX_STEPS; i++) {
//         f32 d = map(ray_origin);
//         total_distance += d;

//         if (d < 0.) {
//             color.z = 1.;
//             return vec2<f32>(total_distance, f32(i));
//         }
//         if (d < MIN_DIST) {
//             color.x = 1.;
//             return vec2<f32>(total_distance, f32(i));
//         }
//         if (total_distance > MAX_DIST) {
//             color.y = 1.;
//             return vec2<f32>(total_distance, f32(i));
//         }

//         ray_origin += ray_direction * d;
//     }
//     //color.z = 1.;
//     return vec2<f32>(total_distance, f32(i));
// }


struct VertexInput {
    @location(0) @interpolate(perspective) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec3<f32>
};
//     in vec2 position;
//     in vec2 coordinates;
//     out vec2 fragCoord;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.position = model.position;
    return out;
}
//     void main() {
//         fragCoord = coordinates * iResolution.xy;
//         gl_Position = vec4(position, 0.0, 1.0);
//     }");


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // var uv: vec2<f32> = in.clip_position.xy / vec2<f32>(800.0, 600.0) - 0.5;
    // var uv: vec2<f32>;
    // uv.x = in.position.y / 2.0;
    // uv.y = in.position.x / 2.0;
    var uv: vec2<f32> = in.position.xy / 2.0;
    
    // uv.x *= -camera_uni.aspect;
    // uv.y *= -1.0;

    var ray_direction: vec4<f32> = normalize(vec4<f32>(uv, 1.0, 0.0));
    ray_direction *= camera_uni.cam_rot;

    let camera_position = camera_uni.cam_pos;

    var cam_pos: vec4<f32> = vec4<f32>(camera_position.xyz, camera_position.y * 0.8);

    var dist_and_depth: vec2<f32> = ray_march(cam_pos, ray_direction); 

    var normal: vec3<f32> = get_normal(dist_and_depth.x * ray_direction + cam_pos);

    var shade: f32 = dot(normal, normalize(vec3<f32>(0.2, 1., 0.5)));

    var color: vec3<f32> = vec3<f32>(shade * 1.33) + (dist_and_depth.x / MAX_DIST);

    var c = dist_and_depth.y / f32(f32(MAX_STEPS) / 3.0);
    color.g -= c;
    color.b -= c;

    // if (dist_and_depth.x >= MAX_DIST) {
    //     color = vec3<f32>(1);
    // }

    return vec4<f32>(color, 1.0);
    // return vec4<f32>(in.clip_position - 100.0);
}
// void main() {
//     vec2<f32> uv = (fragCoord / iResolution.xy - 0.5) * 2.;
//     uv.x *= aspect;

//     vec3<f32> ray_direction = normalize(vec3<f32>(uv, 1.));
//     ray_direction *= rotation_matrix;

//     vec4<f32> cam_pos = vec4<f32>(camera_position, camera_position.y * 0.8);

//     vec2<f32> dist_and_depth = ray_march(cam_pos, vec4<f32>(ray_direction, 0.0)); 

//     vec3<f32> normal = get_normal(dist_and_depth.x * vec4<f32>(ray_direction, 0.0) + cam_pos);

//     f32 shade = dot(normal, normalize(vec3<f32>(0.2, 1, 0.5)));

//     vec3<f32> color = vec3<f32>(shade * 1.33) + (dist_and_depth.x / MAX_DIST);

//     color.gb -= dist_and_depth.y / f32(MAX_STEPS / 3.0);

//     // if (dist_and_depth.x >= MAX_DIST) {
//     //     color = vec3<f32>(1);
//     // }

//     fragColor = vec4<f32>(color, 1.0);
// }


// @fragment
// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//     var res = 0.0;
//     for(var i: i32 = 0; i < 70; i++) {
//         res = res + sin(f32(i)/0.1231445564);
//     }
//     return vec4<f32>(vec3<f32>(in.coordinates, res), 1.0);
// }