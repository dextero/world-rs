extern crate gfx;

#[phase(plugin)]
extern crate gfx_macros;

use std::f32::consts::{PI_2, FRAC_PI_3};
use std::num::{Float};

#[vertex_format]
pub struct Vertex {
    #[name = "a_pos"]
    pub pos: [f32, ..3],

    #[name = "a_color"]
    pub color: [f32, ..4],

    #[name = "a_id"]
    pub id: i32
}

#[shader_param(PolyhedronBatch)]
pub struct Uniforms {
    #[name = "u_world"]
    pub world_mat: [[f32, ..4], ..4],

    #[name = "u_view"]
    pub view_mat: [[f32, ..4], ..4],

    #[name = "u_proj"]
    pub proj_mat: [[f32, ..4], ..4],

    #[name = "u_highlighted_id"]
    pub highlighted_id: i32
}

pub static VS_SOURCE: gfx::ShaderSource<'static> = shaders! {
GLSL_150: b"
#version 150 core

in vec3 a_pos;
in vec4 a_color;
in int a_id;

out vec4 v_color;

uniform mat4 u_world;
uniform mat4 u_view;
uniform mat4 u_proj;
uniform int u_highlighted_id;

void main() {
    gl_Position = u_proj * u_view * u_world * vec4(a_pos, 1.0);
    if (a_id == u_highlighted_id) {
        v_color = -vec4(1.0, 1.0, 1.0, 0.0) * 0.3 + a_color;
    } else {
        v_color = a_color;
    }
}
"
};

pub static FS_SOURCE: gfx::ShaderSource<'static> = shaders! {
GLSL_150: b"
#version 150 core

in vec4 v_color;

out vec4 out_color;

void main() {
    out_color = v_color;
}
"
};

pub fn color_for_hue(hue: f32) -> [f32, ..4] {
    let c = 0.5;
    let x = c * (1.0 - (hue % 2.0 - 1.0).abs());

    let rgb = match hue {
        0.0 ... 1.0 => [c, x, 0.0],
        1.0 ... 2.0 => [x, c, 0.0],
        2.0 ... 3.0 => [0.0, c, x],
        3.0 ... 4.0 => [0.0, x, c],
        4.0 ... 5.0 => [x, 0.0, c],
        _           => [c, 0.0, x]
    };

    [rgb[0], rgb[1], rgb[2], 1.0]
}

pub fn color_by_height(height: f32, min_height: f32, max_height: f32) -> [f32, ..4] {
    let diff = max_height - min_height;
    let relative_height = (height - min_height) / diff;
    let hue = ((FRAC_PI_3 * 4.0 - relative_height * PI_2) + PI_2) % PI_2;
    color_for_hue(hue)
}

pub fn color_by_index(idx: uint,
                      max_idx: uint) -> [f32, ..4] {
    let hue = idx as f32 / max_idx as f32 * PI_2;
    color_for_hue(hue)
}

