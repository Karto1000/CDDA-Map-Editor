#import bevy_pbr::forward_io::VertexOutput

@group(1) @binding(0) var<uniform> viewport_width: f32;
@group(1) @binding(1) var<uniform> viewport_height: f32;
@group(1) @binding(2) var<uniform> tile_size: f32;
@group(1) @binding(3) var<uniform> offset: vec2<f32>;
@group(1) @binding(4) var<uniform> mouse_pos: vec2<f32>;
@group(1) @binding(5) var<uniform> map_size: vec2<f32>;

@fragment
fn fragment(output: VertexOutput) -> @location(0) vec4<f32> {
    var color = vec3<f32>(0.23, 0.26, 0.26);
    var alpha = 0.05;

    // EXAMPLE -> 32. or 64. ...
    var tile_start_x = abs(i32(mouse_pos.x + offset.x)) - abs(i32(mouse_pos.x + offset.x)) % i32(tile_size);
    var tile_start_y = abs(i32(mouse_pos.y + offset.y)) - abs(i32(mouse_pos.y + offset.y)) % i32(tile_size);

    if (mouse_pos.y + offset.y < 0.) {
        tile_start_y *= -1;
        tile_start_y -= i32(tile_size);
    }

    if (mouse_pos.x + offset.x < 0.) {
        tile_start_x *= -1;
        tile_start_x -= i32(tile_size);
    }

    if (
        i32(output.position.x + offset.x) > tile_start_x &&
        i32(output.position.x + offset.x) < tile_start_x + i32(tile_size) &&
        i32(output.position.y + offset.y) > tile_start_y &&
        i32(output.position.y + offset.y) < tile_start_y + i32(tile_size)
    ) {
         color.x = 1.0;
         color.y = 1.0;
         color.z = 1.0;
    }

    if (abs(i32(output.position.x + offset.x)) % i32(tile_size) == i32(0) || abs(i32(output.position.y + offset.y)) % i32(tile_size) == i32(0)) {
         color.x = 1.0;
         color.y = 1.0;
         color.z = 1.0;
         alpha = 0.025;
    }

    let tile_x = i32(output.position.x + offset.x) / i32(tile_size);
    let tile_y = i32(output.position.y + offset.y) / i32(tile_size);

    if (
        i32(abs(tile_x)) >= i32(map_size.x) ||
        i32(tile_x) <= 0 ||
        i32(abs(tile_y)) >= i32(map_size.y) ||
        i32(tile_y) <= 0
    ) {
        color.x -= 0.8;
        color.y -= 0.8;
        color.z -= 0.8;
    }

   return vec4<f32>(color, alpha);
}