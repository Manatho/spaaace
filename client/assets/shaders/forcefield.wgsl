#import bevy_pbr::mesh_view_bindings
// #import bevy_pbr::prepass_utils



struct ForceFieldMaterial {
    color: vec4<f32>,
    prev_color: vec4<f32>,
    last_color_change: f32,
};

@group(1) @binding(0)
var<uniform> material: ForceFieldMaterial;

@group(1) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(1) @binding(2)
var base_color_sampler: sampler;


fn equalArea(vertexPosition: vec3<f32>) -> vec2<f32> {
    // Convert vertex position to spherical coordinates
    let spherical = vec3(length(vertexPosition), acos(vertexPosition.z / length(vertexPosition)), atan2(vertexPosition.y, vertexPosition.x));
    // Convert spherical coordinates to Cartesian coordinates
    let cartesian = vec3(spherical.x * sin(spherical.y) * cos(spherical.z), spherical.x * sin(spherical.y) * sin(spherical.z), spherical.x * cos(spherical.y));

    // Perform equal area projection
    let x = cartesian.x / (1.0 + abs(cartesian.z));
    let y = cartesian.y / (1.0 + abs(cartesian.z));

    var uv = vec2(x, y);

    // Check if the vertex is on the back of the sphere
    if vertexPosition.z < 0.0 {
        // Flip the UV coordinates on the back of the sphere
        uv.x = 1.0 - uv.x;
    }
    return uv;
}


// https://github.com/bevyengine/bevy/blob/main/crates/bevy_pbr/src/render/mesh.wgsl
struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct VertexOutput {
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv_coords: vec2<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.uv_coords = equalArea(vertex.position);
    out.world_normal = vertex.normal;
    out.world_position = vertex.position;
    return out;
}

struct FragmentInput {
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv_coords: vec2<f32>,
};

@fragment
fn fragment(
    input: FragmentInput,
    @builtin(sample_index) sample_index: u32,
    @builtin(position) frag_coord: vec4<f32>,
) -> @location(0) vec4<f32> {

    // let depth = prepass_depth(frag_coord, sample_index);
    let Normal = normalize(input.world_normal);

    let V = normalize(view.world_position.xyz - input.world_position.xyz);

    let NdotV = max(dot(Normal, V), 0.0001);
    var fresnel = clamp(1.0 - NdotV, 0.0, 1.0);

    fresnel = pow(fresnel, 8.0) * 8.0;

    let time_diff = clamp((globals.time - material.last_color_change - fresnel) / 0.1, 0.0, 1.0);
    let lerp_color = (1.0 - time_diff) * material.prev_color + time_diff * material.color;

    let s = textureSample(base_color_texture, base_color_sampler, view.viewport.xy);

    // return depth;
    return lerp_color * vec4(1.0, 1.0, 1.0, 1.0) * fresnel ;
}