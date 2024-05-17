#import bevy_pbr::forward_io::VertexOutput

@group(2) @binding(0) var<uniform> tile_size: f32;
@group(2) @binding(1) var<uniform> offset: vec2<f32>;
@group(2) @binding(2) var<uniform> mouse_pos: vec2<f32>;
@group(2) @binding(3) var<uniform> map_size: vec2<f32>;
@group(2) @binding(4) var<uniform> is_cursor_captured: i32;
@group(2) @binding(5) var<uniform> scale_factor: f32;
@group(2) @binding(6) var<uniform> inside_grid_color: vec3<f32>;
@group(2) @binding(7) var<uniform> outside_grid_color: vec3<f32>;

fn to_linear(nonlinear_color: vec4<f32>) -> vec4<f32> {
    let cutoff = step(nonlinear_color, vec4<f32>(0.04045));
    let higher = pow((nonlinear_color + vec4<f32>(0.055)) / vec4<f32>(1.055), vec4<f32>(2.4));
    let lower = nonlinear_color / vec4<f32>(12.92);
    return mix(higher, lower, cutoff);
}

@fragment
fn fragment(output: VertexOutput) -> @location(0) vec4<f32> {
    var color = inside_grid_color;
    var alpha = 1.;

    let offset = vec2<f32>(offset.x * scale_factor, offset.y * scale_factor);
    let tile_size = tile_size * scale_factor;
    let mouse_pos = vec2<f32>(mouse_pos.x * scale_factor, mouse_pos.y * scale_factor);

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

    // Highlight the tile that the mouse is currently on
    if (is_cursor_captured == 0) {
        if (
            i32(output.position.x + offset.x) > tile_start_x &&
            i32(output.position.x + offset.x) < tile_start_x + i32(tile_size) &&
            i32(output.position.y + offset.y) > tile_start_y &&
            i32(output.position.y + offset.y) < tile_start_y + i32(tile_size)
        ) {
             color.x = 1.0;
             color.y = 1.0;
             color.z = 1.0;
             alpha = 0.05;
        }
    }

    // Make Lines at 0, 0 more noticable
    if (abs(i32(output.position.x + offset.x)) % i32(tile_size) == i32(0) ||
    abs(i32(output.position.y + offset.y)) % i32(tile_size) == i32(0)) {
         color.x = 1.0;
         color.y = 1.0;
         color.z = 1.0;
         alpha = 0.1;
    }

    let tile_x = i32(output.position.x + tile_size + offset.x) / i32(tile_size);
    let tile_y = i32(output.position.y + tile_size + offset.y) / i32(tile_size);

    // Make any tiles on the ouside of the map darker
    if (
        i32(abs(tile_x)) > i32(map_size.x) ||
        i32(tile_x) <= 0 ||
        i32(abs(tile_y)) > i32(map_size.y) ||
        i32(tile_y) <= 0
    ) {
        color = outside_grid_color;
    }

    // We need to call to_linear here because for some reason the color that WGSL displays is not linear????
    // From: https://github.com/bevyengine/bevy/discussions/8937
    return to_linear(vec4<f32>(color, alpha));
}