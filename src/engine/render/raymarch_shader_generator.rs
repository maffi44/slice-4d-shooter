use core::panic;
use std::{cmp::Ordering, f32::consts::PI};

use glam::Vec2;

use crate::engine::render::render_data::Shape;

use super::render_data::static_render_data::StaticRenderData;


const MAX_BSP_TREE_DEPTH: usize = 8;
const MIN_BSP_DIVISION_EFFICIENCY: usize = 2;

pub fn generate_raymarch_shader_with_static_bsp_tree(original_shader: &str, static_data: &StaticRenderData) -> String
{
    let bsp_tree = Box::new(BSPElement::create_binary_space_partition_tree(static_data));

    let shader_pieces: Vec<&str> = original_shader.split("//###map###").collect();
    
    let mut shader = String::new();
    
    if shader_pieces.len() == 3
    {
        shader += shader_pieces[0];
        shader += &generate_map_function_body(&bsp_tree);
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
        shader += &generate_find_intersections_function_body(static_data);
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
        shader += &generate_get_mats_function_body(&bsp_tree);
        shader += shader_pieces[2];
    }
    else
    {
        panic!("Generate raymarch shader Error! pattern //###get_mats### occurs {} times", shader_pieces.len() - 1)
    }
    
    shader
}


fn generate_find_intersections_function_body(static_data: &StaticRenderData) -> String
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
            string_from_vec4(add_vec4_and_float(shape.size, shape.roundness*0.707106781*0.88)),
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
            string_from_vec4(add_vec4_and_float(shape.size, shape.roundness*0.707106781*0.88)),
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

    "for (var i = dynamic_data.shapes_arrays_metadata.neg_spheres_start; i < dynamic_data.shapes_arrays_metadata.neg_spheres_start + dynamic_data.shapes_arrays_metadata.neg_spheres_amount; i++) {
        let intr = sph_intersection(
            ro - dyn_negatives_shapes[i].pos,
            rd,
            dyn_negatives_shapes[i].size.x + dyn_negatives_shapes[i].roundness
        );
        
        if intr.y > 0.0 {
            store_intersection_entrance_and_exit_for_neg(intr, intrs);
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
            (*intrs).intr_players = true;
            store_intersection_entrance_and_exit_for_unbreakables(intr, intrs);
        }
    }\n";

    func_body += "combine_interscted_entrances_and_exites_for_all_intrs(intrs);\n";

    func_body
}


fn generate_map_function_body(bsp_tree: &Box<BSPElement>) -> String
{
    let mut func_body = String::new();

    func_body += "var d = MAX_DIST*2.0;";

    write_bsp_tree_content_for_map_func(&mut func_body, bsp_tree);

    func_body
}


fn generate_get_mats_function_body_simple_for_gebug(static_data: &StaticRenderData) -> String
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

    output.materials_count = 1u;
    output.material_weights[0] = 1.0;
    output.materials[0] = 3;
    return output;";

    func_body
}


fn write_bsp_tree_content_for_get_mats_func(func_body: &mut String, bsp_elem: &Box<BSPElement>)
{
    match bsp_elem.as_ref() {
        BSPElement::Branches(slice, left_branch, right_branch) => 
        {
            let (axis, value) = match slice
            {
                Slice::X(value) => ("x", value),
                Slice::Y(value) => ("y", value),
                Slice::Z(value) => ("z", value),
                Slice::W(value) => ("w", value),
            };

            func_body.push_str(
                &format!
                (
                    "if p.{} > {} {}\n",
                    axis,
                    value,
                    "{"
                )
            );

            write_bsp_tree_content_for_get_mats_func(func_body, right_branch);

            func_body.push_str("}\nelse\n{");

            write_bsp_tree_content_for_get_mats_func(func_body, left_branch);

            func_body.push_str("}");

        },
        BSPElement::Leaf(objects) =>
        {
            let stickiness = objects.stickiness;
            
            for obj in &objects.undestroyable_cubes
            {
                func_body.push_str(&format!
                (
                    "{}let dd = min(d, sd_box(p - {}, {}) - {});\n",
                    "{\n",
                    string_from_vec4(obj.shape.pos),
                    string_from_vec4(obj.shape.size),
                    obj.shape.roundness,
                ));

                func_body.push_str(&format!
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
                    obj.shape.material,
                    "}",
                    "{",
                    obj.shape.material,
                    "}\n}\n",
                ));
            }

            // normal
            for obj in &objects.cubes
            {

                func_body.push_str(&format!
                (
                    "{}let dd = min(d, sd_box(p - {}, {}) - {});\n",
                    "{\n",
                    string_from_vec4(obj.shape.pos),
                    string_from_vec4(obj.shape.size),
                    obj.shape.roundness,
                ));

                func_body.push_str(&format!
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
                    obj.shape.material,
                    "}",
                    "{",
                    obj.shape.material,
                    "}\n}\n",
                ));
            }

            for obj in &objects.spheres
            {
                func_body.push_str(&format!
                (
                    "{}let dd = min(d, sd_sphere(p - {}, {}) - {});\n",
                    "{\n",
                    string_from_vec4(obj.shape.pos),
                    obj.shape.size[0],
                    obj.shape.roundness,
                ));

                func_body.push_str(&format!
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
                    obj.shape.material,
                    "}",
                    "{",
                    obj.shape.material,
                    "}\n}\n",
                ));
            }

            for obj in &objects.sph_cubes 
            {

                func_body.push_str(&format!
                (
                    "{}let dd = min(d, sd_sph_box(p - {}, {}) - {});\n",
                    "{\n",
                    string_from_vec4(obj.shape.pos),
                    string_from_vec4(obj.shape.size),
                    obj.shape.roundness,
                ));

                func_body.push_str(&format!
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
                    obj.shape.material,
                    "}",
                    "{",
                    obj.shape.material,
                    "}\n}\n",
                ));
            }

            // stickiness
            for obj in &objects.s_cubes
            {
                func_body.push_str(&format!
                (
                    "{}let dd = min(d, sd_box(p - {}, {}) - {});\n",
                    "{\n",
                    string_from_vec4(obj.shape.pos),
                    string_from_vec4(obj.shape.size),
                    obj.shape.roundness,
                ));

                func_body.push_str(&format!
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
                    obj.shape.material,
                    "}",
                    "{",
                    "{",
                    obj.shape.material,
                    "}",
                    "{",
                    "{",
                    "}",
                    "{",
                    "}",
                    obj.shape.material,
                    "{",
                    "}",
                    "}",
                    "}\n}\n",
                ));
            }

            for obj in &objects.s_spheres
            {
                func_body.push_str(&format!
                (
                    "{}let dd = min(d, sd_sphere(p - {}, {}) - {});\n",
                    "{\n",
                    string_from_vec4(obj.shape.pos),
                    obj.shape.size[0],
                    obj.shape.roundness,
                ));

                func_body.push_str(&format!
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
                    obj.shape.material,
                    "}",
                    "{",
                    "{",
                    obj.shape.material,
                    "}",
                    "{",
                    "{",
                    "}",
                    "{",
                    "}",
                    obj.shape.material,
                    "{",
                    "}",
                    "}",
                    "}\n}\n",
                ));
            }

            for obj in &objects.s_sph_cubes 
            {
                func_body.push_str(&format!
                (
                    "{}let dd = min(d, sd_sph_box(p - {}, {}) - {});\n",
                    "{\n",
                    string_from_vec4(obj.shape.pos),
                    string_from_vec4(obj.shape.size),
                    obj.shape.roundness,
                ));

                func_body.push_str(&format!
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
                    obj.shape.material,
                    "}",
                    "{",
                    "{",
                    obj.shape.material,
                    "}",
                    "{",
                    "{",
                    "}",
                    "{",
                    "}",
                    obj.shape.material,
                    "{",
                    "}",
                    "}",
                    "}\n}\n",
                ));
            }
        },
    }
}


fn generate_get_mats_function_body(bsp_tree: &Box<BSPElement>) -> String
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

    write_bsp_tree_content_for_get_mats_func(&mut func_body, bsp_tree);

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


fn write_bsp_tree_content_for_map_func(func_body: &mut String, bsp_elem: &Box<BSPElement>)
{
    match bsp_elem.as_ref() {
        BSPElement::Branches(slice, left_branch, right_branch) => 
        {
            let (axis, value) = match slice
            {
                Slice::X(value) => ("x", value),
                Slice::Y(value) => ("y", value),
                Slice::Z(value) => ("z", value),
                Slice::W(value) => ("w", value),
            };

            func_body.push_str(
                &format!
                (
                    "if p.{} > {} {}\n",
                    axis,
                    value,
                    "{"
                )
            );

            write_bsp_tree_content_for_map_func(func_body, right_branch);

            func_body.push_str("}\nelse\n{");

            write_bsp_tree_content_for_map_func(func_body, left_branch);

            func_body.push_str("}");

        },
        BSPElement::Leaf(objects) =>
        {
            let stickiness = objects.stickiness;
            
            for obj in &objects.cubes
            {
                func_body.push_str(
                    &format!
                    (
                        "d = min(d, sd_box(p - {}, {}) - {});\n",
                        string_from_vec4(obj.shape.pos),
                        string_from_vec4(obj.shape.size),
                        obj.shape.roundness,
                    )
                );

            }

            for obj in &objects.spheres
            {
                func_body.push_str(
                    &format!
                    (
                        "d = min(d, sd_sphere(p - {}, {}) - {});\n",
                        string_from_vec4(obj.shape.pos),
                        obj.shape.size[0],
                        obj.shape.roundness,
                    )
                );
            }

            for obj in &objects.sph_cubes 
            {
                func_body.push_str(
                    &format!
                    (
                        "d = min(d, sd_sph_box(p - {}, {}) - {});\n",
                        string_from_vec4(obj.shape.pos),
                        string_from_vec4(obj.shape.size),
                        obj.shape.roundness,
                    )
                );
            }

            // stickiness
            for obj in &objects.s_cubes
            {
                func_body.push_str(
                    &format!
                    (
                        "d = smin(d, sd_box(p - {}, {}) - {}, {});\n",
                        string_from_vec4(obj.shape.pos),
                        string_from_vec4(obj.shape.size),
                        obj.shape.roundness,
                        stickiness,
                    )
                );
            }

            for obj in &objects.s_spheres
            {
                func_body.push_str(
                    &format!
                    (
                        "d = smin(d, sd_sphere(p - {}, {}) - {}, {});\n",
                        string_from_vec4(obj.shape.pos),
                        obj.shape.size[0],
                        obj.shape.roundness,
                        stickiness,
                    )
                );
            }

            for obj in &objects.s_sph_cubes 
            {
                func_body.push_str(
                    &format!
                    (
                        "d = smin(d, sd_sph_box(p - {}, {}) - {}, {});\n",
                        string_from_vec4(obj.shape.pos),
                        string_from_vec4(obj.shape.size),
                        obj.shape.roundness,
                        stickiness,
                    )
                );
            }

            // negative
            for obj in &objects.neg_cubes
            {
                func_body.push_str(
                    &format!
                    (
                        "d = max(d, -(sd_box(p - {}, {}) - {}));\n",
                        string_from_vec4(obj.shape.pos),
                        string_from_vec4(obj.shape.size),
                        obj.shape.roundness,
                    )
                );
            }

            for obj in &objects.neg_spheres
            {
                func_body.push_str(
                    &format!
                    (
                        "d = max(d, -(sd_sphere(p - {}, {}) - {}));\n",
                        string_from_vec4(obj.shape.pos),
                        obj.shape.size[0],
                        obj.shape.roundness,
                    )
                );
            }

            for obj in &objects.neg_sph_cubes
            {
                func_body.push_str(
                    &format!
                    (
                        "d = max(d, -(sd_sph_box(p - {}, {}) - {}));\n",
                        string_from_vec4(obj.shape.pos),
                        string_from_vec4(obj.shape.size),
                        obj.shape.roundness,
                    )
                );
            }

            func_body.push_str(
                "for (var i = dynamic_data.shapes_arrays_metadata.neg_spheres_start; i < dynamic_data.shapes_arrays_metadata.neg_spheres_start + dynamic_data.shapes_arrays_metadata.neg_spheres_amount; i++) {
                    d = max(d, -(sd_sphere(p - dyn_negatives_shapes[i].pos, dyn_negatives_shapes[i].size.x) - dyn_negatives_shapes[i].roundness));
                }\n"
            );

            // negative stickiness
            for obj in &objects.s_neg_cubes
            {
                func_body.push_str(
                    &format!
                    (
                        "d = smax(d, -(sd_box(p - {}, {}) - {}), {});\n",
                        string_from_vec4(obj.shape.pos),
                        string_from_vec4(obj.shape.size),
                        obj.shape.roundness,
                        stickiness,
                    )
                );
            }

            for obj in &objects.s_neg_spheres
            {
                func_body.push_str(
                    &format!
                    (
                        "d = smax(d, -(sd_sphere(p - {}, {}) - {}), {});\n",
                        string_from_vec4(obj.shape.pos),
                        obj.shape.size[0],
                        obj.shape.roundness,
                        stickiness,
                    )
                );
            }

            for obj in &objects.s_neg_sph_cubes
            {
                func_body.push_str(
                    &format!
                    (
                        "d = smax(d, -(sd_sph_box(p - {}, {}) - {}), {});\n",
                        string_from_vec4(obj.shape.pos),
                        string_from_vec4(obj.shape.size),
                        obj.shape.roundness,
                        stickiness,
                    )
                );
            }

            // undestroyable
            for obj in &objects.undestroyable_cubes
            {
                func_body.push_str(
                    &format!
                    (
                        "d = min(d, sd_box(p - {}, {}) - {});\n",
                        string_from_vec4(obj.shape.pos),
                        string_from_vec4(obj.shape.size),
                        obj.shape.roundness,
                    )
                );
            }
        },
    }
}

enum BSPElement
{
    Branches(Slice, Box<BSPElement>, Box<BSPElement>),
    Leaf(Objects)
}

impl BSPElement
{
    pub fn create_binary_space_partition_tree(static_data: &StaticRenderData) -> Self
    {
        let objects = Objects::init(static_data);

        if objects.len() == 0
        {
            BSPElement::Leaf(objects)
        }
        else
        {
            split_leaf(objects, 0)
        }
    }
}

fn split_leaf
(
    objects: Objects,
    current_depth: usize
) -> BSPElement
{
    if current_depth >= MAX_BSP_TREE_DEPTH
    {
        return BSPElement::Leaf(objects);
    }
    else
    {
        let silce_info = objects.find_best_dividing_slice();

        if silce_info.original_amount_of_objects -
            silce_info.left_branch_objects_amount
            .max
            (
                silce_info.right_branch_objects_amount
            )
            <
            MIN_BSP_DIVISION_EFFICIENCY
        {
            return BSPElement::Leaf(objects);
        }
        else
        {
            let (left_objects, right_objects) = objects
                .divide_by_slice(silce_info.slice);
            

            return BSPElement::Branches
            (
                silce_info.slice,
                Box::new(split_leaf(left_objects, current_depth+1)),
                Box::new(split_leaf(right_objects, current_depth+1))
            );
        }
    }
}

struct Objects
{
    pub stickiness: f32,

    pub cubes: Vec<Object>,
    pub s_cubes: Vec<Object>,
    pub neg_cubes: Vec<Object>,
    pub s_neg_cubes: Vec<Object>,
    pub spheres: Vec<Object>,
    pub s_spheres: Vec<Object>,
    pub neg_spheres: Vec<Object>,
    pub s_neg_spheres: Vec<Object>,
    pub sph_cubes: Vec<Object>,
    pub s_sph_cubes: Vec<Object>,
    pub neg_sph_cubes: Vec<Object>,
    pub s_neg_sph_cubes: Vec<Object>,
    pub undestroyable_cubes: Vec<Object>,

    pub object_edges_list_along_x: Vec<f32>,
    pub object_edges_list_along_y: Vec<f32>,
    pub object_edges_list_along_z: Vec<f32>,
    pub object_edges_list_along_w: Vec<f32>,
}


#[derive(Clone, Copy)]
pub struct SliceInfo
{
    pub slice: Slice,
    pub original_amount_of_objects: usize,
    pub left_branch_objects_amount: usize,
    pub right_branch_objects_amount: usize,
}


#[derive(Clone, Copy)]
enum Slice
{
    X(f32),
    Y(f32),
    Z(f32),
    W(f32)
}


impl Slice
{
    pub fn set_slice_pos(&mut self, pos: f32)
    {
        match self
        {
            Slice::X(p) => *p = pos,
            Slice::Y(p) => *p = pos,
            Slice::Z(p) => *p = pos,
            Slice::W(p) => *p = pos,
        }
    }
}

impl Objects
{

    pub fn len(&self) -> usize
    {
        let mut len = 0;

        len += self.cubes.len();
        len += self.s_cubes.len();
        len += self.neg_cubes.len();
        len += self.s_neg_cubes.len();
        len += self.spheres.len();
        len += self.s_spheres.len();
        len += self.neg_spheres.len();
        len += self.s_neg_spheres.len();
        len += self.sph_cubes.len();
        len += self.s_sph_cubes.len();
        len += self.neg_sph_cubes.len();
        len += self.s_neg_sph_cubes.len();
        len += self.undestroyable_cubes.len();

        len  
    }
    

    pub fn find_best_dividing_slice(&self) -> SliceInfo
    {
        let mut slices = Vec::new();

        slices.push(self.find_best_dividing_slice_along_axis(Slice::X(0.0))); 
        slices.push(self.find_best_dividing_slice_along_axis(Slice::Y(0.0))); 
        slices.push(self.find_best_dividing_slice_along_axis(Slice::Z(0.0))); 
        slices.push(self.find_best_dividing_slice_along_axis(Slice::W(0.0)));

        slices.sort_by(|a, b| {
            if a.left_branch_objects_amount.max(a.right_branch_objects_amount) >
                b.left_branch_objects_amount.max(b.right_branch_objects_amount)
            {
                Ordering::Greater
            }
            else if a.left_branch_objects_amount.max(a.right_branch_objects_amount) <
                b.left_branch_objects_amount.max(b.right_branch_objects_amount)
            {
                Ordering::Less
            }
            else
            {
                Ordering::Equal
            }
        });

        slices[0]
    }


    fn find_best_dividing_slice_along_axis(&self, mut slice: Slice) -> SliceInfo
    {
        let edges_list = match slice
        {
            Slice::X(_) => &self.object_edges_list_along_x,
            Slice::Y(_) => &self.object_edges_list_along_y,
            Slice::Z(_) => &self.object_edges_list_along_z,
            Slice::W(_) => &self.object_edges_list_along_w,
        };

        let mut index = 0;

        let mut slices = Vec::new();

        while index < edges_list.len() - 1
        {
            let slice_pos = (edges_list[index] + edges_list[index+1]) / 2.0;
    
            slice.set_slice_pos(slice_pos);
    
            slices.push(self.get_slice_info(slice));

            index += 1;
        }

        slices.sort_by(|a, b| {
            if a.left_branch_objects_amount.max(a.right_branch_objects_amount) >
                b.left_branch_objects_amount.max(b.right_branch_objects_amount)
            {
                Ordering::Greater
            }
            else if a.left_branch_objects_amount.max(a.right_branch_objects_amount) <
                b.left_branch_objects_amount.max(b.right_branch_objects_amount)
            {
                Ordering::Less
            }
            else
            {
                Ordering::Equal
            }
        });

        slices[0]
    }


    pub fn get_slice_info(&self, slice: Slice) -> SliceInfo
    {
        let mut slice_info = SliceInfo {
            slice,
            original_amount_of_objects: self.len(),
            left_branch_objects_amount: 0usize,
            right_branch_objects_amount: 0usize
        };

        for obj in &self.cubes
        {
            match obj.get_side_after_slice(slice)
            {
                SideAfterSlice::Left => {
                    slice_info.left_branch_objects_amount += 1;
                },
                SideAfterSlice::Right => {
                    slice_info.right_branch_objects_amount += 1;
                },
                SideAfterSlice::Both => {
                    slice_info.left_branch_objects_amount += 1;
                    slice_info.right_branch_objects_amount += 1;
                },
            }
        }
        for obj in &self.neg_cubes
        {
            match obj.get_side_after_slice(slice)
            {
                SideAfterSlice::Left => {
                    slice_info.left_branch_objects_amount += 1;
                },
                SideAfterSlice::Right => {
                    slice_info.right_branch_objects_amount += 1;
                },
                SideAfterSlice::Both => {
                    slice_info.left_branch_objects_amount += 1;
                    slice_info.right_branch_objects_amount += 1;
                },
            }
        }
        for obj in &self.s_cubes
        {
            match obj.get_side_after_slice(slice)
            {
                SideAfterSlice::Left => {
                    slice_info.left_branch_objects_amount += 1;
                },
                SideAfterSlice::Right => {
                    slice_info.right_branch_objects_amount += 1;
                },
                SideAfterSlice::Both => {
                    slice_info.left_branch_objects_amount += 1;
                    slice_info.right_branch_objects_amount += 1;
                },
            }
        }
        for obj in &self.s_neg_cubes
        {
            match obj.get_side_after_slice(slice)
            {
                SideAfterSlice::Left => {
                    slice_info.left_branch_objects_amount += 1;
                },
                SideAfterSlice::Right => {
                    slice_info.right_branch_objects_amount += 1;
                },
                SideAfterSlice::Both => {
                    slice_info.left_branch_objects_amount += 1;
                    slice_info.right_branch_objects_amount += 1;
                },
            }
        }

        for obj in &self.spheres
        {
            match obj.get_side_after_slice(slice)
            {
                SideAfterSlice::Left => {
                    slice_info.left_branch_objects_amount += 1;
                },
                SideAfterSlice::Right => {
                    slice_info.right_branch_objects_amount += 1;
                },
                SideAfterSlice::Both => {
                    slice_info.left_branch_objects_amount += 1;
                    slice_info.right_branch_objects_amount += 1;
                },
            }
        }
        for obj in &self.neg_spheres
        {
            match obj.get_side_after_slice(slice)
            {
                SideAfterSlice::Left => {
                    slice_info.left_branch_objects_amount += 1;
                },
                SideAfterSlice::Right => {
                    slice_info.right_branch_objects_amount += 1;
                },
                SideAfterSlice::Both => {
                    slice_info.left_branch_objects_amount += 1;
                    slice_info.right_branch_objects_amount += 1;
                },
            }
        }
        for obj in &self.s_spheres
        {
            match obj.get_side_after_slice(slice)
            {
                SideAfterSlice::Left => {
                    slice_info.left_branch_objects_amount += 1;
                },
                SideAfterSlice::Right => {
                    slice_info.right_branch_objects_amount += 1;
                },
                SideAfterSlice::Both => {
                    slice_info.left_branch_objects_amount += 1;
                    slice_info.right_branch_objects_amount += 1;
                },
            }
        }
        for obj in &self.s_neg_spheres
        {
            match obj.get_side_after_slice(slice)
            {
                SideAfterSlice::Left => {
                    slice_info.left_branch_objects_amount += 1;
                },
                SideAfterSlice::Right => {
                    slice_info.right_branch_objects_amount += 1;
                },
                SideAfterSlice::Both => {
                    slice_info.left_branch_objects_amount += 1;
                    slice_info.right_branch_objects_amount += 1;
                },
            }
        }

        for obj in &self.sph_cubes
        {
            match obj.get_side_after_slice(slice)
            {
                SideAfterSlice::Left => {
                    slice_info.left_branch_objects_amount += 1;
                },
                SideAfterSlice::Right => {
                    slice_info.right_branch_objects_amount += 1;
                },
                SideAfterSlice::Both => {
                    slice_info.left_branch_objects_amount += 1;
                    slice_info.right_branch_objects_amount += 1;
                },
            }
        }
        for obj in &self.neg_sph_cubes
        {
            match obj.get_side_after_slice(slice)
            {
                SideAfterSlice::Left => {
                    slice_info.left_branch_objects_amount += 1;
                },
                SideAfterSlice::Right => {
                    slice_info.right_branch_objects_amount += 1;
                },
                SideAfterSlice::Both => {
                    slice_info.left_branch_objects_amount += 1;
                    slice_info.right_branch_objects_amount += 1;
                },
            }
        }
        for obj in &self.s_sph_cubes
        {
            match obj.get_side_after_slice(slice)
            {
                SideAfterSlice::Left => {
                    slice_info.left_branch_objects_amount += 1;
                },
                SideAfterSlice::Right => {
                    slice_info.right_branch_objects_amount += 1;
                },
                SideAfterSlice::Both => {
                    slice_info.left_branch_objects_amount += 1;
                    slice_info.right_branch_objects_amount += 1;
                },
            }
        }
        for obj in &self.s_neg_sph_cubes
        {
            match obj.get_side_after_slice(slice)
            {
                SideAfterSlice::Left => {
                    slice_info.left_branch_objects_amount += 1;
                },
                SideAfterSlice::Right => {
                    slice_info.right_branch_objects_amount += 1;
                },
                SideAfterSlice::Both => {
                    slice_info.left_branch_objects_amount += 1;
                    slice_info.right_branch_objects_amount += 1;
                },
            }
        }

        for obj in &self.undestroyable_cubes
        {
            match obj.get_side_after_slice(slice)
            {
                SideAfterSlice::Left => {
                    slice_info.left_branch_objects_amount += 1;
                },
                SideAfterSlice::Right => {
                    slice_info.right_branch_objects_amount += 1;
                },
                SideAfterSlice::Both => {
                    slice_info.left_branch_objects_amount += 1;
                    slice_info.right_branch_objects_amount += 1;
                },
            }
        }

        slice_info
    }


    pub fn divide_by_slice(self, slice: Slice) -> (Self, Self)
    {
        let mut left_objects = Objects::new_empty(self.stickiness);
        let mut right_objects = Objects::new_empty(self.stickiness);

        for obj in self.cubes
        {
            match obj.get_side_after_slice(slice)
            {
                SideAfterSlice::Left => {
                    left_objects.cubes.push(obj);
                },
                SideAfterSlice::Right => {
                    right_objects.cubes.push(obj);
                },
                SideAfterSlice::Both => {
                    left_objects.cubes.push(obj.clone());
                    right_objects.cubes.push(obj.clone());
                },
            }
        }
        for obj in self.neg_cubes
        {
            match obj.get_side_after_slice(slice)
            {
                SideAfterSlice::Left => {
                    left_objects.neg_cubes.push(obj);
                },
                SideAfterSlice::Right => {
                    right_objects.neg_cubes.push(obj);
                },
                SideAfterSlice::Both => {
                    left_objects.neg_cubes.push(obj.clone());
                    right_objects.neg_cubes.push(obj.clone());
                },
            }
        }
        for obj in self.s_cubes
        {
            match obj.get_side_after_slice(slice)
            {
                SideAfterSlice::Left => {
                    left_objects.s_cubes.push(obj);
                },
                SideAfterSlice::Right => {
                    right_objects.s_cubes.push(obj);
                },
                SideAfterSlice::Both => {
                    left_objects.s_cubes.push(obj.clone());
                    right_objects.s_cubes.push(obj.clone());
                },
            }
        }
        for obj in self.s_neg_cubes
        {
            match obj.get_side_after_slice(slice)
            {
                SideAfterSlice::Left => {
                    left_objects.s_neg_cubes.push(obj);
                },
                SideAfterSlice::Right => {
                    right_objects.s_neg_cubes.push(obj);
                },
                SideAfterSlice::Both => {
                    left_objects.s_neg_cubes.push(obj.clone());
                    right_objects.s_neg_cubes.push(obj.clone());
                },
            }
        }

        for obj in self.spheres
        {
            match obj.get_side_after_slice(slice)
            {
                SideAfterSlice::Left => {
                    left_objects.spheres.push(obj);
                },
                SideAfterSlice::Right => {
                    right_objects.spheres.push(obj);
                },
                SideAfterSlice::Both => {
                    left_objects.spheres.push(obj.clone());
                    right_objects.spheres.push(obj.clone());
                },
            }
        }
        for obj in self.neg_spheres
        {
            match obj.get_side_after_slice(slice)
            {
                SideAfterSlice::Left => {
                    left_objects.neg_spheres.push(obj);
                },
                SideAfterSlice::Right => {
                    right_objects.neg_spheres.push(obj);
                },
                SideAfterSlice::Both => {
                    left_objects.neg_spheres.push(obj.clone());
                    right_objects.neg_spheres.push(obj.clone());
                },
            }
        }
        for obj in self.s_spheres
        {
            match obj.get_side_after_slice(slice)
            {
                SideAfterSlice::Left => {
                    left_objects.s_spheres.push(obj);
                },
                SideAfterSlice::Right => {
                    right_objects.s_spheres.push(obj);
                },
                SideAfterSlice::Both => {
                    left_objects.s_spheres.push(obj.clone());
                    right_objects.s_spheres.push(obj.clone());
                },
            }
        }
        for obj in self.s_neg_spheres
        {
            match obj.get_side_after_slice(slice)
            {
                SideAfterSlice::Left => {
                    left_objects.s_neg_spheres.push(obj);
                },
                SideAfterSlice::Right => {
                    right_objects.s_neg_spheres.push(obj);
                },
                SideAfterSlice::Both => {
                    left_objects.s_neg_spheres.push(obj.clone());
                    right_objects.s_neg_spheres.push(obj.clone());
                },
            }
        }

        for obj in self.sph_cubes
        {
            match obj.get_side_after_slice(slice)
            {
                SideAfterSlice::Left => {
                    left_objects.sph_cubes.push(obj);
                },
                SideAfterSlice::Right => {
                    right_objects.sph_cubes.push(obj);
                },
                SideAfterSlice::Both => {
                    left_objects.sph_cubes.push(obj.clone());
                    right_objects.sph_cubes.push(obj.clone());
                },
            }
        }
        for obj in self.neg_sph_cubes
        {
            match obj.get_side_after_slice(slice)
            {
                SideAfterSlice::Left => {
                    left_objects.neg_sph_cubes.push(obj);
                },
                SideAfterSlice::Right => {
                    right_objects.neg_sph_cubes.push(obj);
                },
                SideAfterSlice::Both => {
                    left_objects.neg_sph_cubes.push(obj.clone());
                    right_objects.neg_sph_cubes.push(obj.clone());
                },
            }
        }
        for obj in self.s_sph_cubes
        {
            match obj.get_side_after_slice(slice)
            {
                SideAfterSlice::Left => {
                    left_objects.s_sph_cubes.push(obj);
                },
                SideAfterSlice::Right => {
                    right_objects.s_sph_cubes.push(obj);
                },
                SideAfterSlice::Both => {
                    left_objects.s_sph_cubes.push(obj.clone());
                    right_objects.s_sph_cubes.push(obj.clone());
                },
            }
        }
        for obj in self.s_neg_sph_cubes
        {
            match obj.get_side_after_slice(slice)
            {
                SideAfterSlice::Left => {
                    left_objects.s_neg_sph_cubes.push(obj);
                },
                SideAfterSlice::Right => {
                    right_objects.s_neg_sph_cubes.push(obj);
                },
                SideAfterSlice::Both => {
                    left_objects.s_neg_sph_cubes.push(obj.clone());
                    right_objects.s_neg_sph_cubes.push(obj.clone());
                },
            }
        }

        for obj in self.undestroyable_cubes
        {
            match obj.get_side_after_slice(slice)
            {
                SideAfterSlice::Left => {
                    left_objects.undestroyable_cubes.push(obj);
                },
                SideAfterSlice::Right => {
                    right_objects.undestroyable_cubes.push(obj);
                },
                SideAfterSlice::Both => {
                    left_objects.undestroyable_cubes.push(obj.clone());
                    right_objects.undestroyable_cubes.push(obj.clone());
                },
            }
        }

        left_objects.calculate_object_edges_lists();
        right_objects.calculate_object_edges_lists();

        (left_objects, right_objects)
    }


    pub fn new_empty(stickiness: f32) -> Self
    {
        let cubes = Vec::new();
        let s_cubes = Vec::new();
        let neg_cubes = Vec::new();
        let s_neg_cubes = Vec::new();
        let spheres = Vec::new();
        let s_spheres = Vec::new();
        let neg_spheres = Vec::new();
        let s_neg_spheres = Vec::new();
        let sph_cubes = Vec::new();
        let s_sph_cubes = Vec::new();
        let neg_sph_cubes = Vec::new();
        let s_neg_sph_cubes = Vec::new();
        let undestroyable_cubes = Vec::new();
        let object_edges_list_along_x = Vec::new();
        let object_edges_list_along_y = Vec::new();
        let object_edges_list_along_z = Vec::new();
        let object_edges_list_along_w = Vec::new();

        Objects {
            stickiness,
            cubes,
            s_cubes,
            neg_cubes,
            s_neg_cubes,
            spheres,
            s_spheres,
            neg_spheres,
            s_neg_spheres,
            sph_cubes,
            s_sph_cubes,
            neg_sph_cubes,
            s_neg_sph_cubes,
            undestroyable_cubes,
            object_edges_list_along_x,
            object_edges_list_along_y,
            object_edges_list_along_z,
            object_edges_list_along_w,
        }
    }

    pub fn calculate_object_edges_lists(&mut self)
    {
        for obj in &self.cubes
        {
            self.object_edges_list_along_x.push(obj.x_bounds.x);
            self.object_edges_list_along_x.push(obj.x_bounds.y);
        }
        for obj in &self.s_cubes
        {
            self.object_edges_list_along_x.push(obj.x_bounds.x);
            self.object_edges_list_along_x.push(obj.x_bounds.y);
        }

        for obj in &self.s_neg_cubes
        {
            self.object_edges_list_along_x.push(obj.x_bounds.x);
            self.object_edges_list_along_x.push(obj.x_bounds.y);
        }

        for obj in &self.neg_cubes
        {
            self.object_edges_list_along_x.push(obj.x_bounds.x);
            self.object_edges_list_along_x.push(obj.x_bounds.y);
        }

        for obj in &self.spheres
        {
            self.object_edges_list_along_x.push(obj.x_bounds.x);
            self.object_edges_list_along_x.push(obj.x_bounds.y);
        }

        for obj in &self.s_spheres
        {
            self.object_edges_list_along_x.push(obj.x_bounds.x);
            self.object_edges_list_along_x.push(obj.x_bounds.y);
        }

        for obj in &self.s_neg_spheres
        {
            self.object_edges_list_along_x.push(obj.x_bounds.x);
            self.object_edges_list_along_x.push(obj.x_bounds.y);
        }

        for obj in &self.neg_spheres
        {
            self.object_edges_list_along_x.push(obj.x_bounds.x);
            self.object_edges_list_along_x.push(obj.x_bounds.y);
        }

        for obj in &self.sph_cubes
        {
            self.object_edges_list_along_x.push(obj.x_bounds.x);
            self.object_edges_list_along_x.push(obj.x_bounds.y);
        }

        for obj in &self.s_sph_cubes
        {
            self.object_edges_list_along_x.push(obj.x_bounds.x);
            self.object_edges_list_along_x.push(obj.x_bounds.y);
        }

        for obj in &self.s_neg_sph_cubes
        {
            self.object_edges_list_along_x.push(obj.x_bounds.x);
            self.object_edges_list_along_x.push(obj.x_bounds.y);
        }

        for obj in &self.neg_sph_cubes
        {
            self.object_edges_list_along_x.push(obj.x_bounds.x);
            self.object_edges_list_along_x.push(obj.x_bounds.y);
        }

        for obj in &self.undestroyable_cubes
        {
            self.object_edges_list_along_x.push(obj.x_bounds.x);
            self.object_edges_list_along_x.push(obj.x_bounds.y);
        }

        self.object_edges_list_along_x.sort_by(|a, b| {
            if *a < *b
            {
                Ordering::Less
            }
            else if *a > *b
            {
                Ordering::Greater    
            }
            else
            {
                Ordering::Equal
            }
        });

        for obj in &self.cubes
        {
            self.object_edges_list_along_y.push(obj.y_bounds.x);
            self.object_edges_list_along_y.push(obj.y_bounds.y);
        }
        for obj in &self.s_cubes
        {
            self.object_edges_list_along_y.push(obj.y_bounds.x);
            self.object_edges_list_along_y.push(obj.y_bounds.y);
        }

        for obj in &self.s_neg_cubes
        {
            self.object_edges_list_along_y.push(obj.y_bounds.x);
            self.object_edges_list_along_y.push(obj.y_bounds.y);
        }

        for obj in &self.neg_cubes
        {
            self.object_edges_list_along_y.push(obj.y_bounds.x);
            self.object_edges_list_along_y.push(obj.y_bounds.y);
        }

        for obj in &self.spheres
        {
            self.object_edges_list_along_y.push(obj.y_bounds.x);
            self.object_edges_list_along_y.push(obj.y_bounds.y);
        }

        for obj in &self.s_spheres
        {
            self.object_edges_list_along_y.push(obj.y_bounds.x);
            self.object_edges_list_along_y.push(obj.y_bounds.y);
        }

        for obj in &self.s_neg_spheres
        {
            self.object_edges_list_along_y.push(obj.y_bounds.x);
            self.object_edges_list_along_y.push(obj.y_bounds.y);
        }

        for obj in &self.neg_spheres
        {
            self.object_edges_list_along_y.push(obj.y_bounds.x);
            self.object_edges_list_along_y.push(obj.y_bounds.y);
        }

        for obj in &self.sph_cubes
        {
            self.object_edges_list_along_y.push(obj.y_bounds.x);
            self.object_edges_list_along_y.push(obj.y_bounds.y);
        }

        for obj in &self.s_sph_cubes
        {
            self.object_edges_list_along_y.push(obj.y_bounds.x);
            self.object_edges_list_along_y.push(obj.y_bounds.y);
        }

        for obj in &self.s_neg_sph_cubes
        {
            self.object_edges_list_along_y.push(obj.y_bounds.x);
            self.object_edges_list_along_y.push(obj.y_bounds.y);
        }

        for obj in &self.neg_sph_cubes
        {
            self.object_edges_list_along_y.push(obj.y_bounds.x);
            self.object_edges_list_along_y.push(obj.y_bounds.y);
        }

        for obj in &self.undestroyable_cubes
        {
            self.object_edges_list_along_y.push(obj.y_bounds.x);
            self.object_edges_list_along_y.push(obj.y_bounds.y);
        }

        self.object_edges_list_along_y.sort_by(|a, b| {
            if *a < *b
            {
                Ordering::Less
            }
            else if *a > *b
            {
                Ordering::Greater    
            }
            else
            {
                Ordering::Equal
            }
        });

        for obj in &self.cubes
        {
            self.object_edges_list_along_z.push(obj.z_bounds.x);
            self.object_edges_list_along_z.push(obj.z_bounds.y);
        }
        for obj in &self.s_cubes
        {
            self.object_edges_list_along_z.push(obj.z_bounds.x);
            self.object_edges_list_along_z.push(obj.z_bounds.y);
        }

        for obj in &self.s_neg_cubes
        {
            self.object_edges_list_along_z.push(obj.z_bounds.x);
            self.object_edges_list_along_z.push(obj.z_bounds.y);
        }

        for obj in &self.neg_cubes
        {
            self.object_edges_list_along_z.push(obj.z_bounds.x);
            self.object_edges_list_along_z.push(obj.z_bounds.y);
        }

        for obj in &self.spheres
        {
            self.object_edges_list_along_z.push(obj.z_bounds.x);
            self.object_edges_list_along_z.push(obj.z_bounds.y);
        }

        for obj in &self.s_spheres
        {
            self.object_edges_list_along_z.push(obj.z_bounds.x);
            self.object_edges_list_along_z.push(obj.z_bounds.y);
        }

        for obj in &self.s_neg_spheres
        {
            self.object_edges_list_along_z.push(obj.z_bounds.x);
            self.object_edges_list_along_z.push(obj.z_bounds.y);
        }

        for obj in &self.neg_spheres
        {
            self.object_edges_list_along_z.push(obj.z_bounds.x);
            self.object_edges_list_along_z.push(obj.z_bounds.y);
        }

        for obj in &self.sph_cubes
        {
            self.object_edges_list_along_z.push(obj.z_bounds.x);
            self.object_edges_list_along_z.push(obj.z_bounds.y);
        }

        for obj in &self.s_sph_cubes
        {
            self.object_edges_list_along_z.push(obj.z_bounds.x);
            self.object_edges_list_along_z.push(obj.z_bounds.y);
        }

        for obj in &self.s_neg_sph_cubes
        {
            self.object_edges_list_along_z.push(obj.z_bounds.x);
            self.object_edges_list_along_z.push(obj.z_bounds.y);
        }

        for obj in &self.neg_sph_cubes
        {
            self.object_edges_list_along_z.push(obj.z_bounds.x);
            self.object_edges_list_along_z.push(obj.z_bounds.y);
        }

        for obj in &self.undestroyable_cubes
        {
            self.object_edges_list_along_z.push(obj.z_bounds.x);
            self.object_edges_list_along_z.push(obj.z_bounds.y);
        }


        self.object_edges_list_along_z.sort_by(|a, b| {
            if *a < *b
            {
                Ordering::Less
            }
            else if *a > *b
            {
                Ordering::Greater    
            }
            else
            {
                Ordering::Equal
            }
        });

        for obj in &self.cubes
        {
            self.object_edges_list_along_w.push(obj.w_bounds.x);
            self.object_edges_list_along_w.push(obj.w_bounds.y);
        }
        for obj in &self.s_cubes
        {
            self.object_edges_list_along_w.push(obj.w_bounds.x);
            self.object_edges_list_along_w.push(obj.w_bounds.y);
        }

        for obj in &self.s_neg_cubes
        {
            self.object_edges_list_along_w.push(obj.w_bounds.x);
            self.object_edges_list_along_w.push(obj.w_bounds.y);
        }

        for obj in &self.neg_cubes
        {
            self.object_edges_list_along_w.push(obj.w_bounds.x);
            self.object_edges_list_along_w.push(obj.w_bounds.y);
        }

        for obj in &self.spheres
        {
            self.object_edges_list_along_w.push(obj.w_bounds.x);
            self.object_edges_list_along_w.push(obj.w_bounds.y);
        }

        for obj in &self.s_spheres
        {
            self.object_edges_list_along_w.push(obj.w_bounds.x);
            self.object_edges_list_along_w.push(obj.w_bounds.y);
        }

        for obj in &self.s_neg_spheres
        {
            self.object_edges_list_along_w.push(obj.w_bounds.x);
            self.object_edges_list_along_w.push(obj.w_bounds.y);
        }

        for obj in &self.neg_spheres
        {
            self.object_edges_list_along_w.push(obj.w_bounds.x);
            self.object_edges_list_along_w.push(obj.w_bounds.y);
        }

        for obj in &self.sph_cubes
        {
            self.object_edges_list_along_w.push(obj.w_bounds.x);
            self.object_edges_list_along_w.push(obj.w_bounds.y);
        }

        for obj in &self.s_sph_cubes
        {
            self.object_edges_list_along_w.push(obj.w_bounds.x);
            self.object_edges_list_along_w.push(obj.w_bounds.y);
        }

        for obj in &self.s_neg_sph_cubes
        {
            self.object_edges_list_along_w.push(obj.w_bounds.x);
            self.object_edges_list_along_w.push(obj.w_bounds.y);
        }

        for obj in &self.neg_sph_cubes
        {
            self.object_edges_list_along_w.push(obj.w_bounds.x);
            self.object_edges_list_along_w.push(obj.w_bounds.y);
        }

        for obj in &self.undestroyable_cubes
        {
            self.object_edges_list_along_w.push(obj.w_bounds.x);
            self.object_edges_list_along_w.push(obj.w_bounds.y);
        }

        self.object_edges_list_along_w.sort_by(|a, b| {
            if *a < *b
            {
                Ordering::Less
            }
            else if *a > *b
            {
                Ordering::Greater    
            }
            else
            {
                Ordering::Equal
            }
        });
    }

    pub fn init(static_data: &StaticRenderData) -> Self
    {
        let stickiness = static_data.other_static_data.static_shapes_stickiness;

        let mut cubes = Vec::new();
        for shape in &static_data.cubes
        {
            let obj_info = ObjectInfo {
                shape_type: ShapeType::Cube,
                obj_type: ObjectType::Normal
            };

            let object = Object::new(shape, obj_info, stickiness);

            cubes.push(object);
        }


        let mut s_cubes = Vec::new();
        for shape in &static_data.s_cubes
        {
            let obj_info = ObjectInfo {
                shape_type: ShapeType::Cube,
                obj_type: ObjectType::NormalStickiness
            };

            let object = Object::new(shape, obj_info, stickiness);

            s_cubes.push(object);
        }


        let mut neg_cubes = Vec::new();
        for shape in &static_data.neg_cubes
        {
            let obj_info = ObjectInfo {
                shape_type: ShapeType::Cube,
                obj_type: ObjectType::Negative
            };

            let object = Object::new(shape, obj_info, stickiness);

            neg_cubes.push(object);
        }


        let mut s_neg_cubes = Vec::new();
        for shape in &static_data.s_neg_cubes
        {
            let obj_info = ObjectInfo {
                shape_type: ShapeType::Cube,
                obj_type: ObjectType::NegativeStickiness
            };

            let object = Object::new(shape, obj_info, stickiness);

            s_neg_cubes.push(object);
        }


        let mut spheres = Vec::new();
        for shape in &static_data.spheres
        {
            let obj_info = ObjectInfo {
                shape_type: ShapeType::Sphere,
                obj_type: ObjectType::Normal
            };

            let object = Object::new(shape, obj_info, stickiness);

            spheres.push(object);
        }


        let mut s_spheres = Vec::new();
        for shape in &static_data.s_spheres
        {
            let obj_info = ObjectInfo {
                shape_type: ShapeType::Sphere,
                obj_type: ObjectType::NormalStickiness
            };

            let object = Object::new(shape, obj_info, stickiness);

            s_spheres.push(object);
        }


        let mut neg_spheres = Vec::new();
        for shape in &static_data.neg_spheres
        {
            let obj_info = ObjectInfo {
                shape_type: ShapeType::Sphere,
                obj_type: ObjectType::Negative
            };

            let object = Object::new(shape, obj_info, stickiness);

            neg_spheres.push(object);
        }


        let mut s_neg_spheres = Vec::new();
        for shape in &static_data.s_neg_spheres
        {
            let obj_info = ObjectInfo {
                shape_type: ShapeType::Sphere,
                obj_type: ObjectType::NegativeStickiness
            };

            let object = Object::new(shape, obj_info, stickiness);

            s_neg_spheres.push(object);
        }


        let mut sph_cubes = Vec::new();
        for shape in &static_data.sph_cubes
        {
            let obj_info = ObjectInfo {
                shape_type: ShapeType::SphCube,
                obj_type: ObjectType::Normal
            };

            let object = Object::new(shape, obj_info, stickiness);

            sph_cubes.push(object);
        }


        let mut s_sph_cubes = Vec::new();
        for shape in &static_data.s_sph_cubes
        {
            let obj_info = ObjectInfo {
                shape_type: ShapeType::SphCube,
                obj_type: ObjectType::NegativeStickiness
            };

            let object = Object::new(shape, obj_info, stickiness);

            s_sph_cubes.push(object);
        }


        let mut neg_sph_cubes = Vec::new();
        for shape in &static_data.neg_sph_cubes
        {
            let obj_info = ObjectInfo {
                shape_type: ShapeType::SphCube,
                obj_type: ObjectType::Negative
            };

            let object = Object::new(shape, obj_info, stickiness);

            neg_sph_cubes.push(object);
        }


        let mut s_neg_sph_cubes = Vec::new();
        for shape in &static_data.s_neg_sph_cubes
        {
            let obj_info = ObjectInfo {
                shape_type: ShapeType::SphCube,
                obj_type: ObjectType::NegativeStickiness
            };

            let object = Object::new(shape, obj_info, stickiness);

            s_neg_sph_cubes.push(object);
        }


        let mut undestroyable_cubes = Vec::new();
        for shape in &static_data.undestroyable_cubes
        {
            let obj_info = ObjectInfo {
                shape_type: ShapeType::Cube,
                obj_type: ObjectType::Unbreakable
            };

            let object = Object::new(shape, obj_info, stickiness);

            undestroyable_cubes.push(object);
        }
        
        let object_edges_list_along_x = Vec::new();
        let object_edges_list_along_y = Vec::new();
        let object_edges_list_along_z = Vec::new();
        let object_edges_list_along_w = Vec::new();

        let mut objects = Objects {
            stickiness: static_data.other_static_data.static_shapes_stickiness,
            cubes,
            s_cubes,
            neg_cubes,
            s_neg_cubes,
            spheres,
            s_spheres,
            neg_spheres,
            s_neg_spheres,
            sph_cubes,
            s_sph_cubes,
            neg_sph_cubes,
            s_neg_sph_cubes,
            undestroyable_cubes,

            object_edges_list_along_x,
            object_edges_list_along_y,
            object_edges_list_along_z,
            object_edges_list_along_w,
        };

        objects.calculate_object_edges_lists();

        objects
    }
}


#[derive(Clone)]
struct Object
{
    shape: Shape,
    x_bounds: Vec2,
    y_bounds: Vec2,
    z_bounds: Vec2,
    w_bounds: Vec2,
}

pub struct ObjectInfo
{
    shape_type: ShapeType,
    obj_type: ObjectType,
}
enum ObjectType
{
    Normal,
    NormalStickiness,
    Negative,
    NegativeStickiness,
    Unbreakable,
}

enum ShapeType
{
    Cube,
    Sphere,
    SphCube,
}

enum SideAfterSlice
{
    Left,
    Right,
    Both
}

impl Object
{
    pub fn get_side_after_slice(&self, slice: Slice) -> SideAfterSlice
    {
        match slice {
            Slice::X(p) => {
                if self.x_bounds.x < p && self.x_bounds.y < p
                {
                    SideAfterSlice::Left
                }
                else if self.x_bounds.x > p && self.x_bounds.y > p
                {
                    SideAfterSlice::Right
                }
                else
                {
                    SideAfterSlice::Both
                }
            },
            Slice::Y(p) => {
                if self.y_bounds.x < p && self.y_bounds.y < p
                {
                    SideAfterSlice::Left
                }
                else if self.y_bounds.x > p && self.y_bounds.y > p
                {
                    SideAfterSlice::Right
                }
                else
                {
                    SideAfterSlice::Both
                }
            },
            Slice::Z(p) => {
                if self.z_bounds.x < p && self.z_bounds.y < p
                {
                    SideAfterSlice::Left
                }
                else if self.z_bounds.x > p && self.z_bounds.y > p
                {
                    SideAfterSlice::Right
                }
                else
                {
                    SideAfterSlice::Both
                }
            },
            Slice::W(p) => {
                if self.w_bounds.x < p && self.w_bounds.y < p
                {
                    SideAfterSlice::Left
                }
                else if self.w_bounds.x > p && self.w_bounds.y > p
                {
                    SideAfterSlice::Right
                }
                else
                {
                    SideAfterSlice::Both
                }
            },
        }
    }

    pub fn new(shape: &Shape, obj_info: ObjectInfo, stickiness: f32) -> Self
    {
        let (x_bounds, y_bounds, z_bounds, w_bounds) = match obj_info.shape_type
        {
            ShapeType::Cube => {
                match obj_info.obj_type {
                    ObjectType::Normal => {
                        let x_bounds = {
                            Vec2::new(
                                shape.pos[0] - (shape.size[0] + shape.roundness),
                                shape.pos[0] + (shape.size[0] + shape.roundness),
                            )
                        };
                        let y_bounds = {
                            Vec2::new(
                                shape.pos[1] - (shape.size[1] + shape.roundness),
                                shape.pos[1] + (shape.size[1] + shape.roundness),
                            )
                        };
                        let z_bounds = {
                            Vec2::new(
                                shape.pos[2] - (shape.size[2] + shape.roundness),
                                shape.pos[2] + (shape.size[2] + shape.roundness),
                            )
                        };
                        let w_bounds = {
                            Vec2::new(
                                shape.pos[3] - (shape.size[3] + shape.roundness),
                                shape.pos[3] + (shape.size[3] + shape.roundness),
                            )
                        };

                        (x_bounds, y_bounds, z_bounds, w_bounds)
                    },
                    ObjectType::NormalStickiness => {
                        let x_bounds = {
                            Vec2::new(
                                shape.pos[0] - (shape.size[0] + shape.roundness + stickiness*PI),
                                shape.pos[0] + (shape.size[0] + shape.roundness + stickiness*PI),
                            )
                        };
                        let y_bounds = {
                            Vec2::new(
                                shape.pos[1] - (shape.size[1] + shape.roundness + stickiness*PI),
                                shape.pos[1] + (shape.size[1] + shape.roundness + stickiness*PI),
                            )
                        };
                        let z_bounds = {
                            Vec2::new(
                                shape.pos[2] - (shape.size[2] + shape.roundness + stickiness*PI),
                                shape.pos[2] + (shape.size[2] + shape.roundness + stickiness*PI),
                            )
                        };
                        let w_bounds = {
                            Vec2::new(
                                shape.pos[3] - (shape.size[3] + shape.roundness + stickiness*PI),
                                shape.pos[3] + (shape.size[3] + shape.roundness + stickiness*PI),
                            )
                        };

                        (x_bounds, y_bounds, z_bounds, w_bounds)
                    },
                    ObjectType::Negative => {
                        let x_bounds = {
                            Vec2::new(
                                shape.pos[0] - (shape.size[0] + shape.roundness),
                                shape.pos[0] + (shape.size[0] + shape.roundness),
                            )
                        };
                        let y_bounds = {
                            Vec2::new(
                                shape.pos[1] - (shape.size[1] + shape.roundness),
                                shape.pos[1] + (shape.size[1] + shape.roundness),
                            )
                        };
                        let z_bounds = {
                            Vec2::new(
                                shape.pos[2] - (shape.size[2] + shape.roundness),
                                shape.pos[2] + (shape.size[2] + shape.roundness),
                            )
                        };
                        let w_bounds = {
                            Vec2::new(
                                shape.pos[3] - (shape.size[3] + shape.roundness),
                                shape.pos[3] + (shape.size[3] + shape.roundness),
                            )
                        };

                        (x_bounds, y_bounds, z_bounds, w_bounds)
                    },
                    ObjectType::NegativeStickiness => {
                        let x_bounds = {
                            Vec2::new(
                                shape.pos[0] - (shape.size[0] + shape.roundness + stickiness*PI),
                                shape.pos[0] + (shape.size[0] + shape.roundness + stickiness*PI),
                            )
                        };
                        let y_bounds = {
                            Vec2::new(
                                shape.pos[1] - (shape.size[1] + shape.roundness + stickiness*PI),
                                shape.pos[1] + (shape.size[1] + shape.roundness + stickiness*PI),
                            )
                        };
                        let z_bounds = {
                            Vec2::new(
                                shape.pos[2] - (shape.size[2] + shape.roundness + stickiness*PI),
                                shape.pos[2] + (shape.size[2] + shape.roundness + stickiness*PI),
                            )
                        };
                        let w_bounds = {
                            Vec2::new(
                                shape.pos[3] - (shape.size[3] + shape.roundness + stickiness*PI),
                                shape.pos[3] + (shape.size[3] + shape.roundness + stickiness*PI),
                            )
                        };

                        (x_bounds, y_bounds, z_bounds, w_bounds)
                    },
                    ObjectType::Unbreakable => {
                        let x_bounds = {
                            Vec2::new(
                                shape.pos[0] - (shape.size[0] + shape.roundness),
                                shape.pos[0] + (shape.size[0] + shape.roundness),
                            )
                        };
                        let y_bounds = {
                            Vec2::new(
                                shape.pos[1] - (shape.size[1] + shape.roundness),
                                shape.pos[1] + (shape.size[1] + shape.roundness),
                            )
                        };
                        let z_bounds = {
                            Vec2::new(
                                shape.pos[2] - (shape.size[2] + shape.roundness),
                                shape.pos[2] + (shape.size[2] + shape.roundness),
                            )
                        };
                        let w_bounds = {
                            Vec2::new(
                                shape.pos[3] - (shape.size[3] + shape.roundness),
                                shape.pos[3] + (shape.size[3] + shape.roundness),
                            )
                        };

                        (x_bounds, y_bounds, z_bounds, w_bounds)
                    },
                }
            },
            ShapeType::Sphere => {
                match obj_info.obj_type {
                    ObjectType::Normal => {
                        let x_bounds = {
                            Vec2::new(
                                shape.pos[0] - (shape.size[0] + shape.roundness),
                                shape.pos[0] + (shape.size[0] + shape.roundness),
                            )
                        };
                        let y_bounds = {
                            Vec2::new(
                                shape.pos[1] - (shape.size[0] + shape.roundness),
                                shape.pos[1] + (shape.size[0] + shape.roundness),
                            )
                        };
                        let z_bounds = {
                            Vec2::new(
                                shape.pos[2] - (shape.size[0] + shape.roundness),
                                shape.pos[2] + (shape.size[0] + shape.roundness),
                            )
                        };
                        let w_bounds = {
                            Vec2::new(
                                shape.pos[3] - (shape.size[0] + shape.roundness),
                                shape.pos[3] + (shape.size[0] + shape.roundness),
                            )
                        };

                        (x_bounds, y_bounds, z_bounds, w_bounds)
                    },
                    ObjectType::NormalStickiness => {
                        let x_bounds = {
                            Vec2::new(
                                shape.pos[0] - (shape.size[0] + shape.roundness + stickiness*PI),
                                shape.pos[0] + (shape.size[0] + shape.roundness + stickiness*PI),
                            )
                        };
                        let y_bounds = {
                            Vec2::new(
                                shape.pos[1] - (shape.size[0] + shape.roundness + stickiness*PI),
                                shape.pos[1] + (shape.size[0] + shape.roundness + stickiness*PI),
                            )
                        };
                        let z_bounds = {
                            Vec2::new(
                                shape.pos[2] - (shape.size[0] + shape.roundness + stickiness*PI),
                                shape.pos[2] + (shape.size[0] + shape.roundness + stickiness*PI),
                            )
                        };
                        let w_bounds = {
                            Vec2::new(
                                shape.pos[3] - (shape.size[0] + shape.roundness + stickiness*PI),
                                shape.pos[3] + (shape.size[0] + shape.roundness + stickiness*PI),
                            )
                        };

                        (x_bounds, y_bounds, z_bounds, w_bounds)
                    },
                    ObjectType::Negative => {
                        let x_bounds = {
                            Vec2::new(
                                shape.pos[0] - (shape.size[0] + shape.roundness),
                                shape.pos[0] + (shape.size[0] + shape.roundness),
                            )
                        };
                        let y_bounds = {
                            Vec2::new(
                                shape.pos[1] - (shape.size[0] + shape.roundness),
                                shape.pos[1] + (shape.size[0] + shape.roundness),
                            )
                        };
                        let z_bounds = {
                            Vec2::new(
                                shape.pos[2] - (shape.size[0] + shape.roundness),
                                shape.pos[2] + (shape.size[0] + shape.roundness),
                            )
                        };
                        let w_bounds = {
                            Vec2::new(
                                shape.pos[3] - (shape.size[0] + shape.roundness),
                                shape.pos[3] + (shape.size[0] + shape.roundness),
                            )
                        };

                        (x_bounds, y_bounds, z_bounds, w_bounds)
                    },
                    ObjectType::NegativeStickiness => {
                        let x_bounds = {
                            Vec2::new(
                                shape.pos[0] - (shape.size[0] + shape.roundness + stickiness*PI),
                                shape.pos[0] + (shape.size[0] + shape.roundness + stickiness*PI),
                            )
                        };
                        let y_bounds = {
                            Vec2::new(
                                shape.pos[1] - (shape.size[0] + shape.roundness + stickiness*PI),
                                shape.pos[1] + (shape.size[0] + shape.roundness + stickiness*PI),
                            )
                        };
                        let z_bounds = {
                            Vec2::new(
                                shape.pos[2] - (shape.size[0] + shape.roundness + stickiness*PI),
                                shape.pos[2] + (shape.size[0] + shape.roundness + stickiness*PI),
                            )
                        };
                        let w_bounds = {
                            Vec2::new(
                                shape.pos[3] - (shape.size[0] + shape.roundness + stickiness*PI),
                                shape.pos[3] + (shape.size[0] + shape.roundness + stickiness*PI),
                            )
                        };

                        (x_bounds, y_bounds, z_bounds, w_bounds)
                    },
                    ObjectType::Unbreakable => {
                        unimplemented!()
                    },
                }
            },
            ShapeType::SphCube => {
                match obj_info.obj_type {
                    ObjectType::Normal => {
                        let size = calc_size_for_sphcube(shape.size, shape.roundness);

                        let x_bounds = {
                            Vec2::new(
                                shape.pos[0] - (size[0]),
                                shape.pos[0] + (size[0]),
                            )
                        };
                        let y_bounds = {
                            Vec2::new(
                                shape.pos[1] - (size[1]),
                                shape.pos[1] + (size[1]),
                            )
                        };
                        let z_bounds = {
                            Vec2::new(
                                shape.pos[2] - (size[1]),
                                shape.pos[2] + (size[1]),
                            )
                        };
                        let w_bounds = {
                            Vec2::new(
                                shape.pos[3] - (size[1]),
                                shape.pos[3] + (size[1]),
                            )
                        };

                        (x_bounds, y_bounds, z_bounds, w_bounds)
                    },
                    ObjectType::NormalStickiness => {
                        let size = calc_size_for_sphcube(shape.size, shape.roundness);

                        let x_bounds = {
                            Vec2::new(
                                shape.pos[0] - (size[0] + stickiness*PI),
                                shape.pos[0] + (size[0] + stickiness*PI),
                            )
                        };
                        let y_bounds = {
                            Vec2::new(
                                shape.pos[1] - (size[1] + stickiness*PI),
                                shape.pos[1] + (size[1] + stickiness*PI),
                            )
                        };
                        let z_bounds = {
                            Vec2::new(
                                shape.pos[2] - (size[1] + stickiness*PI),
                                shape.pos[2] + (size[1] + stickiness*PI),
                            )
                        };
                        let w_bounds = {
                            Vec2::new(
                                shape.pos[3] - (size[1] + stickiness*PI),
                                shape.pos[3] + (size[1] + stickiness*PI),
                            )
                        };

                        (x_bounds, y_bounds, z_bounds, w_bounds)
                    },
                    ObjectType::Negative => {
                        let size = calc_size_for_sphcube(shape.size, shape.roundness);

                        let x_bounds = {
                            Vec2::new(
                                shape.pos[0] - (size[0]),
                                shape.pos[0] + (size[0]),
                            )
                        };
                        let y_bounds = {
                            Vec2::new(
                                shape.pos[1] - (size[1]),
                                shape.pos[1] + (size[1]),
                            )
                        };
                        let z_bounds = {
                            Vec2::new(
                                shape.pos[2] - (size[1]),
                                shape.pos[2] + (size[1]),
                            )
                        };
                        let w_bounds = {
                            Vec2::new(
                                shape.pos[3] - (size[1]),
                                shape.pos[3] + (size[1]),
                            )
                        };

                        (x_bounds, y_bounds, z_bounds, w_bounds)
                    },
                    ObjectType::NegativeStickiness => {
                        let size = calc_size_for_sphcube(shape.size, shape.roundness);

                        let x_bounds = {
                            Vec2::new(
                                shape.pos[0] - (size[0] + stickiness*PI),
                                shape.pos[0] + (size[0] + stickiness*PI),
                            )
                        };
                        let y_bounds = {
                            Vec2::new(
                                shape.pos[1] - (size[1] + stickiness*PI),
                                shape.pos[1] + (size[1] + stickiness*PI),
                            )
                        };
                        let z_bounds = {
                            Vec2::new(
                                shape.pos[2] - (size[1] + stickiness*PI),
                                shape.pos[2] + (size[1] + stickiness*PI),
                            )
                        };
                        let w_bounds = {
                            Vec2::new(
                                shape.pos[3] - (size[1] + stickiness*PI),
                                shape.pos[3] + (size[1] + stickiness*PI),
                            )
                        };

                        (x_bounds, y_bounds, z_bounds, w_bounds)
                    },
                    ObjectType::Unbreakable => {
                        unimplemented!()
                    },
                }
            },
        };

        Object
        {
            shape: *shape,
            x_bounds,
            y_bounds,
            z_bounds,
            w_bounds,
        }
    }
}