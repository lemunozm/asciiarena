use crate::ids::{SpellSpecId};

use std::collections::{HashMap};

pub struct SpellSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub damage: i32,
    pub speed: f32,
    pub behaviour_name: &'static str,
}

lazy_static! {
    pub static ref SPELL_SPECIFICATIONS: HashMap<SpellSpecId, SpellSpec> =
        vec![
            SpellSpec {
                name: "Fire ball",
                description: "A fire ball that cause burns when explode",
                damage: 5,
                speed: 15.0,
                behaviour_name: "Explotable ball",
            }
        ]
        .into_iter()
        .enumerate()
        .map(|(index, def)| (SpellSpecId(index + 1), def))
        .collect();
}
