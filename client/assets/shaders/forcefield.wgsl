#import bevy_pbr::mesh_view_bindings

// struct ForceFieldMaterial {};

// @group(1) @binding(0)
// var<uniform> material: ForceFieldMaterial;

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {

    var Normal = normalize(world_normal);
    var V = normalize(view.world_position.xyz - world_position.xyz);

    let NdotV = max(dot(Normal, V), 0.0001);
    var fresnel = clamp(1.0 - NdotV, 0.0, 1.0);

    fresnel = pow(fresnel, 3.0) * 2.0;



    return vec4(1., 1., 1., fresnel);
}