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

layout(location = 0) in vec3 _p2vs_location0;
layout(location = 0) smooth out vec3 _vs2fs_location0;

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

void main() {
    VertexInput model = VertexInput(_p2vs_location0);
    VertexOutput out_ = VertexOutput(vec4(0.0), vec3(0.0));
    out_.clip_position = vec4(model.position, 1.0);
    out_.position = model.position;
    VertexOutput _e8 = out_;
    gl_Position = _e8.clip_position;
    _vs2fs_location0 = _e8.position;
    gl_Position.yz = vec2(-gl_Position.y, gl_Position.z * 2.0 - gl_Position.w);
    return;
}

