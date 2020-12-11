use serde::{Serialize, Deserialize};

macro_rules! define_optional_id {
    ($name:ident) => {
        #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
        pub struct $name(pub usize);
        impl $name {
            #[allow(dead_code)]
            pub const NONE: $name = $name(0);

            #[allow(dead_code)]
            pub fn next(id: $name) -> $name {
                $name(id.0 + 1)
            }
        }
    }
}

pub type SessionToken = usize;

define_optional_id!(EntityId);
define_optional_id!(SpellId);
define_optional_id!(SpellSpecId);
define_optional_id!(SkillId);
