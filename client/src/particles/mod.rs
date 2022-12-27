use bevy::prelude::Vec3;
use bevy_hanabi::{InitLayout, InitModifier, ShapeDimension, ToWgslString, Value};

/// An initialization modifier spawning particles on a circle/disc.
#[derive(Clone, Copy)]
pub struct ThrusterModifier {
    /// The circle center, relative to the emitter position.
    pub center: Vec3,
    /// The circle axis, which is the normalized normal of the circle's plane.
    /// Set this to `Vec3::Z` for a 2D game.
    pub axis: Vec3,
    /// The circle radius.
    pub radius: f32,
    /// The radial speed of the particles on spawn.
    pub speed: Value<f32>,
    /// The shape dimension to spawn from.
    pub dimension: ShapeDimension,
}

impl Default for ThrusterModifier {
    fn default() -> Self {
        Self {
            center: Default::default(),
            axis: Vec3::Z,
            radius: Default::default(),
            speed: Default::default(),
            dimension: Default::default(),
        }
    }
}

impl InitModifier for ThrusterModifier {
    fn apply(&self, init_layout: &mut InitLayout) {
        let (tangent, bitangent) = self.axis.any_orthonormal_pair();

        let radius_code = match self.dimension {
            ShapeDimension::Surface => {
                // Constant radius
                format!("let r = {};", self.radius.to_wgsl_string())
            }
            ShapeDimension::Volume => {
                // Radius uniformly distributed in [0:1], then square-rooted
                // to account for the increased perimeter covered by increased radii.
                format!("let r = sqrt(rand()) * {};", self.radius.to_wgsl_string())
            }
        };

        init_layout.position_code = format!(
            r##"
    // >>> [ThrusterModifier]
    // Circle center
    let c = {};
    // Circle basis
    let tangent = {};
    let bitangent = {};
    // Circle radius
    {}
    // Radial speed
    let speed = {};
    // Spawn random point on/in circle
    let theta = rand() * tau;
    let dir = tangent * cos(theta) + bitangent * sin(theta);
    ret.pos = c + r * dir + transform[3].xyz;
    // Velocity along normal
    let normal = cross(bitangent, tangent);
    ret.vel = normal * speed;
    // <<< [ThrusterModifier]
            "##,
            self.center.to_wgsl_string(),
            tangent.to_wgsl_string(),
            bitangent.to_wgsl_string(),
            radius_code,
            self.speed.to_wgsl_string()
        );
    }
}
