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

use gfx::{Device, DeviceHelper};
use gfx::GlCommandBuffer;
use glfw::Context;
use cgmath::{Point3, Vector3, Matrix4, FixedArray, AffineMatrix3, Transform};

use collisions::{intersecting_triangle_id, Ray};
use world::World;
use rendering::Uniforms;
use plate_simulation::PlateSimulation;

mod camera;
mod polyhedron;
mod collisions;
mod world;
mod rendering;
mod plate_simulation;

include!("macros.rs")

struct GameState<'a> {
    wnd: &'a glfw::Window,
    dev: gfx::GlDevice,
    renderer: render::Renderer<gfx::GlCommandBuffer>,
    uniforms: Uniforms,
    camera: camera::Camera,

    update_accumulator: f32,

    world: World
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

        let plate_sim_poly = polyhedron::make_sphere(2);
        let mut plate_sim = PlateSimulation::new(&plate_sim_poly, 10u);
        plate_sim.simulate_plates(10u);

        let world_poly = polyhedron::make_sphere(4);
        let mut world = World::new(world_poly);
        world.apply_heights(&plate_sim);

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
            world: world
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
        let selected_id = intersecting_triangle_id(self.world.get_poly(), &ray);

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
    let batch = game.world.to_batch(&mut ctx, &mut game.dev);

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
