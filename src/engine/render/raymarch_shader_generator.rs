use std::{f32::consts::PI, fmt::format, ops::Add};

use glam::Vec4;

use super::render_data::static_render_data::StaticRenderData;


pub fn generate_raymarch_shader(static_data: &StaticRenderData) -> String
{
    let original_shader = include_str!("shaders/raymarch_shader.wgsl");
    
    let shader_pieces: Vec<&str> = original_shader.split("//###map###").collect();
    
    let mut shader = String::new();
    
    if shader_pieces.len() == 3
    {
        shader += shader_pieces[0];
        shader += &generate_map_function(static_data);
        shader += shader_pieces[2];
    }
    else
    {
        panic!("Generate raymarch shader Error! pattern //###map### occurs {} times", shader_pieces.len() - 1)
    }

    let shader_pieces: Vec<&str> = shader.split("//###find_intersections###").collect();

    let mut shader = String::new();
    
    if shader_pieces.len() == 3
    {
        shader += shader_pieces[0];
        shader += &generate_find_intersections_function(static_data);
        shader += shader_pieces[2];
    }
    else
    {
        panic!("Generate raymarch shader Error! pattern //###find_intersections### occurs {} times", shader_pieces.len() - 1)
    }

    let shader_pieces: Vec<&str> = shader.split("//###get_mats###").collect();

    let mut shader = String::new();
    
    if shader_pieces.len() == 3
    {
        shader += shader_pieces[0];
        shader += &generate_get_mats_function(static_data);
        shader += shader_pieces[2];
    }
    else
    {
        panic!("Generate raymarch shader Error! pattern //###get_mats### occurs {} times", shader_pieces.len() - 1)
    }
    
    shader
}


fn generate_find_intersections_function(static_data: &StaticRenderData) -> String
{
    let stickiness = static_data.other_static_data.static_shapes_stickiness;
    let mut func_body = String::new();

    func_body +=

    "var rd = rdd;
    if rd.x == 0 {
        rd.x += 0.000001; 
    }
    if rd.y == 0 {
        rd.y += 0.000001; 
    }
    if rd.z == 0 {
        rd.z += 0.000001; 
    }
    if rd.w == 0 {
        rd.w += 0.000001; 
    }\n";

    // normal
    for shape in &static_data.cubes
    {
        func_body +=

        &format!
        (
            "{}let intr = cube_intersection(
                ro - {},
                rd,
                {}
            );\n",
            "{\n",
            string_from_vec4(shape.pos),
            string_from_vec4(add_vec4_and_float(shape.size, shape.roundness)),
        );

        func_body +=

        "if intr.y > 0.0 {
            store_intersection_entrance_and_exit(intr, intrs);
        }\n}\n";
    }

    for shape in &static_data.spheres
    {
        func_body +=

        &format!
        (
            "{}let intr = sph_intersection(
                ro - {},
                rd,
                {}
            );\n",
            "{\n",
            string_from_vec4(shape.pos),
            shape.size[0] + shape.roundness,
        );

        func_body +=

        "if intr.y > 0.0 {
            store_intersection_entrance_and_exit(intr, intrs);
        }\n}\n";
    }

    for shape in &static_data.sph_cubes 
    {
        func_body +=

        &format!
        (
            "{}let intr = cube_intersection(
                ro - {},
                rd,
                {}
            );\n",
            "{\n",
            string_from_vec4(shape.pos),
            string_from_vec4(calc_size_for_sphcube(shape.size, shape.roundness)),
        );

        func_body +=

        "if intr.y > 0.0 {
            store_intersection_entrance_and_exit(intr, intrs);
        }\n}\n";
    }

    // stickiness
    for shape in &static_data.s_cubes
    {
        func_body +=

        &format!
        (
            "{}let intr = cube_intersection(
                ro - {},
                rd,
                {}
            );\n",
            "{\n",
            string_from_vec4(shape.pos),
            string_from_vec4(
                add_vec4_and_float(
                    add_vec4_and_float(shape.size, shape.roundness),
                    stickiness * PI
                )
            ),
        );

        func_body +=

        "if intr.y > 0.0 {
            store_intersection_entrance_and_exit(intr, intrs);
        }\n}\n";
    }

    for shape in &static_data.s_spheres
    {
        func_body +=

        &format!
        (
            "{}let intr = sph_intersection(
                ro - {},
                rd,
                {}
            );\n",
            "{\n",
            string_from_vec4(shape.pos),
            shape.size[0] + shape.roundness + stickiness * PI,
        );

        func_body +=

        "if intr.y > 0.0 {
            store_intersection_entrance_and_exit(intr, intrs);
        }\n}\n";
    }

    for shape in &static_data.s_sph_cubes 
    {
        func_body +=

        &format!
        (
            "{}let intr = cube_intersection(
                ro - {},
                rd,
                {}
            );\n",
            "{\n",
            string_from_vec4(shape.pos),
            string_from_vec4(
                add_vec4_and_float(
                    calc_size_for_sphcube(shape.size, shape.roundness),
                    stickiness * PI
                )
            ),
        );

        func_body +=

        "if intr.y > 0.0 {
            store_intersection_entrance_and_exit(intr, intrs);
        }\n}\n";
    }

    // negative
    for shape in &static_data.neg_cubes
    {
        func_body +=

        &format!
        (
            "{}let intr = cube_intersection(
                ro - {},
                rd,
                {}
            );\n",
            "{\n",
            string_from_vec4(shape.pos),
            string_from_vec4(add_vec4_and_float(shape.size, shape.roundness*0.707106781)),
        );

        func_body +=

        "if intr.y > 0.0 {
            store_intersection_entrance_and_exit_for_neg(intr, intrs);
        }\n}\n";
    }

    for shape in &static_data.neg_spheres
    {
        func_body +=

        &format!
        (
            "{}let intr = sph_intersection(
                ro - {},
                rd,
                {}
            );\n",
            "{\n",
            string_from_vec4(shape.pos),
            shape.size[0] + shape.roundness,
        );

        func_body +=

        "if intr.y > 0.0 {
            store_intersection_entrance_and_exit_for_neg(intr, intrs);
        }\n}\n";
    }

    // negative stickiness
    for shape in &static_data.s_neg_cubes
    {
        func_body +=

        &format!
        (
            "{}let intr = cube_intersection(
                ro - {},
                rd,
                {}
            );\n",
            "{\n",
            string_from_vec4(shape.pos),
            string_from_vec4(add_vec4_and_float(shape.size, shape.roundness*0.707106781)),
        );

        func_body +=

        "if intr.y > 0.0 {
            store_intersection_entrance_and_exit_for_neg(intr, intrs);
        }\n}\n";
    }

    for shape in &static_data.s_neg_spheres
    {
        func_body +=

        &format!
        (
            "{}let intr = sph_intersection(
                ro - {},
                rd,
                {}
            );\n",
            "{\n",
            string_from_vec4(shape.pos),
            shape.size[0] + shape.roundness,
        );

        func_body +=

        "if intr.y > 0.0 {
            store_intersection_entrance_and_exit_for_neg(intr, intrs);
        }\n}\n";
    }

    // undetroyable cubes
    for shape in &static_data.undestroyable_cubes
    {
        func_body +=

        &format!
        (
            "{}let intr = cube_intersection(
                ro - {},
                rd,
                {}
            );\n",
            "{\n",
            string_from_vec4(shape.pos),
            string_from_vec4(add_vec4_and_float(shape.size, shape.roundness)),
        );

        func_body +=

        "if intr.y > 0.0 {
            store_intersection_entrance_and_exit_for_unbreakables(intr, intrs);
        }\n}\n";
    }

    func_body +=

    "for (var i = 0u; i < dynamic_data.shapes_arrays_metadata.spheres_amount; i++) {
        let intr = sph_intersection(
            ro - dyn_normal_shapes[i].pos,
            rd,
            dyn_normal_shapes[i].size.x + dyn_normal_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            store_intersection_entrance_and_exit(intr, intrs);
        }
    }\n";

    func_body +=

    "for (var i = 0u; i < dynamic_data.player_forms_amount; i++) {
        let intr = sph_intersection(
            ro - dyn_player_forms[i].pos,
            rd,
            dyn_player_forms[i].radius * 1.7
        );
        
        if intr.y > 0.0 {
            store_intersection_entrance_and_exit_for_unbreakables(intr, intrs);
        }
    }\n";

    func_body += "combine_interscted_entrances_and_exites_for_all_intrs(intrs);\n";

    func_body
}


fn generate_map_function(static_data: &StaticRenderData) -> String
{
    let stickiness = static_data.other_static_data.static_shapes_stickiness;
    let mut func_body = String::new();

    func_body += "var d = MAX_DIST*2.0;";

    // normal
    for shape in &static_data.cubes
    {
        func_body +=

        &format!
        (
            "d = min(d, sd_box(p - {}, {}) - {});\n",
            string_from_vec4(shape.pos),
            string_from_vec4(shape.size),
            shape.roundness,
        );
    }

    for shape in &static_data.spheres
    {
        func_body +=

        &format!
        (
            "d = min(d, sd_sphere(p - {}, {}) - {});\n",
            string_from_vec4(shape.pos),
            shape.size[0],
            shape.roundness,
        );
    }

    for shape in &static_data.sph_cubes 
    {
        func_body +=

        &format!
        (
            "d = min(d, sd_sph_box(p - {}, {}) - {});\n",
            string_from_vec4(shape.pos),
            string_from_vec4(shape.size),
            shape.roundness,
        );
    }

    // stickiness
    for shape in &static_data.s_cubes
    {
        func_body +=

        &format!
        (
            "d = smin(d, sd_box(p - {}, {}) - {}, {});\n",
            string_from_vec4(shape.pos),
            string_from_vec4(shape.size),
            shape.roundness,
            stickiness,
        );
    }

    for shape in &static_data.s_spheres
    {
        func_body +=

        &format!
        (
            "d = smin(d, sd_sphere(p - {}, {}) - {}, {});\n",
            string_from_vec4(shape.pos),
            shape.size[0],
            shape.roundness,
            stickiness,
        );
    }

    for shape in &static_data.s_sph_cubes 
    {
        func_body +=

        &format!
        (
            "d = smin(d, sd_sph_box(p - {}, {}) - {}, {});\n",
            string_from_vec4(shape.pos),
            string_from_vec4(shape.size),
            shape.roundness,
            stickiness,
        );
    }

    // negative
    for shape in &static_data.neg_cubes
    {
        func_body +=

        &format!
        (
            "d = max(d, -(sd_box(p - {}, {}) - {}));\n",
            string_from_vec4(shape.pos),
            string_from_vec4(shape.size),
            shape.roundness,
        );
    }

    for shape in &static_data.neg_spheres
    {
        func_body +=

        &format!
        (
            "d = max(d, -(sd_sphere(p - {}, {}) - {}));\n",
            string_from_vec4(shape.pos),
            shape.size[0],
            shape.roundness,
        );
    }

    for shape in &static_data.neg_sph_cubes
    {
        func_body +=

        &format!
        (
            "d = max(d, -(sd_sph_box(p - {}, {}) - {}));\n",
            string_from_vec4(shape.pos),
            string_from_vec4(shape.size),
            shape.roundness,
        );
    }


    func_body +=

    "for (var i = 0u; i < dynamic_data.shapes_arrays_metadata.neg_spheres_amount; i++) {
        d = max(d, -(sd_sphere(p - dyn_negatives_shapes[i].pos, dyn_negatives_shapes[i].size.x) - dyn_negatives_shapes[i].roundness));
    }\n";


    // negative stickiness
    for shape in &static_data.s_neg_cubes
    {
        func_body +=

        &format!
        (
            "d = smax(d, -(sd_box(p - {}, {}) - {}), {});\n",
            string_from_vec4(shape.pos),
            string_from_vec4(shape.size),
            shape.roundness,
            stickiness,
        );
    }

    for shape in &static_data.s_neg_spheres
    {
        func_body +=

        &format!
        (
            "d = smax(d, -(sd_sphere(p - {}, {}) - {}), {});\n",
            string_from_vec4(shape.pos),
            shape.size[0],
            shape.roundness,
            stickiness,
        );
    }

    for shape in &static_data.s_neg_sph_cubes
    {
        func_body +=

        &format!
        (
            "d = smax(d, -(sd_sph_box(p - {}, {}) - {}), {});\n",
            string_from_vec4(shape.pos),
            string_from_vec4(shape.size),
            shape.roundness,
            stickiness,
        );
    }

    // undestroyable
    for shape in &static_data.undestroyable_cubes
    {
        func_body +=

        &format!
        (
            "d = min(d, sd_box(p - {}, {}) - {});\n",
            string_from_vec4(shape.pos),
            string_from_vec4(shape.size),
            shape.roundness,
        );
    }

    func_body +=

    "var dddd = MAX_DIST;
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
    d = min(d, dddd);\n";

    func_body += "return d;\n";

    func_body
}


fn generate_get_mats_function(static_data: &StaticRenderData) -> String
{

    let mut func_body = String::new();

    func_body +=

    "var output: OutputMaterials;

    if dist > MAX_DIST-MIN_DIST {

        output.materials_count = 1u;
        output.material_weights[0] = 1.0;
        output.materials[0] = -2;
        return output;
    }

    let p = cam_pos + ray_dir * dist;

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
            if shape.is_red.x == 1
            {
                output.materials[0] = static_data.red_players_mat1;
            } else {
                output.materials[0] = static_data.blue_players_mat1;
            }
            return output;
        }

        d = sd_sphere(
                p - shape.pos,
                shape.radius * 0.6
            );

        let dd = sd_sphere(
                rotated_p - vec4(0.0, 0.0, -shape.radius/2.0, 0.0)*0.6,
                shape.radius * 0.24
            );
        
        d = max(
            d,
            -sd_sphere(
                rotated_p - vec4(0.0, 0.0, -shape.radius, 0.0)*0.6,
                shape.radius * 0.34
            )
        );

        if d < MIN_DIST {
            if dd < 0.0 {
                output.materials_count = 2u;
                output.material_weights[0] = 0.26;
                if shape.is_red.x == 1
                {
                    output.materials[0] = -3;
                } else {
                    output.materials[0] = -4;
                }
                output.material_weights[1] = 0.74;
                if shape.is_red.x == 1
                {
                    output.materials[1] = static_data.red_players_mat2;
                } else {
                    output.materials[1] = static_data.blue_players_mat2;
                }
                return output;
            }
            output.materials_count = 1u;
            output.material_weights[0] = 1.0;
            if shape.is_red.x == 1
            {
                output.materials[0] = static_data.red_players_mat2;
            } else {
                output.materials[0] = static_data.blue_players_mat2;
            }
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
            if shape.is_red.x == 1
            {
                output.materials[0] = static_data.red_players_mat1;
            } else {
                output.materials[0] = static_data.blue_players_mat1;
            }
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
            if shape.is_red.x == 1
            {
                output.materials[0] = static_data.red_players_mat2;
            } else {
                output.materials[0] = static_data.blue_players_mat2;
            }
            return output;
        }
    }\n";

    func_body +=
    
    "var d = MAX_DIST * 2.0;
    output.materials_count = 1u;
    output.material_weights[0] = 1.0;\n";

    // undestroyable
    for shape in &static_data.undestroyable_cubes
    {
        func_body +=

        &format!
        (
            "{}let dd = min(d, sd_box(p - {}, {}) - {});\n",
            "{\n",
            string_from_vec4(shape.pos),
            string_from_vec4(shape.size),
            shape.roundness,
        );

        func_body +=

        &format!
        (
            "if dd < MIN_DIST*2.0 {}
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = {};
                return output;
            {}
            
            if dd < d {}
                d = dd;
                output.materials[0] = {};
            {}",

            "{",
            shape.material,
            "}",
            "{",
            shape.material,
            "}\n}\n",
        );
    }

    // normal
    for shape in &static_data.cubes
    {
        func_body +=

        &format!
        (
            "{}let dd = min(d, sd_box(p - {}, {}) - {});\n",
            "{\n",
            string_from_vec4(shape.pos),
            string_from_vec4(shape.size),
            shape.roundness,
        );

        func_body +=

        &format!
        (
            "if dd < MIN_DIST*2.0 {}
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = {};
                return output;
            {}
            
            if dd < d {}
                d = dd;
                output.materials[0] = {};
            {}",

            "{",
            shape.material,
            "}",
            "{",
            shape.material,
            "}\n}\n",
        );
    }

    for shape in &static_data.spheres
    {
        func_body +=

        &format!
        (
            "{}let dd = min(d, sd_sphere(p - {}, {}) - {});\n",
            "{\n",
            string_from_vec4(shape.pos),
            shape.size[0],
            shape.roundness,
        );

        func_body +=

        &format!
        (
            "if dd < MIN_DIST*2.0 {}
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = {};
                return output;
            {}
            
            if dd < d {}
                d = dd;
                output.materials[0] = {};
            {}",

            "{",
            shape.material,
            "}",
            "{",
            shape.material,
            "}\n}\n",
        );
    }

    for shape in &static_data.sph_cubes 
    {
        func_body +=

        &format!
        (
            "{}let dd = min(d, sd_sph_box(p - {}, {}) - {});\n",
            "{\n",
            string_from_vec4(shape.pos),
            string_from_vec4(shape.size),
            shape.roundness,
        );

        func_body +=

        &format!
        (
            "if dd < MIN_DIST*2.0 {}
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = {};
                return output;
            {}
            
            if dd < d {}
                d = dd;
                output.materials[0] = {};
            {}",

            "{",
            shape.material,
            "}",
            "{",
            shape.material,
            "}\n}\n",
        );
    }

    // stickiness
    for shape in &static_data.s_cubes
    {
        func_body +=

        &format!
        (
            "{}let dd = min(d, sd_box(p - {}, {}) - {});\n",
            "{\n",
            string_from_vec4(shape.pos),
            string_from_vec4(shape.size),
            shape.roundness,
        );

        func_body +=


        &format!
        (
            "if dd < MIN_DIST*2.0 {}
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = {};
                return output;
            {}

            if dd < static_data.stickiness * STICKINESS_EFFECT_COEF {}
                if output.materials_count == 0u {}
                    output.materials_count = 1u;
                    output.material_weights[0] = 1.0;
                    output.materials[0] = {};
                    d = dd;
                {} else {}
                    var coef = 0.0;
                    if d<dd {}
                        coef = clamp(pow(d/dd,1.9) * 0.5, 0.0, 1.0);
                    {} else {}
                        coef = 1.0-clamp((pow(dd/d,1.9) * 0.5), 0.0, 1.0);
                    {}
                    output.materials[output.materials_count] = {};
                    output.material_weights[output.materials_count] = coef;

                    let mult = 1.0 - coef;

                    for (var k = 0u; k < output.materials_count; k++) {}
                        output.material_weights[k] *= mult;
                    {}

                    output.materials_count += 1u;
                    d = min(d,dd);
                {}
            {}",

            "{",
            shape.material,
            "}",
            "{",
            "{",
            shape.material,
            "}",
            "{",
            "{",
            "}",
            "{",
            "}",
            shape.material,
            "{",
            "}",
            "}",
            "}\n}\n",
        );
    }

    for shape in &static_data.s_spheres
    {
        func_body +=

        &format!
        (
            "{}let dd = min(d, sd_sphere(p - {}, {}) - {});\n",
            "{\n",
            string_from_vec4(shape.pos),
            shape.size[0],
            shape.roundness,
        );

        func_body +=

        &format!
        (
            "if dd < MIN_DIST*2.0 {}
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = {};
                return output;
            {}

            if dd < static_data.stickiness * STICKINESS_EFFECT_COEF {}
                if output.materials_count == 0u {}
                    output.materials_count = 1u;
                    output.material_weights[0] = 1.0;
                    output.materials[0] = {};
                    d = dd;
                {} else {}
                    var coef = 0.0;
                    if d<dd {}
                        coef = clamp(pow(d/dd,1.9) * 0.5, 0.0, 1.0);
                    {} else {}
                        coef = 1.0-clamp((pow(dd/d,1.9) * 0.5), 0.0, 1.0);
                    {}
                    output.materials[output.materials_count] = {};
                    output.material_weights[output.materials_count] = coef;

                    let mult = 1.0 - coef;

                    for (var k = 0u; k < output.materials_count; k++) {}
                        output.material_weights[k] *= mult;
                    {}

                    output.materials_count += 1u;
                    d = min(d,dd);
                {}
            {}",

            "{",
            shape.material,
            "}",
            "{",
            "{",
            shape.material,
            "}",
            "{",
            "{",
            "}",
            "{",
            "}",
            shape.material,
            "{",
            "}",
            "}",
            "}\n}\n",
        );
    }

    for shape in &static_data.s_sph_cubes 
    {
        func_body +=

        &format!
        (
            "{}let dd = min(d, sd_sph_box(p - {}, {}) - {});\n",
            "{\n",
            string_from_vec4(shape.pos),
            string_from_vec4(shape.size),
            shape.roundness,
        );

        func_body +=

        &format!
        (
            "if dd < MIN_DIST*2.0 {}
                output.materials_count = 1u;
                output.material_weights[0] = 1.0;
                output.materials[0] = {};
                return output;
            {}

            if dd < static_data.stickiness * STICKINESS_EFFECT_COEF {}
                if output.materials_count == 0u {}
                    output.materials_count = 1u;
                    output.material_weights[0] = 1.0;
                    output.materials[0] = {};
                    d = dd;
                {} else {}
                    var coef = 0.0;
                    if d<dd {}
                        coef = clamp(pow(d/dd,1.9) * 0.5, 0.0, 1.0);
                    {} else {}
                        coef = 1.0-clamp((pow(dd/d,1.9) * 0.5), 0.0, 1.0);
                    {}
                    output.materials[output.materials_count] = {};
                    output.material_weights[output.materials_count] = coef;

                    let mult = 1.0 - coef;

                    for (var k = 0u; k < output.materials_count; k++) {}
                        output.material_weights[k] *= mult;
                    {}

                    output.materials_count += 1u;
                    d = min(d,dd);
                {}
            {}",

            "{",
            shape.material,
            "}",
            "{",
            "{",
            shape.material,
            "}",
            "{",
            "{",
            "}",
            "{",
            "}",
            shape.material,
            "{",
            "}",
            "}",
            "}\n}\n",
        );
    }

    func_body += "return output;\n";

    func_body
}


fn string_from_vec4(vec: [f32; 4]) -> String
{
    format!("vec4<f32>({}, {}, {}, {})", vec[0], vec[1], vec[2], vec[3])
}


fn add_two_vec4(mut vec1: [f32; 4], vec2: [f32; 4]) -> [f32; 4]
{
    vec1[0] += vec2[0];
    vec1[1] += vec2[1];
    vec1[2] += vec2[2];
    vec1[3] += vec2[3];

    vec1
}


fn add_vec4_and_float(mut vec: [f32; 4], float: f32) -> [f32; 4]
{
    vec[0] += float;
    vec[1] += float;
    vec[2] += float;
    vec[3] += float;

    vec
}


fn calc_size_for_sphcube(size: [f32; 4], roundness: f32) -> [f32; 4]
{
    [
        (size[1].min(size[2])).min(size[3]) + roundness,
        (size[0].min(size[2])).min(size[3]) + roundness,
        (size[1].min(size[0])).min(size[3]) + roundness,
        size[3] + roundness
    ]
}