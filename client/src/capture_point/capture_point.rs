use bevy::{
    prelude::{
        AlphaMode, Material, Color,
    },
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
};

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct ForceFieldMaterial {
    // #[uniform(0)]
    // pub selection: Vec4,

    #[uniform(0)]
    pub color: Color,
}

impl Material for ForceFieldMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/forcefield.wgsl".into()
    }
 
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

