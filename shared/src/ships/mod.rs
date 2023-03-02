use bevy::{
    gltf::Gltf,
    prelude::{Component, Handle},
};
use phf::phf_map;

#[derive(Clone, Copy)]
pub struct ShipType {
    // model_name is relative to the assets/ships/ folder
    pub model_name: &'static str,
    pub forward_thrust_force: f32,
    pub backward_thrust_force: f32,
    pub lateral_thrust_force: f32,
}

pub static SHIP_TYPES: phf::Map<&'static str, ShipType> = phf_map! {
    "TEST_SHIP" => ShipType{
        model_name: "test_ship/test_ship.gltf",
        forward_thrust_force: 2000.,
        backward_thrust_force: 2000.,
        lateral_thrust_force: 2000.,
    },
};

#[derive(Component)]
pub struct ShipModelLoadHandle(pub Handle<Gltf>);
