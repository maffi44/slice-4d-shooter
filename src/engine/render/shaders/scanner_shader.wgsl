// Fragment shader
struct CameraUniform {
    cam_pos: vec4<f32>,
    cam_rot: mat4x4<f32>,
}

struct SphericalAreasMetadata {
    holegun_colorized_areas_start: u32,
    holegun_colorized_areas_amount: u32,
    explode_areas_start: u32,
    explode_areas_amount: u32,
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

struct RectTransformUniform {
    scale: vec2<f32>,
    translation: vec2<f32>,
    transparency: f32,
    empty_byte: f32,
    rotation_around_rect_center: f32,
    rotation_around_screen_center: f32,
}

struct OtherDynamicData {
    shapes_arrays_metadata: ShapesMetadata,
    spherical_areas_meatadata: SphericalAreasMetadata,
    camera_data: CameraUniform,
    empty_bytes1: vec3<u32>,
    beam_areas_amount: u32,
    player_forms_amount: u32,
    w_scanner_radius: f32,
    w_scanner_intesity: f32,
    death_screen_effect: f32,
    getting_damage_screen_effect: f32,
    stickiness: f32,
    screen_aspect: f32,
    time: f32,
    //all shapes bounding box sides
    bb_pos_side: vec4<f32>,
    bb_neg_side: vec4<f32>,
}

struct PlayerForm {
    pos: vec4<f32>,
    empty_bytes: vec4<u32>,
    color: vec3<f32>,
    radius: f32,
    rotation: mat4x4<f32>,
    weapon_offset: vec4<f32>,
}

struct ScannerData {
    empty_byte0: u32,
    empty_byte1: u32,
    empty_byte2: u32,
    orientation: u32,

    // hits: array<vec4<f32>, 16>,
}

@group(0) @binding(0) var<uniform> rect_transform: RectTransformUniform;
@group(0) @binding(1) var<uniform> dynamic_data: OtherDynamicData;
@group(0) @binding(2) var<uniform> dyn_player_forms: array<PlayerForm, 16>;
@group(0) @binding(3) var<uniform> scanner_data: ScannerData;



struct VertexInput {
    @location(0) @interpolate(perspective) position: vec3<f32>,
    @location(1) @interpolate(perspective) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>
};

fn rotation_mat(angle: f32) -> mat2x2<f32> {
    var c: f32 = cos(angle);
    var s: f32 = sin(angle);

    return mat2x2<f32>(c, -s, s, c);
}

fn sd_sphere(p: vec4<f32>, radius: f32) -> f32 {
    return length(p) - radius;
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {

    var coords = model.position;

    let c = vec2(coords.x, coords.y) * rotation_mat(rect_transform.rotation_around_screen_center);

    coords.x = c.x;
    coords.y = c.y;
    
    coords *= vec3(rect_transform.scale, 0.0);

    coords += vec3(rect_transform.translation, 0.0);


    var out: VertexOutput;
    out.clip_position = vec4<f32>(coords, 1.0);
    out.uv = model.uv;
    return out;
}

const MAX_SCANNER_RADIUS: f32 = 43.0; 

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    if dynamic_data.w_scanner_intesity == 0.0 {
        return vec4(0.0);
    }

    let uv_pos = (in.uv-vec2(0.5))*2.0;

    let dist_to_cntr = length(uv_pos);

    if dist_to_cntr > 1.0 {
        return vec4(0.0);
    }

    var col = vec4(1.0, 1.0, 1.0, 0.0);

    let sc_ring_radius = dynamic_data.w_scanner_radius / MAX_SCANNER_RADIUS;

    let ring_a = pow(1.0-abs(dist_to_cntr-sc_ring_radius), 13.0);

    col.a += ring_a;

    var en_a = 0.0;

    for (var i = 0u; i < dynamic_data.player_forms_amount; i++) {

        let en_pos  = (dynamic_data.camera_data.cam_pos - dyn_player_forms[i].pos) / MAX_SCANNER_RADIUS;
        
        let visible = clamp(((sc_ring_radius + 0.1) - length(en_pos)) * 10.0, 0.0, 1.0);

        if scanner_data.orientation == 0 {
            en_a += clamp(pow(1.0- length(uv_pos-en_pos.zx*vec2(1.0,-1.0)), 4.0)*visible,0.0,1.0);
        } else {
            en_a += clamp(pow(1.0- length(uv_pos-en_pos.zw*vec2(-1.0,1.0)), 4.0)*visible,0.0,1.0);
        }
    }

    col.a += en_a;
    col.g -= en_a;
    col.b -= en_a;

    col.a *= dynamic_data.w_scanner_intesity * rect_transform.transparency;

    return col;
}