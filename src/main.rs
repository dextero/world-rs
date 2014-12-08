#![feature(phase)]
#![feature(macro_rules)]

extern crate time;
extern crate glfw;
extern crate gfx;
extern crate cgmath;

#[phase(plugin)]
extern crate gfx_macros;
extern crate render;
extern crate device;
extern crate core;

use std::vec::Vec;
use std::iter::IteratorExt;
use std::num::{Float, FloatMath};
use core::f32::consts::{PI, FRAC_PI_3};

use gfx::{Device, DeviceHelper, ToSlice};
use gfx::GlCommandBuffer;
use glfw::Context;
use cgmath::{Vector, Point, Point3, Vector3, Matrix4, FixedArray, AffineMatrix3, Transform};

use polyhedron::Polyhedron;
use collisions::{intersecting_triangle_id, Ray};

mod camera;
mod polyhedron;
mod collisions;

macro_rules! time_it(
    ($name:expr, $limit:expr, $expr:block) => ({
        let __start_time = time::precise_time_s();
        let __ret = $expr;
        let __end_time = time::precise_time_s();
        println!("{}: {}s", $name, __end_time - __start_time);
        if ($limit) as f64 > 0.0 && __end_time - __start_time > ($limit) as f64 {
            panic!("time limit ({}s) exceeded", $limit);
        }
        __ret
    });
)

#[vertex_format]
struct Vertex {
    #[name = "a_pos"]
    pos: [f32, ..3],

    #[name = "a_color"]
    color: [f32, ..4],

    #[name = "a_id"]
    id: i32
}

#[shader_param(PolyhedronBatch)]
struct Uniforms {
    #[name = "u_world"]
    world_mat: [[f32, ..4], ..4],

    #[name = "u_view"]
    view_mat: [[f32, ..4], ..4],

    #[name = "u_proj"]
    proj_mat: [[f32, ..4], ..4],

    #[name = "u_highlighted_id"]
    highlighted_id: i32
}

static VS_SOURCE: gfx::ShaderSource<'static> = shaders! {
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

static FS_SOURCE: gfx::ShaderSource<'static> = shaders! {
GLSL_150: b"
#version 150 core

in vec4 v_color;

out vec4 out_color;

void main() {
    out_color = v_color;
}
"
};

fn color_for_y(col: f32, y: f32) -> f32 {
    (y.abs() * 0.8 + col).min(1.0)
}

fn color_for_pos(pos: &Vector3<f32>) -> [f32, ..4] {
    let hue = (pos.z.atan2(pos.x) + PI) / FRAC_PI_3;
    let c = 0.5;
    let x = c * (1.0 - (hue % 2.0 - 1.0).abs());

    let rgb = match hue {
        0.0 ... 1.0  => [c, x, 0.0],
        1.0 ... 2.0 => [x, c, 0.0],
        2.0 ... 3.0 => [0.0, c, x],
        3.0 ... 4.0 => [0.0, x, c],
        4.0 ... 5.0 => [x, 0.0, c],
        _                 => [c, 0.0, x]
    };

    [color_for_y(rgb[0], pos.y),
     color_for_y(rgb[1], pos.y),
     color_for_y(rgb[2], pos.y),
     1.0]
}

fn polyhedron_to_vertices(poly: &Polyhedron) -> Vec<Vertex> {
    let mut vertices = Vec::new();
    vertices.reserve(poly.faces.len() * 3u);

    for face_idx in range(0u, poly.faces.len()) {
        let face = &poly.faces[face_idx];
        let verts = [&poly.vertices[face.vertex_indices[0]].pos,
                     &poly.vertices[face.vertex_indices[1]].pos,
                     &poly.vertices[face.vertex_indices[2]].pos];

        let mean_pos = verts[0].add(verts[1]).add(verts[2]).div_s(3.0);
        let face_col = color_for_pos(&mean_pos);

        for &v in verts.iter() {
            vertices.push(Vertex {
                pos: *v.as_fixed(),
                color: face_col,
                id: face_idx as i32
            });
        }
    }

    vertices
}

fn polyhedron_to_batch(poly: &Polyhedron,
                       ctx: &mut gfx::batch::Context,
                       dev: &mut gfx::GlDevice) -> PolyhedronBatch {
    let vertices = polyhedron_to_vertices(poly);
    let mesh = dev.create_mesh(vertices.as_slice());

    let indices = range(0u32, vertices.len() as u32).collect::<Vec<u32>>();
    let idx_slice = dev.create_buffer_static(indices.as_slice())
                       .to_slice(gfx::PrimitiveType::TriangleList);

    let shader = dev.link_program(VS_SOURCE.clone(), FS_SOURCE.clone())
                    .unwrap();
    let state = gfx::DrawState::new().depth(gfx::state::Comparison::LessEqual, true);

    ctx.make_batch(&shader, &mesh, idx_slice, &state).unwrap()
}

struct GameState<'a> {
    wnd: &'a glfw::Window,
    dev: gfx::GlDevice,
    renderer: render::Renderer<gfx::GlCommandBuffer>,
    uniforms: Uniforms,
    camera: camera::Camera,

    update_accumulator: f32,

    poly: Polyhedron
}

impl<'a> GameState<'a> {
    fn new(wnd: &glfw::Window) -> GameState {
        let (width, height) = wnd.get_size();
        let aspect_ratio = width as f32 / height as f32;
        let view_angle = cgmath::deg(45.0f32);
        let view: AffineMatrix3<f32> = Transform::look_at(
            &Point3::new(-5.0f32, -5.0, 0.0),
            &Point3::new(0.0f32, 0.0, 0.0),
            &Vector3::unit_z()
        );

        let mut dev = gfx::GlDevice::new(|s| wnd.get_proc_address(s));
        let renderer = dev.create_renderer();

        GameState {
            wnd: wnd,
            dev: dev,
            renderer: renderer,
            uniforms: Uniforms {
                world_mat: Matrix4::identity().into_fixed(),
                view_mat: view.mat.into_fixed(),
                proj_mat: cgmath::perspective(view_angle, aspect_ratio, 0.001, 100.0).into_fixed(),
                highlighted_id: -1
            },
            camera: camera::Camera::new(),
            update_accumulator: 0.0,
            poly: polyhedron::make_sphere(4)
        }
    }

    pub fn handle_event(&mut self, evt: &glfw::WindowEvent) {
        match *evt {
            glfw::WindowEvent::Key(key, _, action, _) => match key {
                glfw::Key::Escape =>
                    self.wnd.set_should_close(true),
                glfw::Key::A =>
                    self.camera.handle_key_action(camera::CAMERA_LEFT, action),
                glfw::Key::D =>
                    self.camera.handle_key_action(camera::CAMERA_RIGHT, action),
                glfw::Key::W =>
                    self.camera.handle_key_action(camera::CAMERA_UP, action),
                glfw::Key::S =>
                    self.camera.handle_key_action(camera::CAMERA_DOWN, action),
                glfw::Key::KpAdd | glfw::Key::Equal =>
                    self.camera.handle_key_action(camera::CAMERA_ZOOM_IN, action),
                glfw::Key::KpSubtract | glfw::Key::Minus =>
                    self.camera.handle_key_action(camera::CAMERA_ZOOM_OUT, action),
                _ => {}
            },
            _ => {}
        }
    }

    fn update_step(&mut self, dt: f32) {
        self.camera.update(dt);
        self.uniforms.view_mat = self.camera.to_view_matrix().into_fixed();

        let ray = Ray::towards_center(&self.camera.get_eye());
        let selected_id = intersecting_triangle_id(&self.poly, &ray);

        self.uniforms.highlighted_id = match selected_id {
            Some(id) => id as i32,
            None => -1
        }
    }

    pub fn update(&mut self, dt: f32) {
        const UPDATE_STEP: f32 = 1.0 / 30.0;

        self.update_accumulator += dt;
        while self.update_accumulator > UPDATE_STEP {
            self.update_accumulator -= UPDATE_STEP;

            time_it!("update step", 0.02f64, {
                self.update_step(UPDATE_STEP);
            });
        }
    }
}

fn game_loop<'a>(game: &mut GameState<'a>,
                 glfw: &glfw::Glfw,
                 events: &std::comm::Receiver<(f64, glfw::WindowEvent)>,
                 frame: &gfx::Frame) {
    let mut ctx = gfx::batch::Context::new();

    let batch = polyhedron_to_batch(&game.poly, &mut ctx, &mut game.dev);

    let clear_data = gfx::ClearData {
        color: [0.0, 0.0, 0.2, 1.0],
        depth: 1.0,
        stencil: 0
    };

    let mut frame_start = time::now().to_timespec();

    while !game.wnd.should_close() {
        glfw.poll_events();
        for (_, evt) in glfw::flush_messages(events) {
            game.handle_event(&evt);
        }

        let frame_end = time::now().to_timespec();
        let delta_time = (frame_end - frame_start).num_milliseconds() as f32 / 1000.0;
        game.update(delta_time);
        frame_start = frame_end;

        game.renderer.clear(clear_data, gfx::COLOR | gfx::DEPTH, frame);
        game.renderer.draw((&batch, &game.uniforms, &ctx), frame);
        game.dev.submit(game.renderer.as_buffer());
        game.renderer.reset();

        game.wnd.swap_buffers();
    }
}

fn main() {
    let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.set_error_callback(glfw::FAIL_ON_ERRORS);

    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenglForwardCompat(true));
    glfw.window_hint(glfw::WindowHint::OpenglProfile(glfw::OpenGlProfileHint::Core));

    let (wnd, events) = glfw.create_window(1000, 1000, "world", glfw::WindowMode::Windowed)
                            .expect("Failed to create GLFW window");
    wnd.make_current();
    wnd.set_key_polling(true);

    let (width, height) = wnd.get_framebuffer_size();
    let frame = gfx::Frame::new(width as u16, height as u16);

    let mut state = GameState::new(&wnd);
    game_loop(&mut state, &glfw, &events, &frame);
}
