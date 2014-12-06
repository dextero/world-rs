#![feature(phase)]

extern crate time;
extern crate glfw;
extern crate gfx;
extern crate cgmath;

#[phase(plugin)]
extern crate gfx_macros;
extern crate render;
extern crate device;
extern crate core;

use gfx::{Device, DeviceHelper, ToSlice};
use gfx::GlCommandBuffer;
use glfw::Context;
use cgmath::FixedArray;
use std::vec::Vec;
use std::iter::IteratorExt;
use std::num::FloatMath;
use core::f32;

mod polyhedron;

#[vertex_format]
struct Vertex {
    #[name = "a_pos"]
    pos: [f32, ..3],

    #[name = "a_color"]
    color: [f32, ..4]
}

#[shader_param(PolyhedronBatch)]
struct Uniforms {
    #[name = "u_world"]
    world_mat: [[f32, ..4], ..4],

    #[name = "u_view"]
    view_mat: [[f32, ..4], ..4],

    #[name = "u_proj"]
    proj_mat: [[f32, ..4], ..4]
}

static VS_SOURCE: gfx::ShaderSource<'static> = shaders! {
GLSL_150: b"
#version 150 core

in vec3 a_pos;
in vec4 a_color;

out vec4 v_color;

uniform mat4 u_world;
uniform mat4 u_view;
uniform mat4 u_proj;

void main() {
    gl_Position = u_proj * u_view * u_world * vec4(a_pos, 1.0);
    v_color = a_color;
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

fn random_grey() -> [f32, ..4] {
    let brightness = std::rand::random::<f32>();
    [brightness, brightness, brightness, 1.0f32]
}

fn polyhedron_to_vertices(poly: &polyhedron::Polyhedron) -> Vec<Vertex> {
    let mut vertices = Vec::new();
    vertices.reserve(poly.faces.len() * 3u);

    for &face in poly.faces.iter() {
        let face_col = random_grey();

        for i in range(0u, 3u) {
            let pos = &poly.vertices[face.vertex_indices[i]].pos;

            vertices.push(Vertex {
                pos: *pos.as_fixed(),
                color: face_col
            });
        }
    }

    vertices
}

fn polyhedron_to_batch(poly: &polyhedron::Polyhedron,
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

bitflags! {
    flags CameraRotationFlags: u32 {
        const CAMERA_STILL = 0x0,
        const CAMERA_UP    = 0x1,
        const CAMERA_DOWN  = 0x2,
        const CAMERA_LEFT  = 0x4,
        const CAMERA_RIGHT = 0x8
    }
}

struct Camera {
    angle_xz: f32,
    angle_y: f32,

    rotate: CameraRotationFlags
}

impl Camera {
    fn new() -> Camera {
        Camera {
            angle_xz: 0.0,
            angle_y: 0.0,
            rotate: CAMERA_STILL
        }
    }

    pub fn to_view_matrix(&mut self) -> cgmath::Matrix4<f32> {
        const RADIUS: f32 = 5.0;

        let (sin_xz, cos_xz) = self.angle_xz.sin_cos();
        let (sin_y, cos_y) = self.angle_y.sin_cos();

        let x = RADIUS * cos_xz * cos_y;
        let y = RADIUS * sin_y;
        let z = RADIUS * sin_xz * cos_y;

        cgmath::Matrix4::look_at(&cgmath::Point3::new(x, y, z),
                                 &cgmath::Point3::new(0.0, 0.0, 0.0),
                                 &cgmath::Vector3::unit_y())
    }

    pub fn update(&mut self, dt: f32) {
        const EPSILON: f32 = 0.00001;

        let left = self.rotate.contains(CAMERA_LEFT);
        let right = self.rotate.contains(CAMERA_RIGHT);
        let up = self.rotate.contains(CAMERA_UP);
        let down = self.rotate.contains(CAMERA_DOWN);

        let dir_xz = right as f32 - left as f32;
        let dir_y = down as f32 - up as f32;

        self.angle_xz = (self.angle_xz + dt * dir_xz) % f32::consts::PI_2;
        self.angle_y = (self.angle_y + dt * dir_y).min(f32::consts::FRAC_PI_2 - EPSILON)
                                                  .max(-f32::consts::FRAC_PI_2 + EPSILON);
    }

    pub fn handle_key_action(&mut self,
                             dir: CameraRotationFlags,
                             action: glfw::Action) {
        match action {
            glfw::Action::Press => self.rotate.insert(dir),
            glfw::Action::Release => self.rotate.remove(dir),
            _ => {}
        }
    }
}

struct GameState<'a> {
    wnd: &'a glfw::Window,
    dev: gfx::GlDevice,
    renderer: render::Renderer<gfx::GlCommandBuffer>,
    uniforms: Uniforms,
    camera: Camera
}

impl<'a> GameState<'a> {
    fn new(wnd: &glfw::Window) -> GameState {
        let (width, height) = wnd.get_size();
        let aspect_ratio = width as f32 / height as f32;
        let view_angle = cgmath::deg(45.0f32);
        let view: cgmath::AffineMatrix3<f32> = cgmath::Transform::look_at(
            &cgmath::Point3::new(-5.0f32, -5.0, 0.0),
            &cgmath::Point3::new(0.0f32, 0.0, 0.0),
            &cgmath::Vector3::unit_z()
        );

        let mut dev = gfx::GlDevice::new(|s| wnd.get_proc_address(s));
        let renderer = dev.create_renderer();

        GameState {
            wnd: wnd,
            dev: dev,
            renderer: renderer,
            uniforms: Uniforms {
                world_mat: cgmath::Matrix4::identity().into_fixed(),
                view_mat: view.mat.into_fixed(),
                proj_mat: cgmath::perspective(view_angle, aspect_ratio, 1.0, 100.0).into_fixed()
            },
            camera: Camera::new()
        }
    }

    pub fn handle_event(&mut self, evt: &glfw::WindowEvent) {
        match *evt {
            glfw::WindowEvent::Key(key, _, action, _) => match key {
                glfw::Key::Escape =>
                    self.wnd.set_should_close(true),
                glfw::Key::A =>
                    self.camera.handle_key_action(CAMERA_LEFT, action),
                glfw::Key::D =>
                    self.camera.handle_key_action(CAMERA_RIGHT, action),
                glfw::Key::W =>
                    self.camera.handle_key_action(CAMERA_UP, action),
                glfw::Key::S =>
                    self.camera.handle_key_action(CAMERA_DOWN, action),
                _ => {}
            },
            _ => {}
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.camera.update(dt);
        self.uniforms.view_mat = self.camera.to_view_matrix().into_fixed();
    }
}

fn game_loop<'a>(game: &mut GameState<'a>,
                 glfw: &glfw::Glfw,
                 events: &std::comm::Receiver<(f64, glfw::WindowEvent)>,
                 frame: &gfx::Frame) {
    let mut ctx = gfx::batch::Context::new();

    let sphere = polyhedron::make_sphere();
    let batch = polyhedron_to_batch(&sphere, &mut ctx, &mut game.dev);

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

    let (wnd, events) = glfw.create_window(800, 600, "world", glfw::WindowMode::Windowed)
                            .expect("Failed to create GLFW window");
    wnd.make_current();
    wnd.set_key_polling(true);

    let (width, height) = wnd.get_framebuffer_size();
    let frame = gfx::Frame::new(width as u16, height as u16);

    let mut state = GameState::new(&wnd);
    game_loop(&mut state, &glfw, &events, &frame);
}
