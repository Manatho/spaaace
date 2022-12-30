#import bevy_pbr::mesh_view_bindings


struct ForceFieldMaterial {
    color: vec4<f32>,
    prev_color: vec4<f32>,
    last_color_change: f32,
};

@group(1) @binding(0)
var<uniform> material: ForceFieldMaterial;

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {


    let Normal = normalize(world_normal);
    let V = normalize(view.world_position.xyz - world_position.xyz);

    let NdotV = max(dot(Normal, V), 0.0001);
    var fresnel = clamp(1.0 - NdotV, 0.0, 1.0);

    fresnel = pow(fresnel, 3.0) * 2.0;

    let time_diff = clamp((globals.time - material.last_color_change - fresnel) / 0.1, 0.0, 1.0);
    let lerp_color = (1.0 - time_diff) * material.prev_color + time_diff * material.color;


    return lerp_color * vec4(1.0, 1.0, 1.0, fresnel);
}