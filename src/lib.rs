// cargo t -- --nocapture

use luexks_reassembly::{
    shapes::{
        port::Port, port_flags::PortFlag, ports::Ports, scale::Scale, shape::Shape, shape_id::ShapeId, shapes::Shapes, vertex::vert, vertices::Vertices
    },
    utility::{
        display_oriented_math::don_float_from,
        flags::Flags,
    },
    *,
};
use mlua::{Lua, Table, Value};
use regex::Regex;

mod tests {
    use crate::get_vanilla_shapes;

    use crate::blocks::block::*;
    use crate::blocks::blocks::*;
    use crate::blocks::feature::*;
    use crate::r#mod::Mod;
    use crate::shapes::shapes::Shapes;
    use crate::utility::color::Color;
    use crate::utility::flags::*;
    use crate::utility::funky_string::*;
    use luexks_reassembly::*;

    #[test]
    fn create_vanilla_shape_mod() {
        const USER: &str = "ALX";
        const MOD_NAME: &str = "Vanilla Shape Mod";
        pub const GROUP: i32 = 78;

        let mod_path = create_mod_folder_and_get_mod_path(USER, MOD_NAME);

        let blocks = &mut Blocks::default();
        let shapes = &mut get_vanilla_shapes();

        blocks.add_blocks(
            block!(
                name: funky_string!("Test Hull"),
                blurb: funky_string!("This block's shape was parsed from the vanilla shapes.lua."),
                features: explicit_features!(
                    Palette,
                ),
                group: GROUP,
                color_1: Color::new_rrggbb("7f6538"),
                color_2: Color::new_rrggbb("49351f"),
                line_color: Color::new_rrggbb("221e0f"),
                durability: 4.0,
                density: 0.2,
            )
            .to_extended_blocks_from_plural_shapes(&shapes.0),
        );

        let r#mod = Mod {
            blocks_option: Some(blocks),
            shapes_option: Some(shapes),
        };

        println!("Just debugged.");

        write_mod(mod_path, r#mod);
    }
}

pub fn get_vanilla_shapes() -> Shapes {
    let vanilla_shapes = std::fs::read_to_string("shapes.lua").unwrap();

    let regex_shape_id = r"\{\s*([A-Z_][A-Z0-9_]*)\s*,";
    let re = Regex::new(regex_shape_id).unwrap();
    let vanilla_shapes = re.replace_all(&vanilla_shapes, "{ \"$1\",");

    let regex_port_flag = r"([A-Z_][A-Z-9_]*)\s*\}";
    let re = Regex::new(regex_port_flag).unwrap();
    let vanilla_shapes = re.replace_all(&vanilla_shapes, "\"$1\"}");

    let lua = Lua::new();
    let shapes: Vec<Table> = lua
        .load(&format!("return {}", vanilla_shapes))
        .eval::<mlua::Table>()
        .unwrap()
        .sequence_values::<Table>()
        .map(|shape| shape.unwrap())
        .collect();

    let shapes = shapes.iter().map(|shape| {
        let name = 
            if let Value::String(s) = shape.get(1).unwrap() {
                s.to_str().unwrap().to_string()
            } else {
                panic!()
            };
        let id = ShapeId::Vanilla(name.clone());
        let scales = shape.get::<Table>(2).unwrap().sequence_values::<Table>().map(|scale| scale.unwrap()).collect::<Vec<Table>>();
        let scales: Vec<Scale> = scales.iter().map(|scale| {
            let verts = scale.get::<Table>("verts").unwrap().sequence_values::<Table>().map(|vertex| vertex.unwrap()).collect::<Vec<Table>>();
            let verts = Vertices(verts.iter().map(|vert| {
                let x: f32 = vert.get(1).unwrap();
                let y: f32 = vert.get(2).unwrap();
                vert!(x, y)
            }).collect());
            let ports = scale.get::<Table>("ports").unwrap().sequence_values::<Table>().map(|port| port.unwrap()).collect::<Vec<Table>>();
            let ports= Ports(ports.iter().map(|port| {
                let side_index: usize = port.get(1).unwrap();
                let position = don_float_from(port.get(2).unwrap());
                let flags = match port.get(3).unwrap() {
                    Value::String(s) => Flags(vec![PortFlag::from_str(&s.to_str().unwrap().to_string())]),
                    _ => Flags::default(),
                };
                Port { side_index, position, flags }
            }).collect());
            Scale { verts, ports, name: name.clone() }
        }).collect();
        let shape = Shape::Standard { id, scales };
        shape
    }).collect();

    Shapes(shapes)
}
