use std::ops::{AddAssign, SubAssign};

use bevy::{
    math::vec3,
    prelude::{Component, Vec3},
};

use bevy_inspector_egui::Inspectable;
use naia_shared::{Property, Replicate};

#[derive(Component, Replicate)]
#[protocol_path = "crate::networking::protocol::Protocol"]
pub struct NetworkPosition {
    pub x: Property<f32>,
    pub y: Property<f32>,
    pub z: Property<f32>,
}

impl Inspectable for NetworkPosition {
    type Attributes = ();

    fn ui(
        &mut self,
        ui: &mut bevy_inspector_egui::egui::Ui,
        _: Self::Attributes,
        _: &mut bevy_inspector_egui::Context,
    ) -> bool {
        ui.label(self.x.to_string());
        ui.label(self.y.to_string());
        ui.label(self.z.to_string());
        true
    }
}

impl Into<Vec3> for NetworkPosition {
    fn into(self) -> Vec3 {
        vec3(*self.x, *self.y, *self.z)
    }
}

impl AddAssign<Vec3> for NetworkPosition {
    fn add_assign(&mut self, rhs: Vec3) {
        *self.x = rhs.x + *self.x;
        *self.y = rhs.y + *self.y;
        *self.z = rhs.z + *self.z;
    }
}

impl SubAssign<Vec3> for NetworkPosition {
    fn sub_assign(&mut self, rhs: Vec3) {
        *self.x = rhs.x - *self.x;
        *self.y = rhs.y - *self.y;
        *self.z = rhs.z - *self.z;
    }
}

impl NetworkPosition {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        NetworkPosition::new_complete(x, y, z)
    }
}
