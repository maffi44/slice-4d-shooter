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
layout(set = 1, binding = 0) uniform type_16_block_10Fragment { PlayerForm _group_1_binding_2_fs[16]; };

bool ray_w_rotated = false;

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

void get_mat(vec4 cam_pos, vec4 ray_dir, float dist, inout IntersectedShapesMetadata ismd_1, inout uint ish_1[16], inout int mats_1[32], inout float mats_wieghts_1[32], inout uint mat_count) {
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
    IntersectedShapesMetadata ismda = ismd_1;
    mat_count = 0u;
    j = ismda.player_forms_start;
    bool loop_init_2 = true;
    while(true) {
        if (!loop_init_2) {
            uint _e198 = j;
            j = (_e198 + 1u);
        }
        loop_init_2 = false;
        uint _e21 = j;
        if ((_e21 < (ismda.player_forms_amount + ismda.player_forms_start))) {
        } else {
            break;
        }
        {
            uint _e27 = j;
            uint _e29 = ish_1[_e27];
            PlayerForm shape = _group_1_binding_2_fs[_e29];
            float _e35 = sd_sphere((p_17 - shape.pos), shape.radius);
            d_6 = _e35;
            float _e37 = d_6;
            float _e43 = sd_sphere((p_17 - shape.pos), (shape.radius * 0.86));
            d_6 = max(_e37, -(_e43));
            vec4 rotated_p = (shape.rotation * (p_17 - shape.pos));
            float _e50 = d_6;
            float _e64 = sd_box(rotated_p, vec4((shape.radius * 0.18), (shape.radius * 1.2), (shape.radius * 1.2), (shape.radius * 1.2)));
            d_6 = max(_e50, -(_e64));
            float _e67 = d_6;
            float _e78 = sd_sphere((rotated_p - vec4(0.0, 0.0, -(shape.radius), 0.0)), (shape.radius * 0.53));
            d_6 = max(_e67, -(_e78));
            float _e81 = d_6;
            if ((_e81 < MIN_DIST)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                int _e90 = _group_0_binding_4_fs.players_mat1_;
                mats_1[0] = _e90;
                return;
            }
            float _e96 = sd_sphere((p_17 - shape.pos), (shape.radius * 0.6));
            d_6 = _e96;
            float _e97 = d_6;
            float _e110 = sd_sphere((rotated_p - (vec4(0.0, 0.0, -(shape.radius), 0.0) * 0.6)), (shape.radius * 0.34));
            d_6 = max(_e97, -(_e110));
            float _e113 = d_6;
            if ((_e113 < MIN_DIST)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                int _e122 = _group_0_binding_4_fs.players_mat2_;
                mats_1[0] = _e122;
                return;
            }
            float _e128 = sd_sphere((rotated_p - shape.weapon_offset), (shape.radius * 0.286));
            d_6 = _e128;
            float _e129 = d_6;
            float _e143 = sd_capsule(rotated_p, shape.weapon_offset, (shape.weapon_offset - vec4(0.0, 0.0, (shape.radius * 0.49), 0.0)), (shape.radius * 0.18));
            d_6 = max(_e129, -(_e143));
            float _e146 = d_6;
            if ((_e146 < MIN_DIST)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                int _e155 = _group_0_binding_4_fs.players_mat1_;
                mats_1[0] = _e155;
                return;
            }
            float _e169 = sd_capsule(rotated_p, shape.weapon_offset, (shape.weapon_offset - vec4(0.0, 0.0, (shape.radius * 0.43), 0.0)), (shape.radius * 0.1));
            d_6 = _e169;
            float _e170 = d_6;
            float _e184 = sd_capsule(rotated_p, shape.weapon_offset, (shape.weapon_offset - vec4(0.0, 0.0, (shape.radius * 0.65), 0.0)), (shape.radius * 0.052));
            d_6 = max(_e170, -(_e184));
            float _e187 = d_6;
            if ((_e187 < MIN_DIST)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                int _e196 = _group_0_binding_4_fs.players_mat2_;
                mats_1[0] = _e196;
                return;
            }
        }
    }
    i_6 = ismda.st_cubes_start;
    bool loop_init_3 = true;
    while(true) {
        if (!loop_init_3) {
            uint _e227 = i_6;
            i_6 = (_e227 + 1u);
        }
        loop_init_3 = false;
        uint _e202 = i_6;
        if ((_e202 < (ismda.st_cubes_amount + ismda.st_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e207 = i_6;
            uint j_2 = ish_1[_e207];
            Shape shape_1 = _group_0_binding_0_fs[j_2];
            float _e216 = sd_box((p_17 - shape_1.pos), shape_1.size);
            if (((_e216 - shape_1.roundness) < 0.02)) {
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
            uint _e257 = i_7;
            i_7 = (_e257 + 1u);
        }
        loop_init_4 = false;
        uint _e231 = i_7;
        if ((_e231 < (ismda.st_spheres_amount + ismda.st_spheres_start))) {
        } else {
            break;
        }
        {
            uint _e236 = i_7;
            uint j_3 = ish_1[_e236];
            Shape shape_2 = _group_0_binding_0_fs[j_3];
            float _e246 = sd_sphere((p_17 - shape_2.pos), shape_2.size.x);
            if (((_e246 - shape_2.roundness) < 0.02)) {
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
            uint _e286 = i_8;
            i_8 = (_e286 + 1u);
        }
        loop_init_5 = false;
        uint _e261 = i_8;
        if ((_e261 < (ismda.st_sph_cubes_amount + ismda.st_sph_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e266 = i_8;
            uint j_4 = ish_1[_e266];
            Shape shape_3 = _group_0_binding_0_fs[j_4];
            float _e275 = sd_sph_box((p_17 - shape_3.pos), shape_3.size);
            if (((_e275 - shape_3.roundness) < 0.02)) {
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
            uint _e316 = i_9;
            i_9 = (_e316 + 1u);
        }
        loop_init_6 = false;
        uint _e290 = i_9;
        if ((_e290 < (ismda.st_inf_cubes_amount + ismda.st_inf_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e295 = i_9;
            uint j_5 = ish_1[_e295];
            Shape shape_4 = _group_0_binding_0_fs[j_5];
            float _e305 = sd_inf_box((p_17 - shape_4.pos), shape_4.size.xyz);
            if (((_e305 - shape_4.roundness) < 0.02)) {
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
            uint _e345 = i_10;
            i_10 = (_e345 + 1u);
        }
        loop_init_7 = false;
        uint _e320 = i_10;
        if ((_e320 < (ismda.dyn_cubes_amount + ismda.dyn_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e325 = i_10;
            uint j_6 = ish_1[_e325];
            Shape shape_5 = _group_0_binding_5_fs[j_6];
            float _e334 = sd_box((p_17 - shape_5.pos), shape_5.size);
            if (((_e334 - shape_5.roundness) < 0.02)) {
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
            uint _e375 = i_11;
            i_11 = (_e375 + 1u);
        }
        loop_init_8 = false;
        uint _e349 = i_11;
        if ((_e349 < (ismda.dyn_spheres_amount + ismda.dyn_spheres_start))) {
        } else {
            break;
        }
        {
            uint _e354 = i_11;
            uint j_7 = ish_1[_e354];
            Shape shape_6 = _group_0_binding_5_fs[j_7];
            float _e364 = sd_sphere((p_17 - shape_6.pos), shape_6.size.x);
            if (((_e364 - shape_6.roundness) < 0.02)) {
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
            uint _e404 = i_12;
            i_12 = (_e404 + 1u);
        }
        loop_init_9 = false;
        uint _e379 = i_12;
        if ((_e379 < (ismda.dyn_sph_cubes_amount + ismda.dyn_sph_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e384 = i_12;
            uint j_8 = ish_1[_e384];
            Shape shape_7 = _group_0_binding_5_fs[j_8];
            float _e393 = sd_sph_box((p_17 - shape_7.pos), shape_7.size);
            if (((_e393 - shape_7.roundness) < 0.02)) {
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
            uint _e434 = i_13;
            i_13 = (_e434 + 1u);
        }
        loop_init_10 = false;
        uint _e408 = i_13;
        if ((_e408 < (ismda.dyn_inf_cubes_amount + ismda.dyn_inf_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e413 = i_13;
            uint j_9 = ish_1[_e413];
            Shape shape_8 = _group_0_binding_5_fs[j_9];
            float _e423 = sd_inf_box((p_17 - shape_8.pos), shape_8.size.xyz);
            if (((_e423 - shape_8.roundness) < 0.02)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                mats_1[0] = shape_8.material;
                return;
            }
        }
    }
    int _e438 = _group_0_binding_4_fs.is_w_floor_exist;
    if ((_e438 == 1)) {
        bool _e442 = ray_w_rotated;
        if (_e442) {
            float _e446 = _group_0_binding_4_fs.w_floor;
            if (((p_17.w - _e446) < 0.02)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                int _e456 = _group_0_binding_4_fs.w_cups_mat;
                mats_1[0] = _e456;
                return;
            }
        }
    }
    i_14 = ismda.st_s_cubes_start;
    bool loop_init_11 = true;
    while(true) {
        if (!loop_init_11) {
            uint _e550 = i_14;
            i_14 = (_e550 + 1u);
        }
        loop_init_11 = false;
        uint _e461 = i_14;
        if ((_e461 < (ismda.st_s_cubes_amount + ismda.st_s_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e466 = i_14;
            uint j_10 = ish_1[_e466];
            Shape shape_9 = _group_0_binding_2_fs[j_10];
            float _e475 = sd_box((p_17 - shape_9.pos), shape_9.size);
            float dd_1 = (_e475 - shape_9.roundness);
            if ((dd_1 < 0.02)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                mats_1[0] = shape_9.material;
                return;
            }
            float _e487 = _group_0_binding_4_fs.stickiness;
            if ((dd_1 < (_e487 * STICKINESS_EFFECT_COEF))) {
                uint _e491 = mat_count;
                if ((_e491 == 0u)) {
                    mat_count = 1u;
                    mats_wieghts_1[0] = 1.0;
                    mats_1[0] = shape_9.material;
                    d_7 = dd_1;
                } else {
                    coef = 0.0;
                    float _e501 = d_7;
                    if ((_e501 < dd_1)) {
                        float _e503 = d_7;
                        coef = clamp((pow((_e503 / dd_1), 1.9) * 0.5), 0.0, 1.0);
                    } else {
                        float _e512 = d_7;
                        coef = (1.0 - clamp((pow((dd_1 / _e512), 1.9) * 0.5), 0.0, 1.0));
                    }
                    uint _e523 = mat_count;
                    mats_1[_e523] = shape_9.material;
                    uint _e526 = mat_count;
                    float _e528 = coef;
                    mats_wieghts_1[_e526] = _e528;
                    float _e529 = coef;
                    float mult = (1.0 - _e529);
                    k_1 = 0u;
                    bool loop_init_12 = true;
                    while(true) {
                        if (!loop_init_12) {
                            uint _e542 = k_1;
                            k_1 = (_e542 + 1u);
                        }
                        loop_init_12 = false;
                        uint _e534 = k_1;
                        uint _e535 = mat_count;
                        if ((_e534 < _e535)) {
                        } else {
                            break;
                        }
                        {
                            uint _e537 = k_1;
                            float _e539 = mats_wieghts_1[_e537];
                            mats_wieghts_1[_e537] = (_e539 * mult);
                        }
                    }
                    uint _e545 = mat_count;
                    mat_count = (_e545 + 1u);
                    float _e547 = d_7;
                    d_7 = min(_e547, dd_1);
                }
            }
        }
    }
    i_15 = ismda.st_s_spheres_start;
    bool loop_init_13 = true;
    while(true) {
        if (!loop_init_13) {
            uint _e644 = i_15;
            i_15 = (_e644 + 1u);
        }
        loop_init_13 = false;
        uint _e554 = i_15;
        if ((_e554 < (ismda.st_s_spheres_amount + ismda.st_s_spheres_start))) {
        } else {
            break;
        }
        {
            uint _e559 = i_15;
            uint j_11 = ish_1[_e559];
            Shape shape_10 = _group_0_binding_2_fs[j_11];
            float _e569 = sd_sphere((p_17 - shape_10.pos), shape_10.size.x);
            float dd_2 = (_e569 - shape_10.roundness);
            if ((dd_2 < 0.02)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                mats_1[0] = shape_10.material;
                return;
            }
            float _e581 = _group_0_binding_4_fs.stickiness;
            if ((dd_2 < (_e581 * STICKINESS_EFFECT_COEF))) {
                uint _e585 = mat_count;
                if ((_e585 == 0u)) {
                    mat_count = 1u;
                    mats_wieghts_1[0] = 1.0;
                    mats_1[0] = shape_10.material;
                    d_7 = dd_2;
                } else {
                    coef_1 = 0.0;
                    float _e595 = d_7;
                    if ((_e595 < dd_2)) {
                        float _e597 = d_7;
                        coef_1 = clamp((pow((_e597 / dd_2), 1.9) * 0.5), 0.0, 1.0);
                    } else {
                        float _e606 = d_7;
                        coef_1 = (1.0 - clamp((pow((dd_2 / _e606), 1.9) * 0.5), 0.0, 1.0));
                    }
                    uint _e617 = mat_count;
                    mats_1[_e617] = shape_10.material;
                    uint _e620 = mat_count;
                    float _e622 = coef_1;
                    mats_wieghts_1[_e620] = _e622;
                    float _e623 = coef_1;
                    float mult_1 = (1.0 - _e623);
                    k_2 = 0u;
                    bool loop_init_14 = true;
                    while(true) {
                        if (!loop_init_14) {
                            uint _e636 = k_2;
                            k_2 = (_e636 + 1u);
                        }
                        loop_init_14 = false;
                        uint _e628 = k_2;
                        uint _e629 = mat_count;
                        if ((_e628 < _e629)) {
                        } else {
                            break;
                        }
                        {
                            uint _e631 = k_2;
                            float _e633 = mats_wieghts_1[_e631];
                            mats_wieghts_1[_e631] = (_e633 * mult_1);
                        }
                    }
                    uint _e639 = mat_count;
                    mat_count = (_e639 + 1u);
                    float _e641 = d_7;
                    d_7 = min(_e641, dd_2);
                }
            }
        }
    }
    i_16 = ismda.st_s_sph_cubes_start;
    bool loop_init_15 = true;
    while(true) {
        if (!loop_init_15) {
            uint _e737 = i_16;
            i_16 = (_e737 + 1u);
        }
        loop_init_15 = false;
        uint _e648 = i_16;
        if ((_e648 < (ismda.st_s_sph_cubes_amount + ismda.st_s_sph_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e653 = i_16;
            uint j_12 = ish_1[_e653];
            Shape shape_11 = _group_0_binding_2_fs[j_12];
            float _e662 = sd_sph_box((p_17 - shape_11.pos), shape_11.size);
            float dd_3 = (_e662 - shape_11.roundness);
            if ((dd_3 < 0.02)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                mats_1[0] = shape_11.material;
                return;
            }
            float _e674 = _group_0_binding_4_fs.stickiness;
            if ((dd_3 < (_e674 * STICKINESS_EFFECT_COEF))) {
                uint _e678 = mat_count;
                if ((_e678 == 0u)) {
                    mat_count = 1u;
                    mats_wieghts_1[0] = 1.0;
                    mats_1[0] = shape_11.material;
                    d_7 = dd_3;
                } else {
                    coef_2 = 0.0;
                    float _e688 = d_7;
                    if ((_e688 < dd_3)) {
                        float _e690 = d_7;
                        coef_2 = clamp((pow((_e690 / dd_3), 1.9) * 0.5), 0.0, 1.0);
                    } else {
                        float _e699 = d_7;
                        coef_2 = (1.0 - clamp((pow((dd_3 / _e699), 1.9) * 0.5), 0.0, 1.0));
                    }
                    uint _e710 = mat_count;
                    mats_1[_e710] = shape_11.material;
                    uint _e713 = mat_count;
                    float _e715 = coef_2;
                    mats_wieghts_1[_e713] = _e715;
                    float _e716 = coef_2;
                    float mult_2 = (1.0 - _e716);
                    k_3 = 0u;
                    bool loop_init_16 = true;
                    while(true) {
                        if (!loop_init_16) {
                            uint _e729 = k_3;
                            k_3 = (_e729 + 1u);
                        }
                        loop_init_16 = false;
                        uint _e721 = k_3;
                        uint _e722 = mat_count;
                        if ((_e721 < _e722)) {
                        } else {
                            break;
                        }
                        {
                            uint _e724 = k_3;
                            float _e726 = mats_wieghts_1[_e724];
                            mats_wieghts_1[_e724] = (_e726 * mult_2);
                        }
                    }
                    uint _e732 = mat_count;
                    mat_count = (_e732 + 1u);
                    float _e734 = d_7;
                    d_7 = min(_e734, dd_3);
                }
            }
        }
    }
    i_17 = ismda.st_s_inf_cubes_start;
    bool loop_init_17 = true;
    while(true) {
        if (!loop_init_17) {
            uint _e831 = i_17;
            i_17 = (_e831 + 1u);
        }
        loop_init_17 = false;
        uint _e741 = i_17;
        if ((_e741 < (ismda.st_s_inf_cubes_amount + ismda.st_s_inf_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e746 = i_17;
            uint j_13 = ish_1[_e746];
            Shape shape_12 = _group_0_binding_2_fs[j_13];
            float _e756 = sd_inf_box((p_17 - shape_12.pos), shape_12.size.xyz);
            float dd_4 = (_e756 - shape_12.roundness);
            if ((dd_4 < 0.02)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                mats_1[0] = shape_12.material;
                return;
            }
            float _e768 = _group_0_binding_4_fs.stickiness;
            if ((dd_4 < (_e768 * STICKINESS_EFFECT_COEF))) {
                uint _e772 = mat_count;
                if ((_e772 == 0u)) {
                    mat_count = 1u;
                    mats_wieghts_1[0] = 1.0;
                    mats_1[0] = shape_12.material;
                    d_7 = dd_4;
                } else {
                    coef_3 = 0.0;
                    float _e782 = d_7;
                    if ((_e782 < dd_4)) {
                        float _e784 = d_7;
                        coef_3 = clamp((pow((_e784 / dd_4), 1.9) * 0.5), 0.0, 1.0);
                    } else {
                        float _e793 = d_7;
                        coef_3 = (1.0 - clamp((pow((dd_4 / _e793), 1.9) * 0.5), 0.0, 1.0));
                    }
                    uint _e804 = mat_count;
                    mats_1[_e804] = shape_12.material;
                    uint _e807 = mat_count;
                    float _e809 = coef_3;
                    mats_wieghts_1[_e807] = _e809;
                    float _e810 = coef_3;
                    float mult_3 = (1.0 - _e810);
                    k_4 = 0u;
                    bool loop_init_18 = true;
                    while(true) {
                        if (!loop_init_18) {
                            uint _e823 = k_4;
                            k_4 = (_e823 + 1u);
                        }
                        loop_init_18 = false;
                        uint _e815 = k_4;
                        uint _e816 = mat_count;
                        if ((_e815 < _e816)) {
                        } else {
                            break;
                        }
                        {
                            uint _e818 = k_4;
                            float _e820 = mats_wieghts_1[_e818];
                            mats_wieghts_1[_e818] = (_e820 * mult_3);
                        }
                    }
                    uint _e826 = mat_count;
                    mat_count = (_e826 + 1u);
                    float _e828 = d_7;
                    d_7 = min(_e828, dd_4);
                }
            }
        }
    }
    i_18 = ismda.dyn_s_cubes_start;
    bool loop_init_19 = true;
    while(true) {
        if (!loop_init_19) {
            uint _e924 = i_18;
            i_18 = (_e924 + 1u);
        }
        loop_init_19 = false;
        uint _e835 = i_18;
        if ((_e835 < (ismda.dyn_s_cubes_amount + ismda.dyn_s_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e840 = i_18;
            uint j_14 = ish_1[_e840];
            Shape shape_13 = _group_0_binding_7_fs[j_14];
            float _e849 = sd_box((p_17 - shape_13.pos), shape_13.size);
            float dd_5 = (_e849 - shape_13.roundness);
            if ((dd_5 < 0.02)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                mats_1[0] = shape_13.material;
                return;
            }
            float _e861 = _group_0_binding_4_fs.stickiness;
            if ((dd_5 < (_e861 * STICKINESS_EFFECT_COEF))) {
                uint _e865 = mat_count;
                if ((_e865 == 0u)) {
                    mat_count = 1u;
                    mats_wieghts_1[0] = 1.0;
                    mats_1[0] = shape_13.material;
                    d_7 = dd_5;
                } else {
                    coef_4 = 0.0;
                    float _e875 = d_7;
                    if ((_e875 < dd_5)) {
                        float _e877 = d_7;
                        coef_4 = clamp((pow((_e877 / dd_5), 1.9) * 0.5), 0.0, 1.0);
                    } else {
                        float _e886 = d_7;
                        coef_4 = (1.0 - clamp((pow((dd_5 / _e886), 1.9) * 0.5), 0.0, 1.0));
                    }
                    uint _e897 = mat_count;
                    mats_1[_e897] = shape_13.material;
                    uint _e900 = mat_count;
                    float _e902 = coef_4;
                    mats_wieghts_1[_e900] = _e902;
                    float _e903 = coef_4;
                    float mult_4 = (1.0 - _e903);
                    k_5 = 0u;
                    bool loop_init_20 = true;
                    while(true) {
                        if (!loop_init_20) {
                            uint _e916 = k_5;
                            k_5 = (_e916 + 1u);
                        }
                        loop_init_20 = false;
                        uint _e908 = k_5;
                        uint _e909 = mat_count;
                        if ((_e908 < _e909)) {
                        } else {
                            break;
                        }
                        {
                            uint _e911 = k_5;
                            float _e913 = mats_wieghts_1[_e911];
                            mats_wieghts_1[_e911] = (_e913 * mult_4);
                        }
                    }
                    uint _e919 = mat_count;
                    mat_count = (_e919 + 1u);
                    float _e921 = d_7;
                    d_7 = min(_e921, dd_5);
                }
            }
        }
    }
    i_19 = ismda.dyn_s_spheres_start;
    bool loop_init_21 = true;
    while(true) {
        if (!loop_init_21) {
            uint _e1018 = i_19;
            i_19 = (_e1018 + 1u);
        }
        loop_init_21 = false;
        uint _e928 = i_19;
        if ((_e928 < (ismda.dyn_s_spheres_amount + ismda.dyn_s_spheres_start))) {
        } else {
            break;
        }
        {
            uint _e933 = i_19;
            uint j_15 = ish_1[_e933];
            Shape shape_14 = _group_0_binding_7_fs[j_15];
            float _e943 = sd_sphere((p_17 - shape_14.pos), shape_14.size.x);
            float dd_6 = (_e943 - shape_14.roundness);
            if ((dd_6 < 0.02)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                mats_1[0] = shape_14.material;
                return;
            }
            float _e955 = _group_0_binding_4_fs.stickiness;
            if ((dd_6 < (_e955 * STICKINESS_EFFECT_COEF))) {
                uint _e959 = mat_count;
                if ((_e959 == 0u)) {
                    mat_count = 1u;
                    mats_wieghts_1[0] = 1.0;
                    mats_1[0] = shape_14.material;
                    d_7 = dd_6;
                } else {
                    coef_5 = 0.0;
                    float _e969 = d_7;
                    if ((_e969 < dd_6)) {
                        float _e971 = d_7;
                        coef_5 = clamp((pow((_e971 / dd_6), 1.9) * 0.5), 0.0, 1.0);
                    } else {
                        float _e980 = d_7;
                        coef_5 = (1.0 - clamp((pow((dd_6 / _e980), 1.9) * 0.5), 0.0, 1.0));
                    }
                    uint _e991 = mat_count;
                    mats_1[_e991] = shape_14.material;
                    uint _e994 = mat_count;
                    float _e996 = coef_5;
                    mats_wieghts_1[_e994] = _e996;
                    float _e997 = coef_5;
                    float mult_5 = (1.0 - _e997);
                    k_6 = 0u;
                    bool loop_init_22 = true;
                    while(true) {
                        if (!loop_init_22) {
                            uint _e1010 = k_6;
                            k_6 = (_e1010 + 1u);
                        }
                        loop_init_22 = false;
                        uint _e1002 = k_6;
                        uint _e1003 = mat_count;
                        if ((_e1002 < _e1003)) {
                        } else {
                            break;
                        }
                        {
                            uint _e1005 = k_6;
                            float _e1007 = mats_wieghts_1[_e1005];
                            mats_wieghts_1[_e1005] = (_e1007 * mult_5);
                        }
                    }
                    uint _e1013 = mat_count;
                    mat_count = (_e1013 + 1u);
                    float _e1015 = d_7;
                    d_7 = min(_e1015, dd_6);
                }
            }
        }
    }
    i_20 = ismda.dyn_s_sph_cubes_start;
    bool loop_init_23 = true;
    while(true) {
        if (!loop_init_23) {
            uint _e1111 = i_20;
            i_20 = (_e1111 + 1u);
        }
        loop_init_23 = false;
        uint _e1022 = i_20;
        if ((_e1022 < (ismda.dyn_s_sph_cubes_amount + ismda.dyn_s_sph_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e1027 = i_20;
            uint j_16 = ish_1[_e1027];
            Shape shape_15 = _group_0_binding_7_fs[j_16];
            float _e1036 = sd_sph_box((p_17 - shape_15.pos), shape_15.size);
            float dd_7 = (_e1036 - shape_15.roundness);
            if ((dd_7 < 0.02)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                mats_1[0] = shape_15.material;
                return;
            }
            float _e1048 = _group_0_binding_4_fs.stickiness;
            if ((dd_7 < (_e1048 * STICKINESS_EFFECT_COEF))) {
                uint _e1052 = mat_count;
                if ((_e1052 == 0u)) {
                    mat_count = 1u;
                    mats_wieghts_1[0] = 1.0;
                    mats_1[0] = shape_15.material;
                    d_7 = dd_7;
                } else {
                    coef_6 = 0.0;
                    float _e1062 = d_7;
                    if ((_e1062 < dd_7)) {
                        float _e1064 = d_7;
                        coef_6 = clamp((pow((_e1064 / dd_7), 1.9) * 0.5), 0.0, 1.0);
                    } else {
                        float _e1073 = d_7;
                        coef_6 = (1.0 - clamp((pow((dd_7 / _e1073), 1.9) * 0.5), 0.0, 1.0));
                    }
                    uint _e1084 = mat_count;
                    mats_1[_e1084] = shape_15.material;
                    uint _e1087 = mat_count;
                    float _e1089 = coef_6;
                    mats_wieghts_1[_e1087] = _e1089;
                    float _e1090 = coef_6;
                    float mult_6 = (1.0 - _e1090);
                    k_7 = 0u;
                    bool loop_init_24 = true;
                    while(true) {
                        if (!loop_init_24) {
                            uint _e1103 = k_7;
                            k_7 = (_e1103 + 1u);
                        }
                        loop_init_24 = false;
                        uint _e1095 = k_7;
                        uint _e1096 = mat_count;
                        if ((_e1095 < _e1096)) {
                        } else {
                            break;
                        }
                        {
                            uint _e1098 = k_7;
                            float _e1100 = mats_wieghts_1[_e1098];
                            mats_wieghts_1[_e1098] = (_e1100 * mult_6);
                        }
                    }
                    uint _e1106 = mat_count;
                    mat_count = (_e1106 + 1u);
                    float _e1108 = d_7;
                    d_7 = min(_e1108, dd_7);
                }
            }
        }
    }
    i_21 = ismda.dyn_s_inf_cubes_start;
    bool loop_init_25 = true;
    while(true) {
        if (!loop_init_25) {
            uint _e1205 = i_21;
            i_21 = (_e1205 + 1u);
        }
        loop_init_25 = false;
        uint _e1115 = i_21;
        if ((_e1115 < (ismda.dyn_s_inf_cubes_amount + ismda.dyn_s_inf_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e1120 = i_21;
            uint j_17 = ish_1[_e1120];
            Shape shape_16 = _group_0_binding_7_fs[j_17];
            float _e1130 = sd_inf_box((p_17 - shape_16.pos), shape_16.size.xyz);
            float dd_8 = (_e1130 - shape_16.roundness);
            if ((dd_8 < 0.02)) {
                mat_count = 1u;
                mats_wieghts_1[0] = 1.0;
                mats_1[0] = shape_16.material;
                return;
            }
            float _e1142 = _group_0_binding_4_fs.stickiness;
            if ((dd_8 < (_e1142 * STICKINESS_EFFECT_COEF))) {
                uint _e1146 = mat_count;
                if ((_e1146 == 0u)) {
                    mat_count = 1u;
                    mats_wieghts_1[0] = 1.0;
                    mats_1[0] = shape_16.material;
                    d_7 = dd_8;
                } else {
                    coef_7 = 0.0;
                    float _e1156 = d_7;
                    if ((_e1156 < dd_8)) {
                        float _e1158 = d_7;
                        coef_7 = clamp((pow((_e1158 / dd_8), 1.9) * 0.5), 0.0, 1.0);
                    } else {
                        float _e1167 = d_7;
                        coef_7 = (1.0 - clamp((pow((dd_8 / _e1167), 1.9) * 0.5), 0.0, 1.0));
                    }
                    uint _e1178 = mat_count;
                    mats_1[_e1178] = shape_16.material;
                    uint _e1181 = mat_count;
                    float _e1183 = coef_7;
                    mats_wieghts_1[_e1181] = _e1183;
                    float _e1184 = coef_7;
                    float mult_7 = (1.0 - _e1184);
                    k_8 = 0u;
                    bool loop_init_26 = true;
                    while(true) {
                        if (!loop_init_26) {
                            uint _e1197 = k_8;
                            k_8 = (_e1197 + 1u);
                        }
                        loop_init_26 = false;
                        uint _e1189 = k_8;
                        uint _e1190 = mat_count;
                        if ((_e1189 < _e1190)) {
                        } else {
                            break;
                        }
                        {
                            uint _e1192 = k_8;
                            float _e1194 = mats_wieghts_1[_e1192];
                            mats_wieghts_1[_e1192] = (_e1194 * mult_7);
                        }
                    }
                    uint _e1200 = mat_count;
                    mat_count = (_e1200 + 1u);
                    float _e1202 = d_7;
                    d_7 = min(_e1202, dd_8);
                }
            }
        }
    }
    uint _e1207 = mat_count;
    if ((_e1207 == 0u)) {
        mat_count = 1u;
        mats_wieghts_1[0] = 1.0;
        mats_1[0] = -1;
    }
    return;
}

float map(vec4 p_15, inout IntersectedShapesMetadata ismd_2, inout uint ish_2[16]) {
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
    IntersectedShapesMetadata ismda_1 = ismd_2;
    i_22 = ismda_1.st_s_cubes_start;
    bool loop_init_27 = true;
    while(true) {
        if (!loop_init_27) {
            uint _e31 = i_22;
            i_22 = (_e31 + 1u);
        }
        loop_init_27 = false;
        uint _e8 = i_22;
        if ((_e8 < (ismda_1.st_s_cubes_amount + ismda_1.st_s_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e13 = i_22;
            uint j_18 = ish_2[_e13];
            Shape shape_17 = _group_0_binding_2_fs[j_18];
            float _e19 = d_8;
            float _e23 = sd_box((p_15 - shape_17.pos), shape_17.size);
            float _e28 = _group_0_binding_4_fs.stickiness;
            float _e29 = smin(_e19, (_e23 - shape_17.roundness), _e28);
            d_8 = _e29;
        }
    }
    i_23 = ismda_1.st_s_spheres_start;
    bool loop_init_28 = true;
    while(true) {
        if (!loop_init_28) {
            uint _e59 = i_23;
            i_23 = (_e59 + 1u);
        }
        loop_init_28 = false;
        uint _e35 = i_23;
        if ((_e35 < (ismda_1.st_s_spheres_amount + ismda_1.st_s_spheres_start))) {
        } else {
            break;
        }
        {
            uint _e40 = i_23;
            uint j_19 = ish_2[_e40];
            Shape shape_18 = _group_0_binding_2_fs[j_19];
            float _e46 = d_8;
            float _e51 = sd_sphere((p_15 - shape_18.pos), shape_18.size.x);
            float _e56 = _group_0_binding_4_fs.stickiness;
            float _e57 = smin(_e46, (_e51 - shape_18.roundness), _e56);
            d_8 = _e57;
        }
    }
    i_24 = ismda_1.st_s_sph_cubes_start;
    bool loop_init_29 = true;
    while(true) {
        if (!loop_init_29) {
            uint _e86 = i_24;
            i_24 = (_e86 + 1u);
        }
        loop_init_29 = false;
        uint _e63 = i_24;
        if ((_e63 < (ismda_1.st_s_sph_cubes_amount + ismda_1.st_s_sph_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e68 = i_24;
            uint j_20 = ish_2[_e68];
            Shape shape_19 = _group_0_binding_2_fs[j_20];
            float _e74 = d_8;
            float _e78 = sd_sph_box((p_15 - shape_19.pos), shape_19.size);
            float _e83 = _group_0_binding_4_fs.stickiness;
            float _e84 = smin(_e74, (_e78 - shape_19.roundness), _e83);
            d_8 = _e84;
        }
    }
    i_25 = ismda_1.st_s_inf_cubes_start;
    bool loop_init_30 = true;
    while(true) {
        if (!loop_init_30) {
            uint _e114 = i_25;
            i_25 = (_e114 + 1u);
        }
        loop_init_30 = false;
        uint _e90 = i_25;
        if ((_e90 < (ismda_1.st_s_inf_cubes_amount + ismda_1.st_s_inf_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e95 = i_25;
            uint j_21 = ish_2[_e95];
            Shape shape_20 = _group_0_binding_2_fs[j_21];
            float _e101 = d_8;
            float _e106 = sd_inf_box((p_15 - shape_20.pos), shape_20.size.xyz);
            float _e111 = _group_0_binding_4_fs.stickiness;
            float _e112 = smin(_e101, (_e106 - shape_20.roundness), _e111);
            d_8 = _e112;
        }
    }
    i_26 = ismda_1.dyn_s_cubes_start;
    bool loop_init_31 = true;
    while(true) {
        if (!loop_init_31) {
            uint _e141 = i_26;
            i_26 = (_e141 + 1u);
        }
        loop_init_31 = false;
        uint _e118 = i_26;
        if ((_e118 < (ismda_1.dyn_s_cubes_amount + ismda_1.dyn_s_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e123 = i_26;
            uint j_22 = ish_2[_e123];
            Shape shape_21 = _group_0_binding_7_fs[j_22];
            float _e129 = d_8;
            float _e133 = sd_box((p_15 - shape_21.pos), shape_21.size);
            float _e138 = _group_0_binding_4_fs.stickiness;
            float _e139 = smin(_e129, (_e133 - shape_21.roundness), _e138);
            d_8 = _e139;
        }
    }
    i_27 = ismda_1.dyn_s_spheres_start;
    bool loop_init_32 = true;
    while(true) {
        if (!loop_init_32) {
            uint _e169 = i_27;
            i_27 = (_e169 + 1u);
        }
        loop_init_32 = false;
        uint _e145 = i_27;
        if ((_e145 < (ismda_1.dyn_s_spheres_amount + ismda_1.dyn_s_spheres_start))) {
        } else {
            break;
        }
        {
            uint _e150 = i_27;
            uint j_23 = ish_2[_e150];
            Shape shape_22 = _group_0_binding_7_fs[j_23];
            float _e156 = d_8;
            float _e161 = sd_sphere((p_15 - shape_22.pos), shape_22.size.x);
            float _e166 = _group_0_binding_4_fs.stickiness;
            float _e167 = smin(_e156, (_e161 - shape_22.roundness), _e166);
            d_8 = _e167;
        }
    }
    i_28 = ismda_1.dyn_s_sph_cubes_start;
    bool loop_init_33 = true;
    while(true) {
        if (!loop_init_33) {
            uint _e196 = i_28;
            i_28 = (_e196 + 1u);
        }
        loop_init_33 = false;
        uint _e173 = i_28;
        if ((_e173 < (ismda_1.dyn_s_sph_cubes_amount + ismda_1.dyn_s_sph_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e178 = i_28;
            uint j_24 = ish_2[_e178];
            Shape shape_23 = _group_0_binding_7_fs[j_24];
            float _e184 = d_8;
            float _e188 = sd_sph_box((p_15 - shape_23.pos), shape_23.size);
            float _e193 = _group_0_binding_4_fs.stickiness;
            float _e194 = smin(_e184, (_e188 - shape_23.roundness), _e193);
            d_8 = _e194;
        }
    }
    i_29 = ismda_1.dyn_s_inf_cubes_start;
    bool loop_init_34 = true;
    while(true) {
        if (!loop_init_34) {
            uint _e224 = i_29;
            i_29 = (_e224 + 1u);
        }
        loop_init_34 = false;
        uint _e200 = i_29;
        if ((_e200 < (ismda_1.dyn_s_inf_cubes_amount + ismda_1.dyn_s_inf_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e205 = i_29;
            uint j_25 = ish_2[_e205];
            Shape shape_24 = _group_0_binding_7_fs[j_25];
            float _e211 = d_8;
            float _e216 = sd_inf_box((p_15 - shape_24.pos), shape_24.size.xyz);
            float _e221 = _group_0_binding_4_fs.stickiness;
            float _e222 = smin(_e211, (_e216 - shape_24.roundness), _e221);
            d_8 = _e222;
        }
    }
    i_30 = ismda_1.st_cubes_start;
    bool loop_init_35 = true;
    while(true) {
        if (!loop_init_35) {
            uint _e248 = i_30;
            i_30 = (_e248 + 1u);
        }
        loop_init_35 = false;
        uint _e228 = i_30;
        if ((_e228 < (ismda_1.st_cubes_amount + ismda_1.st_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e233 = i_30;
            uint j_26 = ish_2[_e233];
            Shape shape_25 = _group_0_binding_0_fs[j_26];
            float _e239 = d_8;
            float _e243 = sd_box((p_15 - shape_25.pos), shape_25.size);
            d_8 = min(_e239, (_e243 - shape_25.roundness));
        }
    }
    i_31 = ismda_1.st_spheres_start;
    bool loop_init_36 = true;
    while(true) {
        if (!loop_init_36) {
            uint _e273 = i_31;
            i_31 = (_e273 + 1u);
        }
        loop_init_36 = false;
        uint _e252 = i_31;
        if ((_e252 < (ismda_1.st_spheres_amount + ismda_1.st_spheres_start))) {
        } else {
            break;
        }
        {
            uint _e257 = i_31;
            uint j_27 = ish_2[_e257];
            Shape shape_26 = _group_0_binding_0_fs[j_27];
            float _e263 = d_8;
            float _e268 = sd_sphere((p_15 - shape_26.pos), shape_26.size.x);
            d_8 = min(_e263, (_e268 - shape_26.roundness));
        }
    }
    i_32 = ismda_1.st_sph_cubes_start;
    bool loop_init_37 = true;
    while(true) {
        if (!loop_init_37) {
            uint _e297 = i_32;
            i_32 = (_e297 + 1u);
        }
        loop_init_37 = false;
        uint _e277 = i_32;
        if ((_e277 < (ismda_1.st_sph_cubes_amount + ismda_1.st_sph_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e282 = i_32;
            uint j_28 = ish_2[_e282];
            Shape shape_27 = _group_0_binding_0_fs[j_28];
            float _e288 = d_8;
            float _e292 = sd_sph_box((p_15 - shape_27.pos), shape_27.size);
            d_8 = min(_e288, (_e292 - shape_27.roundness));
        }
    }
    i_33 = ismda_1.st_inf_cubes_start;
    bool loop_init_38 = true;
    while(true) {
        if (!loop_init_38) {
            uint _e322 = i_33;
            i_33 = (_e322 + 1u);
        }
        loop_init_38 = false;
        uint _e301 = i_33;
        if ((_e301 < (ismda_1.st_inf_cubes_amount + ismda_1.st_inf_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e306 = i_33;
            uint j_29 = ish_2[_e306];
            Shape shape_28 = _group_0_binding_0_fs[j_29];
            float _e312 = d_8;
            float _e317 = sd_inf_box((p_15 - shape_28.pos), shape_28.size.xyz);
            d_8 = min(_e312, (_e317 - shape_28.roundness));
        }
    }
    i_34 = ismda_1.dyn_cubes_start;
    bool loop_init_39 = true;
    while(true) {
        if (!loop_init_39) {
            uint _e346 = i_34;
            i_34 = (_e346 + 1u);
        }
        loop_init_39 = false;
        uint _e326 = i_34;
        if ((_e326 < (ismda_1.dyn_cubes_amount + ismda_1.dyn_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e331 = i_34;
            uint j_30 = ish_2[_e331];
            Shape shape_29 = _group_0_binding_5_fs[j_30];
            float _e337 = d_8;
            float _e341 = sd_box((p_15 - shape_29.pos), shape_29.size);
            d_8 = min(_e337, (_e341 - shape_29.roundness));
        }
    }
    i_35 = ismda_1.dyn_spheres_start;
    bool loop_init_40 = true;
    while(true) {
        if (!loop_init_40) {
            uint _e371 = i_35;
            i_35 = (_e371 + 1u);
        }
        loop_init_40 = false;
        uint _e350 = i_35;
        if ((_e350 < (ismda_1.dyn_spheres_amount + ismda_1.dyn_spheres_start))) {
        } else {
            break;
        }
        {
            uint _e355 = i_35;
            uint j_31 = ish_2[_e355];
            Shape shape_30 = _group_0_binding_5_fs[j_31];
            float _e361 = d_8;
            float _e366 = sd_sphere((p_15 - shape_30.pos), shape_30.size.x);
            d_8 = min(_e361, (_e366 - shape_30.roundness));
        }
    }
    i_36 = ismda_1.dyn_sph_cubes_start;
    bool loop_init_41 = true;
    while(true) {
        if (!loop_init_41) {
            uint _e395 = i_36;
            i_36 = (_e395 + 1u);
        }
        loop_init_41 = false;
        uint _e375 = i_36;
        if ((_e375 < (ismda_1.dyn_sph_cubes_amount + ismda_1.dyn_sph_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e380 = i_36;
            uint j_32 = ish_2[_e380];
            Shape shape_31 = _group_0_binding_5_fs[j_32];
            float _e386 = d_8;
            float _e390 = sd_sph_box((p_15 - shape_31.pos), shape_31.size);
            d_8 = min(_e386, (_e390 - shape_31.roundness));
        }
    }
    i_37 = ismda_1.dyn_inf_cubes_start;
    bool loop_init_42 = true;
    while(true) {
        if (!loop_init_42) {
            uint _e420 = i_37;
            i_37 = (_e420 + 1u);
        }
        loop_init_42 = false;
        uint _e399 = i_37;
        if ((_e399 < (ismda_1.dyn_inf_cubes_amount + ismda_1.dyn_inf_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e404 = i_37;
            uint j_33 = ish_2[_e404];
            Shape shape_32 = _group_0_binding_5_fs[j_33];
            float _e410 = d_8;
            float _e415 = sd_inf_box((p_15 - shape_32.pos), shape_32.size.xyz);
            d_8 = min(_e410, (_e415 - shape_32.roundness));
        }
    }
    i_38 = ismda_1.st_s_neg_cubes_start;
    bool loop_init_43 = true;
    while(true) {
        if (!loop_init_43) {
            uint _e449 = i_38;
            i_38 = (_e449 + 1u);
        }
        loop_init_43 = false;
        uint _e426 = i_38;
        if ((_e426 < (ismda_1.st_s_neg_cubes_amount + ismda_1.st_s_neg_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e431 = i_38;
            uint j_34 = ish_2[_e431];
            Shape shape_33 = _group_0_binding_3_fs[j_34];
            float _e437 = dd;
            float _e441 = sd_box((p_15 - shape_33.pos), shape_33.size);
            float _e446 = _group_0_binding_4_fs.stickiness;
            float _e447 = smin(_e437, (_e441 - shape_33.roundness), _e446);
            dd = _e447;
        }
    }
    i_39 = ismda_1.st_s_neg_spheres_start;
    bool loop_init_44 = true;
    while(true) {
        if (!loop_init_44) {
            uint _e477 = i_39;
            i_39 = (_e477 + 1u);
        }
        loop_init_44 = false;
        uint _e453 = i_39;
        if ((_e453 < (ismda_1.st_s_neg_spheres_amount + ismda_1.st_s_neg_spheres_start))) {
        } else {
            break;
        }
        {
            uint _e458 = i_39;
            uint j_35 = ish_2[_e458];
            Shape shape_34 = _group_0_binding_3_fs[j_35];
            float _e464 = dd;
            float _e469 = sd_sphere((p_15 - shape_34.pos), shape_34.size.x);
            float _e474 = _group_0_binding_4_fs.stickiness;
            float _e475 = smin(_e464, (_e469 - shape_34.roundness), _e474);
            dd = _e475;
        }
    }
    i_40 = ismda_1.st_s_neg_sph_cubes_start;
    bool loop_init_45 = true;
    while(true) {
        if (!loop_init_45) {
            uint _e504 = i_40;
            i_40 = (_e504 + 1u);
        }
        loop_init_45 = false;
        uint _e481 = i_40;
        if ((_e481 < (ismda_1.st_s_neg_sph_cubes_amount + ismda_1.st_s_neg_sph_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e486 = i_40;
            uint j_36 = ish_2[_e486];
            Shape shape_35 = _group_0_binding_3_fs[j_36];
            float _e492 = dd;
            float _e496 = sd_sph_box((p_15 - shape_35.pos), shape_35.size);
            float _e501 = _group_0_binding_4_fs.stickiness;
            float _e502 = smin(_e492, (_e496 - shape_35.roundness), _e501);
            dd = _e502;
        }
    }
    i_41 = ismda_1.st_s_neg_inf_cubes_start;
    bool loop_init_46 = true;
    while(true) {
        if (!loop_init_46) {
            uint _e532 = i_41;
            i_41 = (_e532 + 1u);
        }
        loop_init_46 = false;
        uint _e508 = i_41;
        if ((_e508 < (ismda_1.st_s_neg_inf_cubes_amount + ismda_1.st_s_neg_inf_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e513 = i_41;
            uint j_37 = ish_2[_e513];
            Shape shape_36 = _group_0_binding_3_fs[j_37];
            float _e519 = dd;
            float _e524 = sd_inf_box((p_15 - shape_36.pos), shape_36.size.xyz);
            float _e529 = _group_0_binding_4_fs.stickiness;
            float _e530 = smin(_e519, (_e524 - shape_36.roundness), _e529);
            dd = _e530;
        }
    }
    float _e534 = d_8;
    float _e535 = dd;
    d_8 = max(_e534, -(_e535));
    float _e538 = dd;
    ddd = _e538;
    i_42 = ismda_1.dyn_s_neg_cubes_start;
    bool loop_init_47 = true;
    while(true) {
        if (!loop_init_47) {
            uint _e565 = i_42;
            i_42 = (_e565 + 1u);
        }
        loop_init_47 = false;
        uint _e542 = i_42;
        if ((_e542 < (ismda_1.dyn_s_neg_cubes_amount + ismda_1.dyn_s_neg_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e547 = i_42;
            uint j_38 = ish_2[_e547];
            Shape shape_37 = _group_0_binding_8_fs[j_38];
            float _e553 = ddd;
            float _e557 = sd_box((p_15 - shape_37.pos), shape_37.size);
            float _e562 = _group_0_binding_4_fs.stickiness;
            float _e563 = smin(_e553, (_e557 - shape_37.roundness), _e562);
            ddd = _e563;
        }
    }
    i_43 = ismda_1.dyn_s_neg_spheres_start;
    bool loop_init_48 = true;
    while(true) {
        if (!loop_init_48) {
            uint _e593 = i_43;
            i_43 = (_e593 + 1u);
        }
        loop_init_48 = false;
        uint _e569 = i_43;
        if ((_e569 < (ismda_1.dyn_s_neg_spheres_amount + ismda_1.dyn_s_neg_spheres_start))) {
        } else {
            break;
        }
        {
            uint _e574 = i_43;
            uint j_39 = ish_2[_e574];
            Shape shape_38 = _group_0_binding_8_fs[j_39];
            float _e580 = ddd;
            float _e585 = sd_sphere((p_15 - shape_38.pos), shape_38.size.x);
            float _e590 = _group_0_binding_4_fs.stickiness;
            float _e591 = smin(_e580, (_e585 - shape_38.roundness), _e590);
            ddd = _e591;
        }
    }
    i_44 = ismda_1.dyn_s_neg_sph_cubes_start;
    bool loop_init_49 = true;
    while(true) {
        if (!loop_init_49) {
            uint _e620 = i_44;
            i_44 = (_e620 + 1u);
        }
        loop_init_49 = false;
        uint _e597 = i_44;
        if ((_e597 < (ismda_1.dyn_s_neg_sph_cubes_amount + ismda_1.dyn_s_neg_sph_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e602 = i_44;
            uint j_40 = ish_2[_e602];
            Shape shape_39 = _group_0_binding_8_fs[j_40];
            float _e608 = ddd;
            float _e612 = sd_sph_box((p_15 - shape_39.pos), shape_39.size);
            float _e617 = _group_0_binding_4_fs.stickiness;
            float _e618 = smin(_e608, (_e612 - shape_39.roundness), _e617);
            ddd = _e618;
        }
    }
    i_45 = ismda_1.dyn_s_neg_inf_cubes_start;
    bool loop_init_50 = true;
    while(true) {
        if (!loop_init_50) {
            uint _e648 = i_45;
            i_45 = (_e648 + 1u);
        }
        loop_init_50 = false;
        uint _e624 = i_45;
        if ((_e624 < (ismda_1.dyn_s_neg_inf_cubes_amount + ismda_1.dyn_s_neg_inf_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e629 = i_45;
            uint j_41 = ish_2[_e629];
            Shape shape_40 = _group_0_binding_8_fs[j_41];
            float _e635 = ddd;
            float _e640 = sd_inf_box((p_15 - shape_40.pos), shape_40.size.xyz);
            float _e645 = _group_0_binding_4_fs.stickiness;
            float _e646 = smin(_e635, (_e640 - shape_40.roundness), _e645);
            ddd = _e646;
        }
    }
    float _e650 = d_8;
    float _e651 = ddd;
    d_8 = max(_e650, -(_e651));
    i_46 = ismda_1.st_neg_cubes_start;
    bool loop_init_51 = true;
    while(true) {
        if (!loop_init_51) {
            uint _e677 = i_46;
            i_46 = (_e677 + 1u);
        }
        loop_init_51 = false;
        uint _e656 = i_46;
        if ((_e656 < (ismda_1.st_neg_cubes_amount + ismda_1.st_neg_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e661 = i_46;
            uint j_42 = ish_2[_e661];
            Shape shape_41 = _group_0_binding_1_fs[j_42];
            float _e667 = d_8;
            float _e671 = sd_box((p_15 - shape_41.pos), shape_41.size);
            d_8 = max(_e667, -((_e671 - shape_41.roundness)));
        }
    }
    i_47 = ismda_1.st_neg_spheres_start;
    bool loop_init_52 = true;
    while(true) {
        if (!loop_init_52) {
            uint _e703 = i_47;
            i_47 = (_e703 + 1u);
        }
        loop_init_52 = false;
        uint _e681 = i_47;
        if ((_e681 < (ismda_1.st_neg_spheres_amount + ismda_1.st_neg_spheres_start))) {
        } else {
            break;
        }
        {
            uint _e686 = i_47;
            uint j_43 = ish_2[_e686];
            Shape shape_42 = _group_0_binding_1_fs[j_43];
            float _e692 = d_8;
            float _e697 = sd_sphere((p_15 - shape_42.pos), shape_42.size.x);
            d_8 = max(_e692, -((_e697 - shape_42.roundness)));
        }
    }
    i_48 = ismda_1.st_neg_sph_cubes_start;
    bool loop_init_53 = true;
    while(true) {
        if (!loop_init_53) {
            uint _e728 = i_48;
            i_48 = (_e728 + 1u);
        }
        loop_init_53 = false;
        uint _e707 = i_48;
        if ((_e707 < (ismda_1.st_neg_sph_cubes_amount + ismda_1.st_neg_sph_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e712 = i_48;
            uint j_44 = ish_2[_e712];
            Shape shape_43 = _group_0_binding_1_fs[j_44];
            float _e718 = d_8;
            float _e722 = sd_sph_box((p_15 - shape_43.pos), shape_43.size);
            d_8 = max(_e718, -((_e722 - shape_43.roundness)));
        }
    }
    i_49 = ismda_1.st_neg_inf_cubes_start;
    bool loop_init_54 = true;
    while(true) {
        if (!loop_init_54) {
            uint _e754 = i_49;
            i_49 = (_e754 + 1u);
        }
        loop_init_54 = false;
        uint _e732 = i_49;
        if ((_e732 < (ismda_1.st_neg_inf_cubes_amount + ismda_1.st_neg_inf_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e737 = i_49;
            uint j_45 = ish_2[_e737];
            Shape shape_44 = _group_0_binding_1_fs[j_45];
            float _e743 = d_8;
            float _e748 = sd_inf_box((p_15 - shape_44.pos), shape_44.size.xyz);
            d_8 = max(_e743, -((_e748 - shape_44.roundness)));
        }
    }
    i_50 = ismda_1.dyn_neg_cubes_start;
    bool loop_init_55 = true;
    while(true) {
        if (!loop_init_55) {
            uint _e779 = i_50;
            i_50 = (_e779 + 1u);
        }
        loop_init_55 = false;
        uint _e758 = i_50;
        if ((_e758 < (ismda_1.dyn_neg_cubes_amount + ismda_1.dyn_neg_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e763 = i_50;
            uint j_46 = ish_2[_e763];
            Shape shape_45 = _group_0_binding_6_fs[j_46];
            float _e769 = d_8;
            float _e773 = sd_box((p_15 - shape_45.pos), shape_45.size);
            d_8 = max(_e769, -((_e773 - shape_45.roundness)));
        }
    }
    i_51 = ismda_1.dyn_neg_spheres_start;
    bool loop_init_56 = true;
    while(true) {
        if (!loop_init_56) {
            uint _e805 = i_51;
            i_51 = (_e805 + 1u);
        }
        loop_init_56 = false;
        uint _e783 = i_51;
        if ((_e783 < (ismda_1.dyn_neg_spheres_amount + ismda_1.dyn_neg_spheres_start))) {
        } else {
            break;
        }
        {
            uint _e788 = i_51;
            uint j_47 = ish_2[_e788];
            Shape shape_46 = _group_0_binding_6_fs[j_47];
            float _e794 = d_8;
            float _e799 = sd_sphere((p_15 - shape_46.pos), shape_46.size.x);
            d_8 = max(_e794, -((_e799 - shape_46.roundness)));
        }
    }
    i_52 = ismda_1.dyn_neg_sph_cubes_start;
    bool loop_init_57 = true;
    while(true) {
        if (!loop_init_57) {
            uint _e830 = i_52;
            i_52 = (_e830 + 1u);
        }
        loop_init_57 = false;
        uint _e809 = i_52;
        if ((_e809 < (ismda_1.dyn_neg_sph_cubes_amount + ismda_1.dyn_neg_sph_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e814 = i_52;
            uint j_48 = ish_2[_e814];
            Shape shape_47 = _group_0_binding_6_fs[j_48];
            float _e820 = d_8;
            float _e824 = sd_sph_box((p_15 - shape_47.pos), shape_47.size);
            d_8 = max(_e820, -((_e824 - shape_47.roundness)));
        }
    }
    i_53 = ismda_1.dyn_neg_inf_cubes_start;
    bool loop_init_58 = true;
    while(true) {
        if (!loop_init_58) {
            uint _e856 = i_53;
            i_53 = (_e856 + 1u);
        }
        loop_init_58 = false;
        uint _e834 = i_53;
        if ((_e834 < (ismda_1.dyn_neg_inf_cubes_amount + ismda_1.dyn_neg_inf_cubes_start))) {
        } else {
            break;
        }
        {
            uint _e839 = i_53;
            uint j_49 = ish_2[_e839];
            Shape shape_48 = _group_0_binding_6_fs[j_49];
            float _e845 = d_8;
            float _e850 = sd_inf_box((p_15 - shape_48.pos), shape_48.size.xyz);
            d_8 = max(_e845, -((_e850 - shape_48.roundness)));
        }
    }
    j_1 = ismda_1.player_forms_start;
    bool loop_init_59 = true;
    while(true) {
        if (!loop_init_59) {
            uint _e1006 = j_1;
            j_1 = (_e1006 + 1u);
        }
        loop_init_59 = false;
        uint _e862 = j_1;
        if ((_e862 < (ismda_1.player_forms_amount + ismda_1.player_forms_start))) {
        } else {
            break;
        }
        {
            uint _e867 = j_1;
            uint i_89 = ish_2[_e867];
            PlayerForm shape_49 = _group_1_binding_2_fs[i_89];
            float _e873 = dddd;
            float _e877 = sd_sphere((p_15 - shape_49.pos), shape_49.radius);
            dddd = min(_e873, _e877);
            float _e879 = dddd;
            float _e885 = sd_sphere((p_15 - shape_49.pos), (shape_49.radius * 0.86));
            dddd = max(_e879, -(_e885));
            vec4 rotated_p_1 = (shape_49.rotation * (p_15 - shape_49.pos));
            float _e892 = dddd;
            float _e906 = sd_box(rotated_p_1, vec4((shape_49.radius * 0.18), (shape_49.radius * 1.2), (shape_49.radius * 1.2), (shape_49.radius * 1.2)));
            dddd = max(_e892, -(_e906));
            float _e909 = dddd;
            float _e920 = sd_sphere((rotated_p_1 - vec4(0.0, 0.0, -(shape_49.radius), 0.0)), (shape_49.radius * 0.53));
            dddd = max(_e909, -(_e920));
            float _e923 = dddd;
            float _e929 = sd_sphere((p_15 - shape_49.pos), (shape_49.radius * 0.6));
            dddd = min(_e923, _e929);
            float _e931 = dddd;
            float _e944 = sd_sphere((rotated_p_1 - (vec4(0.0, 0.0, -(shape_49.radius), 0.0) * 0.6)), (shape_49.radius * 0.34));
            dddd = max(_e931, -(_e944));
            float _e947 = dddd;
            float _e953 = sd_sphere((rotated_p_1 - shape_49.weapon_offset), (shape_49.radius * 0.286));
            dddd = min(_e947, _e953);
            float _e955 = dddd;
            float _e969 = sd_capsule(rotated_p_1, shape_49.weapon_offset, (shape_49.weapon_offset - vec4(0.0, 0.0, (shape_49.radius * 0.49), 0.0)), (shape_49.radius * 0.18));
            dddd = max(_e955, -(_e969));
            float _e972 = dddd;
            float _e986 = sd_capsule(rotated_p_1, shape_49.weapon_offset, (shape_49.weapon_offset - vec4(0.0, 0.0, (shape_49.radius * 0.43), 0.0)), (shape_49.radius * 0.1));
            dddd = min(_e972, _e986);
            float _e988 = dddd;
            float _e1002 = sd_capsule(rotated_p_1, shape_49.weapon_offset, (shape_49.weapon_offset - vec4(0.0, 0.0, (shape_49.radius * 0.65), 0.0)), (shape_49.radius * 0.052));
            dddd = max(_e988, -(_e1002));
        }
    }
    float _e1008 = d_8;
    float _e1009 = dddd;
    d_8 = min(_e1008, _e1009);
    int _e1013 = _group_0_binding_4_fs.is_w_floor_exist;
    if ((_e1013 == 1)) {
        bool _e1017 = ray_w_rotated;
        if (_e1017) {
            float _e1018 = d_8;
            float _e1022 = _group_0_binding_4_fs.w_floor;
            d_8 = min(_e1018, (p_15.w - _e1022));
        }
    }
    float _e1025 = d_8;
    return _e1025;
}

vec4 get_normal(vec4 p_16, inout IntersectedShapesMetadata ismd_3, inout uint ish_3[16]) {
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
    vec3 _e8 = h_2;
    a_4 = (p_16 + _e8.yxxz);
    vec3 _e12 = h_2;
    b_9 = (p_16 + _e12.xyxz);
    vec3 _e16 = h_2;
    c_4 = (p_16 + _e16.xxyz);
    vec3 _e20 = h_2;
    d_9 = (p_16 + _e20.yyyz);
    vec3 _e24 = h_2;
    e_2 = (p_16 + _e24.zzzx);
    vec3 _e28 = h_2;
    f_2 = (p_16 + _e28.zzzy);
    vec4 _e32 = a_4;
    float _e33 = map(_e32, ismd_3, ish_3);
    fa_2 = _e33;
    vec4 _e35 = b_9;
    float _e36 = map(_e35, ismd_3, ish_3);
    fb_2 = _e36;
    vec4 _e38 = c_4;
    float _e39 = map(_e38, ismd_3, ish_3);
    fc_2 = _e39;
    vec4 _e41 = d_9;
    float _e42 = map(_e41, ismd_3, ish_3);
    fd_2 = _e42;
    vec4 _e44 = e_2;
    float _e45 = map(_e44, ismd_3, ish_3);
    fe_2 = _e45;
    vec4 _e47 = f_2;
    float _e48 = map(_e47, ismd_3, ish_3);
    ff_2 = _e48;
    vec3 _e50 = h_2;
    float _e52 = fa_2;
    vec3 _e54 = h_2;
    float _e56 = fb_2;
    vec3 _e59 = h_2;
    float _e61 = fc_2;
    vec3 _e64 = h_2;
    float _e66 = fd_2;
    vec3 _e69 = h_2;
    float _e71 = fe_2;
    vec3 _e74 = h_2;
    float _e76 = ff_2;
    return normalize(((((((_e50.yxxz * _e52) + (_e54.xyxz * _e56)) + (_e59.xxyz * _e61)) + (_e64.yyyz * _e66)) + (_e69.zzzx * _e71)) + (_e74.zzzy * _e76)));
}

vec2 ray_march(vec4 ray_origin_base, vec4 ray_direction_1, float offset, inout IntersectedShapesMetadata ismd_4, inout uint ish_4[16]) {
    float total_distance = 0.0;
    vec4 ray_origin = vec4(0.0);
    int i_54 = 0;
    float d_10 = 0.0;
    if ((offset > MAX_DIST)) {
        return vec2(700.0, 0.0);
    }
    total_distance = offset;
    ray_origin = (ray_origin_base + (ray_direction_1 * offset));
    bool loop_init_60 = true;
    while(true) {
        if (!loop_init_60) {
            int _e51 = i_54;
            i_54 = (_e51 + 1);
        }
        loop_init_60 = false;
        int _e16 = i_54;
        if ((_e16 < MAX_STEPS)) {
        } else {
            break;
        }
        {
            vec4 _e19 = ray_origin;
            float _e20 = map(_e19, ismd_4, ish_4);
            d_10 = _e20;
            float _e22 = d_10;
            float _e23 = total_distance;
            total_distance = (_e23 + _e22);
            float _e25 = d_10;
            if ((_e25 < 0.0)) {
                float _e28 = total_distance;
                int _e29 = i_54;
                return vec2(_e28, float(_e29));
            }
            float _e32 = d_10;
            if ((_e32 < MIN_DIST)) {
                float _e35 = total_distance;
                int _e36 = i_54;
                return vec2(_e35, float(_e36));
            }
            float _e39 = total_distance;
            if ((_e39 > MAX_DIST)) {
                int _e43 = i_54;
                return vec2(700.0, float(_e43));
            }
            float _e46 = d_10;
            vec4 _e48 = ray_origin;
            ray_origin = (_e48 + (ray_direction_1 * _e46));
        }
    }
    float _e53 = total_distance;
    int _e54 = i_54;
    return vec2(_e53, float(_e54));
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

float find_intersections(vec4 ro_3, vec4 rd_3, inout IntersectedShapesMetadata ismd_5, inout uint ish_5[16]) {
    float offset_1 = 700.0;
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
    ismd_5.st_s_cubes_start = _e9;
    uint _e13 = _group_0_binding_4_fs.shapes_arrays_metadata.s_cubes_start;
    i_56 = _e13;
    bool loop_init_62 = true;
    while(true) {
        if (!loop_init_62) {
            uint _e69 = i_56;
            i_56 = (_e69 + 1u);
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
                uint _e57 = ismd_5.st_s_cubes_amount;
                ismd_5.st_s_cubes_amount = (_e57 + 1u);
                uint _e59 = ish_index;
                uint _e61 = i_56;
                ish_5[_e59] = _e61;
                uint _e63 = ish_index;
                ish_index = (_e63 + 1u);
                float _e65 = offset_1;
                offset_1 = min(_e65, _e51.x);
            }
        }
    }
    uint _e72 = ish_index;
    ismd_5.st_s_spheres_start = _e72;
    uint _e76 = _group_0_binding_4_fs.shapes_arrays_metadata.s_spheres_start;
    i_57 = _e76;
    bool loop_init_63 = true;
    while(true) {
        if (!loop_init_63) {
            uint _e131 = i_57;
            i_57 = (_e131 + 1u);
        }
        loop_init_63 = false;
        uint _e78 = i_57;
        uint _e82 = _group_0_binding_4_fs.shapes_arrays_metadata.s_spheres_amount;
        uint _e86 = _group_0_binding_4_fs.shapes_arrays_metadata.s_spheres_start;
        if ((_e78 < (_e82 + _e86))) {
        } else {
            break;
        }
        {
            uint _e90 = i_57;
            vec4 _e93 = _group_0_binding_2_fs[_e90].pos;
            uint _e96 = i_57;
            float _e100 = _group_0_binding_2_fs[_e96].size.x;
            uint _e102 = i_57;
            float _e105 = _group_0_binding_2_fs[_e102].roundness;
            float _e109 = _group_0_binding_4_fs.stickiness;
            vec2 _e113 = sph_intersection((ro_3 - _e93), rd_3, ((_e100 + _e105) + (_e109 * STICKINESS_EFFECT_COEF)));
            if ((_e113.y > 0.0)) {
                uint _e119 = ismd_5.st_s_spheres_amount;
                ismd_5.st_s_spheres_amount = (_e119 + 1u);
                uint _e121 = ish_index;
                uint _e123 = i_57;
                ish_5[_e121] = _e123;
                uint _e125 = ish_index;
                ish_index = (_e125 + 1u);
                float _e127 = offset_1;
                offset_1 = min(_e127, _e113.x);
            }
        }
    }
    uint _e134 = ish_index;
    ismd_5.st_s_sph_cubes_start = _e134;
    uint _e138 = _group_0_binding_4_fs.shapes_arrays_metadata.s_sph_cubes_start;
    i_58 = _e138;
    bool loop_init_64 = true;
    while(true) {
        if (!loop_init_64) {
            uint _e194 = i_58;
            i_58 = (_e194 + 1u);
        }
        loop_init_64 = false;
        uint _e140 = i_58;
        uint _e144 = _group_0_binding_4_fs.shapes_arrays_metadata.s_sph_cubes_amount;
        uint _e148 = _group_0_binding_4_fs.shapes_arrays_metadata.s_sph_cubes_start;
        if ((_e140 < (_e144 + _e148))) {
        } else {
            break;
        }
        {
            uint _e152 = i_58;
            vec4 _e155 = _group_0_binding_2_fs[_e152].pos;
            uint _e158 = i_58;
            vec4 _e161 = _group_0_binding_2_fs[_e158].size;
            uint _e163 = i_58;
            float _e166 = _group_0_binding_2_fs[_e163].roundness;
            float _e171 = _group_0_binding_4_fs.stickiness;
            vec2 _e176 = cube_intersection((ro_3 - _e155), rd_3, ((_e161 + vec4(_e166)) + vec4((_e171 * STICKINESS_EFFECT_COEF))));
            if ((_e176.y > 0.0)) {
                uint _e182 = ismd_5.st_s_sph_cubes_amount;
                ismd_5.st_s_sph_cubes_amount = (_e182 + 1u);
                uint _e184 = ish_index;
                uint _e186 = i_58;
                ish_5[_e184] = _e186;
                uint _e188 = ish_index;
                ish_index = (_e188 + 1u);
                float _e190 = offset_1;
                offset_1 = min(_e190, _e176.x);
            }
        }
    }
    uint _e197 = ish_index;
    ismd_5.st_s_inf_cubes_start = _e197;
    uint _e201 = _group_0_binding_4_fs.shapes_arrays_metadata.s_inf_cubes_start;
    i_59 = _e201;
    bool loop_init_65 = true;
    while(true) {
        if (!loop_init_65) {
            uint _e258 = i_59;
            i_59 = (_e258 + 1u);
        }
        loop_init_65 = false;
        uint _e203 = i_59;
        uint _e207 = _group_0_binding_4_fs.shapes_arrays_metadata.s_inf_cubes_amount;
        uint _e211 = _group_0_binding_4_fs.shapes_arrays_metadata.s_inf_cubes_start;
        if ((_e203 < (_e207 + _e211))) {
        } else {
            break;
        }
        {
            uint _e215 = i_59;
            vec4 _e218 = _group_0_binding_2_fs[_e215].pos;
            uint _e221 = i_59;
            vec4 _e224 = _group_0_binding_2_fs[_e221].size;
            uint _e227 = i_59;
            float _e230 = _group_0_binding_2_fs[_e227].roundness;
            float _e235 = _group_0_binding_4_fs.stickiness;
            vec2 _e240 = inf_cube_intersection((ro_3 - _e218), rd_3, ((_e224.xyz + vec3(_e230)) + vec3((_e235 * STICKINESS_EFFECT_COEF))));
            if ((_e240.y > 0.0)) {
                uint _e246 = ismd_5.st_s_inf_cubes_amount;
                ismd_5.st_s_inf_cubes_amount = (_e246 + 1u);
                uint _e248 = ish_index;
                uint _e250 = i_59;
                ish_5[_e248] = _e250;
                uint _e252 = ish_index;
                ish_index = (_e252 + 1u);
                float _e254 = offset_1;
                offset_1 = min(_e254, _e240.x);
            }
        }
    }
    uint _e261 = ish_index;
    ismd_5.dyn_s_cubes_start = _e261;
    uint _e265 = _group_0_binding_9_fs.shapes_arrays_metadata.s_cubes_start;
    i_60 = _e265;
    bool loop_init_66 = true;
    while(true) {
        if (!loop_init_66) {
            uint _e321 = i_60;
            i_60 = (_e321 + 1u);
        }
        loop_init_66 = false;
        uint _e267 = i_60;
        uint _e271 = _group_0_binding_9_fs.shapes_arrays_metadata.s_cubes_amount;
        uint _e275 = _group_0_binding_9_fs.shapes_arrays_metadata.s_cubes_start;
        if ((_e267 < (_e271 + _e275))) {
        } else {
            break;
        }
        {
            uint _e279 = i_60;
            vec4 _e282 = _group_0_binding_7_fs[_e279].pos;
            uint _e285 = i_60;
            vec4 _e288 = _group_0_binding_7_fs[_e285].size;
            uint _e290 = i_60;
            float _e293 = _group_0_binding_7_fs[_e290].roundness;
            float _e298 = _group_0_binding_4_fs.stickiness;
            vec2 _e303 = cube_intersection((ro_3 - _e282), rd_3, ((_e288 + vec4(_e293)) + vec4((_e298 * STICKINESS_EFFECT_COEF))));
            if ((_e303.y > 0.0)) {
                uint _e309 = ismd_5.dyn_s_cubes_amount;
                ismd_5.dyn_s_cubes_amount = (_e309 + 1u);
                uint _e311 = ish_index;
                uint _e313 = i_60;
                ish_5[_e311] = _e313;
                uint _e315 = ish_index;
                ish_index = (_e315 + 1u);
                float _e317 = offset_1;
                offset_1 = min(_e317, _e303.x);
            }
        }
    }
    uint _e324 = ish_index;
    ismd_5.dyn_s_spheres_start = _e324;
    uint _e328 = _group_0_binding_9_fs.shapes_arrays_metadata.s_spheres_start;
    i_61 = _e328;
    bool loop_init_67 = true;
    while(true) {
        if (!loop_init_67) {
            uint _e383 = i_61;
            i_61 = (_e383 + 1u);
        }
        loop_init_67 = false;
        uint _e330 = i_61;
        uint _e334 = _group_0_binding_9_fs.shapes_arrays_metadata.s_spheres_amount;
        uint _e338 = _group_0_binding_9_fs.shapes_arrays_metadata.s_spheres_start;
        if ((_e330 < (_e334 + _e338))) {
        } else {
            break;
        }
        {
            uint _e342 = i_61;
            vec4 _e345 = _group_0_binding_7_fs[_e342].pos;
            uint _e348 = i_61;
            float _e352 = _group_0_binding_7_fs[_e348].size.x;
            uint _e354 = i_61;
            float _e357 = _group_0_binding_7_fs[_e354].roundness;
            float _e361 = _group_0_binding_4_fs.stickiness;
            vec2 _e365 = sph_intersection((ro_3 - _e345), rd_3, ((_e352 + _e357) + (_e361 * STICKINESS_EFFECT_COEF)));
            if ((_e365.y > 0.0)) {
                uint _e371 = ismd_5.dyn_s_spheres_amount;
                ismd_5.dyn_s_spheres_amount = (_e371 + 1u);
                uint _e373 = ish_index;
                uint _e375 = i_61;
                ish_5[_e373] = _e375;
                uint _e377 = ish_index;
                ish_index = (_e377 + 1u);
                float _e379 = offset_1;
                offset_1 = min(_e379, _e365.x);
            }
        }
    }
    uint _e386 = ish_index;
    ismd_5.dyn_s_sph_cubes_start = _e386;
    uint _e390 = _group_0_binding_9_fs.shapes_arrays_metadata.s_sph_cubes_start;
    i_62 = _e390;
    bool loop_init_68 = true;
    while(true) {
        if (!loop_init_68) {
            uint _e446 = i_62;
            i_62 = (_e446 + 1u);
        }
        loop_init_68 = false;
        uint _e392 = i_62;
        uint _e396 = _group_0_binding_9_fs.shapes_arrays_metadata.s_sph_cubes_amount;
        uint _e400 = _group_0_binding_9_fs.shapes_arrays_metadata.s_sph_cubes_start;
        if ((_e392 < (_e396 + _e400))) {
        } else {
            break;
        }
        {
            uint _e404 = i_62;
            vec4 _e407 = _group_0_binding_7_fs[_e404].pos;
            uint _e410 = i_62;
            vec4 _e413 = _group_0_binding_7_fs[_e410].size;
            uint _e415 = i_62;
            float _e418 = _group_0_binding_7_fs[_e415].roundness;
            float _e423 = _group_0_binding_4_fs.stickiness;
            vec2 _e428 = cube_intersection((ro_3 - _e407), rd_3, ((_e413 + vec4(_e418)) + vec4((_e423 * STICKINESS_EFFECT_COEF))));
            if ((_e428.y > 0.0)) {
                uint _e434 = ismd_5.dyn_s_sph_cubes_amount;
                ismd_5.dyn_s_sph_cubes_amount = (_e434 + 1u);
                uint _e436 = ish_index;
                uint _e438 = i_62;
                ish_5[_e436] = _e438;
                uint _e440 = ish_index;
                ish_index = (_e440 + 1u);
                float _e442 = offset_1;
                offset_1 = min(_e442, _e428.x);
            }
        }
    }
    uint _e449 = ish_index;
    ismd_5.dyn_s_inf_cubes_start = _e449;
    uint _e453 = _group_0_binding_9_fs.shapes_arrays_metadata.s_inf_cubes_start;
    i_63 = _e453;
    bool loop_init_69 = true;
    while(true) {
        if (!loop_init_69) {
            uint _e513 = i_63;
            i_63 = (_e513 + 1u);
        }
        loop_init_69 = false;
        uint _e455 = i_63;
        uint _e459 = _group_0_binding_9_fs.shapes_arrays_metadata.s_inf_cubes_amount;
        uint _e463 = _group_0_binding_9_fs.shapes_arrays_metadata.s_inf_cubes_start;
        if ((_e455 < (_e459 + _e463))) {
        } else {
            break;
        }
        {
            uint _e467 = i_63;
            vec4 _e470 = _group_0_binding_7_fs[_e467].pos;
            uint _e473 = i_63;
            vec4 _e476 = _group_0_binding_7_fs[_e473].size;
            uint _e479 = i_63;
            float _e482 = _group_0_binding_7_fs[_e479].roundness;
            float _e487 = _group_0_binding_4_fs.stickiness;
            vec2 _e492 = inf_cube_intersection((ro_3 - _e470), rd_3, ((_e476.xyz + vec3(_e482)) + vec3((_e487 * STICKINESS_EFFECT_COEF))));
            if ((_e492.y > 0.0)) {
                uint _e498 = ismd_5.dyn_s_inf_cubes_amount;
                ismd_5.dyn_s_inf_cubes_amount = (_e498 + 1u);
                uint _e500 = ish_index;
                uint _e502 = i_63;
                ish_5[_e500] = _e502;
                uint _e504 = ish_index;
                ish_index = (_e504 + 1u);
                if ((_e492.x >= 0.0)) {
                    float _e509 = offset_1;
                    offset_1 = min(_e509, _e492.x);
                }
            }
        }
    }
    uint _e516 = ish_index;
    ismd_5.st_cubes_start = _e516;
    uint _e520 = _group_0_binding_4_fs.shapes_arrays_metadata.cubes_start;
    i_64 = _e520;
    bool loop_init_70 = true;
    while(true) {
        if (!loop_init_70) {
            uint _e569 = i_64;
            i_64 = (_e569 + 1u);
        }
        loop_init_70 = false;
        uint _e522 = i_64;
        uint _e526 = _group_0_binding_4_fs.shapes_arrays_metadata.cubes_amount;
        uint _e530 = _group_0_binding_4_fs.shapes_arrays_metadata.cubes_start;
        if ((_e522 < (_e526 + _e530))) {
        } else {
            break;
        }
        {
            uint _e534 = i_64;
            vec4 _e537 = _group_0_binding_0_fs[_e534].pos;
            uint _e540 = i_64;
            vec4 _e543 = _group_0_binding_0_fs[_e540].size;
            uint _e545 = i_64;
            float _e548 = _group_0_binding_0_fs[_e545].roundness;
            vec2 _e551 = cube_intersection((ro_3 - _e537), rd_3, (_e543 + vec4(_e548)));
            if ((_e551.y > 0.0)) {
                uint _e557 = ismd_5.st_cubes_amount;
                ismd_5.st_cubes_amount = (_e557 + 1u);
                uint _e559 = ish_index;
                uint _e561 = i_64;
                ish_5[_e559] = _e561;
                uint _e563 = ish_index;
                ish_index = (_e563 + 1u);
                float _e565 = offset_1;
                offset_1 = min(_e565, _e551.x);
            }
        }
    }
    uint _e572 = ish_index;
    ismd_5.st_spheres_start = _e572;
    uint _e576 = _group_0_binding_4_fs.shapes_arrays_metadata.spheres_start;
    i_65 = _e576;
    bool loop_init_71 = true;
    while(true) {
        if (!loop_init_71) {
            uint _e625 = i_65;
            i_65 = (_e625 + 1u);
        }
        loop_init_71 = false;
        uint _e578 = i_65;
        uint _e582 = _group_0_binding_4_fs.shapes_arrays_metadata.spheres_amount;
        uint _e586 = _group_0_binding_4_fs.shapes_arrays_metadata.spheres_start;
        if ((_e578 < (_e582 + _e586))) {
        } else {
            break;
        }
        {
            uint _e590 = i_65;
            vec4 _e593 = _group_0_binding_0_fs[_e590].pos;
            uint _e596 = i_65;
            float _e600 = _group_0_binding_0_fs[_e596].size.x;
            uint _e602 = i_65;
            float _e605 = _group_0_binding_0_fs[_e602].roundness;
            vec2 _e607 = sph_intersection((ro_3 - _e593), rd_3, (_e600 + _e605));
            if ((_e607.y > 0.0)) {
                uint _e613 = ismd_5.st_spheres_amount;
                ismd_5.st_spheres_amount = (_e613 + 1u);
                uint _e615 = ish_index;
                uint _e617 = i_65;
                ish_5[_e615] = _e617;
                uint _e619 = ish_index;
                ish_index = (_e619 + 1u);
                float _e621 = offset_1;
                offset_1 = min(_e621, _e607.x);
            }
        }
    }
    uint _e628 = ish_index;
    ismd_5.st_sph_cubes_start = _e628;
    uint _e632 = _group_0_binding_4_fs.shapes_arrays_metadata.sph_cubes_start;
    i_66 = _e632;
    bool loop_init_72 = true;
    while(true) {
        if (!loop_init_72) {
            uint _e681 = i_66;
            i_66 = (_e681 + 1u);
        }
        loop_init_72 = false;
        uint _e634 = i_66;
        uint _e638 = _group_0_binding_4_fs.shapes_arrays_metadata.sph_cubes_amount;
        uint _e642 = _group_0_binding_4_fs.shapes_arrays_metadata.sph_cubes_start;
        if ((_e634 < (_e638 + _e642))) {
        } else {
            break;
        }
        {
            uint _e646 = i_66;
            vec4 _e649 = _group_0_binding_0_fs[_e646].pos;
            uint _e652 = i_66;
            vec4 _e655 = _group_0_binding_0_fs[_e652].size;
            uint _e657 = i_66;
            float _e660 = _group_0_binding_0_fs[_e657].roundness;
            vec2 _e663 = cube_intersection((ro_3 - _e649), rd_3, (_e655 + vec4(_e660)));
            if ((_e663.y > 0.0)) {
                uint _e669 = ismd_5.st_sph_cubes_amount;
                ismd_5.st_sph_cubes_amount = (_e669 + 1u);
                uint _e671 = ish_index;
                uint _e673 = i_66;
                ish_5[_e671] = _e673;
                uint _e675 = ish_index;
                ish_index = (_e675 + 1u);
                float _e677 = offset_1;
                offset_1 = min(_e677, _e663.x);
            }
        }
    }
    uint _e684 = ish_index;
    ismd_5.st_inf_cubes_start = _e684;
    uint _e688 = _group_0_binding_4_fs.shapes_arrays_metadata.inf_cubes_start;
    i_67 = _e688;
    bool loop_init_73 = true;
    while(true) {
        if (!loop_init_73) {
            uint _e738 = i_67;
            i_67 = (_e738 + 1u);
        }
        loop_init_73 = false;
        uint _e690 = i_67;
        uint _e694 = _group_0_binding_4_fs.shapes_arrays_metadata.inf_cubes_amount;
        uint _e698 = _group_0_binding_4_fs.shapes_arrays_metadata.inf_cubes_start;
        if ((_e690 < (_e694 + _e698))) {
        } else {
            break;
        }
        {
            uint _e702 = i_67;
            vec4 _e705 = _group_0_binding_0_fs[_e702].pos;
            uint _e708 = i_67;
            vec4 _e711 = _group_0_binding_0_fs[_e708].size;
            uint _e714 = i_67;
            float _e717 = _group_0_binding_0_fs[_e714].roundness;
            vec2 _e720 = inf_cube_intersection((ro_3 - _e705), rd_3, (_e711.xyz + vec3(_e717)));
            if ((_e720.y > 0.0)) {
                uint _e726 = ismd_5.st_inf_cubes_amount;
                ismd_5.st_inf_cubes_amount = (_e726 + 1u);
                uint _e728 = ish_index;
                uint _e730 = i_67;
                ish_5[_e728] = _e730;
                uint _e732 = ish_index;
                ish_index = (_e732 + 1u);
                float _e734 = offset_1;
                offset_1 = min(_e734, _e720.x);
            }
        }
    }
    uint _e741 = ish_index;
    ismd_5.dyn_cubes_start = _e741;
    uint _e745 = _group_0_binding_9_fs.shapes_arrays_metadata.cubes_start;
    i_68 = _e745;
    bool loop_init_74 = true;
    while(true) {
        if (!loop_init_74) {
            uint _e794 = i_68;
            i_68 = (_e794 + 1u);
        }
        loop_init_74 = false;
        uint _e747 = i_68;
        uint _e751 = _group_0_binding_9_fs.shapes_arrays_metadata.cubes_amount;
        uint _e755 = _group_0_binding_9_fs.shapes_arrays_metadata.cubes_start;
        if ((_e747 < (_e751 + _e755))) {
        } else {
            break;
        }
        {
            uint _e759 = i_68;
            vec4 _e762 = _group_0_binding_5_fs[_e759].pos;
            uint _e765 = i_68;
            vec4 _e768 = _group_0_binding_5_fs[_e765].size;
            uint _e770 = i_68;
            float _e773 = _group_0_binding_5_fs[_e770].roundness;
            vec2 _e776 = cube_intersection((ro_3 - _e762), rd_3, (_e768 + vec4(_e773)));
            if ((_e776.y > 0.0)) {
                uint _e782 = ismd_5.dyn_cubes_amount;
                ismd_5.dyn_cubes_amount = (_e782 + 1u);
                uint _e784 = ish_index;
                uint _e786 = i_68;
                ish_5[_e784] = _e786;
                uint _e788 = ish_index;
                ish_index = (_e788 + 1u);
                float _e790 = offset_1;
                offset_1 = min(_e790, _e776.x);
            }
        }
    }
    uint _e797 = ish_index;
    ismd_5.dyn_spheres_start = _e797;
    uint _e801 = _group_0_binding_9_fs.shapes_arrays_metadata.spheres_start;
    i_69 = _e801;
    bool loop_init_75 = true;
    while(true) {
        if (!loop_init_75) {
            uint _e850 = i_69;
            i_69 = (_e850 + 1u);
        }
        loop_init_75 = false;
        uint _e803 = i_69;
        uint _e807 = _group_0_binding_9_fs.shapes_arrays_metadata.spheres_amount;
        uint _e811 = _group_0_binding_9_fs.shapes_arrays_metadata.spheres_start;
        if ((_e803 < (_e807 + _e811))) {
        } else {
            break;
        }
        {
            uint _e815 = i_69;
            vec4 _e818 = _group_0_binding_5_fs[_e815].pos;
            uint _e821 = i_69;
            float _e825 = _group_0_binding_5_fs[_e821].size.x;
            uint _e827 = i_69;
            float _e830 = _group_0_binding_5_fs[_e827].roundness;
            vec2 _e832 = sph_intersection((ro_3 - _e818), rd_3, (_e825 + _e830));
            if ((_e832.y > 0.0)) {
                uint _e838 = ismd_5.dyn_spheres_amount;
                ismd_5.dyn_spheres_amount = (_e838 + 1u);
                uint _e840 = ish_index;
                uint _e842 = i_69;
                ish_5[_e840] = _e842;
                uint _e844 = ish_index;
                ish_index = (_e844 + 1u);
                float _e846 = offset_1;
                offset_1 = min(_e846, _e832.x);
            }
        }
    }
    uint _e853 = ish_index;
    ismd_5.dyn_sph_cubes_start = _e853;
    uint _e857 = _group_0_binding_9_fs.shapes_arrays_metadata.sph_cubes_start;
    i_70 = _e857;
    bool loop_init_76 = true;
    while(true) {
        if (!loop_init_76) {
            uint _e906 = i_70;
            i_70 = (_e906 + 1u);
        }
        loop_init_76 = false;
        uint _e859 = i_70;
        uint _e863 = _group_0_binding_9_fs.shapes_arrays_metadata.sph_cubes_amount;
        uint _e867 = _group_0_binding_9_fs.shapes_arrays_metadata.sph_cubes_start;
        if ((_e859 < (_e863 + _e867))) {
        } else {
            break;
        }
        {
            uint _e871 = i_70;
            vec4 _e874 = _group_0_binding_5_fs[_e871].pos;
            uint _e877 = i_70;
            vec4 _e880 = _group_0_binding_5_fs[_e877].size;
            uint _e882 = i_70;
            float _e885 = _group_0_binding_5_fs[_e882].roundness;
            vec2 _e888 = cube_intersection((ro_3 - _e874), rd_3, (_e880 + vec4(_e885)));
            if ((_e888.y > 0.0)) {
                uint _e894 = ismd_5.dyn_sph_cubes_amount;
                ismd_5.dyn_sph_cubes_amount = (_e894 + 1u);
                uint _e896 = ish_index;
                uint _e898 = i_70;
                ish_5[_e896] = _e898;
                uint _e900 = ish_index;
                ish_index = (_e900 + 1u);
                float _e902 = offset_1;
                offset_1 = min(_e902, _e888.x);
            }
        }
    }
    uint _e909 = ish_index;
    ismd_5.dyn_inf_cubes_start = _e909;
    uint _e913 = _group_0_binding_9_fs.shapes_arrays_metadata.inf_cubes_start;
    i_71 = _e913;
    bool loop_init_77 = true;
    while(true) {
        if (!loop_init_77) {
            uint _e963 = i_71;
            i_71 = (_e963 + 1u);
        }
        loop_init_77 = false;
        uint _e915 = i_71;
        uint _e919 = _group_0_binding_9_fs.shapes_arrays_metadata.inf_cubes_amount;
        uint _e923 = _group_0_binding_9_fs.shapes_arrays_metadata.inf_cubes_start;
        if ((_e915 < (_e919 + _e923))) {
        } else {
            break;
        }
        {
            uint _e927 = i_71;
            vec4 _e930 = _group_0_binding_5_fs[_e927].pos;
            uint _e933 = i_71;
            vec4 _e936 = _group_0_binding_5_fs[_e933].size;
            uint _e939 = i_71;
            float _e942 = _group_0_binding_5_fs[_e939].roundness;
            vec2 _e945 = inf_cube_intersection((ro_3 - _e930), rd_3, (_e936.xyz + vec3(_e942)));
            if ((_e945.y > 0.0)) {
                uint _e951 = ismd_5.dyn_inf_cubes_amount;
                ismd_5.dyn_inf_cubes_amount = (_e951 + 1u);
                uint _e953 = ish_index;
                uint _e955 = i_71;
                ish_5[_e953] = _e955;
                uint _e957 = ish_index;
                ish_index = (_e957 + 1u);
                float _e959 = offset_1;
                offset_1 = min(_e959, _e945.x);
            }
        }
    }
    uint _e966 = ish_index;
    ismd_5.st_s_neg_cubes_start = _e966;
    uint _e970 = _group_0_binding_4_fs.shapes_arrays_metadata.s_neg_cubes_start;
    i_72 = _e970;
    bool loop_init_78 = true;
    while(true) {
        if (!loop_init_78) {
            uint _e1023 = i_72;
            i_72 = (_e1023 + 1u);
        }
        loop_init_78 = false;
        uint _e972 = i_72;
        uint _e976 = _group_0_binding_4_fs.shapes_arrays_metadata.s_neg_cubes_amount;
        uint _e980 = _group_0_binding_4_fs.shapes_arrays_metadata.s_neg_cubes_start;
        if ((_e972 < (_e976 + _e980))) {
        } else {
            break;
        }
        {
            uint _e984 = i_72;
            vec4 _e987 = _group_0_binding_3_fs[_e984].pos;
            uint _e990 = i_72;
            vec4 _e993 = _group_0_binding_3_fs[_e990].size;
            uint _e995 = i_72;
            float _e998 = _group_0_binding_3_fs[_e995].roundness;
            float _e1003 = _group_0_binding_4_fs.stickiness;
            vec2 _e1008 = cube_intersection((ro_3 - _e987), rd_3, ((_e993 + vec4(_e998)) + vec4((_e1003 * STICKINESS_EFFECT_COEF))));
            if ((_e1008.y > 0.0)) {
                uint _e1014 = ismd_5.st_s_neg_cubes_amount;
                ismd_5.st_s_neg_cubes_amount = (_e1014 + 1u);
                uint _e1016 = ish_index;
                uint _e1018 = i_72;
                ish_5[_e1016] = _e1018;
                uint _e1020 = ish_index;
                ish_index = (_e1020 + 1u);
            }
        }
    }
    uint _e1026 = ish_index;
    ismd_5.st_s_neg_spheres_start = _e1026;
    uint _e1030 = _group_0_binding_4_fs.shapes_arrays_metadata.s_neg_spheres_start;
    i_73 = _e1030;
    bool loop_init_79 = true;
    while(true) {
        if (!loop_init_79) {
            uint _e1082 = i_73;
            i_73 = (_e1082 + 1u);
        }
        loop_init_79 = false;
        uint _e1032 = i_73;
        uint _e1036 = _group_0_binding_4_fs.shapes_arrays_metadata.s_neg_spheres_amount;
        uint _e1040 = _group_0_binding_4_fs.shapes_arrays_metadata.s_neg_spheres_start;
        if ((_e1032 < (_e1036 + _e1040))) {
        } else {
            break;
        }
        {
            uint _e1044 = i_73;
            vec4 _e1047 = _group_0_binding_3_fs[_e1044].pos;
            uint _e1050 = i_73;
            float _e1054 = _group_0_binding_3_fs[_e1050].size.x;
            uint _e1056 = i_73;
            float _e1059 = _group_0_binding_3_fs[_e1056].roundness;
            float _e1063 = _group_0_binding_4_fs.stickiness;
            vec2 _e1067 = sph_intersection((ro_3 - _e1047), rd_3, ((_e1054 + _e1059) + (_e1063 * STICKINESS_EFFECT_COEF)));
            if ((_e1067.y > 0.0)) {
                uint _e1073 = ismd_5.st_s_neg_spheres_amount;
                ismd_5.st_s_neg_spheres_amount = (_e1073 + 1u);
                uint _e1075 = ish_index;
                uint _e1077 = i_73;
                ish_5[_e1075] = _e1077;
                uint _e1079 = ish_index;
                ish_index = (_e1079 + 1u);
            }
        }
    }
    uint _e1085 = ish_index;
    ismd_5.st_s_neg_sph_cubes_start = _e1085;
    uint _e1089 = _group_0_binding_4_fs.shapes_arrays_metadata.s_neg_sph_cubes_start;
    i_74 = _e1089;
    bool loop_init_80 = true;
    while(true) {
        if (!loop_init_80) {
            uint _e1142 = i_74;
            i_74 = (_e1142 + 1u);
        }
        loop_init_80 = false;
        uint _e1091 = i_74;
        uint _e1095 = _group_0_binding_4_fs.shapes_arrays_metadata.s_neg_sph_cubes_amount;
        uint _e1099 = _group_0_binding_4_fs.shapes_arrays_metadata.s_neg_sph_cubes_start;
        if ((_e1091 < (_e1095 + _e1099))) {
        } else {
            break;
        }
        {
            uint _e1103 = i_74;
            vec4 _e1106 = _group_0_binding_3_fs[_e1103].pos;
            uint _e1109 = i_74;
            vec4 _e1112 = _group_0_binding_3_fs[_e1109].size;
            uint _e1114 = i_74;
            float _e1117 = _group_0_binding_3_fs[_e1114].roundness;
            float _e1122 = _group_0_binding_4_fs.stickiness;
            vec2 _e1127 = cube_intersection((ro_3 - _e1106), rd_3, ((_e1112 + vec4(_e1117)) + vec4((_e1122 * STICKINESS_EFFECT_COEF))));
            if ((_e1127.y > 0.0)) {
                uint _e1133 = ismd_5.st_s_neg_sph_cubes_amount;
                ismd_5.st_s_neg_sph_cubes_amount = (_e1133 + 1u);
                uint _e1135 = ish_index;
                uint _e1137 = i_74;
                ish_5[_e1135] = _e1137;
                uint _e1139 = ish_index;
                ish_index = (_e1139 + 1u);
            }
        }
    }
    uint _e1145 = ish_index;
    ismd_5.st_s_neg_inf_cubes_start = _e1145;
    uint _e1149 = _group_0_binding_4_fs.shapes_arrays_metadata.s_neg_inf_cubes_start;
    i_75 = _e1149;
    bool loop_init_81 = true;
    while(true) {
        if (!loop_init_81) {
            uint _e1203 = i_75;
            i_75 = (_e1203 + 1u);
        }
        loop_init_81 = false;
        uint _e1151 = i_75;
        uint _e1155 = _group_0_binding_4_fs.shapes_arrays_metadata.s_neg_inf_cubes_amount;
        uint _e1159 = _group_0_binding_4_fs.shapes_arrays_metadata.s_neg_inf_cubes_start;
        if ((_e1151 < (_e1155 + _e1159))) {
        } else {
            break;
        }
        {
            uint _e1163 = i_75;
            vec4 _e1166 = _group_0_binding_3_fs[_e1163].pos;
            uint _e1169 = i_75;
            vec4 _e1172 = _group_0_binding_3_fs[_e1169].size;
            uint _e1175 = i_75;
            float _e1178 = _group_0_binding_3_fs[_e1175].roundness;
            float _e1183 = _group_0_binding_4_fs.stickiness;
            vec2 _e1188 = inf_cube_intersection((ro_3 - _e1166), rd_3, ((_e1172.xyz + vec3(_e1178)) + vec3((_e1183 * STICKINESS_EFFECT_COEF))));
            if ((_e1188.y > 0.0)) {
                uint _e1194 = ismd_5.st_s_neg_inf_cubes_amount;
                ismd_5.st_s_neg_inf_cubes_amount = (_e1194 + 1u);
                uint _e1196 = ish_index;
                uint _e1198 = i_75;
                ish_5[_e1196] = _e1198;
                uint _e1200 = ish_index;
                ish_index = (_e1200 + 1u);
            }
        }
    }
    uint _e1206 = ish_index;
    ismd_5.dyn_s_neg_cubes_start = _e1206;
    uint _e1210 = _group_0_binding_9_fs.shapes_arrays_metadata.s_neg_cubes_start;
    i_76 = _e1210;
    bool loop_init_82 = true;
    while(true) {
        if (!loop_init_82) {
            uint _e1263 = i_76;
            i_76 = (_e1263 + 1u);
        }
        loop_init_82 = false;
        uint _e1212 = i_76;
        uint _e1216 = _group_0_binding_9_fs.shapes_arrays_metadata.s_neg_cubes_amount;
        uint _e1220 = _group_0_binding_9_fs.shapes_arrays_metadata.s_neg_cubes_start;
        if ((_e1212 < (_e1216 + _e1220))) {
        } else {
            break;
        }
        {
            uint _e1224 = i_76;
            vec4 _e1227 = _group_0_binding_8_fs[_e1224].pos;
            uint _e1230 = i_76;
            vec4 _e1233 = _group_0_binding_8_fs[_e1230].size;
            uint _e1235 = i_76;
            float _e1238 = _group_0_binding_8_fs[_e1235].roundness;
            float _e1243 = _group_0_binding_4_fs.stickiness;
            vec2 _e1248 = cube_intersection((ro_3 - _e1227), rd_3, ((_e1233 + vec4(_e1238)) + vec4((_e1243 * STICKINESS_EFFECT_COEF))));
            if ((_e1248.y > 0.0)) {
                uint _e1254 = ismd_5.dyn_s_neg_cubes_amount;
                ismd_5.dyn_s_neg_cubes_amount = (_e1254 + 1u);
                uint _e1256 = ish_index;
                uint _e1258 = i_76;
                ish_5[_e1256] = _e1258;
                uint _e1260 = ish_index;
                ish_index = (_e1260 + 1u);
            }
        }
    }
    uint _e1266 = ish_index;
    ismd_5.dyn_s_neg_spheres_start = _e1266;
    uint _e1270 = _group_0_binding_9_fs.shapes_arrays_metadata.s_neg_spheres_start;
    i_77 = _e1270;
    bool loop_init_83 = true;
    while(true) {
        if (!loop_init_83) {
            uint _e1322 = i_77;
            i_77 = (_e1322 + 1u);
        }
        loop_init_83 = false;
        uint _e1272 = i_77;
        uint _e1276 = _group_0_binding_9_fs.shapes_arrays_metadata.s_neg_spheres_amount;
        uint _e1280 = _group_0_binding_9_fs.shapes_arrays_metadata.s_neg_spheres_start;
        if ((_e1272 < (_e1276 + _e1280))) {
        } else {
            break;
        }
        {
            uint _e1284 = i_77;
            vec4 _e1287 = _group_0_binding_8_fs[_e1284].pos;
            uint _e1290 = i_77;
            float _e1294 = _group_0_binding_8_fs[_e1290].size.x;
            uint _e1296 = i_77;
            float _e1299 = _group_0_binding_8_fs[_e1296].roundness;
            float _e1303 = _group_0_binding_4_fs.stickiness;
            vec2 _e1307 = sph_intersection((ro_3 - _e1287), rd_3, ((_e1294 + _e1299) + (_e1303 * STICKINESS_EFFECT_COEF)));
            if ((_e1307.y > 0.0)) {
                uint _e1313 = ismd_5.dyn_s_neg_spheres_amount;
                ismd_5.dyn_s_neg_spheres_amount = (_e1313 + 1u);
                uint _e1315 = ish_index;
                uint _e1317 = i_77;
                ish_5[_e1315] = _e1317;
                uint _e1319 = ish_index;
                ish_index = (_e1319 + 1u);
            }
        }
    }
    uint _e1325 = ish_index;
    ismd_5.dyn_s_neg_sph_cubes_start = _e1325;
    uint _e1329 = _group_0_binding_9_fs.shapes_arrays_metadata.s_neg_sph_cubes_start;
    i_78 = _e1329;
    bool loop_init_84 = true;
    while(true) {
        if (!loop_init_84) {
            uint _e1382 = i_78;
            i_78 = (_e1382 + 1u);
        }
        loop_init_84 = false;
        uint _e1331 = i_78;
        uint _e1335 = _group_0_binding_9_fs.shapes_arrays_metadata.s_neg_sph_cubes_amount;
        uint _e1339 = _group_0_binding_9_fs.shapes_arrays_metadata.s_neg_sph_cubes_start;
        if ((_e1331 < (_e1335 + _e1339))) {
        } else {
            break;
        }
        {
            uint _e1343 = i_78;
            vec4 _e1346 = _group_0_binding_8_fs[_e1343].pos;
            uint _e1349 = i_78;
            vec4 _e1352 = _group_0_binding_8_fs[_e1349].size;
            uint _e1354 = i_78;
            float _e1357 = _group_0_binding_8_fs[_e1354].roundness;
            float _e1362 = _group_0_binding_4_fs.stickiness;
            vec2 _e1367 = cube_intersection((ro_3 - _e1346), rd_3, ((_e1352 + vec4(_e1357)) + vec4((_e1362 * STICKINESS_EFFECT_COEF))));
            if ((_e1367.y > 0.0)) {
                uint _e1373 = ismd_5.dyn_s_neg_sph_cubes_amount;
                ismd_5.dyn_s_neg_sph_cubes_amount = (_e1373 + 1u);
                uint _e1375 = ish_index;
                uint _e1377 = i_78;
                ish_5[_e1375] = _e1377;
                uint _e1379 = ish_index;
                ish_index = (_e1379 + 1u);
            }
        }
    }
    uint _e1385 = ish_index;
    ismd_5.dyn_s_neg_inf_cubes_start = _e1385;
    uint _e1389 = _group_0_binding_9_fs.shapes_arrays_metadata.s_neg_inf_cubes_start;
    i_79 = _e1389;
    bool loop_init_85 = true;
    while(true) {
        if (!loop_init_85) {
            uint _e1443 = i_79;
            i_79 = (_e1443 + 1u);
        }
        loop_init_85 = false;
        uint _e1391 = i_79;
        uint _e1395 = _group_0_binding_9_fs.shapes_arrays_metadata.s_neg_inf_cubes_amount;
        uint _e1399 = _group_0_binding_9_fs.shapes_arrays_metadata.s_neg_inf_cubes_start;
        if ((_e1391 < (_e1395 + _e1399))) {
        } else {
            break;
        }
        {
            uint _e1403 = i_79;
            vec4 _e1406 = _group_0_binding_8_fs[_e1403].pos;
            uint _e1409 = i_79;
            vec4 _e1412 = _group_0_binding_8_fs[_e1409].size;
            uint _e1415 = i_79;
            float _e1418 = _group_0_binding_8_fs[_e1415].roundness;
            float _e1423 = _group_0_binding_4_fs.stickiness;
            vec2 _e1428 = inf_cube_intersection((ro_3 - _e1406), rd_3, ((_e1412.xyz + vec3(_e1418)) + vec3((_e1423 * STICKINESS_EFFECT_COEF))));
            if ((_e1428.y > 0.0)) {
                uint _e1434 = ismd_5.dyn_s_neg_inf_cubes_amount;
                ismd_5.dyn_s_neg_inf_cubes_amount = (_e1434 + 1u);
                uint _e1436 = ish_index;
                uint _e1438 = i_79;
                ish_5[_e1436] = _e1438;
                uint _e1440 = ish_index;
                ish_index = (_e1440 + 1u);
            }
        }
    }
    uint _e1446 = ish_index;
    ismd_5.st_neg_cubes_start = _e1446;
    uint _e1450 = _group_0_binding_4_fs.shapes_arrays_metadata.neg_cubes_start;
    i_80 = _e1450;
    bool loop_init_86 = true;
    while(true) {
        if (!loop_init_86) {
            uint _e1496 = i_80;
            i_80 = (_e1496 + 1u);
        }
        loop_init_86 = false;
        uint _e1452 = i_80;
        uint _e1456 = _group_0_binding_4_fs.shapes_arrays_metadata.neg_cubes_amount;
        uint _e1460 = _group_0_binding_4_fs.shapes_arrays_metadata.neg_cubes_start;
        if ((_e1452 < (_e1456 + _e1460))) {
        } else {
            break;
        }
        {
            uint _e1464 = i_80;
            vec4 _e1467 = _group_0_binding_1_fs[_e1464].pos;
            uint _e1470 = i_80;
            vec4 _e1473 = _group_0_binding_1_fs[_e1470].size;
            uint _e1475 = i_80;
            float _e1478 = _group_0_binding_1_fs[_e1475].roundness;
            vec2 _e1481 = cube_intersection((ro_3 - _e1467), rd_3, (_e1473 + vec4(_e1478)));
            if ((_e1481.y > 0.0)) {
                uint _e1487 = ismd_5.st_neg_cubes_amount;
                ismd_5.st_neg_cubes_amount = (_e1487 + 1u);
                uint _e1489 = ish_index;
                uint _e1491 = i_80;
                ish_5[_e1489] = _e1491;
                uint _e1493 = ish_index;
                ish_index = (_e1493 + 1u);
            }
        }
    }
    uint _e1499 = ish_index;
    ismd_5.st_neg_spheres_start = _e1499;
    uint _e1503 = _group_0_binding_4_fs.shapes_arrays_metadata.neg_spheres_start;
    i_81 = _e1503;
    bool loop_init_87 = true;
    while(true) {
        if (!loop_init_87) {
            uint _e1549 = i_81;
            i_81 = (_e1549 + 1u);
        }
        loop_init_87 = false;
        uint _e1505 = i_81;
        uint _e1509 = _group_0_binding_4_fs.shapes_arrays_metadata.neg_spheres_amount;
        uint _e1513 = _group_0_binding_4_fs.shapes_arrays_metadata.neg_spheres_start;
        if ((_e1505 < (_e1509 + _e1513))) {
        } else {
            break;
        }
        {
            uint _e1517 = i_81;
            vec4 _e1520 = _group_0_binding_1_fs[_e1517].pos;
            uint _e1523 = i_81;
            float _e1527 = _group_0_binding_1_fs[_e1523].size.x;
            uint _e1529 = i_81;
            float _e1532 = _group_0_binding_1_fs[_e1529].roundness;
            vec2 _e1534 = sph_intersection((ro_3 - _e1520), rd_3, (_e1527 + _e1532));
            if ((_e1534.y > 0.0)) {
                uint _e1540 = ismd_5.st_neg_spheres_amount;
                ismd_5.st_neg_spheres_amount = (_e1540 + 1u);
                uint _e1542 = ish_index;
                uint _e1544 = i_81;
                ish_5[_e1542] = _e1544;
                uint _e1546 = ish_index;
                ish_index = (_e1546 + 1u);
            }
        }
    }
    uint _e1552 = ish_index;
    ismd_5.st_neg_sph_cubes_start = _e1552;
    uint _e1556 = _group_0_binding_4_fs.shapes_arrays_metadata.neg_sph_cubes_start;
    i_82 = _e1556;
    bool loop_init_88 = true;
    while(true) {
        if (!loop_init_88) {
            uint _e1602 = i_82;
            i_82 = (_e1602 + 1u);
        }
        loop_init_88 = false;
        uint _e1558 = i_82;
        uint _e1562 = _group_0_binding_4_fs.shapes_arrays_metadata.neg_sph_cubes_amount;
        uint _e1566 = _group_0_binding_4_fs.shapes_arrays_metadata.neg_sph_cubes_start;
        if ((_e1558 < (_e1562 + _e1566))) {
        } else {
            break;
        }
        {
            uint _e1570 = i_82;
            vec4 _e1573 = _group_0_binding_1_fs[_e1570].pos;
            uint _e1576 = i_82;
            vec4 _e1579 = _group_0_binding_1_fs[_e1576].size;
            uint _e1581 = i_82;
            float _e1584 = _group_0_binding_1_fs[_e1581].roundness;
            vec2 _e1587 = cube_intersection((ro_3 - _e1573), rd_3, (_e1579 + vec4(_e1584)));
            if ((_e1587.y > 0.0)) {
                uint _e1593 = ismd_5.st_neg_sph_cubes_amount;
                ismd_5.st_neg_sph_cubes_amount = (_e1593 + 1u);
                uint _e1595 = ish_index;
                uint _e1597 = i_82;
                ish_5[_e1595] = _e1597;
                uint _e1599 = ish_index;
                ish_index = (_e1599 + 1u);
            }
        }
    }
    uint _e1605 = ish_index;
    ismd_5.st_neg_inf_cubes_start = _e1605;
    uint _e1609 = _group_0_binding_4_fs.shapes_arrays_metadata.neg_inf_cubes_start;
    i_83 = _e1609;
    bool loop_init_89 = true;
    while(true) {
        if (!loop_init_89) {
            uint _e1656 = i_83;
            i_83 = (_e1656 + 1u);
        }
        loop_init_89 = false;
        uint _e1611 = i_83;
        uint _e1615 = _group_0_binding_4_fs.shapes_arrays_metadata.neg_inf_cubes_amount;
        uint _e1619 = _group_0_binding_4_fs.shapes_arrays_metadata.neg_inf_cubes_start;
        if ((_e1611 < (_e1615 + _e1619))) {
        } else {
            break;
        }
        {
            uint _e1623 = i_83;
            vec4 _e1626 = _group_0_binding_1_fs[_e1623].pos;
            uint _e1629 = i_83;
            vec4 _e1632 = _group_0_binding_1_fs[_e1629].size;
            uint _e1635 = i_83;
            float _e1638 = _group_0_binding_1_fs[_e1635].roundness;
            vec2 _e1641 = inf_cube_intersection((ro_3 - _e1626), rd_3, (_e1632.xyz + vec3(_e1638)));
            if ((_e1641.y > 0.0)) {
                uint _e1647 = ismd_5.st_neg_inf_cubes_amount;
                ismd_5.st_neg_inf_cubes_amount = (_e1647 + 1u);
                uint _e1649 = ish_index;
                uint _e1651 = i_83;
                ish_5[_e1649] = _e1651;
                uint _e1653 = ish_index;
                ish_index = (_e1653 + 1u);
            }
        }
    }
    uint _e1659 = ish_index;
    ismd_5.dyn_neg_cubes_start = _e1659;
    uint _e1663 = _group_0_binding_9_fs.shapes_arrays_metadata.neg_cubes_start;
    i_84 = _e1663;
    bool loop_init_90 = true;
    while(true) {
        if (!loop_init_90) {
            uint _e1709 = i_84;
            i_84 = (_e1709 + 1u);
        }
        loop_init_90 = false;
        uint _e1665 = i_84;
        uint _e1669 = _group_0_binding_9_fs.shapes_arrays_metadata.neg_cubes_amount;
        uint _e1673 = _group_0_binding_9_fs.shapes_arrays_metadata.neg_cubes_start;
        if ((_e1665 < (_e1669 + _e1673))) {
        } else {
            break;
        }
        {
            uint _e1677 = i_84;
            vec4 _e1680 = _group_0_binding_6_fs[_e1677].pos;
            uint _e1683 = i_84;
            vec4 _e1686 = _group_0_binding_6_fs[_e1683].size;
            uint _e1688 = i_84;
            float _e1691 = _group_0_binding_6_fs[_e1688].roundness;
            vec2 _e1694 = cube_intersection((ro_3 - _e1680), rd_3, (_e1686 + vec4(_e1691)));
            if ((_e1694.y > 0.0)) {
                uint _e1700 = ismd_5.dyn_neg_cubes_amount;
                ismd_5.dyn_neg_cubes_amount = (_e1700 + 1u);
                uint _e1702 = ish_index;
                uint _e1704 = i_84;
                ish_5[_e1702] = _e1704;
                uint _e1706 = ish_index;
                ish_index = (_e1706 + 1u);
            }
        }
    }
    uint _e1712 = ish_index;
    ismd_5.dyn_neg_spheres_start = _e1712;
    uint _e1716 = _group_0_binding_9_fs.shapes_arrays_metadata.neg_spheres_start;
    i_85 = _e1716;
    bool loop_init_91 = true;
    while(true) {
        if (!loop_init_91) {
            uint _e1762 = i_85;
            i_85 = (_e1762 + 1u);
        }
        loop_init_91 = false;
        uint _e1718 = i_85;
        uint _e1722 = _group_0_binding_9_fs.shapes_arrays_metadata.neg_spheres_amount;
        uint _e1726 = _group_0_binding_9_fs.shapes_arrays_metadata.neg_spheres_start;
        if ((_e1718 < (_e1722 + _e1726))) {
        } else {
            break;
        }
        {
            uint _e1730 = i_85;
            vec4 _e1733 = _group_0_binding_6_fs[_e1730].pos;
            uint _e1736 = i_85;
            float _e1740 = _group_0_binding_6_fs[_e1736].size.x;
            uint _e1742 = i_85;
            float _e1745 = _group_0_binding_6_fs[_e1742].roundness;
            vec2 _e1747 = sph_intersection((ro_3 - _e1733), rd_3, (_e1740 + _e1745));
            if ((_e1747.y > 0.0)) {
                uint _e1753 = ismd_5.dyn_neg_spheres_amount;
                ismd_5.dyn_neg_spheres_amount = (_e1753 + 1u);
                uint _e1755 = ish_index;
                uint _e1757 = i_85;
                ish_5[_e1755] = _e1757;
                uint _e1759 = ish_index;
                ish_index = (_e1759 + 1u);
            }
        }
    }
    uint _e1765 = ish_index;
    ismd_5.dyn_neg_sph_cubes_start = _e1765;
    uint _e1769 = _group_0_binding_9_fs.shapes_arrays_metadata.neg_sph_cubes_start;
    i_86 = _e1769;
    bool loop_init_92 = true;
    while(true) {
        if (!loop_init_92) {
            uint _e1815 = i_86;
            i_86 = (_e1815 + 1u);
        }
        loop_init_92 = false;
        uint _e1771 = i_86;
        uint _e1775 = _group_0_binding_9_fs.shapes_arrays_metadata.neg_sph_cubes_amount;
        uint _e1779 = _group_0_binding_9_fs.shapes_arrays_metadata.neg_sph_cubes_start;
        if ((_e1771 < (_e1775 + _e1779))) {
        } else {
            break;
        }
        {
            uint _e1783 = i_86;
            vec4 _e1786 = _group_0_binding_6_fs[_e1783].pos;
            uint _e1789 = i_86;
            vec4 _e1792 = _group_0_binding_6_fs[_e1789].size;
            uint _e1794 = i_86;
            float _e1797 = _group_0_binding_6_fs[_e1794].roundness;
            vec2 _e1800 = cube_intersection((ro_3 - _e1786), rd_3, (_e1792 + vec4(_e1797)));
            if ((_e1800.y > 0.0)) {
                uint _e1806 = ismd_5.dyn_neg_sph_cubes_amount;
                ismd_5.dyn_neg_sph_cubes_amount = (_e1806 + 1u);
                uint _e1808 = ish_index;
                uint _e1810 = i_86;
                ish_5[_e1808] = _e1810;
                uint _e1812 = ish_index;
                ish_index = (_e1812 + 1u);
            }
        }
    }
    uint _e1818 = ish_index;
    ismd_5.dyn_neg_inf_cubes_start = _e1818;
    uint _e1822 = _group_0_binding_9_fs.shapes_arrays_metadata.neg_inf_cubes_start;
    i_87 = _e1822;
    bool loop_init_93 = true;
    while(true) {
        if (!loop_init_93) {
            uint _e1869 = i_87;
            i_87 = (_e1869 + 1u);
        }
        loop_init_93 = false;
        uint _e1824 = i_87;
        uint _e1828 = _group_0_binding_9_fs.shapes_arrays_metadata.neg_inf_cubes_amount;
        uint _e1832 = _group_0_binding_9_fs.shapes_arrays_metadata.neg_inf_cubes_start;
        if ((_e1824 < (_e1828 + _e1832))) {
        } else {
            break;
        }
        {
            uint _e1836 = i_87;
            vec4 _e1839 = _group_0_binding_6_fs[_e1836].pos;
            uint _e1842 = i_87;
            vec4 _e1845 = _group_0_binding_6_fs[_e1842].size;
            uint _e1848 = i_87;
            float _e1851 = _group_0_binding_6_fs[_e1848].roundness;
            vec2 _e1854 = inf_cube_intersection((ro_3 - _e1839), rd_3, (_e1845.xyz + vec3(_e1851)));
            if ((_e1854.y > 0.0)) {
                uint _e1860 = ismd_5.dyn_neg_inf_cubes_amount;
                ismd_5.dyn_neg_inf_cubes_amount = (_e1860 + 1u);
                uint _e1862 = ish_index;
                uint _e1864 = i_87;
                ish_5[_e1862] = _e1864;
                uint _e1866 = ish_index;
                ish_index = (_e1866 + 1u);
            }
        }
    }
    uint _e1872 = ish_index;
    ismd_5.player_forms_start = _e1872;
    bool loop_init_94 = true;
    while(true) {
        if (!loop_init_94) {
            uint _e1911 = i_88;
            i_88 = (_e1911 + 1u);
        }
        loop_init_94 = false;
        uint _e1875 = i_88;
        uint _e1878 = _group_0_binding_9_fs.player_forms_amount;
        if ((_e1875 < _e1878)) {
        } else {
            break;
        }
        {
            uint _e1881 = i_88;
            vec4 _e1884 = _group_1_binding_2_fs[_e1881].pos;
            uint _e1887 = i_88;
            float _e1890 = _group_1_binding_2_fs[_e1887].radius;
            vec2 _e1893 = sph_intersection((ro_3 - _e1884), rd_3, (_e1890 * 1.5));
            if ((_e1893.y > 0.0)) {
                uint _e1899 = ismd_5.player_forms_amount;
                ismd_5.player_forms_amount = (_e1899 + 1u);
                uint _e1901 = ish_index;
                uint _e1903 = i_88;
                ish_5[_e1901] = _e1903;
                uint _e1905 = ish_index;
                ish_index = (_e1905 + 1u);
                float _e1907 = offset_1;
                offset_1 = min(_e1907, _e1893.x);
            }
        }
    }
    ray_w_rotated = false;
    if ((rd_3.w < -0.0002)) {
        ray_w_rotated = true;
    }
    float _e1920 = offset_1;
    offset_1 = clamp(_e1920, 0.0, 1400.0);
    float _e1924 = offset_1;
    return _e1924;
}

vec3 apply_material(vec4 pos_1, vec4 ray_dir_1, float dist_2, inout IntersectedShapesMetadata ismd_6, inout uint ish_6[16], int material) {
    vec3 color_5 = vec3(0.0);
    if ((material < 0)) {
        return vec3(0.7);
    }
    vec4 _e14 = _group_0_binding_4_fs.materials[material].color;
    vec3 diffuse = _e14.xyz;
    vec4 _e18 = get_normal((pos_1 + (ray_dir_1 * dist_2)), ismd_6, ish_6);
    float dir_shade = dot(_e18, normalize(vec4(1.0, 0.5, 0.3, 0.1)));
    color_5 = (diffuse * dir_shade);
    vec3 _e28 = color_5;
    return _e28;
}

void main() {
    VertexOutput inn = VertexOutput(gl_FragCoord, _vs2fs_location0);
    vec2 uv = vec2(0.0);
    vec4 ray_direction = vec4(0.0);
    IntersectedShapesMetadata ismd = IntersectedShapesMetadata(0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u);
    uint ish[16] = uint[16](0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u);
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
    vec4 _e30 = ray_direction;
    float _e31 = find_intersections(camera_position, _e30, ismd, ish);
    vec4 _e32 = ray_direction;
    vec2 _e33 = ray_march(camera_position, _e32, _e31, ismd, ish);
    vec4 _e37 = ray_direction;
    get_mat(camera_position, _e37, _e33.x, ismd, ish, mats, mats_wieghts, mats_count);
    vec4 _e39 = ray_direction;
    int _e42 = mats[0];
    vec3 _e43 = apply_material(camera_position, _e39, _e33.x, ismd, ish, _e42);
    color = _e43;
    bool loop_init_95 = true;
    while(true) {
        if (!loop_init_95) {
            uint _e62 = i;
            i = (_e62 + 1u);
        }
        loop_init_95 = false;
        uint _e47 = i;
        uint _e48 = mats_count;
        if ((_e47 < _e48)) {
        } else {
            break;
        }
        {
            vec4 _e50 = ray_direction;
            uint _e52 = i;
            int _e54 = mats[_e52];
            vec3 _e55 = apply_material(camera_position, _e50, _e33.x, ismd, ish, _e54);
            vec3 _e56 = color;
            uint _e57 = i;
            float _e59 = mats_wieghts[_e57];
            color = mix(_e56, _e55, _e59);
        }
    }
    vec3 _e64 = color;
    color = pow(_e64, vec3(0.4545));
    vec2 _e68 = uv;
    vec3 _e77 = color;
    color = (_e77 + vec3(((0.007 - clamp(length(_e68), 0.0, 0.007)) * 1000.0)));
    vec3 _e80 = color;
    _fs2p_location0 = vec4(_e80, 1.0);
    return;
}

