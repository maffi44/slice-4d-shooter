struct RectTransformUniform {
    scale: vec2<f32>,
    translation: vec2<f32>,
    transparency: f32,
    empty_byte: f32,
    rotation_around_rect_center: f32,
    rotation_around_screen_center: f32,
}


struct ProgressBarUniform {
    value: f32,
    v_from: f32,
    v_to: f32,
    direction: f32,
}

@group(0) @binding(0) var<uniform> rect_transform: RectTransformUniform;
@group(0) @binding(1) var texture: texture_2d<f32>;
@group(0) @binding(2) var mask: texture_2d<f32>;
@group(0) @binding(3) var tex_sampler: sampler;
@group(0) @binding(4) var<uniform> bar_uni: ProgressBarUniform;


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


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    var col = textureSample(texture, tex_sampler, in.uv);

    if bar_uni.direction > 2.5 {
        // from top to down direction

        // unimplemented

    } else if bar_uni.direction > 1.5 {
        // from down to top direction

        // unimplemented

    } else if bar_uni.direction > 0.5 {
        // from right to left direction

        let x = 1.0 - in.uv.x;

        if x > bar_uni.v_to && x < bar_uni.v_from {
            let d = abs(bar_uni.v_from - bar_uni.v_to);

            let ot = bar_uni.v_to + bar_uni.value * d;

            if x < ot {
                let bar_col = textureSample(mask, tex_sampler, in.uv);

                let bar_edge_coof = pow((d-(ot-x))/d, 5.0);

                let bar_edge_col = bar_edge_coof * (bar_col*bar_col);

                col += bar_col + bar_edge_col + vec4(pow(bar_edge_coof, 20.0)*0.6)*bar_col.a;
            }
        }

    } else {
        // from left to right direction

        if in.uv.x > bar_uni.v_from && in.uv.x < bar_uni.v_to {
            let d = abs(bar_uni.v_to - bar_uni.v_from);

            let ot = bar_uni.v_from + bar_uni.value * d;

            if in.uv.x < ot {
                let bar_col = textureSample(mask, tex_sampler, in.uv);

                let bar_edge_coof = pow((d-(ot-in.uv.x))/d, 5.0);

                let bar_edge_col = bar_edge_coof * (bar_col*bar_col);

                col += bar_col + bar_edge_col + vec4(pow(bar_edge_coof, 20.0)*0.6)*bar_col.a;

                
            }
        }

    }

    col.a *= rect_transform.transparency;

    return col;
}