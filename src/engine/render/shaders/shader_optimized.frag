#version 440

precision highp float;
precision highp int;

struct CameraUniform {
    vec4 cam_pos;
    mat4x4 cam_rot;
};
struct Shape {
    vec4 pos;
    vec4 size;
    int material;
    uint empty_bytes1_;
    uint empty_bytes2_;
    float roundness;
};
struct PlayerForm {
    vec4 pos;
    uvec4 empty_bytes;
    vec3 color;
    float radius;
    mat4x4 rotation;
    vec4 weapon_offset;
};
struct ShapesMetadata {
    uint cubes_start;
    uint cubes_amount;
    uint spheres_start;
    uint spheres_amount;
    uint inf_cubes_start;
    uint inf_cubes_amount;
    uint sph_cubes_start;
    uint sph_cubes_amount;
    uint s_cubes_start;
    uint s_cubes_amount;
    uint s_spheres_start;
    uint s_spheres_amount;
    uint s_inf_cubes_start;
    uint s_inf_cubes_amount;
    uint s_sph_cubes_start;
    uint s_sph_cubes_amount;
    uint neg_cubes_start;
    uint neg_cubes_amount;
    uint neg_spheres_start;
    uint neg_spheres_amount;
    uint neg_inf_cubes_start;
    uint neg_inf_cubes_amount;
    uint neg_sph_cubes_start;
    uint neg_sph_cubes_amount;
    uint s_neg_cubes_start;
    uint s_neg_cubes_amount;
    uint s_neg_spheres_start;
    uint s_neg_spheres_amount;
    uint s_neg_inf_cubes_start;
    uint s_neg_inf_cubes_amount;
    uint s_neg_sph_cubes_start;
    uint s_neg_sph_cubes_amount;
};
struct IntersectedShapesMetadata {
    uint st_cubes_start;
    uint st_cubes_amount;
    uint dyn_cubes_start;
    uint dyn_cubes_amount;
    uint st_spheres_start;
    uint st_spheres_amount;
    uint dyn_spheres_start;
    uint dyn_spheres_amount;
    uint st_inf_cubes_start;
    uint st_inf_cubes_amount;
    uint dyn_inf_cubes_start;
    uint dyn_inf_cubes_amount;
    uint st_sph_cubes_start;
    uint st_sph_cubes_amount;
    uint dyn_sph_cubes_start;
    uint dyn_sph_cubes_amount;
    uint st_s_cubes_start;
    uint st_s_cubes_amount;
    uint dyn_s_cubes_start;
    uint dyn_s_cubes_amount;
    uint st_s_spheres_start;
    uint st_s_spheres_amount;
    uint dyn_s_spheres_start;
    uint dyn_s_spheres_amount;
    uint st_s_inf_cubes_start;
    uint st_s_inf_cubes_amount;
    uint dyn_s_inf_cubes_start;
    uint dyn_s_inf_cubes_amount;
    uint st_s_sph_cubes_start;
    uint st_s_sph_cubes_amount;
    uint dyn_s_sph_cubes_start;
    uint dyn_s_sph_cubes_amount;
    uint st_neg_cubes_start;
    uint st_neg_cubes_amount;
    uint dyn_neg_cubes_start;
    uint dyn_neg_cubes_amount;
    uint st_neg_spheres_start;
    uint st_neg_spheres_amount;
    uint dyn_neg_spheres_start;
    uint dyn_neg_spheres_amount;
    uint st_neg_inf_cubes_start;
    uint st_neg_inf_cubes_amount;
    uint dyn_neg_inf_cubes_start;
    uint dyn_neg_inf_cubes_amount;
    uint st_neg_sph_cubes_start;
    uint st_neg_sph_cubes_amount;
    uint dyn_neg_sph_cubes_start;
    uint dyn_neg_sph_cubes_amount;
    uint st_s_neg_cubes_start;
    uint st_s_neg_cubes_amount;
    uint dyn_s_neg_cubes_start;
    uint dyn_s_neg_cubes_amount;
    uint st_s_neg_spheres_start;
    uint st_s_neg_spheres_amount;
    uint dyn_s_neg_spheres_start;
    uint dyn_s_neg_spheres_amount;
    uint st_s_neg_inf_cubes_start;
    uint st_s_neg_inf_cubes_amount;
    uint dyn_s_neg_inf_cubes_start;
    uint dyn_s_neg_inf_cubes_amount;
    uint st_s_neg_sph_cubes_start;
    uint st_s_neg_sph_cubes_amount;
    uint dyn_s_neg_sph_cubes_start;
    uint dyn_s_neg_sph_cubes_amount;
    uint player_forms_start;
    uint player_forms_amount;
};
struct Intersections {
    IntersectedShapesMetadata ismd;
    uint ish[16];
    float offset;
    bool ray_w_rotated;
};
struct SphericalAreasMetadata {
    uint holegun_colorized_areas_start;
    uint holegun_colorized_areas_amount;
    uint explode_areas_start;
    uint explode_areas_amount;
};
struct SphericalArea {
    vec4 pos;
    vec3 color;
    float radius;
};
struct BeamArea {
    vec4 pos1_;
    vec4 pos2_;
    vec3 color;
    float radius;
};
struct OutputMaterials {
    uint materials_count;
    vec3 empty_bytes;
    int materials[16];
    float material_weights[16];
};
struct OtherDynamicData {
    ShapesMetadata shapes_arrays_metadata;
    SphericalAreasMetadata spherical_areas_meatadata;
    CameraUniform camera_data;
    uvec3 empty_bytes1_;
    uint beam_areas_amount;
    uint player_forms_amount;
    float w_scaner_radius;
    float w_scaner_intesity;
    float death_screen_effect;
    float getting_damage_screen_effect;
    float stickiness;
    float screen_aspect;
    float time;
};
struct Material {
    vec4 color;
};
struct OtherStaticData {
    ShapesMetadata shapes_arrays_metadata;
    int is_w_floor_exist;
    float w_floor;
    int players_mat1_;
    int players_mat2_;
    int w_cups_mat;
    float stickiness;
    uint empty_byte1_;
    uint empty_byte2_;
    Material materials[32];
};
struct VertexInput {
    vec3 position;
};
struct VertexOutput {
    vec4 clip_position;
    vec3 position;
};
const int MAX_STEPS = 128;
const float PI = 3.1415927;
const float MIN_DIST = 0.01;
const float MAX_DIST = 350.0;
const float STICKINESS_EFFECT_COEF = 3.1415927;
const float MIN_STEP = 0.005;


layout(set = 0, binding = 0) uniform type_13_block_0Fragment { Shape _group_0_binding_0_fs[256]; };
layout(set = 0, binding = 1) uniform type_13_block_1Fragment { Shape _group_0_binding_1_fs[256]; };
layout(set = 0, binding = 2) uniform type_13_block_2Fragment { Shape _group_0_binding_2_fs[256]; };
layout(set = 0, binding = 3) uniform type_13_block_3Fragment { Shape _group_0_binding_3_fs[256]; };
layout(set = 0, binding = 4) uniform OtherStaticData_block_4Fragment { OtherStaticData _group_0_binding_4_fs; };
layout(set = 0, binding = 5) uniform type_13_block_5Fragment { Shape _group_0_binding_5_fs[256]; };
layout(set = 0, binding = 6) uniform type_13_block_6Fragment { Shape _group_0_binding_6_fs[256]; };
layout(set = 0, binding = 7) uniform type_13_block_7Fragment { Shape _group_0_binding_7_fs[256]; };
layout(set = 0, binding = 8) uniform type_13_block_8Fragment { Shape _group_0_binding_8_fs[256]; };
layout(set = 0, binding = 9) uniform OtherDynamicData_block_9Fragment { OtherDynamicData _group_0_binding_9_fs; };
layout(set = 1, binding = 2) uniform type_16_block_10Fragment { PlayerForm _group_1_binding_2_fs[16]; };

layout(location = 0) smooth in vec3 _vs2fs_location0;
layout(location = 0) out vec4 _fs2p_location0;

mat2x2 rotate(float angle) {
    float c = 0.0;
    float s = 0.0;
    c = cos(angle);
    s = sin(angle);
    float _e5 = c;
    float _e6 = s;
    float _e8 = s;
    float _e9 = c;
    return mat2x2(vec2(_e5, -(_e6)), vec2(_e8, _e9));
}

float sd_sphere(vec4 p, float radius) {
    return (length(p) - radius);
}

float sd_inf_sphere(vec4 p_1, float radius_1) {
    return (length(p_1.xyz) - radius_1);
}

float sd_inf_box(vec4 p_2, vec3 b) {
    vec3 d = vec3(0.0);
    d = (abs(p_2.xyz) - b);
    float _e7 = d.x;
    float _e9 = d.y;
    float _e11 = d.z;
    vec3 _e16 = d;
    return (min(max(_e7, max(_e9, _e11)), 0.0) + length(max(_e16, vec3(0.0))));
}

float sd_box(vec4 p_3, vec4 b_1) {
    vec4 d_1 = vec4(0.0);
    d_1 = (abs(p_3) - b_1);
    float _e6 = d_1.x;
    float _e8 = d_1.y;
    float _e10 = d_1.z;
    float _e12 = d_1.w;
    vec4 _e18 = d_1;
    return (min(max(_e6, max(_e8, max(_e10, _e12))), 0.0) + length(max(_e18, vec4(0.0))));
}

float sd_sph_inf_box(vec4 p_4, vec4 b_2) {
    float d1_ = 0.0;
    float d2_ = 0.0;
    vec2 d_2 = vec2(0.0);
    d1_ = (length(p_4.wx) - b_2.x);
    d2_ = (length(p_4.wy) - b_2.y);
    d_2 = (abs(p_4.xy) - b_2.xy);
    float _e17 = d1_;
    float _e18 = d2_;
    float _e20 = d_2.x;
    float _e22 = d_2.y;
    vec2 _e26 = d_2;
    return max(_e17, max(_e18, (min(max(_e20, _e22), 0.0) + length(max(_e26, vec2(0.0))))));
}

float sd_sph_box(vec4 p_5, vec4 b_3) {
    float d1_1 = 0.0;
    float d2_1 = 0.0;
    float d3_ = 0.0;
    float d4_ = 0.0;
    float d5_ = 0.0;
    float d6_ = 0.0;
    d1_1 = (length(p_5.xy) - b_3.x);
    d2_1 = (length(p_5.xz) - b_3.y);
    d3_ = (length(p_5.yz) - b_3.z);
    d4_ = (length(p_5.wx) - b_3.w);
    d5_ = (length(p_5.wy) - b_3.w);
    d6_ = (length(p_5.wz) - b_3.w);
    float _e32 = d6_;
    float _e33 = d5_;
    float _e34 = d4_;
    float _e35 = d1_1;
    float _e36 = d2_1;
    float _e37 = d3_;
    return max(_e32, max(_e33, max(_e34, max(_e35, max(_e36, _e37)))));
}

float sd_box_sph(vec4 p_6, vec4 b_4) {
    float ds = 0.0;
    vec4 d_3 = vec4(0.0);
    ds = (length(p_6) - b_4.w);
    d_3 = (abs(p_6) - b_4);
    float _e9 = ds;
    float _e11 = d_3.x;
    float _e13 = d_3.y;
    float _e15 = d_3.z;
    float _e17 = d_3.w;
    vec4 _e23 = d_3;
    return max(_e9, (min(max(_e11, max(_e13, max(_e15, _e17))), 0.0) + length(max(_e23, vec4(0.0)))));
}

float sd_solid_angle(vec4 p_7, vec2 c_1, float ra) {
    vec2 q = vec2(0.0);
    float l = 0.0;
    float m = 0.0;
    q = vec2(length(p_7.xz), p_7.y);
    vec2 _e8 = q;
    l = (length(_e8) - ra);
    vec2 _e12 = q;
    vec2 _e13 = q;
    m = length((_e12 - (c_1 * clamp(dot(_e13, c_1), 0.0, ra))));
    float _e21 = l;
    float _e22 = m;
    float _e25 = q.x;
    float _e29 = q.y;
    return max(_e21, (_e22 * sign(((c_1.y * _e25) - (c_1.x * _e29)))));
}

float sd_octahedron(vec4 point, float s_1) {
    vec4 p_8 = vec4(0.0);
    p_8 = abs(point);
    float _e5 = p_8.x;
    float _e7 = p_8.y;
    float _e10 = p_8.z;
    float _e13 = p_8.w;
    return (((((_e5 + _e7) + _e10) + _e13) - s_1) * 0.57725626);
}

float sd_capsule(vec4 p_9, vec4 a, vec4 b_5, float r) {
    vec4 pa = (p_9 - a);
    vec4 ba = (b_5 - a);
    float h_4 = clamp((dot(pa, ba) / dot(ba, ba)), 0.0, 1.0);
    return (length((pa - (ba * h_4))) - r);
}

float smin(float a_1, float b_6, float k) {
    float kk = ((k * 1.0) / 0.29289323);
    float h_5 = (max((kk - abs((a_1 - b_6))), 0.0) / kk);
    return (min(a_1, b_6) - ((kk * 0.5) * ((1.0 + h_5) - sqrt((1.0 - (h_5 * (h_5 - 2.0)))))));
}

vec4 get_sphere_normal(vec4 p_11, vec4 sphere_pos, float sphere_radius) {
    vec3 h = vec3(0.001, -0.001, 0.0);
    vec4 a_2 = vec4(0.0);
    vec4 b_7 = vec4(0.0);
    vec4 c_2 = vec4(0.0);
    vec4 d_4 = vec4(0.0);
    vec4 e = vec4(0.0);
    vec4 f = vec4(0.0);
    float fa = 0.0;
    float fb = 0.0;
    float fc = 0.0;
    float fd = 0.0;
    float fe = 0.0;
    float ff = 0.0;
    vec3 _e8 = h;
    a_2 = (p_11 + _e8.yxxz);
    vec3 _e12 = h;
    b_7 = (p_11 + _e12.xyxz);
    vec3 _e16 = h;
    c_2 = (p_11 + _e16.xxyz);
    vec3 _e20 = h;
    d_4 = (p_11 + _e20.yyyz);
    vec3 _e24 = h;
    e = (p_11 + _e24.zzzx);
    vec3 _e28 = h;
    f = (p_11 + _e28.zzzy);
    vec4 _e32 = a_2;
    float _e34 = sd_sphere((_e32 - sphere_pos), sphere_radius);
    fa = _e34;
    vec4 _e36 = b_7;
    float _e38 = sd_sphere((_e36 - sphere_pos), sphere_radius);
    fb = _e38;
    vec4 _e40 = c_2;
    float _e42 = sd_sphere((_e40 - sphere_pos), sphere_radius);
    fc = _e42;
    vec4 _e44 = d_4;
    float _e46 = sd_sphere((_e44 - sphere_pos), sphere_radius);
    fd = _e46;
    vec4 _e48 = e;
    float _e50 = sd_sphere((_e48 - sphere_pos), sphere_radius);
    fe = _e50;
    vec4 _e52 = f;
    float _e54 = sd_sphere((_e52 - sphere_pos), sphere_radius);
    ff = _e54;
    vec3 _e56 = h;
    float _e58 = fa;
    vec3 _e60 = h;
    float _e62 = fb;
    vec3 _e65 = h;
    float _e67 = fc;
    vec3 _e70 = h;
    float _e72 = fd;
    vec3 _e75 = h;
    float _e77 = fe;
    vec3 _e80 = h;
    float _e82 = ff;
    return normalize(((((((_e56.yxxz * _e58) + (_e60.xyxz * _e62)) + (_e65.xxyz * _e67)) + (_e70.yyyz * _e72)) + (_e75.zzzx * _e77)) + (_e80.zzzy * _e82)));
}

vec4 ray_march_individual_volume_sphere(SphericalArea sphere, vec4 start_pos, vec4 direction, float max_distance) {
    vec4 color_2 = vec4(0.0);
    float total_dist = 0.0;
    vec4 p_12 = vec4(0.0);
    float prev_d = MAX_DIST;
    int i_2 = 0;
    p_12 = start_pos;
    bool loop_init = true;
    while(true) {
        if (!loop_init) {
            int _e55 = i_2;
            i_2 = (_e55 + 1);
        }
        loop_init = false;
        int _e14 = i_2;
        if ((_e14 < MAX_STEPS)) {
        } else {
            break;
        }
        {
            float _e17 = total_dist;
            if ((_e17 > max_distance)) {
                break;
            }
            vec4 _e19 = p_12;
            float _e23 = sd_sphere((_e19 - sphere.pos), sphere.radius);
            float _e24 = prev_d;
            if ((_e23 > _e24)) {
                break;
            }
            prev_d = _e23;
            if ((_e23 < MIN_DIST)) {
                vec4 _e28 = p_12;
                vec4 _e31 = get_sphere_normal(_e28, sphere.pos, sphere.radius);
                float color_coef = abs(dot(_e31, direction));
                vec3 color_rgb = ((mix(sphere.color, vec3(1.0), pow(color_coef, 40.5)) * pow(color_coef, 4.0)) + vec3(0.05));
                color_2 = vec4(color_rgb, pow(color_coef, 15.0));
                break;
            }
            float _e49 = total_dist;
            total_dist = (_e49 + _e23);
            vec4 _e52 = p_12;
            p_12 = (_e52 + (direction * _e23));
        }
    }
    vec4 _e57 = color_2;
    return _e57;
}

vec4 get_capsule_normal(vec4 p_13, vec4 beam_pos1_, vec4 beam_pos2_, float beam_radius) {
    vec3 h_1 = vec3(0.001, -0.001, 0.0);
    vec4 a_3 = vec4(0.0);
    vec4 b_8 = vec4(0.0);
    vec4 c_3 = vec4(0.0);
    vec4 d_5 = vec4(0.0);
    vec4 e_1 = vec4(0.0);
    vec4 f_1 = vec4(0.0);
    float fa_1 = 0.0;
    float fb_1 = 0.0;
    float fc_1 = 0.0;
    float fd_1 = 0.0;
    float fe_1 = 0.0;
    float ff_1 = 0.0;
    vec3 _e9 = h_1;
    a_3 = (p_13 + _e9.yxxz);
    vec3 _e13 = h_1;
    b_8 = (p_13 + _e13.xyxz);
    vec3 _e17 = h_1;
    c_3 = (p_13 + _e17.xxyz);
    vec3 _e21 = h_1;
    d_5 = (p_13 + _e21.yyyz);
    vec3 _e25 = h_1;
    e_1 = (p_13 + _e25.zzzx);
    vec3 _e29 = h_1;
    f_1 = (p_13 + _e29.zzzy);
    vec4 _e33 = a_3;
    float _e34 = sd_capsule(_e33, beam_pos1_, beam_pos2_, beam_radius);
    fa_1 = _e34;
    vec4 _e36 = b_8;
    float _e37 = sd_capsule(_e36, beam_pos1_, beam_pos2_, beam_radius);
    fb_1 = _e37;
    vec4 _e39 = c_3;
    float _e40 = sd_capsule(_e39, beam_pos1_, beam_pos2_, beam_radius);
    fc_1 = _e40;
    vec4 _e42 = d_5;
    float _e43 = sd_capsule(_e42, beam_pos1_, beam_pos2_, beam_radius);
    fd_1 = _e43;
    vec4 _e45 = e_1;
    float _e46 = sd_capsule(_e45, beam_pos1_, beam_pos2_, beam_radius);
    fe_1 = _e46;
    vec4 _e48 = f_1;
    float _e49 = sd_capsule(_e48, beam_pos1_, beam_pos2_, beam_radius);
    ff_1 = _e49;
    vec3 _e51 = h_1;
    float _e53 = fa_1;
    vec3 _e55 = h_1;
    float _e57 = fb_1;
    vec3 _e60 = h_1;
    float _e62 = fc_1;
    vec3 _e65 = h_1;
    float _e67 = fd_1;
    vec3 _e70 = h_1;
    float _e72 = fe_1;
    vec3 _e75 = h_1;
    float _e77 = ff_1;
    return normalize(((((((_e51.yxxz * _e53) + (_e55.xyxz * _e57)) + (_e60.xxyz * _e62)) + (_e65.yyyz * _e67)) + (_e70.zzzx * _e72)) + (_e75.zzzy * _e77)));
}

vec4 ray_march_indicidual_volume_beam(BeamArea beam, vec4 start_pos_1, vec4 direction_1, float max_distance_1) {
    vec4 color_3 = vec4(0.0);
    float total_dist_1 = 0.0;
    vec4 p_14 = vec4(0.0);
    float prev_d_1 = MAX_DIST;
    int i_3 = 0;
    p_14 = start_pos_1;
    bool loop_init_1 = true;
    while(true) {
        if (!loop_init_1) {
            int _e61 = i_3;
            i_3 = (_e61 + 1);
        }
        loop_init_1 = false;
        int _e14 = i_3;
        if ((_e14 < MAX_STEPS)) {
        } else {
            break;
        }
        {
            float _e17 = total_dist_1;
            if ((_e17 > max_distance_1)) {
                break;
            }
            vec4 _e19 = p_14;
            float _e23 = sd_capsule(_e19, beam.pos1_, beam.pos2_, beam.radius);
            float _e24 = prev_d_1;
            if ((_e23 > _e24)) {
                break;
            }
            prev_d_1 = _e23;
            if ((_e23 < MIN_DIST)) {
                vec4 _e28 = p_14;
                vec4 _e32 = get_capsule_normal(_e28, beam.pos1_, beam.pos2_, beam.radius);
                vec4 beam_dir = normalize((beam.pos1_ - beam.pos2_));
                vec4 beam_perpendicular = normalize((direction_1 - (dot(direction_1, beam_dir) * beam_dir)));
                float color_coef_1 = abs(dot(_e32, beam_perpendicular));
                vec3 color_rgb_1 = (mix(beam.color, vec3(1.0), pow(color_coef_1, 40.5)) * pow(color_coef_1, 4.0));
                color_3 = vec4(color_rgb_1, pow(color_coef_1, 15.0));
                break;
            }
            float _e55 = total_dist_1;
            total_dist_1 = (_e55 + _e23);
            vec4 _e58 = p_14;
            p_14 = (_e58 + (direction_1 * _e23));
        }
    }
    vec4 _e63 = color_3;
    return _e63;
}

void get_mat(vec4 cam_pos, vec4 ray_dir, float dist, inout Intersections in_1, inout int mats_1[32], inout float mats_wieghts_1[32], inout uint mat_count) {
    uint j = 0u;
    float d_6 = 0.0;
    uint i_6 = 0u;
    uint i_7 = 0u;
    uint i_8 = 0u;
    uint i_9 = 0u;
    uint i_10 = 0u;
    uint i_11 = 0u;
    uint i_12 = 0u;
    uint i_13 = 0u;
    float d_7 = 700.0;
    uint i_14 = 0u;
    float coef = 0.0;
    uint k_1 = 0u;
    uint i_15 = 0u;
    float coef_1 = 0.0;
    uint k_2 = 0u;
    uint i_16 = 0u;
    float coef_2 = 0.0;
    uint k_3 = 0u;
    uint i_17 = 0u;
    float coef_3 = 0.0;
    uint k_4 = 0u;
    uint i_18 = 0u;
    float coef_4 = 0.0;
    uint k_5 = 0u;
    uint i_19 = 0u;
    float coef_5 = 0.0;
    uint k_6 = 0u;
    uint i_20 = 0u;
    float coef_6 = 0.0;
    uint k_7 = 0u;
    uint i_21 = 0u;
    float coef_7 = 0.0;
    uint k_8 = 0u;
    if ((dist > MAX_DIST)) {
        mat_count = 1u;
        mats_wieghts_1[0] = 1.0;
        mats_1[0] = -1;
        return;
    }
    vec4 p_17 = (cam_pos + (ray_dir * dist));
    IntersectedShapesMetadata ismda = in_1.ismd;
    mat_count = 0u;
    j = ismda.player_forms_start;
    bool loop_init_2 = true;
    while(true) {
        if (!loop_init_2) {
            uint _e199 = j;
            j = (_e199 + 1u);
        }
        loop_init_2 = false;
        uint _e21 = j;
        if ((_e21 < (ismda.player_forms_amount + ismda.player_forms_start))) {
        } else {
            break;
        }
        {
            uint _e28 = j;
            uint _e30 = in_1.ish[_e28];
            PlayerForm shape = _group_1_binding_2_fs[_e30];
            float _e36 = sd_sphere((p_17 - shape.pos), shape.radius);
            d_6 = _e36;
            float _e38 = d_6;
            float _e44 = sd_sphere((p_17 - shape.pos), (shape.radius * 0.86));
            d_6 = max(_e38, -(_e44));
            vec4 rotated_p = (shape.rotation * (p_17 - shape.pos));
            float _e51 = d_6;
            float _e65 = sd_box(rotated_p, vec4((shape.radius * 0.18), (shape.radius * 1.2), (shape.radius * 1.2), (shape.radius * 1.2)));
            d_6 = max(_e51, -(_e65));
            float _e68 = d_6;
            float _e79 = sd_sphere((rotated_p - vec4(0.0, 0.0, -(shape.radius), 0.0)), (shape.radius * 0.53));
            d_6 = max(_e68, -(_e79));
            float _e82 = d_6;
            if ((_e82 < MIN_DIST)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                int _e91 = _group_0_binding_4_fs.players_mat1_;
                mats_1[0] = _e91;
                return;
            }
            float _e97 = sd_sphere((p_17 - shape.pos), (shape.radius * 0.6));
            d_6 = _e97;
            float _e98 = d_6;
            float _e111 = sd_sphere((rotated_p - (vec4(0.0, 0.0, -(shape.radius), 0.0) * 0.6)), (shape.radius * 0.34));
            d_6 = max(_e98, -(_e111));
            float _e114 = d_6;
            if ((_e114 < MIN_DIST)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                int _e123 = _group_0_binding_4_fs.players_mat2_;
                mats_1[0] = _e123;
                return;
            }
            float _e129 = sd_sphere((rotated_p - shape.weapon_offset), (shape.radius * 0.286));
            d_6 = _e129;
            float _e130 = d_6;
            float _e144 = sd_capsule(rotated_p, shape.weapon_offset, (shape.weapon_offset - vec4(0.0, 0.0, (shape.radius * 0.49), 0.0)), (shape.radius * 0.18));
            d_6 = max(_e130, -(_e144));
            float _e147 = d_6;
            if ((_e147 < MIN_DIST)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                int _e156 = _group_0_binding_4_fs.players_mat1_;
                mats_1[0] = _e156;
                return;
            }
            float _e170 = sd_capsule(rotated_p, shape.weapon_offset, (shape.weapon_offset - vec4(0.0, 0.0, (shape.radius * 0.43), 0.0)), (shape.radius * 0.1));
            d_6 = _e170;
            float _e171 = d_6;
            float _e185 = sd_capsule(rotated_p, shape.weapon_offset, (shape.weapon_offset - vec4(0.0, 0.0, (shape.radius * 0.65), 0.0)), (shape.radius * 0.052));
            d_6 = max(_e171, -(_e185));
            float _e188 = d_6;
            if ((_e188 < MIN_DIST)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                int _e197 = _group_0_binding_4_fs.players_mat2_;
                mats_1[0] = _e197;
                return;
            }
        }
    }
    i_6 = ismda.st_cubes_start;
    bool loop_init_3 = true;
    while(true) {
        if (!loop_init_3) {
            uint _e229 = i_6;
            i_6 = (_e229 + 1u);
        }
        loop_init_3 = false;
        uint _e203 = i_6;
        if ((_e203 < (ismda.st_cubes_amount + ismda.st_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e209 = i_6;
            uint j_2 = in_1.ish[_e209];
            Shape shape_1 = _group_0_binding_0_fs[j_2];
            float _e218 = sd_box((p_17 - shape_1.pos), shape_1.size);
            if (((_e218 - shape_1.roundness) < 0.02)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                mats_1[0] = shape_1.material;
                return;
            }
        }
    }
    i_7 = ismda.st_spheres_start;
    bool loop_init_4 = true;
    while(true) {
        if (!loop_init_4) {
            uint _e260 = i_7;
            i_7 = (_e260 + 1u);
        }
        loop_init_4 = false;
        uint _e233 = i_7;
        if ((_e233 < (ismda.st_spheres_amount + ismda.st_spheres_start))) {
        } else {
            break;
        }
        {
            uint _e239 = i_7;
            uint j_3 = in_1.ish[_e239];
            Shape shape_2 = _group_0_binding_0_fs[j_3];
            float _e249 = sd_sphere((p_17 - shape_2.pos), shape_2.size.x);
            if (((_e249 - shape_2.roundness) < 0.02)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                mats_1[0] = shape_2.material;
                return;
            }
        }
    }
    i_8 = ismda.st_sph_cubes_start;
    bool loop_init_5 = true;
    while(true) {
        if (!loop_init_5) {
            uint _e290 = i_8;
            i_8 = (_e290 + 1u);
        }
        loop_init_5 = false;
        uint _e264 = i_8;
        if ((_e264 < (ismda.st_sph_cubes_amount + ismda.st_sph_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e270 = i_8;
            uint j_4 = in_1.ish[_e270];
            Shape shape_3 = _group_0_binding_0_fs[j_4];
            float _e279 = sd_sph_box((p_17 - shape_3.pos), shape_3.size);
            if (((_e279 - shape_3.roundness) < 0.02)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                mats_1[0] = shape_3.material;
                return;
            }
        }
    }
    i_9 = ismda.st_inf_cubes_start;
    bool loop_init_6 = true;
    while(true) {
        if (!loop_init_6) {
            uint _e321 = i_9;
            i_9 = (_e321 + 1u);
        }
        loop_init_6 = false;
        uint _e294 = i_9;
        if ((_e294 < (ismda.st_inf_cubes_amount + ismda.st_inf_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e300 = i_9;
            uint j_5 = in_1.ish[_e300];
            Shape shape_4 = _group_0_binding_0_fs[j_5];
            float _e310 = sd_inf_box((p_17 - shape_4.pos), shape_4.size.xyz);
            if (((_e310 - shape_4.roundness) < 0.02)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                mats_1[0] = shape_4.material;
                return;
            }
        }
    }
    i_10 = ismda.dyn_cubes_start;
    bool loop_init_7 = true;
    while(true) {
        if (!loop_init_7) {
            uint _e351 = i_10;
            i_10 = (_e351 + 1u);
        }
        loop_init_7 = false;
        uint _e325 = i_10;
        if ((_e325 < (ismda.dyn_cubes_amount + ismda.dyn_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e331 = i_10;
            uint j_6 = in_1.ish[_e331];
            Shape shape_5 = _group_0_binding_5_fs[j_6];
            float _e340 = sd_box((p_17 - shape_5.pos), shape_5.size);
            if (((_e340 - shape_5.roundness) < 0.02)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                mats_1[0] = shape_5.material;
                return;
            }
        }
    }
    i_11 = ismda.dyn_spheres_start;
    bool loop_init_8 = true;
    while(true) {
        if (!loop_init_8) {
            uint _e382 = i_11;
            i_11 = (_e382 + 1u);
        }
        loop_init_8 = false;
        uint _e355 = i_11;
        if ((_e355 < (ismda.dyn_spheres_amount + ismda.dyn_spheres_start))) {
        } else {
            break;
        }
        {
            uint _e361 = i_11;
            uint j_7 = in_1.ish[_e361];
            Shape shape_6 = _group_0_binding_5_fs[j_7];
            float _e371 = sd_sphere((p_17 - shape_6.pos), shape_6.size.x);
            if (((_e371 - shape_6.roundness) < 0.02)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                mats_1[0] = shape_6.material;
                return;
            }
        }
    }
    i_12 = ismda.dyn_sph_cubes_start;
    bool loop_init_9 = true;
    while(true) {
        if (!loop_init_9) {
            uint _e412 = i_12;
            i_12 = (_e412 + 1u);
        }
        loop_init_9 = false;
        uint _e386 = i_12;
        if ((_e386 < (ismda.dyn_sph_cubes_amount + ismda.dyn_sph_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e392 = i_12;
            uint j_8 = in_1.ish[_e392];
            Shape shape_7 = _group_0_binding_5_fs[j_8];
            float _e401 = sd_sph_box((p_17 - shape_7.pos), shape_7.size);
            if (((_e401 - shape_7.roundness) < 0.02)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                mats_1[0] = shape_7.material;
                return;
            }
        }
    }
    i_13 = ismda.dyn_inf_cubes_start;
    bool loop_init_10 = true;
    while(true) {
        if (!loop_init_10) {
            uint _e443 = i_13;
            i_13 = (_e443 + 1u);
        }
        loop_init_10 = false;
        uint _e416 = i_13;
        if ((_e416 < (ismda.dyn_inf_cubes_amount + ismda.dyn_inf_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e422 = i_13;
            uint j_9 = in_1.ish[_e422];
            Shape shape_8 = _group_0_binding_5_fs[j_9];
            float _e432 = sd_inf_box((p_17 - shape_8.pos), shape_8.size.xyz);
            if (((_e432 - shape_8.roundness) < 0.02)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                mats_1[0] = shape_8.material;
                return;
            }
        }
    }
    int _e447 = _group_0_binding_4_fs.is_w_floor_exist;
    if ((_e447 == 1)) {
        bool _e451 = in_1.ray_w_rotated;
        if (_e451) {
            float _e455 = _group_0_binding_4_fs.w_floor;
            if (((p_17.w - _e455) < 0.02)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                int _e465 = _group_0_binding_4_fs.w_cups_mat;
                mats_1[0] = _e465;
                return;
            }
        }
    }
    i_14 = ismda.st_s_cubes_start;
    bool loop_init_11 = true;
    while(true) {
        if (!loop_init_11) {
            uint _e560 = i_14;
            i_14 = (_e560 + 1u);
        }
        loop_init_11 = false;
        uint _e470 = i_14;
        if ((_e470 < (ismda.st_s_cubes_amount + ismda.st_s_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e476 = i_14;
            uint j_10 = in_1.ish[_e476];
            Shape shape_9 = _group_0_binding_2_fs[j_10];
            float _e485 = sd_box((p_17 - shape_9.pos), shape_9.size);
            float dd_1 = (_e485 - shape_9.roundness);
            if ((dd_1 < 0.02)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                mats_1[0] = shape_9.material;
                return;
            }
            float _e497 = _group_0_binding_4_fs.stickiness;
            if ((dd_1 < (_e497 * STICKINESS_EFFECT_COEF))) {
                uint _e501 = mat_count;
                if ((_e501 == 0u)) {
                    mat_count = 1u;
                    mats_wieghts_1[0] = 1.0;
                    mats_1[0] = shape_9.material;
                    d_7 = dd_1;
                } else {
                    coef = 0.0;
                    float _e511 = d_7;
                    if ((_e511 < dd_1)) {
                        float _e513 = d_7;
                        coef = clamp((pow((_e513 / dd_1), 1.9) * 0.5), 0.0, 1.0);
                    } else {
                        float _e522 = d_7;
                        coef = (1.0 - clamp((pow((dd_1 / _e522), 1.9) * 0.5), 0.0, 1.0));
                    }
                    uint _e533 = mat_count;
                    mats_1[_e533] = shape_9.material;
                    uint _e536 = mat_count;
                    float _e538 = coef;
                    mats_wieghts_1[_e536] = _e538;
                    float _e539 = coef;
                    float mult = (1.0 - _e539);
                    k_1 = 0u;
                    bool loop_init_12 = true;
                    while(true) {
                        if (!loop_init_12) {
                            uint _e552 = k_1;
                            k_1 = (_e552 + 1u);
                        }
                        loop_init_12 = false;
                        uint _e544 = k_1;
                        uint _e545 = mat_count;
                        if ((_e544 < _e545)) {
                        } else {
                            break;
                        }
                        {
                            uint _e547 = k_1;
                            float _e549 = mats_wieghts_1[_e547];
                            mats_wieghts_1[_e547] = (_e549 * mult);
                        }
                    }
                    uint _e555 = mat_count;
                    mat_count = (_e555 + 1u);
                    float _e557 = d_7;
                    d_7 = min(_e557, dd_1);
                }
            }
        }
    }
    i_15 = ismda.st_s_spheres_start;
    bool loop_init_13 = true;
    while(true) {
        if (!loop_init_13) {
            uint _e655 = i_15;
            i_15 = (_e655 + 1u);
        }
        loop_init_13 = false;
        uint _e564 = i_15;
        if ((_e564 < (ismda.st_s_spheres_amount + ismda.st_s_spheres_start))) {
        } else {
            break;
        }
        {
            uint _e570 = i_15;
            uint j_11 = in_1.ish[_e570];
            Shape shape_10 = _group_0_binding_2_fs[j_11];
            float _e580 = sd_sphere((p_17 - shape_10.pos), shape_10.size.x);
            float dd_2 = (_e580 - shape_10.roundness);
            if ((dd_2 < 0.02)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                mats_1[0] = shape_10.material;
                return;
            }
            float _e592 = _group_0_binding_4_fs.stickiness;
            if ((dd_2 < (_e592 * STICKINESS_EFFECT_COEF))) {
                uint _e596 = mat_count;
                if ((_e596 == 0u)) {
                    mat_count = 1u;
                    mats_wieghts_1[0] = 1.0;
                    mats_1[0] = shape_10.material;
                    d_7 = dd_2;
                } else {
                    coef_1 = 0.0;
                    float _e606 = d_7;
                    if ((_e606 < dd_2)) {
                        float _e608 = d_7;
                        coef_1 = clamp((pow((_e608 / dd_2), 1.9) * 0.5), 0.0, 1.0);
                    } else {
                        float _e617 = d_7;
                        coef_1 = (1.0 - clamp((pow((dd_2 / _e617), 1.9) * 0.5), 0.0, 1.0));
                    }
                    uint _e628 = mat_count;
                    mats_1[_e628] = shape_10.material;
                    uint _e631 = mat_count;
                    float _e633 = coef_1;
                    mats_wieghts_1[_e631] = _e633;
                    float _e634 = coef_1;
                    float mult_1 = (1.0 - _e634);
                    k_2 = 0u;
                    bool loop_init_14 = true;
                    while(true) {
                        if (!loop_init_14) {
                            uint _e647 = k_2;
                            k_2 = (_e647 + 1u);
                        }
                        loop_init_14 = false;
                        uint _e639 = k_2;
                        uint _e640 = mat_count;
                        if ((_e639 < _e640)) {
                        } else {
                            break;
                        }
                        {
                            uint _e642 = k_2;
                            float _e644 = mats_wieghts_1[_e642];
                            mats_wieghts_1[_e642] = (_e644 * mult_1);
                        }
                    }
                    uint _e650 = mat_count;
                    mat_count = (_e650 + 1u);
                    float _e652 = d_7;
                    d_7 = min(_e652, dd_2);
                }
            }
        }
    }
    i_16 = ismda.st_s_sph_cubes_start;
    bool loop_init_15 = true;
    while(true) {
        if (!loop_init_15) {
            uint _e749 = i_16;
            i_16 = (_e749 + 1u);
        }
        loop_init_15 = false;
        uint _e659 = i_16;
        if ((_e659 < (ismda.st_s_sph_cubes_amount + ismda.st_s_sph_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e665 = i_16;
            uint j_12 = in_1.ish[_e665];
            Shape shape_11 = _group_0_binding_2_fs[j_12];
            float _e674 = sd_sph_box((p_17 - shape_11.pos), shape_11.size);
            float dd_3 = (_e674 - shape_11.roundness);
            if ((dd_3 < 0.02)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                mats_1[0] = shape_11.material;
                return;
            }
            float _e686 = _group_0_binding_4_fs.stickiness;
            if ((dd_3 < (_e686 * STICKINESS_EFFECT_COEF))) {
                uint _e690 = mat_count;
                if ((_e690 == 0u)) {
                    mat_count = 1u;
                    mats_wieghts_1[0] = 1.0;
                    mats_1[0] = shape_11.material;
                    d_7 = dd_3;
                } else {
                    coef_2 = 0.0;
                    float _e700 = d_7;
                    if ((_e700 < dd_3)) {
                        float _e702 = d_7;
                        coef_2 = clamp((pow((_e702 / dd_3), 1.9) * 0.5), 0.0, 1.0);
                    } else {
                        float _e711 = d_7;
                        coef_2 = (1.0 - clamp((pow((dd_3 / _e711), 1.9) * 0.5), 0.0, 1.0));
                    }
                    uint _e722 = mat_count;
                    mats_1[_e722] = shape_11.material;
                    uint _e725 = mat_count;
                    float _e727 = coef_2;
                    mats_wieghts_1[_e725] = _e727;
                    float _e728 = coef_2;
                    float mult_2 = (1.0 - _e728);
                    k_3 = 0u;
                    bool loop_init_16 = true;
                    while(true) {
                        if (!loop_init_16) {
                            uint _e741 = k_3;
                            k_3 = (_e741 + 1u);
                        }
                        loop_init_16 = false;
                        uint _e733 = k_3;
                        uint _e734 = mat_count;
                        if ((_e733 < _e734)) {
                        } else {
                            break;
                        }
                        {
                            uint _e736 = k_3;
                            float _e738 = mats_wieghts_1[_e736];
                            mats_wieghts_1[_e736] = (_e738 * mult_2);
                        }
                    }
                    uint _e744 = mat_count;
                    mat_count = (_e744 + 1u);
                    float _e746 = d_7;
                    d_7 = min(_e746, dd_3);
                }
            }
        }
    }
    i_17 = ismda.st_s_inf_cubes_start;
    bool loop_init_17 = true;
    while(true) {
        if (!loop_init_17) {
            uint _e844 = i_17;
            i_17 = (_e844 + 1u);
        }
        loop_init_17 = false;
        uint _e753 = i_17;
        if ((_e753 < (ismda.st_s_inf_cubes_amount + ismda.st_s_inf_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e759 = i_17;
            uint j_13 = in_1.ish[_e759];
            Shape shape_12 = _group_0_binding_2_fs[j_13];
            float _e769 = sd_inf_box((p_17 - shape_12.pos), shape_12.size.xyz);
            float dd_4 = (_e769 - shape_12.roundness);
            if ((dd_4 < 0.02)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                mats_1[0] = shape_12.material;
                return;
            }
            float _e781 = _group_0_binding_4_fs.stickiness;
            if ((dd_4 < (_e781 * STICKINESS_EFFECT_COEF))) {
                uint _e785 = mat_count;
                if ((_e785 == 0u)) {
                    mat_count = 1u;
                    mats_wieghts_1[0] = 1.0;
                    mats_1[0] = shape_12.material;
                    d_7 = dd_4;
                } else {
                    coef_3 = 0.0;
                    float _e795 = d_7;
                    if ((_e795 < dd_4)) {
                        float _e797 = d_7;
                        coef_3 = clamp((pow((_e797 / dd_4), 1.9) * 0.5), 0.0, 1.0);
                    } else {
                        float _e806 = d_7;
                        coef_3 = (1.0 - clamp((pow((dd_4 / _e806), 1.9) * 0.5), 0.0, 1.0));
                    }
                    uint _e817 = mat_count;
                    mats_1[_e817] = shape_12.material;
                    uint _e820 = mat_count;
                    float _e822 = coef_3;
                    mats_wieghts_1[_e820] = _e822;
                    float _e823 = coef_3;
                    float mult_3 = (1.0 - _e823);
                    k_4 = 0u;
                    bool loop_init_18 = true;
                    while(true) {
                        if (!loop_init_18) {
                            uint _e836 = k_4;
                            k_4 = (_e836 + 1u);
                        }
                        loop_init_18 = false;
                        uint _e828 = k_4;
                        uint _e829 = mat_count;
                        if ((_e828 < _e829)) {
                        } else {
                            break;
                        }
                        {
                            uint _e831 = k_4;
                            float _e833 = mats_wieghts_1[_e831];
                            mats_wieghts_1[_e831] = (_e833 * mult_3);
                        }
                    }
                    uint _e839 = mat_count;
                    mat_count = (_e839 + 1u);
                    float _e841 = d_7;
                    d_7 = min(_e841, dd_4);
                }
            }
        }
    }
    i_18 = ismda.dyn_s_cubes_start;
    bool loop_init_19 = true;
    while(true) {
        if (!loop_init_19) {
            uint _e938 = i_18;
            i_18 = (_e938 + 1u);
        }
        loop_init_19 = false;
        uint _e848 = i_18;
        if ((_e848 < (ismda.dyn_s_cubes_amount + ismda.dyn_s_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e854 = i_18;
            uint j_14 = in_1.ish[_e854];
            Shape shape_13 = _group_0_binding_7_fs[j_14];
            float _e863 = sd_box((p_17 - shape_13.pos), shape_13.size);
            float dd_5 = (_e863 - shape_13.roundness);
            if ((dd_5 < 0.02)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                mats_1[0] = shape_13.material;
                return;
            }
            float _e875 = _group_0_binding_4_fs.stickiness;
            if ((dd_5 < (_e875 * STICKINESS_EFFECT_COEF))) {
                uint _e879 = mat_count;
                if ((_e879 == 0u)) {
                    mat_count = 1u;
                    mats_wieghts_1[0] = 1.0;
                    mats_1[0] = shape_13.material;
                    d_7 = dd_5;
                } else {
                    coef_4 = 0.0;
                    float _e889 = d_7;
                    if ((_e889 < dd_5)) {
                        float _e891 = d_7;
                        coef_4 = clamp((pow((_e891 / dd_5), 1.9) * 0.5), 0.0, 1.0);
                    } else {
                        float _e900 = d_7;
                        coef_4 = (1.0 - clamp((pow((dd_5 / _e900), 1.9) * 0.5), 0.0, 1.0));
                    }
                    uint _e911 = mat_count;
                    mats_1[_e911] = shape_13.material;
                    uint _e914 = mat_count;
                    float _e916 = coef_4;
                    mats_wieghts_1[_e914] = _e916;
                    float _e917 = coef_4;
                    float mult_4 = (1.0 - _e917);
                    k_5 = 0u;
                    bool loop_init_20 = true;
                    while(true) {
                        if (!loop_init_20) {
                            uint _e930 = k_5;
                            k_5 = (_e930 + 1u);
                        }
                        loop_init_20 = false;
                        uint _e922 = k_5;
                        uint _e923 = mat_count;
                        if ((_e922 < _e923)) {
                        } else {
                            break;
                        }
                        {
                            uint _e925 = k_5;
                            float _e927 = mats_wieghts_1[_e925];
                            mats_wieghts_1[_e925] = (_e927 * mult_4);
                        }
                    }
                    uint _e933 = mat_count;
                    mat_count = (_e933 + 1u);
                    float _e935 = d_7;
                    d_7 = min(_e935, dd_5);
                }
            }
        }
    }
    i_19 = ismda.dyn_s_spheres_start;
    bool loop_init_21 = true;
    while(true) {
        if (!loop_init_21) {
            uint _e1033 = i_19;
            i_19 = (_e1033 + 1u);
        }
        loop_init_21 = false;
        uint _e942 = i_19;
        if ((_e942 < (ismda.dyn_s_spheres_amount + ismda.dyn_s_spheres_start))) {
        } else {
            break;
        }
        {
            uint _e948 = i_19;
            uint j_15 = in_1.ish[_e948];
            Shape shape_14 = _group_0_binding_7_fs[j_15];
            float _e958 = sd_sphere((p_17 - shape_14.pos), shape_14.size.x);
            float dd_6 = (_e958 - shape_14.roundness);
            if ((dd_6 < 0.02)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                mats_1[0] = shape_14.material;
                return;
            }
            float _e970 = _group_0_binding_4_fs.stickiness;
            if ((dd_6 < (_e970 * STICKINESS_EFFECT_COEF))) {
                uint _e974 = mat_count;
                if ((_e974 == 0u)) {
                    mat_count = 1u;
                    mats_wieghts_1[0] = 1.0;
                    mats_1[0] = shape_14.material;
                    d_7 = dd_6;
                } else {
                    coef_5 = 0.0;
                    float _e984 = d_7;
                    if ((_e984 < dd_6)) {
                        float _e986 = d_7;
                        coef_5 = clamp((pow((_e986 / dd_6), 1.9) * 0.5), 0.0, 1.0);
                    } else {
                        float _e995 = d_7;
                        coef_5 = (1.0 - clamp((pow((dd_6 / _e995), 1.9) * 0.5), 0.0, 1.0));
                    }
                    uint _e1006 = mat_count;
                    mats_1[_e1006] = shape_14.material;
                    uint _e1009 = mat_count;
                    float _e1011 = coef_5;
                    mats_wieghts_1[_e1009] = _e1011;
                    float _e1012 = coef_5;
                    float mult_5 = (1.0 - _e1012);
                    k_6 = 0u;
                    bool loop_init_22 = true;
                    while(true) {
                        if (!loop_init_22) {
                            uint _e1025 = k_6;
                            k_6 = (_e1025 + 1u);
                        }
                        loop_init_22 = false;
                        uint _e1017 = k_6;
                        uint _e1018 = mat_count;
                        if ((_e1017 < _e1018)) {
                        } else {
                            break;
                        }
                        {
                            uint _e1020 = k_6;
                            float _e1022 = mats_wieghts_1[_e1020];
                            mats_wieghts_1[_e1020] = (_e1022 * mult_5);
                        }
                    }
                    uint _e1028 = mat_count;
                    mat_count = (_e1028 + 1u);
                    float _e1030 = d_7;
                    d_7 = min(_e1030, dd_6);
                }
            }
        }
    }
    i_20 = ismda.dyn_s_sph_cubes_start;
    bool loop_init_23 = true;
    while(true) {
        if (!loop_init_23) {
            uint _e1127 = i_20;
            i_20 = (_e1127 + 1u);
        }
        loop_init_23 = false;
        uint _e1037 = i_20;
        if ((_e1037 < (ismda.dyn_s_sph_cubes_amount + ismda.dyn_s_sph_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e1043 = i_20;
            uint j_16 = in_1.ish[_e1043];
            Shape shape_15 = _group_0_binding_7_fs[j_16];
            float _e1052 = sd_sph_box((p_17 - shape_15.pos), shape_15.size);
            float dd_7 = (_e1052 - shape_15.roundness);
            if ((dd_7 < 0.02)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                mats_1[0] = shape_15.material;
                return;
            }
            float _e1064 = _group_0_binding_4_fs.stickiness;
            if ((dd_7 < (_e1064 * STICKINESS_EFFECT_COEF))) {
                uint _e1068 = mat_count;
                if ((_e1068 == 0u)) {
                    mat_count = 1u;
                    mats_wieghts_1[0] = 1.0;
                    mats_1[0] = shape_15.material;
                    d_7 = dd_7;
                } else {
                    coef_6 = 0.0;
                    float _e1078 = d_7;
                    if ((_e1078 < dd_7)) {
                        float _e1080 = d_7;
                        coef_6 = clamp((pow((_e1080 / dd_7), 1.9) * 0.5), 0.0, 1.0);
                    } else {
                        float _e1089 = d_7;
                        coef_6 = (1.0 - clamp((pow((dd_7 / _e1089), 1.9) * 0.5), 0.0, 1.0));
                    }
                    uint _e1100 = mat_count;
                    mats_1[_e1100] = shape_15.material;
                    uint _e1103 = mat_count;
                    float _e1105 = coef_6;
                    mats_wieghts_1[_e1103] = _e1105;
                    float _e1106 = coef_6;
                    float mult_6 = (1.0 - _e1106);
                    k_7 = 0u;
                    bool loop_init_24 = true;
                    while(true) {
                        if (!loop_init_24) {
                            uint _e1119 = k_7;
                            k_7 = (_e1119 + 1u);
                        }
                        loop_init_24 = false;
                        uint _e1111 = k_7;
                        uint _e1112 = mat_count;
                        if ((_e1111 < _e1112)) {
                        } else {
                            break;
                        }
                        {
                            uint _e1114 = k_7;
                            float _e1116 = mats_wieghts_1[_e1114];
                            mats_wieghts_1[_e1114] = (_e1116 * mult_6);
                        }
                    }
                    uint _e1122 = mat_count;
                    mat_count = (_e1122 + 1u);
                    float _e1124 = d_7;
                    d_7 = min(_e1124, dd_7);
                }
            }
        }
    }
    i_21 = ismda.dyn_s_inf_cubes_start;
    bool loop_init_25 = true;
    while(true) {
        if (!loop_init_25) {
            uint _e1222 = i_21;
            i_21 = (_e1222 + 1u);
        }
        loop_init_25 = false;
        uint _e1131 = i_21;
        if ((_e1131 < (ismda.dyn_s_inf_cubes_amount + ismda.dyn_s_inf_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e1137 = i_21;
            uint j_17 = in_1.ish[_e1137];
            Shape shape_16 = _group_0_binding_7_fs[j_17];
            float _e1147 = sd_inf_box((p_17 - shape_16.pos), shape_16.size.xyz);
            float dd_8 = (_e1147 - shape_16.roundness);
            if ((dd_8 < 0.02)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                mats_1[0] = shape_16.material;
                return;
            }
            float _e1159 = _group_0_binding_4_fs.stickiness;
            if ((dd_8 < (_e1159 * STICKINESS_EFFECT_COEF))) {
                uint _e1163 = mat_count;
                if ((_e1163 == 0u)) {
                    mat_count = 1u;
                    mats_wieghts_1[0] = 1.0;
                    mats_1[0] = shape_16.material;
                    d_7 = dd_8;
                } else {
                    coef_7 = 0.0;
                    float _e1173 = d_7;
                    if ((_e1173 < dd_8)) {
                        float _e1175 = d_7;
                        coef_7 = clamp((pow((_e1175 / dd_8), 1.9) * 0.5), 0.0, 1.0);
                    } else {
                        float _e1184 = d_7;
                        coef_7 = (1.0 - clamp((pow((dd_8 / _e1184), 1.9) * 0.5), 0.0, 1.0));
                    }
                    uint _e1195 = mat_count;
                    mats_1[_e1195] = shape_16.material;
                    uint _e1198 = mat_count;
                    float _e1200 = coef_7;
                    mats_wieghts_1[_e1198] = _e1200;
                    float _e1201 = coef_7;
                    float mult_7 = (1.0 - _e1201);
                    k_8 = 0u;
                    bool loop_init_26 = true;
                    while(true) {
                        if (!loop_init_26) {
                            uint _e1214 = k_8;
                            k_8 = (_e1214 + 1u);
                        }
                        loop_init_26 = false;
                        uint _e1206 = k_8;
                        uint _e1207 = mat_count;
                        if ((_e1206 < _e1207)) {
                        } else {
                            break;
                        }
                        {
                            uint _e1209 = k_8;
                            float _e1211 = mats_wieghts_1[_e1209];
                            mats_wieghts_1[_e1209] = (_e1211 * mult_7);
                        }
                    }
                    uint _e1217 = mat_count;
                    mat_count = (_e1217 + 1u);
                    float _e1219 = d_7;
                    d_7 = min(_e1219, dd_8);
                }
            }
        }
    }
    uint _e1224 = mat_count;
    if ((_e1224 == 0u)) {
        mat_count = 1u;
        mats_wieghts_1[0] = 1.0;
        mats_1[0] = -1;
    }
    return;
}

float map(vec4 p_15, inout Intersections in_2) {
    float d_8 = MAX_DIST;
    uint i_22 = 0u;
    uint i_23 = 0u;
    uint i_24 = 0u;
    uint i_25 = 0u;
    uint i_26 = 0u;
    uint i_27 = 0u;
    uint i_28 = 0u;
    uint i_29 = 0u;
    uint i_30 = 0u;
    uint i_31 = 0u;
    uint i_32 = 0u;
    uint i_33 = 0u;
    uint i_34 = 0u;
    uint i_35 = 0u;
    uint i_36 = 0u;
    uint i_37 = 0u;
    float dd = MAX_DIST;
    uint i_38 = 0u;
    uint i_39 = 0u;
    uint i_40 = 0u;
    uint i_41 = 0u;
    float ddd = 0.0;
    uint i_42 = 0u;
    uint i_43 = 0u;
    uint i_44 = 0u;
    uint i_45 = 0u;
    uint i_46 = 0u;
    uint i_47 = 0u;
    uint i_48 = 0u;
    uint i_49 = 0u;
    uint i_50 = 0u;
    uint i_51 = 0u;
    uint i_52 = 0u;
    uint i_53 = 0u;
    float dddd = MAX_DIST;
    uint j_1 = 0u;
    IntersectedShapesMetadata ismda_1 = in_2.ismd;
    i_22 = ismda_1.st_s_cubes_start;
    bool loop_init_27 = true;
    while(true) {
        if (!loop_init_27) {
            uint _e32 = i_22;
            i_22 = (_e32 + 1u);
        }
        loop_init_27 = false;
        uint _e8 = i_22;
        if ((_e8 < (ismda_1.st_s_cubes_amount + ismda_1.st_s_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e14 = i_22;
            uint j_18 = in_2.ish[_e14];
            Shape shape_17 = _group_0_binding_2_fs[j_18];
            float _e20 = d_8;
            float _e24 = sd_box((p_15 - shape_17.pos), shape_17.size);
            float _e29 = _group_0_binding_4_fs.stickiness;
            float _e30 = smin(_e20, (_e24 - shape_17.roundness), _e29);
            d_8 = _e30;
        }
    }
    i_23 = ismda_1.st_s_spheres_start;
    bool loop_init_28 = true;
    while(true) {
        if (!loop_init_28) {
            uint _e61 = i_23;
            i_23 = (_e61 + 1u);
        }
        loop_init_28 = false;
        uint _e36 = i_23;
        if ((_e36 < (ismda_1.st_s_spheres_amount + ismda_1.st_s_spheres_start))) {
        } else {
            break;
        }
        {
            uint _e42 = i_23;
            uint j_19 = in_2.ish[_e42];
            Shape shape_18 = _group_0_binding_2_fs[j_19];
            float _e48 = d_8;
            float _e53 = sd_sphere((p_15 - shape_18.pos), shape_18.size.x);
            float _e58 = _group_0_binding_4_fs.stickiness;
            float _e59 = smin(_e48, (_e53 - shape_18.roundness), _e58);
            d_8 = _e59;
        }
    }
    i_24 = ismda_1.st_s_sph_cubes_start;
    bool loop_init_29 = true;
    while(true) {
        if (!loop_init_29) {
            uint _e89 = i_24;
            i_24 = (_e89 + 1u);
        }
        loop_init_29 = false;
        uint _e65 = i_24;
        if ((_e65 < (ismda_1.st_s_sph_cubes_amount + ismda_1.st_s_sph_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e71 = i_24;
            uint j_20 = in_2.ish[_e71];
            Shape shape_19 = _group_0_binding_2_fs[j_20];
            float _e77 = d_8;
            float _e81 = sd_sph_box((p_15 - shape_19.pos), shape_19.size);
            float _e86 = _group_0_binding_4_fs.stickiness;
            float _e87 = smin(_e77, (_e81 - shape_19.roundness), _e86);
            d_8 = _e87;
        }
    }
    i_25 = ismda_1.st_s_inf_cubes_start;
    bool loop_init_30 = true;
    while(true) {
        if (!loop_init_30) {
            uint _e118 = i_25;
            i_25 = (_e118 + 1u);
        }
        loop_init_30 = false;
        uint _e93 = i_25;
        if ((_e93 < (ismda_1.st_s_inf_cubes_amount + ismda_1.st_s_inf_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e99 = i_25;
            uint j_21 = in_2.ish[_e99];
            Shape shape_20 = _group_0_binding_2_fs[j_21];
            float _e105 = d_8;
            float _e110 = sd_inf_box((p_15 - shape_20.pos), shape_20.size.xyz);
            float _e115 = _group_0_binding_4_fs.stickiness;
            float _e116 = smin(_e105, (_e110 - shape_20.roundness), _e115);
            d_8 = _e116;
        }
    }
    i_26 = ismda_1.dyn_s_cubes_start;
    bool loop_init_31 = true;
    while(true) {
        if (!loop_init_31) {
            uint _e146 = i_26;
            i_26 = (_e146 + 1u);
        }
        loop_init_31 = false;
        uint _e122 = i_26;
        if ((_e122 < (ismda_1.dyn_s_cubes_amount + ismda_1.dyn_s_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e128 = i_26;
            uint j_22 = in_2.ish[_e128];
            Shape shape_21 = _group_0_binding_7_fs[j_22];
            float _e134 = d_8;
            float _e138 = sd_box((p_15 - shape_21.pos), shape_21.size);
            float _e143 = _group_0_binding_4_fs.stickiness;
            float _e144 = smin(_e134, (_e138 - shape_21.roundness), _e143);
            d_8 = _e144;
        }
    }
    i_27 = ismda_1.dyn_s_spheres_start;
    bool loop_init_32 = true;
    while(true) {
        if (!loop_init_32) {
            uint _e175 = i_27;
            i_27 = (_e175 + 1u);
        }
        loop_init_32 = false;
        uint _e150 = i_27;
        if ((_e150 < (ismda_1.dyn_s_spheres_amount + ismda_1.dyn_s_spheres_start))) {
        } else {
            break;
        }
        {
            uint _e156 = i_27;
            uint j_23 = in_2.ish[_e156];
            Shape shape_22 = _group_0_binding_7_fs[j_23];
            float _e162 = d_8;
            float _e167 = sd_sphere((p_15 - shape_22.pos), shape_22.size.x);
            float _e172 = _group_0_binding_4_fs.stickiness;
            float _e173 = smin(_e162, (_e167 - shape_22.roundness), _e172);
            d_8 = _e173;
        }
    }
    i_28 = ismda_1.dyn_s_sph_cubes_start;
    bool loop_init_33 = true;
    while(true) {
        if (!loop_init_33) {
            uint _e203 = i_28;
            i_28 = (_e203 + 1u);
        }
        loop_init_33 = false;
        uint _e179 = i_28;
        if ((_e179 < (ismda_1.dyn_s_sph_cubes_amount + ismda_1.dyn_s_sph_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e185 = i_28;
            uint j_24 = in_2.ish[_e185];
            Shape shape_23 = _group_0_binding_7_fs[j_24];
            float _e191 = d_8;
            float _e195 = sd_sph_box((p_15 - shape_23.pos), shape_23.size);
            float _e200 = _group_0_binding_4_fs.stickiness;
            float _e201 = smin(_e191, (_e195 - shape_23.roundness), _e200);
            d_8 = _e201;
        }
    }
    i_29 = ismda_1.dyn_s_inf_cubes_start;
    bool loop_init_34 = true;
    while(true) {
        if (!loop_init_34) {
            uint _e232 = i_29;
            i_29 = (_e232 + 1u);
        }
        loop_init_34 = false;
        uint _e207 = i_29;
        if ((_e207 < (ismda_1.dyn_s_inf_cubes_amount + ismda_1.dyn_s_inf_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e213 = i_29;
            uint j_25 = in_2.ish[_e213];
            Shape shape_24 = _group_0_binding_7_fs[j_25];
            float _e219 = d_8;
            float _e224 = sd_inf_box((p_15 - shape_24.pos), shape_24.size.xyz);
            float _e229 = _group_0_binding_4_fs.stickiness;
            float _e230 = smin(_e219, (_e224 - shape_24.roundness), _e229);
            d_8 = _e230;
        }
    }
    i_30 = ismda_1.st_cubes_start;
    bool loop_init_35 = true;
    while(true) {
        if (!loop_init_35) {
            uint _e257 = i_30;
            i_30 = (_e257 + 1u);
        }
        loop_init_35 = false;
        uint _e236 = i_30;
        if ((_e236 < (ismda_1.st_cubes_amount + ismda_1.st_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e242 = i_30;
            uint j_26 = in_2.ish[_e242];
            Shape shape_25 = _group_0_binding_0_fs[j_26];
            float _e248 = d_8;
            float _e252 = sd_box((p_15 - shape_25.pos), shape_25.size);
            d_8 = min(_e248, (_e252 - shape_25.roundness));
        }
    }
    i_31 = ismda_1.st_spheres_start;
    bool loop_init_36 = true;
    while(true) {
        if (!loop_init_36) {
            uint _e283 = i_31;
            i_31 = (_e283 + 1u);
        }
        loop_init_36 = false;
        uint _e261 = i_31;
        if ((_e261 < (ismda_1.st_spheres_amount + ismda_1.st_spheres_start))) {
        } else {
            break;
        }
        {
            uint _e267 = i_31;
            uint j_27 = in_2.ish[_e267];
            Shape shape_26 = _group_0_binding_0_fs[j_27];
            float _e273 = d_8;
            float _e278 = sd_sphere((p_15 - shape_26.pos), shape_26.size.x);
            d_8 = min(_e273, (_e278 - shape_26.roundness));
        }
    }
    i_32 = ismda_1.st_sph_cubes_start;
    bool loop_init_37 = true;
    while(true) {
        if (!loop_init_37) {
            uint _e308 = i_32;
            i_32 = (_e308 + 1u);
        }
        loop_init_37 = false;
        uint _e287 = i_32;
        if ((_e287 < (ismda_1.st_sph_cubes_amount + ismda_1.st_sph_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e293 = i_32;
            uint j_28 = in_2.ish[_e293];
            Shape shape_27 = _group_0_binding_0_fs[j_28];
            float _e299 = d_8;
            float _e303 = sd_sph_box((p_15 - shape_27.pos), shape_27.size);
            d_8 = min(_e299, (_e303 - shape_27.roundness));
        }
    }
    i_33 = ismda_1.st_inf_cubes_start;
    bool loop_init_38 = true;
    while(true) {
        if (!loop_init_38) {
            uint _e334 = i_33;
            i_33 = (_e334 + 1u);
        }
        loop_init_38 = false;
        uint _e312 = i_33;
        if ((_e312 < (ismda_1.st_inf_cubes_amount + ismda_1.st_inf_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e318 = i_33;
            uint j_29 = in_2.ish[_e318];
            Shape shape_28 = _group_0_binding_0_fs[j_29];
            float _e324 = d_8;
            float _e329 = sd_inf_box((p_15 - shape_28.pos), shape_28.size.xyz);
            d_8 = min(_e324, (_e329 - shape_28.roundness));
        }
    }
    i_34 = ismda_1.dyn_cubes_start;
    bool loop_init_39 = true;
    while(true) {
        if (!loop_init_39) {
            uint _e359 = i_34;
            i_34 = (_e359 + 1u);
        }
        loop_init_39 = false;
        uint _e338 = i_34;
        if ((_e338 < (ismda_1.dyn_cubes_amount + ismda_1.dyn_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e344 = i_34;
            uint j_30 = in_2.ish[_e344];
            Shape shape_29 = _group_0_binding_5_fs[j_30];
            float _e350 = d_8;
            float _e354 = sd_box((p_15 - shape_29.pos), shape_29.size);
            d_8 = min(_e350, (_e354 - shape_29.roundness));
        }
    }
    i_35 = ismda_1.dyn_spheres_start;
    bool loop_init_40 = true;
    while(true) {
        if (!loop_init_40) {
            uint _e385 = i_35;
            i_35 = (_e385 + 1u);
        }
        loop_init_40 = false;
        uint _e363 = i_35;
        if ((_e363 < (ismda_1.dyn_spheres_amount + ismda_1.dyn_spheres_start))) {
        } else {
            break;
        }
        {
            uint _e369 = i_35;
            uint j_31 = in_2.ish[_e369];
            Shape shape_30 = _group_0_binding_5_fs[j_31];
            float _e375 = d_8;
            float _e380 = sd_sphere((p_15 - shape_30.pos), shape_30.size.x);
            d_8 = min(_e375, (_e380 - shape_30.roundness));
        }
    }
    i_36 = ismda_1.dyn_sph_cubes_start;
    bool loop_init_41 = true;
    while(true) {
        if (!loop_init_41) {
            uint _e410 = i_36;
            i_36 = (_e410 + 1u);
        }
        loop_init_41 = false;
        uint _e389 = i_36;
        if ((_e389 < (ismda_1.dyn_sph_cubes_amount + ismda_1.dyn_sph_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e395 = i_36;
            uint j_32 = in_2.ish[_e395];
            Shape shape_31 = _group_0_binding_5_fs[j_32];
            float _e401 = d_8;
            float _e405 = sd_sph_box((p_15 - shape_31.pos), shape_31.size);
            d_8 = min(_e401, (_e405 - shape_31.roundness));
        }
    }
    i_37 = ismda_1.dyn_inf_cubes_start;
    bool loop_init_42 = true;
    while(true) {
        if (!loop_init_42) {
            uint _e436 = i_37;
            i_37 = (_e436 + 1u);
        }
        loop_init_42 = false;
        uint _e414 = i_37;
        if ((_e414 < (ismda_1.dyn_inf_cubes_amount + ismda_1.dyn_inf_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e420 = i_37;
            uint j_33 = in_2.ish[_e420];
            Shape shape_32 = _group_0_binding_5_fs[j_33];
            float _e426 = d_8;
            float _e431 = sd_inf_box((p_15 - shape_32.pos), shape_32.size.xyz);
            d_8 = min(_e426, (_e431 - shape_32.roundness));
        }
    }
    i_38 = ismda_1.st_s_neg_cubes_start;
    bool loop_init_43 = true;
    while(true) {
        if (!loop_init_43) {
            uint _e466 = i_38;
            i_38 = (_e466 + 1u);
        }
        loop_init_43 = false;
        uint _e442 = i_38;
        if ((_e442 < (ismda_1.st_s_neg_cubes_amount + ismda_1.st_s_neg_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e448 = i_38;
            uint j_34 = in_2.ish[_e448];
            Shape shape_33 = _group_0_binding_3_fs[j_34];
            float _e454 = dd;
            float _e458 = sd_box((p_15 - shape_33.pos), shape_33.size);
            float _e463 = _group_0_binding_4_fs.stickiness;
            float _e464 = smin(_e454, (_e458 - shape_33.roundness), _e463);
            dd = _e464;
        }
    }
    i_39 = ismda_1.st_s_neg_spheres_start;
    bool loop_init_44 = true;
    while(true) {
        if (!loop_init_44) {
            uint _e495 = i_39;
            i_39 = (_e495 + 1u);
        }
        loop_init_44 = false;
        uint _e470 = i_39;
        if ((_e470 < (ismda_1.st_s_neg_spheres_amount + ismda_1.st_s_neg_spheres_start))) {
        } else {
            break;
        }
        {
            uint _e476 = i_39;
            uint j_35 = in_2.ish[_e476];
            Shape shape_34 = _group_0_binding_3_fs[j_35];
            float _e482 = dd;
            float _e487 = sd_sphere((p_15 - shape_34.pos), shape_34.size.x);
            float _e492 = _group_0_binding_4_fs.stickiness;
            float _e493 = smin(_e482, (_e487 - shape_34.roundness), _e492);
            dd = _e493;
        }
    }
    i_40 = ismda_1.st_s_neg_sph_cubes_start;
    bool loop_init_45 = true;
    while(true) {
        if (!loop_init_45) {
            uint _e523 = i_40;
            i_40 = (_e523 + 1u);
        }
        loop_init_45 = false;
        uint _e499 = i_40;
        if ((_e499 < (ismda_1.st_s_neg_sph_cubes_amount + ismda_1.st_s_neg_sph_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e505 = i_40;
            uint j_36 = in_2.ish[_e505];
            Shape shape_35 = _group_0_binding_3_fs[j_36];
            float _e511 = dd;
            float _e515 = sd_sph_box((p_15 - shape_35.pos), shape_35.size);
            float _e520 = _group_0_binding_4_fs.stickiness;
            float _e521 = smin(_e511, (_e515 - shape_35.roundness), _e520);
            dd = _e521;
        }
    }
    i_41 = ismda_1.st_s_neg_inf_cubes_start;
    bool loop_init_46 = true;
    while(true) {
        if (!loop_init_46) {
            uint _e552 = i_41;
            i_41 = (_e552 + 1u);
        }
        loop_init_46 = false;
        uint _e527 = i_41;
        if ((_e527 < (ismda_1.st_s_neg_inf_cubes_amount + ismda_1.st_s_neg_inf_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e533 = i_41;
            uint j_37 = in_2.ish[_e533];
            Shape shape_36 = _group_0_binding_3_fs[j_37];
            float _e539 = dd;
            float _e544 = sd_inf_box((p_15 - shape_36.pos), shape_36.size.xyz);
            float _e549 = _group_0_binding_4_fs.stickiness;
            float _e550 = smin(_e539, (_e544 - shape_36.roundness), _e549);
            dd = _e550;
        }
    }
    float _e554 = d_8;
    float _e555 = dd;
    d_8 = max(_e554, -(_e555));
    float _e558 = dd;
    ddd = _e558;
    i_42 = ismda_1.dyn_s_neg_cubes_start;
    bool loop_init_47 = true;
    while(true) {
        if (!loop_init_47) {
            uint _e586 = i_42;
            i_42 = (_e586 + 1u);
        }
        loop_init_47 = false;
        uint _e562 = i_42;
        if ((_e562 < (ismda_1.dyn_s_neg_cubes_amount + ismda_1.dyn_s_neg_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e568 = i_42;
            uint j_38 = in_2.ish[_e568];
            Shape shape_37 = _group_0_binding_8_fs[j_38];
            float _e574 = ddd;
            float _e578 = sd_box((p_15 - shape_37.pos), shape_37.size);
            float _e583 = _group_0_binding_4_fs.stickiness;
            float _e584 = smin(_e574, (_e578 - shape_37.roundness), _e583);
            ddd = _e584;
        }
    }
    i_43 = ismda_1.dyn_s_neg_spheres_start;
    bool loop_init_48 = true;
    while(true) {
        if (!loop_init_48) {
            uint _e615 = i_43;
            i_43 = (_e615 + 1u);
        }
        loop_init_48 = false;
        uint _e590 = i_43;
        if ((_e590 < (ismda_1.dyn_s_neg_spheres_amount + ismda_1.dyn_s_neg_spheres_start))) {
        } else {
            break;
        }
        {
            uint _e596 = i_43;
            uint j_39 = in_2.ish[_e596];
            Shape shape_38 = _group_0_binding_8_fs[j_39];
            float _e602 = ddd;
            float _e607 = sd_sphere((p_15 - shape_38.pos), shape_38.size.x);
            float _e612 = _group_0_binding_4_fs.stickiness;
            float _e613 = smin(_e602, (_e607 - shape_38.roundness), _e612);
            ddd = _e613;
        }
    }
    i_44 = ismda_1.dyn_s_neg_sph_cubes_start;
    bool loop_init_49 = true;
    while(true) {
        if (!loop_init_49) {
            uint _e643 = i_44;
            i_44 = (_e643 + 1u);
        }
        loop_init_49 = false;
        uint _e619 = i_44;
        if ((_e619 < (ismda_1.dyn_s_neg_sph_cubes_amount + ismda_1.dyn_s_neg_sph_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e625 = i_44;
            uint j_40 = in_2.ish[_e625];
            Shape shape_39 = _group_0_binding_8_fs[j_40];
            float _e631 = ddd;
            float _e635 = sd_sph_box((p_15 - shape_39.pos), shape_39.size);
            float _e640 = _group_0_binding_4_fs.stickiness;
            float _e641 = smin(_e631, (_e635 - shape_39.roundness), _e640);
            ddd = _e641;
        }
    }
    i_45 = ismda_1.dyn_s_neg_inf_cubes_start;
    bool loop_init_50 = true;
    while(true) {
        if (!loop_init_50) {
            uint _e672 = i_45;
            i_45 = (_e672 + 1u);
        }
        loop_init_50 = false;
        uint _e647 = i_45;
        if ((_e647 < (ismda_1.dyn_s_neg_inf_cubes_amount + ismda_1.dyn_s_neg_inf_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e653 = i_45;
            uint j_41 = in_2.ish[_e653];
            Shape shape_40 = _group_0_binding_8_fs[j_41];
            float _e659 = ddd;
            float _e664 = sd_inf_box((p_15 - shape_40.pos), shape_40.size.xyz);
            float _e669 = _group_0_binding_4_fs.stickiness;
            float _e670 = smin(_e659, (_e664 - shape_40.roundness), _e669);
            ddd = _e670;
        }
    }
    float _e674 = d_8;
    float _e675 = ddd;
    d_8 = max(_e674, -(_e675));
    i_46 = ismda_1.st_neg_cubes_start;
    bool loop_init_51 = true;
    while(true) {
        if (!loop_init_51) {
            uint _e702 = i_46;
            i_46 = (_e702 + 1u);
        }
        loop_init_51 = false;
        uint _e680 = i_46;
        if ((_e680 < (ismda_1.st_neg_cubes_amount + ismda_1.st_neg_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e686 = i_46;
            uint j_42 = in_2.ish[_e686];
            Shape shape_41 = _group_0_binding_1_fs[j_42];
            float _e692 = d_8;
            float _e696 = sd_box((p_15 - shape_41.pos), shape_41.size);
            d_8 = max(_e692, -((_e696 - shape_41.roundness)));
        }
    }
    i_47 = ismda_1.st_neg_spheres_start;
    bool loop_init_52 = true;
    while(true) {
        if (!loop_init_52) {
            uint _e729 = i_47;
            i_47 = (_e729 + 1u);
        }
        loop_init_52 = false;
        uint _e706 = i_47;
        if ((_e706 < (ismda_1.st_neg_spheres_amount + ismda_1.st_neg_spheres_start))) {
        } else {
            break;
        }
        {
            uint _e712 = i_47;
            uint j_43 = in_2.ish[_e712];
            Shape shape_42 = _group_0_binding_1_fs[j_43];
            float _e718 = d_8;
            float _e723 = sd_sphere((p_15 - shape_42.pos), shape_42.size.x);
            d_8 = max(_e718, -((_e723 - shape_42.roundness)));
        }
    }
    i_48 = ismda_1.st_neg_sph_cubes_start;
    bool loop_init_53 = true;
    while(true) {
        if (!loop_init_53) {
            uint _e755 = i_48;
            i_48 = (_e755 + 1u);
        }
        loop_init_53 = false;
        uint _e733 = i_48;
        if ((_e733 < (ismda_1.st_neg_sph_cubes_amount + ismda_1.st_neg_sph_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e739 = i_48;
            uint j_44 = in_2.ish[_e739];
            Shape shape_43 = _group_0_binding_1_fs[j_44];
            float _e745 = d_8;
            float _e749 = sd_sph_box((p_15 - shape_43.pos), shape_43.size);
            d_8 = max(_e745, -((_e749 - shape_43.roundness)));
        }
    }
    i_49 = ismda_1.st_neg_inf_cubes_start;
    bool loop_init_54 = true;
    while(true) {
        if (!loop_init_54) {
            uint _e782 = i_49;
            i_49 = (_e782 + 1u);
        }
        loop_init_54 = false;
        uint _e759 = i_49;
        if ((_e759 < (ismda_1.st_neg_inf_cubes_amount + ismda_1.st_neg_inf_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e765 = i_49;
            uint j_45 = in_2.ish[_e765];
            Shape shape_44 = _group_0_binding_1_fs[j_45];
            float _e771 = d_8;
            float _e776 = sd_inf_box((p_15 - shape_44.pos), shape_44.size.xyz);
            d_8 = max(_e771, -((_e776 - shape_44.roundness)));
        }
    }
    i_50 = ismda_1.dyn_neg_cubes_start;
    bool loop_init_55 = true;
    while(true) {
        if (!loop_init_55) {
            uint _e808 = i_50;
            i_50 = (_e808 + 1u);
        }
        loop_init_55 = false;
        uint _e786 = i_50;
        if ((_e786 < (ismda_1.dyn_neg_cubes_amount + ismda_1.dyn_neg_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e792 = i_50;
            uint j_46 = in_2.ish[_e792];
            Shape shape_45 = _group_0_binding_6_fs[j_46];
            float _e798 = d_8;
            float _e802 = sd_box((p_15 - shape_45.pos), shape_45.size);
            d_8 = max(_e798, -((_e802 - shape_45.roundness)));
        }
    }
    i_51 = ismda_1.dyn_neg_spheres_start;
    bool loop_init_56 = true;
    while(true) {
        if (!loop_init_56) {
            uint _e835 = i_51;
            i_51 = (_e835 + 1u);
        }
        loop_init_56 = false;
        uint _e812 = i_51;
        if ((_e812 < (ismda_1.dyn_neg_spheres_amount + ismda_1.dyn_neg_spheres_start))) {
        } else {
            break;
        }
        {
            uint _e818 = i_51;
            uint j_47 = in_2.ish[_e818];
            Shape shape_46 = _group_0_binding_6_fs[j_47];
            float _e824 = d_8;
            float _e829 = sd_sphere((p_15 - shape_46.pos), shape_46.size.x);
            d_8 = max(_e824, -((_e829 - shape_46.roundness)));
        }
    }
    i_52 = ismda_1.dyn_neg_sph_cubes_start;
    bool loop_init_57 = true;
    while(true) {
        if (!loop_init_57) {
            uint _e861 = i_52;
            i_52 = (_e861 + 1u);
        }
        loop_init_57 = false;
        uint _e839 = i_52;
        if ((_e839 < (ismda_1.dyn_neg_sph_cubes_amount + ismda_1.dyn_neg_sph_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e845 = i_52;
            uint j_48 = in_2.ish[_e845];
            Shape shape_47 = _group_0_binding_6_fs[j_48];
            float _e851 = d_8;
            float _e855 = sd_sph_box((p_15 - shape_47.pos), shape_47.size);
            d_8 = max(_e851, -((_e855 - shape_47.roundness)));
        }
    }
    i_53 = ismda_1.dyn_neg_inf_cubes_start;
    bool loop_init_58 = true;
    while(true) {
        if (!loop_init_58) {
            uint _e888 = i_53;
            i_53 = (_e888 + 1u);
        }
        loop_init_58 = false;
        uint _e865 = i_53;
        if ((_e865 < (ismda_1.dyn_neg_inf_cubes_amount + ismda_1.dyn_neg_inf_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e871 = i_53;
            uint j_49 = in_2.ish[_e871];
            Shape shape_48 = _group_0_binding_6_fs[j_49];
            float _e877 = d_8;
            float _e882 = sd_inf_box((p_15 - shape_48.pos), shape_48.size.xyz);
            d_8 = max(_e877, -((_e882 - shape_48.roundness)));
        }
    }
    j_1 = ismda_1.player_forms_start;
    bool loop_init_59 = true;
    while(true) {
        if (!loop_init_59) {
            uint _e1039 = j_1;
            j_1 = (_e1039 + 1u);
        }
        loop_init_59 = false;
        uint _e894 = j_1;
        if ((_e894 < (ismda_1.player_forms_amount + ismda_1.player_forms_start))) {
        } else {
            break;
        }
        {
            uint _e900 = j_1;
            uint i_89 = in_2.ish[_e900];
            PlayerForm shape_49 = _group_1_binding_2_fs[i_89];
            float _e906 = dddd;
            float _e910 = sd_sphere((p_15 - shape_49.pos), shape_49.radius);
            dddd = min(_e906, _e910);
            float _e912 = dddd;
            float _e918 = sd_sphere((p_15 - shape_49.pos), (shape_49.radius * 0.86));
            dddd = max(_e912, -(_e918));
            vec4 rotated_p_1 = (shape_49.rotation * (p_15 - shape_49.pos));
            float _e925 = dddd;
            float _e939 = sd_box(rotated_p_1, vec4((shape_49.radius * 0.18), (shape_49.radius * 1.2), (shape_49.radius * 1.2), (shape_49.radius * 1.2)));
            dddd = max(_e925, -(_e939));
            float _e942 = dddd;
            float _e953 = sd_sphere((rotated_p_1 - vec4(0.0, 0.0, -(shape_49.radius), 0.0)), (shape_49.radius * 0.53));
            dddd = max(_e942, -(_e953));
            float _e956 = dddd;
            float _e962 = sd_sphere((p_15 - shape_49.pos), (shape_49.radius * 0.6));
            dddd = min(_e956, _e962);
            float _e964 = dddd;
            float _e977 = sd_sphere((rotated_p_1 - (vec4(0.0, 0.0, -(shape_49.radius), 0.0) * 0.6)), (shape_49.radius * 0.34));
            dddd = max(_e964, -(_e977));
            float _e980 = dddd;
            float _e986 = sd_sphere((rotated_p_1 - shape_49.weapon_offset), (shape_49.radius * 0.286));
            dddd = min(_e980, _e986);
            float _e988 = dddd;
            float _e1002 = sd_capsule(rotated_p_1, shape_49.weapon_offset, (shape_49.weapon_offset - vec4(0.0, 0.0, (shape_49.radius * 0.49), 0.0)), (shape_49.radius * 0.18));
            dddd = max(_e988, -(_e1002));
            float _e1005 = dddd;
            float _e1019 = sd_capsule(rotated_p_1, shape_49.weapon_offset, (shape_49.weapon_offset - vec4(0.0, 0.0, (shape_49.radius * 0.43), 0.0)), (shape_49.radius * 0.1));
            dddd = min(_e1005, _e1019);
            float _e1021 = dddd;
            float _e1035 = sd_capsule(rotated_p_1, shape_49.weapon_offset, (shape_49.weapon_offset - vec4(0.0, 0.0, (shape_49.radius * 0.65), 0.0)), (shape_49.radius * 0.052));
            dddd = max(_e1021, -(_e1035));
        }
    }
    float _e1041 = d_8;
    float _e1042 = dddd;
    d_8 = min(_e1041, _e1042);
    int _e1046 = _group_0_binding_4_fs.is_w_floor_exist;
    if ((_e1046 == 1)) {
        bool _e1050 = in_2.ray_w_rotated;
        if (_e1050) {
            float _e1051 = d_8;
            float _e1055 = _group_0_binding_4_fs.w_floor;
            d_8 = min(_e1051, (p_15.w - _e1055));
        }
    }
    float _e1058 = d_8;
    return _e1058;
}

vec4 get_normal(vec4 p_16, inout Intersections in_3) {
    vec3 h_2 = vec3(0.001, -0.001, 0.0);
    vec4 a_4 = vec4(0.0);
    vec4 b_9 = vec4(0.0);
    vec4 c_4 = vec4(0.0);
    vec4 d_9 = vec4(0.0);
    vec4 e_2 = vec4(0.0);
    vec4 f_2 = vec4(0.0);
    float fa_2 = 0.0;
    float fb_2 = 0.0;
    float fc_2 = 0.0;
    float fd_2 = 0.0;
    float fe_2 = 0.0;
    float ff_2 = 0.0;
    vec3 _e7 = h_2;
    a_4 = (p_16 + _e7.yxxz);
    vec3 _e11 = h_2;
    b_9 = (p_16 + _e11.xyxz);
    vec3 _e15 = h_2;
    c_4 = (p_16 + _e15.xxyz);
    vec3 _e19 = h_2;
    d_9 = (p_16 + _e19.yyyz);
    vec3 _e23 = h_2;
    e_2 = (p_16 + _e23.zzzx);
    vec3 _e27 = h_2;
    f_2 = (p_16 + _e27.zzzy);
    vec4 _e31 = a_4;
    float _e32 = map(_e31, in_3);
    fa_2 = _e32;
    vec4 _e34 = b_9;
    float _e35 = map(_e34, in_3);
    fb_2 = _e35;
    vec4 _e37 = c_4;
    float _e38 = map(_e37, in_3);
    fc_2 = _e38;
    vec4 _e40 = d_9;
    float _e41 = map(_e40, in_3);
    fd_2 = _e41;
    vec4 _e43 = e_2;
    float _e44 = map(_e43, in_3);
    fe_2 = _e44;
    vec4 _e46 = f_2;
    float _e47 = map(_e46, in_3);
    ff_2 = _e47;
    vec3 _e49 = h_2;
    float _e51 = fa_2;
    vec3 _e53 = h_2;
    float _e55 = fb_2;
    vec3 _e58 = h_2;
    float _e60 = fc_2;
    vec3 _e63 = h_2;
    float _e65 = fd_2;
    vec3 _e68 = h_2;
    float _e70 = fe_2;
    vec3 _e73 = h_2;
    float _e75 = ff_2;
    return normalize(((((((_e49.yxxz * _e51) + (_e53.xyxz * _e55)) + (_e58.xxyz * _e60)) + (_e63.yyyz * _e65)) + (_e68.zzzx * _e70)) + (_e73.zzzy * _e75)));
}

vec2 ray_march(vec4 ray_origin_base, vec4 ray_direction_1, inout Intersections in_4) {
    float total_distance = 0.0;
    vec4 ray_origin = vec4(0.0);
    int i_54 = 0;
    float d_10 = 0.0;
    float _e4 = in_4.offset;
    if ((_e4 > MAX_DIST)) {
        return vec2(700.0, 0.0);
    }
    float _e11 = in_4.offset;
    total_distance = _e11;
    float _e14 = in_4.offset;
    ray_origin = (ray_origin_base + (ray_direction_1 * _e14));
    bool loop_init_60 = true;
    while(true) {
        if (!loop_init_60) {
            int _e55 = i_54;
            i_54 = (_e55 + 1);
        }
        loop_init_60 = false;
        int _e20 = i_54;
        if ((_e20 < MAX_STEPS)) {
        } else {
            break;
        }
        {
            vec4 _e23 = ray_origin;
            float _e24 = map(_e23, in_4);
            d_10 = _e24;
            float _e26 = d_10;
            float _e27 = total_distance;
            total_distance = (_e27 + _e26);
            float _e29 = d_10;
            if ((_e29 < 0.0)) {
                float _e32 = total_distance;
                int _e33 = i_54;
                return vec2(_e32, float(_e33));
            }
            float _e36 = d_10;
            if ((_e36 < MIN_DIST)) {
                float _e39 = total_distance;
                int _e40 = i_54;
                return vec2(_e39, float(_e40));
            }
            float _e43 = total_distance;
            if ((_e43 > MAX_DIST)) {
                int _e47 = i_54;
                return vec2(700.0, float(_e47));
            }
            float _e50 = d_10;
            vec4 _e52 = ray_origin;
            ray_origin = (_e52 + (ray_direction_1 * _e50));
        }
    }
    float _e57 = total_distance;
    int _e58 = i_54;
    return vec2(_e57, float(_e58));
}

vec3 add_w_scnner_color(vec4 pos, float dist_1, vec4 dir) {
    vec3 scanner_color = vec3(0.0);
    uint i_55 = 0u;
    float red = 0.0;
    float _e8 = _group_0_binding_9_fs.w_scaner_radius;
    if ((dist_1 > _e8)) {
        float y_coof = clamp(pow((1.0 - dir.y), 3.0), 0.0, 1.0);
        scanner_color = vec3((0.4 * y_coof));
    }
    float _e23 = _group_0_binding_9_fs.w_scaner_radius;
    vec3 _e33 = scanner_color;
    scanner_color = (_e33 + vec3(clamp(pow((1.0 - abs((dist_1 - _e23))), 5.0), 0.0, 1.0)));
    float _e38 = _group_0_binding_9_fs.w_scaner_intesity;
    vec3 _e39 = scanner_color;
    scanner_color = (_e39 * _e38);
    bool loop_init_61 = true;
    while(true) {
        if (!loop_init_61) {
            uint _e133 = i_55;
            i_55 = (_e133 + 1u);
        }
        loop_init_61 = false;
        uint _e43 = i_55;
        uint _e46 = _group_0_binding_9_fs.player_forms_amount;
        if ((_e43 < _e46)) {
        } else {
            break;
        }
        {
            uint _e49 = i_55;
            vec4 _e52 = _group_1_binding_2_fs[_e49].pos;
            uint _e55 = i_55;
            float _e58 = _group_1_binding_2_fs[_e55].radius;
            float _e59 = sd_sphere((pos - _e52), _e58);
            float _e62 = _group_0_binding_9_fs.w_scaner_radius;
            float visible = clamp(((_e62 - _e59) * 5.0), 0.0, 1.0);
            float _e71 = _group_0_binding_9_fs.w_scaner_radius;
            uint _e74 = i_55;
            vec4 _e77 = _group_1_binding_2_fs[_e74].pos;
            uint _e85 = i_55;
            vec4 _e88 = _group_1_binding_2_fs[_e85].pos;
            uint _e93 = i_55;
            float _e96 = _group_1_binding_2_fs[_e93].radius;
            float vis_d = (length(((pos + (dir * min(_e71, length((pos.xyz - _e77.xyz))))) - _e88).xyz) - _e96);
            red = (pow(clamp((1.0 - abs((vis_d * 10.0))), 0.0, 1.0), 2.0) * visible);
            float _e119 = red;
            red = (_e119 + (pow(clamp((-(vis_d) * 2.5), 0.0, 1.0), 2.0) * visible));
            float _e123 = _group_0_binding_9_fs.w_scaner_intesity;
            float _e126 = red;
            red = (_e126 * (_e123 * 2.0));
            float _e129 = red;
            float _e130 = scanner_color.x;
            scanner_color.x = (_e130 + _e129);
        }
    }
    vec3 _e135 = scanner_color;
    return clamp(_e135, vec3(0.0), vec3(1.0));
}

vec2 cube_intersection(vec4 ro, vec4 rd, vec4 size) {
    vec4 m_1 = (vec4(1.0) / rd);
    vec4 n = (m_1 * ro);
    vec4 k_9 = (abs(m_1) * size);
    vec4 t1_ = (-(n) - k_9);
    vec4 t2_ = (-(n) + k_9);
    float tN = max(max(max(t1_.x, t1_.y), t1_.z), t1_.w);
    float tF = min(min(min(t2_.x, t2_.y), t2_.z), t2_.w);
    if (((tN > tF) || (tF < 0.0))) {
        return vec2(-1.0);
    }
    return vec2(tN, tF);
}

vec2 inf_cube_intersection(vec4 ro_1, vec4 rd_1, vec3 size_1) {
    vec4 m_2 = (vec4(1.0) / rd_1);
    vec4 n_1 = (m_2 * ro_1);
    vec3 k_10 = (abs(m_2.xyz) * size_1);
    vec3 t1_1 = (-(n_1.xyz) - k_10.xyz);
    vec3 t2_1 = (-(n_1.xyz) + k_10.xyz);
    float tN_1 = max(max(t1_1.x, t1_1.y), t1_1.z);
    float tF_1 = min(min(t2_1.x, t2_1.y), t2_1.z);
    if (((tN_1 > tF_1) || (tF_1 < 0.0))) {
        return vec2(-1.0);
    }
    return vec2(tN_1, tF_1);
}

vec2 sph_intersection(vec4 ro_2, vec4 rd_2, float ra_1) {
    float h_3 = 0.0;
    float b_10 = dot(ro_2, rd_2);
    float c_5 = (dot(ro_2, ro_2) - (ra_1 * ra_1));
    h_3 = ((b_10 * b_10) - c_5);
    float _e10 = h_3;
    if ((_e10 < 0.0)) {
        return vec2(-1.0);
    }
    float _e15 = h_3;
    h_3 = sqrt(_e15);
    float _e18 = h_3;
    float _e21 = h_3;
    return vec2((-(b_10) - _e18), (-(b_10) + _e21));
}

Intersections find_intersections(vec4 ro_3, vec4 rd_3) {
    Intersections in_5 = Intersections(IntersectedShapesMetadata(0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u), uint[16](0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u), 0.0, false);
    float offset = 700.0;
    uint ish_index = 0u;
    uint i_56 = 0u;
    uint i_57 = 0u;
    uint i_58 = 0u;
    uint i_59 = 0u;
    uint i_60 = 0u;
    uint i_61 = 0u;
    uint i_62 = 0u;
    uint i_63 = 0u;
    uint i_64 = 0u;
    uint i_65 = 0u;
    uint i_66 = 0u;
    uint i_67 = 0u;
    uint i_68 = 0u;
    uint i_69 = 0u;
    uint i_70 = 0u;
    uint i_71 = 0u;
    uint i_72 = 0u;
    uint i_73 = 0u;
    uint i_74 = 0u;
    uint i_75 = 0u;
    uint i_76 = 0u;
    uint i_77 = 0u;
    uint i_78 = 0u;
    uint i_79 = 0u;
    uint i_80 = 0u;
    uint i_81 = 0u;
    uint i_82 = 0u;
    uint i_83 = 0u;
    uint i_84 = 0u;
    uint i_85 = 0u;
    uint i_86 = 0u;
    uint i_87 = 0u;
    uint i_88 = 0u;
    uint _e9 = ish_index;
    in_5.ismd.st_s_cubes_start = _e9;
    uint _e13 = _group_0_binding_4_fs.shapes_arrays_metadata.s_cubes_start;
    i_56 = _e13;
    bool loop_init_62 = true;
    while(true) {
        if (!loop_init_62) {
            uint _e71 = i_56;
            i_56 = (_e71 + 1u);
        }
        loop_init_62 = false;
        uint _e15 = i_56;
        uint _e19 = _group_0_binding_4_fs.shapes_arrays_metadata.s_cubes_amount;
        uint _e23 = _group_0_binding_4_fs.shapes_arrays_metadata.s_cubes_start;
        if ((_e15 < (_e19 + _e23))) {
        } else {
            break;
        }
        {
            uint _e27 = i_56;
            vec4 _e30 = _group_0_binding_2_fs[_e27].pos;
            uint _e33 = i_56;
            vec4 _e36 = _group_0_binding_2_fs[_e33].size;
            uint _e38 = i_56;
            float _e41 = _group_0_binding_2_fs[_e38].roundness;
            float _e46 = _group_0_binding_4_fs.stickiness;
            vec2 _e51 = cube_intersection((ro_3 - _e30), rd_3, ((_e36 + vec4(_e41)) + vec4((_e46 * STICKINESS_EFFECT_COEF))));
            if ((_e51.y > 0.0)) {
                uint _e58 = in_5.ismd.st_s_cubes_amount;
                in_5.ismd.st_s_cubes_amount = (_e58 + 1u);
                uint _e61 = ish_index;
                uint _e63 = i_56;
                in_5.ish[_e61] = _e63;
                uint _e65 = ish_index;
                ish_index = (_e65 + 1u);
                float _e67 = offset;
                offset = min(_e67, _e51.x);
            }
        }
    }
    uint _e75 = ish_index;
    in_5.ismd.st_s_spheres_start = _e75;
    uint _e79 = _group_0_binding_4_fs.shapes_arrays_metadata.s_spheres_start;
    i_57 = _e79;
    bool loop_init_63 = true;
    while(true) {
        if (!loop_init_63) {
            uint _e136 = i_57;
            i_57 = (_e136 + 1u);
        }
        loop_init_63 = false;
        uint _e81 = i_57;
        uint _e85 = _group_0_binding_4_fs.shapes_arrays_metadata.s_spheres_amount;
        uint _e89 = _group_0_binding_4_fs.shapes_arrays_metadata.s_spheres_start;
        if ((_e81 < (_e85 + _e89))) {
        } else {
            break;
        }
        {
            uint _e93 = i_57;
            vec4 _e96 = _group_0_binding_2_fs[_e93].pos;
            uint _e99 = i_57;
            float _e103 = _group_0_binding_2_fs[_e99].size.x;
            uint _e105 = i_57;
            float _e108 = _group_0_binding_2_fs[_e105].roundness;
            float _e112 = _group_0_binding_4_fs.stickiness;
            vec2 _e116 = sph_intersection((ro_3 - _e96), rd_3, ((_e103 + _e108) + (_e112 * STICKINESS_EFFECT_COEF)));
            if ((_e116.y > 0.0)) {
                uint _e123 = in_5.ismd.st_s_spheres_amount;
                in_5.ismd.st_s_spheres_amount = (_e123 + 1u);
                uint _e126 = ish_index;
                uint _e128 = i_57;
                in_5.ish[_e126] = _e128;
                uint _e130 = ish_index;
                ish_index = (_e130 + 1u);
                float _e132 = offset;
                offset = min(_e132, _e116.x);
            }
        }
    }
    uint _e140 = ish_index;
    in_5.ismd.st_s_sph_cubes_start = _e140;
    uint _e144 = _group_0_binding_4_fs.shapes_arrays_metadata.s_sph_cubes_start;
    i_58 = _e144;
    bool loop_init_64 = true;
    while(true) {
        if (!loop_init_64) {
            uint _e202 = i_58;
            i_58 = (_e202 + 1u);
        }
        loop_init_64 = false;
        uint _e146 = i_58;
        uint _e150 = _group_0_binding_4_fs.shapes_arrays_metadata.s_sph_cubes_amount;
        uint _e154 = _group_0_binding_4_fs.shapes_arrays_metadata.s_sph_cubes_start;
        if ((_e146 < (_e150 + _e154))) {
        } else {
            break;
        }
        {
            uint _e158 = i_58;
            vec4 _e161 = _group_0_binding_2_fs[_e158].pos;
            uint _e164 = i_58;
            vec4 _e167 = _group_0_binding_2_fs[_e164].size;
            uint _e169 = i_58;
            float _e172 = _group_0_binding_2_fs[_e169].roundness;
            float _e177 = _group_0_binding_4_fs.stickiness;
            vec2 _e182 = cube_intersection((ro_3 - _e161), rd_3, ((_e167 + vec4(_e172)) + vec4((_e177 * STICKINESS_EFFECT_COEF))));
            if ((_e182.y > 0.0)) {
                uint _e189 = in_5.ismd.st_s_sph_cubes_amount;
                in_5.ismd.st_s_sph_cubes_amount = (_e189 + 1u);
                uint _e192 = ish_index;
                uint _e194 = i_58;
                in_5.ish[_e192] = _e194;
                uint _e196 = ish_index;
                ish_index = (_e196 + 1u);
                float _e198 = offset;
                offset = min(_e198, _e182.x);
            }
        }
    }
    uint _e206 = ish_index;
    in_5.ismd.st_s_inf_cubes_start = _e206;
    uint _e210 = _group_0_binding_4_fs.shapes_arrays_metadata.s_inf_cubes_start;
    i_59 = _e210;
    bool loop_init_65 = true;
    while(true) {
        if (!loop_init_65) {
            uint _e269 = i_59;
            i_59 = (_e269 + 1u);
        }
        loop_init_65 = false;
        uint _e212 = i_59;
        uint _e216 = _group_0_binding_4_fs.shapes_arrays_metadata.s_inf_cubes_amount;
        uint _e220 = _group_0_binding_4_fs.shapes_arrays_metadata.s_inf_cubes_start;
        if ((_e212 < (_e216 + _e220))) {
        } else {
            break;
        }
        {
            uint _e224 = i_59;
            vec4 _e227 = _group_0_binding_2_fs[_e224].pos;
            uint _e230 = i_59;
            vec4 _e233 = _group_0_binding_2_fs[_e230].size;
            uint _e236 = i_59;
            float _e239 = _group_0_binding_2_fs[_e236].roundness;
            float _e244 = _group_0_binding_4_fs.stickiness;
            vec2 _e249 = inf_cube_intersection((ro_3 - _e227), rd_3, ((_e233.xyz + vec3(_e239)) + vec3((_e244 * STICKINESS_EFFECT_COEF))));
            if ((_e249.y > 0.0)) {
                uint _e256 = in_5.ismd.st_s_inf_cubes_amount;
                in_5.ismd.st_s_inf_cubes_amount = (_e256 + 1u);
                uint _e259 = ish_index;
                uint _e261 = i_59;
                in_5.ish[_e259] = _e261;
                uint _e263 = ish_index;
                ish_index = (_e263 + 1u);
                float _e265 = offset;
                offset = min(_e265, _e249.x);
            }
        }
    }
    uint _e273 = ish_index;
    in_5.ismd.dyn_s_cubes_start = _e273;
    uint _e277 = _group_0_binding_9_fs.shapes_arrays_metadata.s_cubes_start;
    i_60 = _e277;
    bool loop_init_66 = true;
    while(true) {
        if (!loop_init_66) {
            uint _e335 = i_60;
            i_60 = (_e335 + 1u);
        }
        loop_init_66 = false;
        uint _e279 = i_60;
        uint _e283 = _group_0_binding_9_fs.shapes_arrays_metadata.s_cubes_amount;
        uint _e287 = _group_0_binding_9_fs.shapes_arrays_metadata.s_cubes_start;
        if ((_e279 < (_e283 + _e287))) {
        } else {
            break;
        }
        {
            uint _e291 = i_60;
            vec4 _e294 = _group_0_binding_7_fs[_e291].pos;
            uint _e297 = i_60;
            vec4 _e300 = _group_0_binding_7_fs[_e297].size;
            uint _e302 = i_60;
            float _e305 = _group_0_binding_7_fs[_e302].roundness;
            float _e310 = _group_0_binding_4_fs.stickiness;
            vec2 _e315 = cube_intersection((ro_3 - _e294), rd_3, ((_e300 + vec4(_e305)) + vec4((_e310 * STICKINESS_EFFECT_COEF))));
            if ((_e315.y > 0.0)) {
                uint _e322 = in_5.ismd.dyn_s_cubes_amount;
                in_5.ismd.dyn_s_cubes_amount = (_e322 + 1u);
                uint _e325 = ish_index;
                uint _e327 = i_60;
                in_5.ish[_e325] = _e327;
                uint _e329 = ish_index;
                ish_index = (_e329 + 1u);
                float _e331 = offset;
                offset = min(_e331, _e315.x);
            }
        }
    }
    uint _e339 = ish_index;
    in_5.ismd.dyn_s_spheres_start = _e339;
    uint _e343 = _group_0_binding_9_fs.shapes_arrays_metadata.s_spheres_start;
    i_61 = _e343;
    bool loop_init_67 = true;
    while(true) {
        if (!loop_init_67) {
            uint _e400 = i_61;
            i_61 = (_e400 + 1u);
        }
        loop_init_67 = false;
        uint _e345 = i_61;
        uint _e349 = _group_0_binding_9_fs.shapes_arrays_metadata.s_spheres_amount;
        uint _e353 = _group_0_binding_9_fs.shapes_arrays_metadata.s_spheres_start;
        if ((_e345 < (_e349 + _e353))) {
        } else {
            break;
        }
        {
            uint _e357 = i_61;
            vec4 _e360 = _group_0_binding_7_fs[_e357].pos;
            uint _e363 = i_61;
            float _e367 = _group_0_binding_7_fs[_e363].size.x;
            uint _e369 = i_61;
            float _e372 = _group_0_binding_7_fs[_e369].roundness;
            float _e376 = _group_0_binding_4_fs.stickiness;
            vec2 _e380 = sph_intersection((ro_3 - _e360), rd_3, ((_e367 + _e372) + (_e376 * STICKINESS_EFFECT_COEF)));
            if ((_e380.y > 0.0)) {
                uint _e387 = in_5.ismd.dyn_s_spheres_amount;
                in_5.ismd.dyn_s_spheres_amount = (_e387 + 1u);
                uint _e390 = ish_index;
                uint _e392 = i_61;
                in_5.ish[_e390] = _e392;
                uint _e394 = ish_index;
                ish_index = (_e394 + 1u);
                float _e396 = offset;
                offset = min(_e396, _e380.x);
            }
        }
    }
    uint _e404 = ish_index;
    in_5.ismd.dyn_s_sph_cubes_start = _e404;
    uint _e408 = _group_0_binding_9_fs.shapes_arrays_metadata.s_sph_cubes_start;
    i_62 = _e408;
    bool loop_init_68 = true;
    while(true) {
        if (!loop_init_68) {
            uint _e466 = i_62;
            i_62 = (_e466 + 1u);
        }
        loop_init_68 = false;
        uint _e410 = i_62;
        uint _e414 = _group_0_binding_9_fs.shapes_arrays_metadata.s_sph_cubes_amount;
        uint _e418 = _group_0_binding_9_fs.shapes_arrays_metadata.s_sph_cubes_start;
        if ((_e410 < (_e414 + _e418))) {
        } else {
            break;
        }
        {
            uint _e422 = i_62;
            vec4 _e425 = _group_0_binding_7_fs[_e422].pos;
            uint _e428 = i_62;
            vec4 _e431 = _group_0_binding_7_fs[_e428].size;
            uint _e433 = i_62;
            float _e436 = _group_0_binding_7_fs[_e433].roundness;
            float _e441 = _group_0_binding_4_fs.stickiness;
            vec2 _e446 = cube_intersection((ro_3 - _e425), rd_3, ((_e431 + vec4(_e436)) + vec4((_e441 * STICKINESS_EFFECT_COEF))));
            if ((_e446.y > 0.0)) {
                uint _e453 = in_5.ismd.dyn_s_sph_cubes_amount;
                in_5.ismd.dyn_s_sph_cubes_amount = (_e453 + 1u);
                uint _e456 = ish_index;
                uint _e458 = i_62;
                in_5.ish[_e456] = _e458;
                uint _e460 = ish_index;
                ish_index = (_e460 + 1u);
                float _e462 = offset;
                offset = min(_e462, _e446.x);
            }
        }
    }
    uint _e470 = ish_index;
    in_5.ismd.dyn_s_inf_cubes_start = _e470;
    uint _e474 = _group_0_binding_9_fs.shapes_arrays_metadata.s_inf_cubes_start;
    i_63 = _e474;
    bool loop_init_69 = true;
    while(true) {
        if (!loop_init_69) {
            uint _e536 = i_63;
            i_63 = (_e536 + 1u);
        }
        loop_init_69 = false;
        uint _e476 = i_63;
        uint _e480 = _group_0_binding_9_fs.shapes_arrays_metadata.s_inf_cubes_amount;
        uint _e484 = _group_0_binding_9_fs.shapes_arrays_metadata.s_inf_cubes_start;
        if ((_e476 < (_e480 + _e484))) {
        } else {
            break;
        }
        {
            uint _e488 = i_63;
            vec4 _e491 = _group_0_binding_7_fs[_e488].pos;
            uint _e494 = i_63;
            vec4 _e497 = _group_0_binding_7_fs[_e494].size;
            uint _e500 = i_63;
            float _e503 = _group_0_binding_7_fs[_e500].roundness;
            float _e508 = _group_0_binding_4_fs.stickiness;
            vec2 _e513 = inf_cube_intersection((ro_3 - _e491), rd_3, ((_e497.xyz + vec3(_e503)) + vec3((_e508 * STICKINESS_EFFECT_COEF))));
            if ((_e513.y > 0.0)) {
                uint _e520 = in_5.ismd.dyn_s_inf_cubes_amount;
                in_5.ismd.dyn_s_inf_cubes_amount = (_e520 + 1u);
                uint _e523 = ish_index;
                uint _e525 = i_63;
                in_5.ish[_e523] = _e525;
                uint _e527 = ish_index;
                ish_index = (_e527 + 1u);
                if ((_e513.x >= 0.0)) {
                    float _e532 = offset;
                    offset = min(_e532, _e513.x);
                }
            }
        }
    }
    uint _e540 = ish_index;
    in_5.ismd.st_cubes_start = _e540;
    uint _e544 = _group_0_binding_4_fs.shapes_arrays_metadata.cubes_start;
    i_64 = _e544;
    bool loop_init_70 = true;
    while(true) {
        if (!loop_init_70) {
            uint _e595 = i_64;
            i_64 = (_e595 + 1u);
        }
        loop_init_70 = false;
        uint _e546 = i_64;
        uint _e550 = _group_0_binding_4_fs.shapes_arrays_metadata.cubes_amount;
        uint _e554 = _group_0_binding_4_fs.shapes_arrays_metadata.cubes_start;
        if ((_e546 < (_e550 + _e554))) {
        } else {
            break;
        }
        {
            uint _e558 = i_64;
            vec4 _e561 = _group_0_binding_0_fs[_e558].pos;
            uint _e564 = i_64;
            vec4 _e567 = _group_0_binding_0_fs[_e564].size;
            uint _e569 = i_64;
            float _e572 = _group_0_binding_0_fs[_e569].roundness;
            vec2 _e575 = cube_intersection((ro_3 - _e561), rd_3, (_e567 + vec4(_e572)));
            if ((_e575.y > 0.0)) {
                uint _e582 = in_5.ismd.st_cubes_amount;
                in_5.ismd.st_cubes_amount = (_e582 + 1u);
                uint _e585 = ish_index;
                uint _e587 = i_64;
                in_5.ish[_e585] = _e587;
                uint _e589 = ish_index;
                ish_index = (_e589 + 1u);
                float _e591 = offset;
                offset = min(_e591, _e575.x);
            }
        }
    }
    uint _e599 = ish_index;
    in_5.ismd.st_spheres_start = _e599;
    uint _e603 = _group_0_binding_4_fs.shapes_arrays_metadata.spheres_start;
    i_65 = _e603;
    bool loop_init_71 = true;
    while(true) {
        if (!loop_init_71) {
            uint _e654 = i_65;
            i_65 = (_e654 + 1u);
        }
        loop_init_71 = false;
        uint _e605 = i_65;
        uint _e609 = _group_0_binding_4_fs.shapes_arrays_metadata.spheres_amount;
        uint _e613 = _group_0_binding_4_fs.shapes_arrays_metadata.spheres_start;
        if ((_e605 < (_e609 + _e613))) {
        } else {
            break;
        }
        {
            uint _e617 = i_65;
            vec4 _e620 = _group_0_binding_0_fs[_e617].pos;
            uint _e623 = i_65;
            float _e627 = _group_0_binding_0_fs[_e623].size.x;
            uint _e629 = i_65;
            float _e632 = _group_0_binding_0_fs[_e629].roundness;
            vec2 _e634 = sph_intersection((ro_3 - _e620), rd_3, (_e627 + _e632));
            if ((_e634.y > 0.0)) {
                uint _e641 = in_5.ismd.st_spheres_amount;
                in_5.ismd.st_spheres_amount = (_e641 + 1u);
                uint _e644 = ish_index;
                uint _e646 = i_65;
                in_5.ish[_e644] = _e646;
                uint _e648 = ish_index;
                ish_index = (_e648 + 1u);
                float _e650 = offset;
                offset = min(_e650, _e634.x);
            }
        }
    }
    uint _e658 = ish_index;
    in_5.ismd.st_sph_cubes_start = _e658;
    uint _e662 = _group_0_binding_4_fs.shapes_arrays_metadata.sph_cubes_start;
    i_66 = _e662;
    bool loop_init_72 = true;
    while(true) {
        if (!loop_init_72) {
            uint _e713 = i_66;
            i_66 = (_e713 + 1u);
        }
        loop_init_72 = false;
        uint _e664 = i_66;
        uint _e668 = _group_0_binding_4_fs.shapes_arrays_metadata.sph_cubes_amount;
        uint _e672 = _group_0_binding_4_fs.shapes_arrays_metadata.sph_cubes_start;
        if ((_e664 < (_e668 + _e672))) {
        } else {
            break;
        }
        {
            uint _e676 = i_66;
            vec4 _e679 = _group_0_binding_0_fs[_e676].pos;
            uint _e682 = i_66;
            vec4 _e685 = _group_0_binding_0_fs[_e682].size;
            uint _e687 = i_66;
            float _e690 = _group_0_binding_0_fs[_e687].roundness;
            vec2 _e693 = cube_intersection((ro_3 - _e679), rd_3, (_e685 + vec4(_e690)));
            if ((_e693.y > 0.0)) {
                uint _e700 = in_5.ismd.st_sph_cubes_amount;
                in_5.ismd.st_sph_cubes_amount = (_e700 + 1u);
                uint _e703 = ish_index;
                uint _e705 = i_66;
                in_5.ish[_e703] = _e705;
                uint _e707 = ish_index;
                ish_index = (_e707 + 1u);
                float _e709 = offset;
                offset = min(_e709, _e693.x);
            }
        }
    }
    uint _e717 = ish_index;
    in_5.ismd.st_inf_cubes_start = _e717;
    uint _e721 = _group_0_binding_4_fs.shapes_arrays_metadata.inf_cubes_start;
    i_67 = _e721;
    bool loop_init_73 = true;
    while(true) {
        if (!loop_init_73) {
            uint _e773 = i_67;
            i_67 = (_e773 + 1u);
        }
        loop_init_73 = false;
        uint _e723 = i_67;
        uint _e727 = _group_0_binding_4_fs.shapes_arrays_metadata.inf_cubes_amount;
        uint _e731 = _group_0_binding_4_fs.shapes_arrays_metadata.inf_cubes_start;
        if ((_e723 < (_e727 + _e731))) {
        } else {
            break;
        }
        {
            uint _e735 = i_67;
            vec4 _e738 = _group_0_binding_0_fs[_e735].pos;
            uint _e741 = i_67;
            vec4 _e744 = _group_0_binding_0_fs[_e741].size;
            uint _e747 = i_67;
            float _e750 = _group_0_binding_0_fs[_e747].roundness;
            vec2 _e753 = inf_cube_intersection((ro_3 - _e738), rd_3, (_e744.xyz + vec3(_e750)));
            if ((_e753.y > 0.0)) {
                uint _e760 = in_5.ismd.st_inf_cubes_amount;
                in_5.ismd.st_inf_cubes_amount = (_e760 + 1u);
                uint _e763 = ish_index;
                uint _e765 = i_67;
                in_5.ish[_e763] = _e765;
                uint _e767 = ish_index;
                ish_index = (_e767 + 1u);
                float _e769 = offset;
                offset = min(_e769, _e753.x);
            }
        }
    }
    uint _e777 = ish_index;
    in_5.ismd.dyn_cubes_start = _e777;
    uint _e781 = _group_0_binding_9_fs.shapes_arrays_metadata.cubes_start;
    i_68 = _e781;
    bool loop_init_74 = true;
    while(true) {
        if (!loop_init_74) {
            uint _e832 = i_68;
            i_68 = (_e832 + 1u);
        }
        loop_init_74 = false;
        uint _e783 = i_68;
        uint _e787 = _group_0_binding_9_fs.shapes_arrays_metadata.cubes_amount;
        uint _e791 = _group_0_binding_9_fs.shapes_arrays_metadata.cubes_start;
        if ((_e783 < (_e787 + _e791))) {
        } else {
            break;
        }
        {
            uint _e795 = i_68;
            vec4 _e798 = _group_0_binding_5_fs[_e795].pos;
            uint _e801 = i_68;
            vec4 _e804 = _group_0_binding_5_fs[_e801].size;
            uint _e806 = i_68;
            float _e809 = _group_0_binding_5_fs[_e806].roundness;
            vec2 _e812 = cube_intersection((ro_3 - _e798), rd_3, (_e804 + vec4(_e809)));
            if ((_e812.y > 0.0)) {
                uint _e819 = in_5.ismd.dyn_cubes_amount;
                in_5.ismd.dyn_cubes_amount = (_e819 + 1u);
                uint _e822 = ish_index;
                uint _e824 = i_68;
                in_5.ish[_e822] = _e824;
                uint _e826 = ish_index;
                ish_index = (_e826 + 1u);
                float _e828 = offset;
                offset = min(_e828, _e812.x);
            }
        }
    }
    uint _e836 = ish_index;
    in_5.ismd.dyn_spheres_start = _e836;
    uint _e840 = _group_0_binding_9_fs.shapes_arrays_metadata.spheres_start;
    i_69 = _e840;
    bool loop_init_75 = true;
    while(true) {
        if (!loop_init_75) {
            uint _e891 = i_69;
            i_69 = (_e891 + 1u);
        }
        loop_init_75 = false;
        uint _e842 = i_69;
        uint _e846 = _group_0_binding_9_fs.shapes_arrays_metadata.spheres_amount;
        uint _e850 = _group_0_binding_9_fs.shapes_arrays_metadata.spheres_start;
        if ((_e842 < (_e846 + _e850))) {
        } else {
            break;
        }
        {
            uint _e854 = i_69;
            vec4 _e857 = _group_0_binding_5_fs[_e854].pos;
            uint _e860 = i_69;
            float _e864 = _group_0_binding_5_fs[_e860].size.x;
            uint _e866 = i_69;
            float _e869 = _group_0_binding_5_fs[_e866].roundness;
            vec2 _e871 = sph_intersection((ro_3 - _e857), rd_3, (_e864 + _e869));
            if ((_e871.y > 0.0)) {
                uint _e878 = in_5.ismd.dyn_spheres_amount;
                in_5.ismd.dyn_spheres_amount = (_e878 + 1u);
                uint _e881 = ish_index;
                uint _e883 = i_69;
                in_5.ish[_e881] = _e883;
                uint _e885 = ish_index;
                ish_index = (_e885 + 1u);
                float _e887 = offset;
                offset = min(_e887, _e871.x);
            }
        }
    }
    uint _e895 = ish_index;
    in_5.ismd.dyn_sph_cubes_start = _e895;
    uint _e899 = _group_0_binding_9_fs.shapes_arrays_metadata.sph_cubes_start;
    i_70 = _e899;
    bool loop_init_76 = true;
    while(true) {
        if (!loop_init_76) {
            uint _e950 = i_70;
            i_70 = (_e950 + 1u);
        }
        loop_init_76 = false;
        uint _e901 = i_70;
        uint _e905 = _group_0_binding_9_fs.shapes_arrays_metadata.sph_cubes_amount;
        uint _e909 = _group_0_binding_9_fs.shapes_arrays_metadata.sph_cubes_start;
        if ((_e901 < (_e905 + _e909))) {
        } else {
            break;
        }
        {
            uint _e913 = i_70;
            vec4 _e916 = _group_0_binding_5_fs[_e913].pos;
            uint _e919 = i_70;
            vec4 _e922 = _group_0_binding_5_fs[_e919].size;
            uint _e924 = i_70;
            float _e927 = _group_0_binding_5_fs[_e924].roundness;
            vec2 _e930 = cube_intersection((ro_3 - _e916), rd_3, (_e922 + vec4(_e927)));
            if ((_e930.y > 0.0)) {
                uint _e937 = in_5.ismd.dyn_sph_cubes_amount;
                in_5.ismd.dyn_sph_cubes_amount = (_e937 + 1u);
                uint _e940 = ish_index;
                uint _e942 = i_70;
                in_5.ish[_e940] = _e942;
                uint _e944 = ish_index;
                ish_index = (_e944 + 1u);
                float _e946 = offset;
                offset = min(_e946, _e930.x);
            }
        }
    }
    uint _e954 = ish_index;
    in_5.ismd.dyn_inf_cubes_start = _e954;
    uint _e958 = _group_0_binding_9_fs.shapes_arrays_metadata.inf_cubes_start;
    i_71 = _e958;
    bool loop_init_77 = true;
    while(true) {
        if (!loop_init_77) {
            uint _e1010 = i_71;
            i_71 = (_e1010 + 1u);
        }
        loop_init_77 = false;
        uint _e960 = i_71;
        uint _e964 = _group_0_binding_9_fs.shapes_arrays_metadata.inf_cubes_amount;
        uint _e968 = _group_0_binding_9_fs.shapes_arrays_metadata.inf_cubes_start;
        if ((_e960 < (_e964 + _e968))) {
        } else {
            break;
        }
        {
            uint _e972 = i_71;
            vec4 _e975 = _group_0_binding_5_fs[_e972].pos;
            uint _e978 = i_71;
            vec4 _e981 = _group_0_binding_5_fs[_e978].size;
            uint _e984 = i_71;
            float _e987 = _group_0_binding_5_fs[_e984].roundness;
            vec2 _e990 = inf_cube_intersection((ro_3 - _e975), rd_3, (_e981.xyz + vec3(_e987)));
            if ((_e990.y > 0.0)) {
                uint _e997 = in_5.ismd.dyn_inf_cubes_amount;
                in_5.ismd.dyn_inf_cubes_amount = (_e997 + 1u);
                uint _e1000 = ish_index;
                uint _e1002 = i_71;
                in_5.ish[_e1000] = _e1002;
                uint _e1004 = ish_index;
                ish_index = (_e1004 + 1u);
                float _e1006 = offset;
                offset = min(_e1006, _e990.x);
            }
        }
    }
    uint _e1014 = ish_index;
    in_5.ismd.st_s_neg_cubes_start = _e1014;
    uint _e1018 = _group_0_binding_4_fs.shapes_arrays_metadata.s_neg_cubes_start;
    i_72 = _e1018;
    bool loop_init_78 = true;
    while(true) {
        if (!loop_init_78) {
            uint _e1073 = i_72;
            i_72 = (_e1073 + 1u);
        }
        loop_init_78 = false;
        uint _e1020 = i_72;
        uint _e1024 = _group_0_binding_4_fs.shapes_arrays_metadata.s_neg_cubes_amount;
        uint _e1028 = _group_0_binding_4_fs.shapes_arrays_metadata.s_neg_cubes_start;
        if ((_e1020 < (_e1024 + _e1028))) {
        } else {
            break;
        }
        {
            uint _e1032 = i_72;
            vec4 _e1035 = _group_0_binding_3_fs[_e1032].pos;
            uint _e1038 = i_72;
            vec4 _e1041 = _group_0_binding_3_fs[_e1038].size;
            uint _e1043 = i_72;
            float _e1046 = _group_0_binding_3_fs[_e1043].roundness;
            float _e1051 = _group_0_binding_4_fs.stickiness;
            vec2 _e1056 = cube_intersection((ro_3 - _e1035), rd_3, ((_e1041 + vec4(_e1046)) + vec4((_e1051 * STICKINESS_EFFECT_COEF))));
            if ((_e1056.y > 0.0)) {
                uint _e1063 = in_5.ismd.st_s_neg_cubes_amount;
                in_5.ismd.st_s_neg_cubes_amount = (_e1063 + 1u);
                uint _e1066 = ish_index;
                uint _e1068 = i_72;
                in_5.ish[_e1066] = _e1068;
                uint _e1070 = ish_index;
                ish_index = (_e1070 + 1u);
            }
        }
    }
    uint _e1077 = ish_index;
    in_5.ismd.st_s_neg_spheres_start = _e1077;
    uint _e1081 = _group_0_binding_4_fs.shapes_arrays_metadata.s_neg_spheres_start;
    i_73 = _e1081;
    bool loop_init_79 = true;
    while(true) {
        if (!loop_init_79) {
            uint _e1135 = i_73;
            i_73 = (_e1135 + 1u);
        }
        loop_init_79 = false;
        uint _e1083 = i_73;
        uint _e1087 = _group_0_binding_4_fs.shapes_arrays_metadata.s_neg_spheres_amount;
        uint _e1091 = _group_0_binding_4_fs.shapes_arrays_metadata.s_neg_spheres_start;
        if ((_e1083 < (_e1087 + _e1091))) {
        } else {
            break;
        }
        {
            uint _e1095 = i_73;
            vec4 _e1098 = _group_0_binding_3_fs[_e1095].pos;
            uint _e1101 = i_73;
            float _e1105 = _group_0_binding_3_fs[_e1101].size.x;
            uint _e1107 = i_73;
            float _e1110 = _group_0_binding_3_fs[_e1107].roundness;
            float _e1114 = _group_0_binding_4_fs.stickiness;
            vec2 _e1118 = sph_intersection((ro_3 - _e1098), rd_3, ((_e1105 + _e1110) + (_e1114 * STICKINESS_EFFECT_COEF)));
            if ((_e1118.y > 0.0)) {
                uint _e1125 = in_5.ismd.st_s_neg_spheres_amount;
                in_5.ismd.st_s_neg_spheres_amount = (_e1125 + 1u);
                uint _e1128 = ish_index;
                uint _e1130 = i_73;
                in_5.ish[_e1128] = _e1130;
                uint _e1132 = ish_index;
                ish_index = (_e1132 + 1u);
            }
        }
    }
    uint _e1139 = ish_index;
    in_5.ismd.st_s_neg_sph_cubes_start = _e1139;
    uint _e1143 = _group_0_binding_4_fs.shapes_arrays_metadata.s_neg_sph_cubes_start;
    i_74 = _e1143;
    bool loop_init_80 = true;
    while(true) {
        if (!loop_init_80) {
            uint _e1198 = i_74;
            i_74 = (_e1198 + 1u);
        }
        loop_init_80 = false;
        uint _e1145 = i_74;
        uint _e1149 = _group_0_binding_4_fs.shapes_arrays_metadata.s_neg_sph_cubes_amount;
        uint _e1153 = _group_0_binding_4_fs.shapes_arrays_metadata.s_neg_sph_cubes_start;
        if ((_e1145 < (_e1149 + _e1153))) {
        } else {
            break;
        }
        {
            uint _e1157 = i_74;
            vec4 _e1160 = _group_0_binding_3_fs[_e1157].pos;
            uint _e1163 = i_74;
            vec4 _e1166 = _group_0_binding_3_fs[_e1163].size;
            uint _e1168 = i_74;
            float _e1171 = _group_0_binding_3_fs[_e1168].roundness;
            float _e1176 = _group_0_binding_4_fs.stickiness;
            vec2 _e1181 = cube_intersection((ro_3 - _e1160), rd_3, ((_e1166 + vec4(_e1171)) + vec4((_e1176 * STICKINESS_EFFECT_COEF))));
            if ((_e1181.y > 0.0)) {
                uint _e1188 = in_5.ismd.st_s_neg_sph_cubes_amount;
                in_5.ismd.st_s_neg_sph_cubes_amount = (_e1188 + 1u);
                uint _e1191 = ish_index;
                uint _e1193 = i_74;
                in_5.ish[_e1191] = _e1193;
                uint _e1195 = ish_index;
                ish_index = (_e1195 + 1u);
            }
        }
    }
    uint _e1202 = ish_index;
    in_5.ismd.st_s_neg_inf_cubes_start = _e1202;
    uint _e1206 = _group_0_binding_4_fs.shapes_arrays_metadata.s_neg_inf_cubes_start;
    i_75 = _e1206;
    bool loop_init_81 = true;
    while(true) {
        if (!loop_init_81) {
            uint _e1262 = i_75;
            i_75 = (_e1262 + 1u);
        }
        loop_init_81 = false;
        uint _e1208 = i_75;
        uint _e1212 = _group_0_binding_4_fs.shapes_arrays_metadata.s_neg_inf_cubes_amount;
        uint _e1216 = _group_0_binding_4_fs.shapes_arrays_metadata.s_neg_inf_cubes_start;
        if ((_e1208 < (_e1212 + _e1216))) {
        } else {
            break;
        }
        {
            uint _e1220 = i_75;
            vec4 _e1223 = _group_0_binding_3_fs[_e1220].pos;
            uint _e1226 = i_75;
            vec4 _e1229 = _group_0_binding_3_fs[_e1226].size;
            uint _e1232 = i_75;
            float _e1235 = _group_0_binding_3_fs[_e1232].roundness;
            float _e1240 = _group_0_binding_4_fs.stickiness;
            vec2 _e1245 = inf_cube_intersection((ro_3 - _e1223), rd_3, ((_e1229.xyz + vec3(_e1235)) + vec3((_e1240 * STICKINESS_EFFECT_COEF))));
            if ((_e1245.y > 0.0)) {
                uint _e1252 = in_5.ismd.st_s_neg_inf_cubes_amount;
                in_5.ismd.st_s_neg_inf_cubes_amount = (_e1252 + 1u);
                uint _e1255 = ish_index;
                uint _e1257 = i_75;
                in_5.ish[_e1255] = _e1257;
                uint _e1259 = ish_index;
                ish_index = (_e1259 + 1u);
            }
        }
    }
    uint _e1266 = ish_index;
    in_5.ismd.dyn_s_neg_cubes_start = _e1266;
    uint _e1270 = _group_0_binding_9_fs.shapes_arrays_metadata.s_neg_cubes_start;
    i_76 = _e1270;
    bool loop_init_82 = true;
    while(true) {
        if (!loop_init_82) {
            uint _e1325 = i_76;
            i_76 = (_e1325 + 1u);
        }
        loop_init_82 = false;
        uint _e1272 = i_76;
        uint _e1276 = _group_0_binding_9_fs.shapes_arrays_metadata.s_neg_cubes_amount;
        uint _e1280 = _group_0_binding_9_fs.shapes_arrays_metadata.s_neg_cubes_start;
        if ((_e1272 < (_e1276 + _e1280))) {
        } else {
            break;
        }
        {
            uint _e1284 = i_76;
            vec4 _e1287 = _group_0_binding_8_fs[_e1284].pos;
            uint _e1290 = i_76;
            vec4 _e1293 = _group_0_binding_8_fs[_e1290].size;
            uint _e1295 = i_76;
            float _e1298 = _group_0_binding_8_fs[_e1295].roundness;
            float _e1303 = _group_0_binding_4_fs.stickiness;
            vec2 _e1308 = cube_intersection((ro_3 - _e1287), rd_3, ((_e1293 + vec4(_e1298)) + vec4((_e1303 * STICKINESS_EFFECT_COEF))));
            if ((_e1308.y > 0.0)) {
                uint _e1315 = in_5.ismd.dyn_s_neg_cubes_amount;
                in_5.ismd.dyn_s_neg_cubes_amount = (_e1315 + 1u);
                uint _e1318 = ish_index;
                uint _e1320 = i_76;
                in_5.ish[_e1318] = _e1320;
                uint _e1322 = ish_index;
                ish_index = (_e1322 + 1u);
            }
        }
    }
    uint _e1329 = ish_index;
    in_5.ismd.dyn_s_neg_spheres_start = _e1329;
    uint _e1333 = _group_0_binding_9_fs.shapes_arrays_metadata.s_neg_spheres_start;
    i_77 = _e1333;
    bool loop_init_83 = true;
    while(true) {
        if (!loop_init_83) {
            uint _e1387 = i_77;
            i_77 = (_e1387 + 1u);
        }
        loop_init_83 = false;
        uint _e1335 = i_77;
        uint _e1339 = _group_0_binding_9_fs.shapes_arrays_metadata.s_neg_spheres_amount;
        uint _e1343 = _group_0_binding_9_fs.shapes_arrays_metadata.s_neg_spheres_start;
        if ((_e1335 < (_e1339 + _e1343))) {
        } else {
            break;
        }
        {
            uint _e1347 = i_77;
            vec4 _e1350 = _group_0_binding_8_fs[_e1347].pos;
            uint _e1353 = i_77;
            float _e1357 = _group_0_binding_8_fs[_e1353].size.x;
            uint _e1359 = i_77;
            float _e1362 = _group_0_binding_8_fs[_e1359].roundness;
            float _e1366 = _group_0_binding_4_fs.stickiness;
            vec2 _e1370 = sph_intersection((ro_3 - _e1350), rd_3, ((_e1357 + _e1362) + (_e1366 * STICKINESS_EFFECT_COEF)));
            if ((_e1370.y > 0.0)) {
                uint _e1377 = in_5.ismd.dyn_s_neg_spheres_amount;
                in_5.ismd.dyn_s_neg_spheres_amount = (_e1377 + 1u);
                uint _e1380 = ish_index;
                uint _e1382 = i_77;
                in_5.ish[_e1380] = _e1382;
                uint _e1384 = ish_index;
                ish_index = (_e1384 + 1u);
            }
        }
    }
    uint _e1391 = ish_index;
    in_5.ismd.dyn_s_neg_sph_cubes_start = _e1391;
    uint _e1395 = _group_0_binding_9_fs.shapes_arrays_metadata.s_neg_sph_cubes_start;
    i_78 = _e1395;
    bool loop_init_84 = true;
    while(true) {
        if (!loop_init_84) {
            uint _e1450 = i_78;
            i_78 = (_e1450 + 1u);
        }
        loop_init_84 = false;
        uint _e1397 = i_78;
        uint _e1401 = _group_0_binding_9_fs.shapes_arrays_metadata.s_neg_sph_cubes_amount;
        uint _e1405 = _group_0_binding_9_fs.shapes_arrays_metadata.s_neg_sph_cubes_start;
        if ((_e1397 < (_e1401 + _e1405))) {
        } else {
            break;
        }
        {
            uint _e1409 = i_78;
            vec4 _e1412 = _group_0_binding_8_fs[_e1409].pos;
            uint _e1415 = i_78;
            vec4 _e1418 = _group_0_binding_8_fs[_e1415].size;
            uint _e1420 = i_78;
            float _e1423 = _group_0_binding_8_fs[_e1420].roundness;
            float _e1428 = _group_0_binding_4_fs.stickiness;
            vec2 _e1433 = cube_intersection((ro_3 - _e1412), rd_3, ((_e1418 + vec4(_e1423)) + vec4((_e1428 * STICKINESS_EFFECT_COEF))));
            if ((_e1433.y > 0.0)) {
                uint _e1440 = in_5.ismd.dyn_s_neg_sph_cubes_amount;
                in_5.ismd.dyn_s_neg_sph_cubes_amount = (_e1440 + 1u);
                uint _e1443 = ish_index;
                uint _e1445 = i_78;
                in_5.ish[_e1443] = _e1445;
                uint _e1447 = ish_index;
                ish_index = (_e1447 + 1u);
            }
        }
    }
    uint _e1454 = ish_index;
    in_5.ismd.dyn_s_neg_inf_cubes_start = _e1454;
    uint _e1458 = _group_0_binding_9_fs.shapes_arrays_metadata.s_neg_inf_cubes_start;
    i_79 = _e1458;
    bool loop_init_85 = true;
    while(true) {
        if (!loop_init_85) {
            uint _e1514 = i_79;
            i_79 = (_e1514 + 1u);
        }
        loop_init_85 = false;
        uint _e1460 = i_79;
        uint _e1464 = _group_0_binding_9_fs.shapes_arrays_metadata.s_neg_inf_cubes_amount;
        uint _e1468 = _group_0_binding_9_fs.shapes_arrays_metadata.s_neg_inf_cubes_start;
        if ((_e1460 < (_e1464 + _e1468))) {
        } else {
            break;
        }
        {
            uint _e1472 = i_79;
            vec4 _e1475 = _group_0_binding_8_fs[_e1472].pos;
            uint _e1478 = i_79;
            vec4 _e1481 = _group_0_binding_8_fs[_e1478].size;
            uint _e1484 = i_79;
            float _e1487 = _group_0_binding_8_fs[_e1484].roundness;
            float _e1492 = _group_0_binding_4_fs.stickiness;
            vec2 _e1497 = inf_cube_intersection((ro_3 - _e1475), rd_3, ((_e1481.xyz + vec3(_e1487)) + vec3((_e1492 * STICKINESS_EFFECT_COEF))));
            if ((_e1497.y > 0.0)) {
                uint _e1504 = in_5.ismd.dyn_s_neg_inf_cubes_amount;
                in_5.ismd.dyn_s_neg_inf_cubes_amount = (_e1504 + 1u);
                uint _e1507 = ish_index;
                uint _e1509 = i_79;
                in_5.ish[_e1507] = _e1509;
                uint _e1511 = ish_index;
                ish_index = (_e1511 + 1u);
            }
        }
    }
    uint _e1518 = ish_index;
    in_5.ismd.st_neg_cubes_start = _e1518;
    uint _e1522 = _group_0_binding_4_fs.shapes_arrays_metadata.neg_cubes_start;
    i_80 = _e1522;
    bool loop_init_86 = true;
    while(true) {
        if (!loop_init_86) {
            uint _e1570 = i_80;
            i_80 = (_e1570 + 1u);
        }
        loop_init_86 = false;
        uint _e1524 = i_80;
        uint _e1528 = _group_0_binding_4_fs.shapes_arrays_metadata.neg_cubes_amount;
        uint _e1532 = _group_0_binding_4_fs.shapes_arrays_metadata.neg_cubes_start;
        if ((_e1524 < (_e1528 + _e1532))) {
        } else {
            break;
        }
        {
            uint _e1536 = i_80;
            vec4 _e1539 = _group_0_binding_1_fs[_e1536].pos;
            uint _e1542 = i_80;
            vec4 _e1545 = _group_0_binding_1_fs[_e1542].size;
            uint _e1547 = i_80;
            float _e1550 = _group_0_binding_1_fs[_e1547].roundness;
            vec2 _e1553 = cube_intersection((ro_3 - _e1539), rd_3, (_e1545 + vec4(_e1550)));
            if ((_e1553.y > 0.0)) {
                uint _e1560 = in_5.ismd.st_neg_cubes_amount;
                in_5.ismd.st_neg_cubes_amount = (_e1560 + 1u);
                uint _e1563 = ish_index;
                uint _e1565 = i_80;
                in_5.ish[_e1563] = _e1565;
                uint _e1567 = ish_index;
                ish_index = (_e1567 + 1u);
            }
        }
    }
    uint _e1574 = ish_index;
    in_5.ismd.st_neg_spheres_start = _e1574;
    uint _e1578 = _group_0_binding_4_fs.shapes_arrays_metadata.neg_spheres_start;
    i_81 = _e1578;
    bool loop_init_87 = true;
    while(true) {
        if (!loop_init_87) {
            uint _e1626 = i_81;
            i_81 = (_e1626 + 1u);
        }
        loop_init_87 = false;
        uint _e1580 = i_81;
        uint _e1584 = _group_0_binding_4_fs.shapes_arrays_metadata.neg_spheres_amount;
        uint _e1588 = _group_0_binding_4_fs.shapes_arrays_metadata.neg_spheres_start;
        if ((_e1580 < (_e1584 + _e1588))) {
        } else {
            break;
        }
        {
            uint _e1592 = i_81;
            vec4 _e1595 = _group_0_binding_1_fs[_e1592].pos;
            uint _e1598 = i_81;
            float _e1602 = _group_0_binding_1_fs[_e1598].size.x;
            uint _e1604 = i_81;
            float _e1607 = _group_0_binding_1_fs[_e1604].roundness;
            vec2 _e1609 = sph_intersection((ro_3 - _e1595), rd_3, (_e1602 + _e1607));
            if ((_e1609.y > 0.0)) {
                uint _e1616 = in_5.ismd.st_neg_spheres_amount;
                in_5.ismd.st_neg_spheres_amount = (_e1616 + 1u);
                uint _e1619 = ish_index;
                uint _e1621 = i_81;
                in_5.ish[_e1619] = _e1621;
                uint _e1623 = ish_index;
                ish_index = (_e1623 + 1u);
            }
        }
    }
    uint _e1630 = ish_index;
    in_5.ismd.st_neg_sph_cubes_start = _e1630;
    uint _e1634 = _group_0_binding_4_fs.shapes_arrays_metadata.neg_sph_cubes_start;
    i_82 = _e1634;
    bool loop_init_88 = true;
    while(true) {
        if (!loop_init_88) {
            uint _e1682 = i_82;
            i_82 = (_e1682 + 1u);
        }
        loop_init_88 = false;
        uint _e1636 = i_82;
        uint _e1640 = _group_0_binding_4_fs.shapes_arrays_metadata.neg_sph_cubes_amount;
        uint _e1644 = _group_0_binding_4_fs.shapes_arrays_metadata.neg_sph_cubes_start;
        if ((_e1636 < (_e1640 + _e1644))) {
        } else {
            break;
        }
        {
            uint _e1648 = i_82;
            vec4 _e1651 = _group_0_binding_1_fs[_e1648].pos;
            uint _e1654 = i_82;
            vec4 _e1657 = _group_0_binding_1_fs[_e1654].size;
            uint _e1659 = i_82;
            float _e1662 = _group_0_binding_1_fs[_e1659].roundness;
            vec2 _e1665 = cube_intersection((ro_3 - _e1651), rd_3, (_e1657 + vec4(_e1662)));
            if ((_e1665.y > 0.0)) {
                uint _e1672 = in_5.ismd.st_neg_sph_cubes_amount;
                in_5.ismd.st_neg_sph_cubes_amount = (_e1672 + 1u);
                uint _e1675 = ish_index;
                uint _e1677 = i_82;
                in_5.ish[_e1675] = _e1677;
                uint _e1679 = ish_index;
                ish_index = (_e1679 + 1u);
            }
        }
    }
    uint _e1686 = ish_index;
    in_5.ismd.st_neg_inf_cubes_start = _e1686;
    uint _e1690 = _group_0_binding_4_fs.shapes_arrays_metadata.neg_inf_cubes_start;
    i_83 = _e1690;
    bool loop_init_89 = true;
    while(true) {
        if (!loop_init_89) {
            uint _e1739 = i_83;
            i_83 = (_e1739 + 1u);
        }
        loop_init_89 = false;
        uint _e1692 = i_83;
        uint _e1696 = _group_0_binding_4_fs.shapes_arrays_metadata.neg_inf_cubes_amount;
        uint _e1700 = _group_0_binding_4_fs.shapes_arrays_metadata.neg_inf_cubes_start;
        if ((_e1692 < (_e1696 + _e1700))) {
        } else {
            break;
        }
        {
            uint _e1704 = i_83;
            vec4 _e1707 = _group_0_binding_1_fs[_e1704].pos;
            uint _e1710 = i_83;
            vec4 _e1713 = _group_0_binding_1_fs[_e1710].size;
            uint _e1716 = i_83;
            float _e1719 = _group_0_binding_1_fs[_e1716].roundness;
            vec2 _e1722 = inf_cube_intersection((ro_3 - _e1707), rd_3, (_e1713.xyz + vec3(_e1719)));
            if ((_e1722.y > 0.0)) {
                uint _e1729 = in_5.ismd.st_neg_inf_cubes_amount;
                in_5.ismd.st_neg_inf_cubes_amount = (_e1729 + 1u);
                uint _e1732 = ish_index;
                uint _e1734 = i_83;
                in_5.ish[_e1732] = _e1734;
                uint _e1736 = ish_index;
                ish_index = (_e1736 + 1u);
            }
        }
    }
    uint _e1743 = ish_index;
    in_5.ismd.dyn_neg_cubes_start = _e1743;
    uint _e1747 = _group_0_binding_9_fs.shapes_arrays_metadata.neg_cubes_start;
    i_84 = _e1747;
    bool loop_init_90 = true;
    while(true) {
        if (!loop_init_90) {
            uint _e1795 = i_84;
            i_84 = (_e1795 + 1u);
        }
        loop_init_90 = false;
        uint _e1749 = i_84;
        uint _e1753 = _group_0_binding_9_fs.shapes_arrays_metadata.neg_cubes_amount;
        uint _e1757 = _group_0_binding_9_fs.shapes_arrays_metadata.neg_cubes_start;
        if ((_e1749 < (_e1753 + _e1757))) {
        } else {
            break;
        }
        {
            uint _e1761 = i_84;
            vec4 _e1764 = _group_0_binding_6_fs[_e1761].pos;
            uint _e1767 = i_84;
            vec4 _e1770 = _group_0_binding_6_fs[_e1767].size;
            uint _e1772 = i_84;
            float _e1775 = _group_0_binding_6_fs[_e1772].roundness;
            vec2 _e1778 = cube_intersection((ro_3 - _e1764), rd_3, (_e1770 + vec4(_e1775)));
            if ((_e1778.y > 0.0)) {
                uint _e1785 = in_5.ismd.dyn_neg_cubes_amount;
                in_5.ismd.dyn_neg_cubes_amount = (_e1785 + 1u);
                uint _e1788 = ish_index;
                uint _e1790 = i_84;
                in_5.ish[_e1788] = _e1790;
                uint _e1792 = ish_index;
                ish_index = (_e1792 + 1u);
            }
        }
    }
    uint _e1799 = ish_index;
    in_5.ismd.dyn_neg_spheres_start = _e1799;
    uint _e1803 = _group_0_binding_9_fs.shapes_arrays_metadata.neg_spheres_start;
    i_85 = _e1803;
    bool loop_init_91 = true;
    while(true) {
        if (!loop_init_91) {
            uint _e1851 = i_85;
            i_85 = (_e1851 + 1u);
        }
        loop_init_91 = false;
        uint _e1805 = i_85;
        uint _e1809 = _group_0_binding_9_fs.shapes_arrays_metadata.neg_spheres_amount;
        uint _e1813 = _group_0_binding_9_fs.shapes_arrays_metadata.neg_spheres_start;
        if ((_e1805 < (_e1809 + _e1813))) {
        } else {
            break;
        }
        {
            uint _e1817 = i_85;
            vec4 _e1820 = _group_0_binding_6_fs[_e1817].pos;
            uint _e1823 = i_85;
            float _e1827 = _group_0_binding_6_fs[_e1823].size.x;
            uint _e1829 = i_85;
            float _e1832 = _group_0_binding_6_fs[_e1829].roundness;
            vec2 _e1834 = sph_intersection((ro_3 - _e1820), rd_3, (_e1827 + _e1832));
            if ((_e1834.y > 0.0)) {
                uint _e1841 = in_5.ismd.dyn_neg_spheres_amount;
                in_5.ismd.dyn_neg_spheres_amount = (_e1841 + 1u);
                uint _e1844 = ish_index;
                uint _e1846 = i_85;
                in_5.ish[_e1844] = _e1846;
                uint _e1848 = ish_index;
                ish_index = (_e1848 + 1u);
            }
        }
    }
    uint _e1855 = ish_index;
    in_5.ismd.dyn_neg_sph_cubes_start = _e1855;
    uint _e1859 = _group_0_binding_9_fs.shapes_arrays_metadata.neg_sph_cubes_start;
    i_86 = _e1859;
    bool loop_init_92 = true;
    while(true) {
        if (!loop_init_92) {
            uint _e1907 = i_86;
            i_86 = (_e1907 + 1u);
        }
        loop_init_92 = false;
        uint _e1861 = i_86;
        uint _e1865 = _group_0_binding_9_fs.shapes_arrays_metadata.neg_sph_cubes_amount;
        uint _e1869 = _group_0_binding_9_fs.shapes_arrays_metadata.neg_sph_cubes_start;
        if ((_e1861 < (_e1865 + _e1869))) {
        } else {
            break;
        }
        {
            uint _e1873 = i_86;
            vec4 _e1876 = _group_0_binding_6_fs[_e1873].pos;
            uint _e1879 = i_86;
            vec4 _e1882 = _group_0_binding_6_fs[_e1879].size;
            uint _e1884 = i_86;
            float _e1887 = _group_0_binding_6_fs[_e1884].roundness;
            vec2 _e1890 = cube_intersection((ro_3 - _e1876), rd_3, (_e1882 + vec4(_e1887)));
            if ((_e1890.y > 0.0)) {
                uint _e1897 = in_5.ismd.dyn_neg_sph_cubes_amount;
                in_5.ismd.dyn_neg_sph_cubes_amount = (_e1897 + 1u);
                uint _e1900 = ish_index;
                uint _e1902 = i_86;
                in_5.ish[_e1900] = _e1902;
                uint _e1904 = ish_index;
                ish_index = (_e1904 + 1u);
            }
        }
    }
    uint _e1911 = ish_index;
    in_5.ismd.dyn_neg_inf_cubes_start = _e1911;
    uint _e1915 = _group_0_binding_9_fs.shapes_arrays_metadata.neg_inf_cubes_start;
    i_87 = _e1915;
    bool loop_init_93 = true;
    while(true) {
        if (!loop_init_93) {
            uint _e1964 = i_87;
            i_87 = (_e1964 + 1u);
        }
        loop_init_93 = false;
        uint _e1917 = i_87;
        uint _e1921 = _group_0_binding_9_fs.shapes_arrays_metadata.neg_inf_cubes_amount;
        uint _e1925 = _group_0_binding_9_fs.shapes_arrays_metadata.neg_inf_cubes_start;
        if ((_e1917 < (_e1921 + _e1925))) {
        } else {
            break;
        }
        {
            uint _e1929 = i_87;
            vec4 _e1932 = _group_0_binding_6_fs[_e1929].pos;
            uint _e1935 = i_87;
            vec4 _e1938 = _group_0_binding_6_fs[_e1935].size;
            uint _e1941 = i_87;
            float _e1944 = _group_0_binding_6_fs[_e1941].roundness;
            vec2 _e1947 = inf_cube_intersection((ro_3 - _e1932), rd_3, (_e1938.xyz + vec3(_e1944)));
            if ((_e1947.y > 0.0)) {
                uint _e1954 = in_5.ismd.dyn_neg_inf_cubes_amount;
                in_5.ismd.dyn_neg_inf_cubes_amount = (_e1954 + 1u);
                uint _e1957 = ish_index;
                uint _e1959 = i_87;
                in_5.ish[_e1957] = _e1959;
                uint _e1961 = ish_index;
                ish_index = (_e1961 + 1u);
            }
        }
    }
    uint _e1968 = ish_index;
    in_5.ismd.player_forms_start = _e1968;
    bool loop_init_94 = true;
    while(true) {
        if (!loop_init_94) {
            uint _e2009 = i_88;
            i_88 = (_e2009 + 1u);
        }
        loop_init_94 = false;
        uint _e1971 = i_88;
        uint _e1974 = _group_0_binding_9_fs.player_forms_amount;
        if ((_e1971 < _e1974)) {
        } else {
            break;
        }
        {
            uint _e1977 = i_88;
            vec4 _e1980 = _group_1_binding_2_fs[_e1977].pos;
            uint _e1983 = i_88;
            float _e1986 = _group_1_binding_2_fs[_e1983].radius;
            vec2 _e1989 = sph_intersection((ro_3 - _e1980), rd_3, (_e1986 * 1.5));
            if ((_e1989.y > 0.0)) {
                uint _e1996 = in_5.ismd.player_forms_amount;
                in_5.ismd.player_forms_amount = (_e1996 + 1u);
                uint _e1999 = ish_index;
                uint _e2001 = i_88;
                in_5.ish[_e1999] = _e2001;
                uint _e2003 = ish_index;
                ish_index = (_e2003 + 1u);
                float _e2005 = offset;
                offset = min(_e2005, _e1989.x);
            }
        }
    }
    in_5.ray_w_rotated = false;
    if ((rd_3.w < -0.0002)) {
        in_5.ray_w_rotated = true;
    }
    float _e2018 = offset;
    offset = clamp(_e2018, 0.0, 1400.0);
    float _e2023 = offset;
    in_5.offset = _e2023;
    Intersections _e2024 = in_5;
    return _e2024;
}

vec3 apply_material(vec4 pos_1, vec4 ray_dir_1, float dist_2, inout Intersections in_6, int material) {
    vec3 color_5 = vec3(0.0);
    if ((material < 0)) {
        return vec3(0.7);
    }
    vec4 _e13 = _group_0_binding_4_fs.materials[material].color;
    vec3 diffuse = _e13.xyz;
    vec4 _e17 = get_normal((pos_1 + (ray_dir_1 * dist_2)), in_6);
    float dir_shade = dot(_e17, normalize(vec4(1.0, 0.5, 0.3, 0.1)));
    color_5 = (diffuse * dir_shade);
    vec3 _e27 = color_5;
    return _e27;
}

void main() {
    VertexOutput inn = VertexOutput(gl_FragCoord, _vs2fs_location0);
    vec2 uv = vec2(0.0);
    vec4 ray_direction = vec4(0.0);
    Intersections in_ = Intersections(IntersectedShapesMetadata(0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u), uint[16](0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u), 0.0, false);
    int mats[32] = int[32](0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
    float mats_wieghts[32] = float[32](0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    uint mats_count = 0u;
    vec3 color = vec3(0.0);
    uint i = 1u;
    uv = (inn.position.xy * 0.7);
    float _e9 = _group_0_binding_9_fs.screen_aspect;
    float _e10 = uv.x;
    uv.x = (_e10 * _e9);
    vec2 _e12 = uv;
    ray_direction = normalize(vec4(_e12, -1.0, 0.0));
    mat4x4 _e21 = _group_0_binding_9_fs.camera_data.cam_rot;
    vec4 _e22 = ray_direction;
    ray_direction = (_e22 * _e21);
    vec4 camera_position = _group_0_binding_9_fs.camera_data.cam_pos;
    vec4 _e28 = ray_direction;
    Intersections _e29 = find_intersections(camera_position, _e28);
    in_ = _e29;
    vec4 _e31 = ray_direction;
    vec2 _e32 = ray_march(camera_position, _e31, in_);
    vec4 _e36 = ray_direction;
    get_mat(camera_position, _e36, _e32.x, in_, mats, mats_wieghts, mats_count);
    vec4 _e38 = ray_direction;
    int _e41 = mats[0];
    vec3 _e42 = apply_material(camera_position, _e38, _e32.x, in_, _e41);
    color = _e42;
    bool loop_init_95 = true;
    while(true) {
        if (!loop_init_95) {
            uint _e61 = i;
            i = (_e61 + 1u);
        }
        loop_init_95 = false;
        uint _e46 = i;
        uint _e47 = mats_count;
        if ((_e46 < _e47)) {
        } else {
            break;
        }
        {
            vec4 _e49 = ray_direction;
            uint _e51 = i;
            int _e53 = mats[_e51];
            vec3 _e54 = apply_material(camera_position, _e49, _e32.x, in_, _e53);
            vec3 _e55 = color;
            uint _e56 = i;
            float _e58 = mats_wieghts[_e56];
            color = mix(_e55, _e54, _e58);
        }
    }
    vec3 _e63 = color;
    color = pow(_e63, vec3(0.4545));
    vec2 _e67 = uv;
    vec3 _e76 = color;
    color = (_e76 + vec3(((0.007 - clamp(length(_e67), 0.0, 0.007)) * 1000.0)));
    vec3 _e79 = color;
    _fs2p_location0 = vec4(_e79, 1.0);
    return;
}

