extern crate gfx;

#[phase(plugin)]
extern crate gfx_macros;

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

