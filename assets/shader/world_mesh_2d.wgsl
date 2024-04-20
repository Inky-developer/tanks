// Import the standard 2d mesh uniforms and set their bind groups
#import bevy_sprite::mesh2d_functions

// The structure of the vertex buffer is as specified in `specialize()`
struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) color: u32,
    @location(2) local_pos: vec2<f32>,
    @location(3) neighbors: u32,
};

struct VertexOutput {
    // The vertex shader must set the on-screen position of the vertex
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local_pos: vec2<f32>,
    @location(2) neighbors: u32,
};

/// Entry point for the vertex shader
@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    // Project the world position of the mesh into screen position
    let model = mesh2d_functions::get_model_matrix(vertex.instance_index);
    out.clip_position = mesh2d_functions::mesh2d_position_local_to_clip(model, vec4<f32>(vertex.position, 1.0));
    // Unpack the `u32` from the vertex buffer into the `vec4<f32>` used by the fragment shader
    out.color = vec4<f32>((vec4<u32>(vertex.color) >> vec4<u32>(0u, 8u, 16u, 24u)) & vec4<u32>(255u)) / 255.0;
    out.local_pos = vertex.local_pos;
    out.neighbors = vertex.neighbors;
    return out;
}

// The input of the fragment shader must correspond to the output of the vertex shader for all `location`s
struct FragmentInput {
    @location(0) color: vec4<f32>,
    @location(1) local_pos: vec2<f32>,
    @location(2) neighbors: u32,
};

/// Entry point for the fragment shader
@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    var color = in.color;

    let top = (in.neighbors & 1) == 1;
    let left = ((in.neighbors >> 1) & 1) == 1;
    let bottom = ((in.neighbors >> 2) & 1) == 1;
    let right = ((in.neighbors >> 3) & 1) == 1;
    let self_on = ((in.neighbors >> 4) & 1) == 1;

    let total = u32(top) + u32(left) + u32(bottom) + u32(right);
    var pixel_on = false;

    if self_on {
        if total >= 3 || (top && bottom) || (left && right) {
            pixel_on |= true;
        } 

        if total == 0 {
            pixel_on |= abs(in.local_pos.x - 0.5) + abs(in.local_pos.y - 0.5) < 0.25;   
        }

        let in_bottom_right_half = in.local_pos.y < in.local_pos.x;
        let in_top_right_half = (1.0 - in.local_pos.y) < in.local_pos.x;
        let in_top_left_half = !in_bottom_right_half;
        let in_bottom_left_half = !in_top_right_half;
        if top {
            pixel_on |= in_top_left_half && in_top_right_half;
        }
        if left {
            pixel_on |= in_top_left_half && in_bottom_left_half;
        }
        if bottom {
            pixel_on |= in_bottom_left_half && in_bottom_right_half;
        }
        if right {
            pixel_on |= in_top_right_half && in_bottom_right_half;
        }
    }


    if !pixel_on {
        color.a = 0.0;
    }

    return color;
}

