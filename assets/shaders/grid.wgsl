#import bevy_pbr::forward_io::VertexOutput

@group(1) @binding(0) var<uniform> viewport_width: f32;
@group(1) @binding(1) var<uniform> viewport_height: f32;
@group(1) @binding(2) var<uniform> tile_size: f32;

@fragment
fn fragment(output: VertexOutput) -> @location(0) vec4<f32> {
    var color = vec3<f32>(0.0, 0.0, 0.0);

    let offset_x = viewport_width % tile_size;
    let offset_y = -(viewport_height % tile_size);

    if (u32(output.position.x) % u32(tile_size) == u32(0) || u32(output.position.y) % u32(tile_size) == u32(0)) {
         color.x = 1.0;
         color.y = 1.0;
         color.z = 1.0;
    }

   return vec4<f32>(color, 0.05);
}