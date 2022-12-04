use bevy_ecs::component::Component;

use naia_shared::{EntityProperty, Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::protocol::Protocol"]
pub struct KeyCommand {
    pub entity: EntityProperty,

    pub forward: Property<bool>,
    pub backward: Property<bool>,
    pub left: Property<bool>,
    pub right: Property<bool>,

    pub primary_fire: Property<bool>,
}

impl KeyCommand {
    pub fn new(w: bool, s: bool, a: bool, d: bool, primary_fire: bool) -> Self {
        KeyCommand::new_complete(w, s, a, d, primary_fire)
    }
}
