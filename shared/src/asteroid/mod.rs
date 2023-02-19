use bevy::prelude::{Component, Entity};

#[derive(Component)]
pub struct Asteroid;


#[derive(Component)]
pub struct Bullet {
    pub speed: f32,
    pub lifetime: f32,
}