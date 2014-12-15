#![feature(phase)]
#![feature(slicing_syntax)]

extern crate time;
extern crate getopts;

extern crate glfw;
extern crate gfx;
extern crate cgmath;

#[phase(plugin)]
extern crate gfx_macros;
extern crate render;
extern crate device;

use std::rand::{SeedableRng, XorShiftRng};
use std::os;

use gfx::batch;
use gfx::{Device, DeviceHelper};
use gfx::GlCommandBuffer;
use glfw::Context;
use cgmath::{Point3, Vector3, Matrix4, FixedArray, AffineMatrix3, Transform};

use collisions::{intersecting_triangle_id, Ray};
use world::World;
use rendering::{PolyhedronBatch, Uniforms};
use plate_simulation::PlateSimulation;

mod camera;
mod polyhedron;
mod collisions;
mod world;
mod rendering;
mod plate_simulation;
mod cmdline;

include!("macros.rs")

enum DisplayState {
    World,
    PlateSimBatch(uint)
}

struct GameState<'a> {
    wnd: &'a glfw::Window,
    dev: gfx::GlDevice,
    renderer: render::Renderer<gfx::GlCommandBuffer>,
    uniforms: Uniforms,
    camera: camera::Camera,

    update_accumulator: f32,
    display_state: DisplayState,

    plate_sim_batches: Vec<(PolyhedronBatch, batch::Context)>,
    world: World,
}

fn world_from_plate_sim(sim: &PlateSimulation,
                        detail_level: uint) -> World {
    let mut world_poly = polyhedron::make_sphere(detail_level);
    let mut world = World::new(world_poly);

    time_it!("world.apply_heights", 0.0f64, {
        world.apply_heights(sim);
    });

    world
}

impl<'a> GameState<'a> {
    fn new(cmdline_args: &cmdline::Args,
           wnd: &'a glfw::Window) -> GameState<'a> {
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

        let mut rng: XorShiftRng = SeedableRng::from_seed(cmdline_args.rng_seed);
        let plate_sim_poly = polyhedron::make_sphere(cmdline_args.plate_sim_detail_level);
        let mut plate_sim = PlateSimulation::new(&plate_sim_poly,
                                                 cmdline_args.plate_sim_plates,
                                                 &mut rng);

        let mut plate_sim_batches = Vec::with_capacity(cmdline_args.plate_sim_steps + 1);
        let mut ctx = batch::Context::new();
        let sim_world = world_from_plate_sim(&plate_sim, cmdline_args.world_detail_level);
        plate_sim_batches.push((sim_world.to_batch(&mut ctx, &mut dev), ctx));
        for _ in range(0u, cmdline_args.plate_sim_steps) {
            let mut ctx = batch::Context::new();
            plate_sim.simulate_plates(1);

            let world = world_from_plate_sim(&plate_sim, cmdline_args.world_detail_level);
            plate_sim_batches.push((world.to_batch(&mut ctx, &mut dev), ctx));
        }

        let world_poly = polyhedron::make_sphere(cmdline_args.world_detail_level);
        let mut world = World::new(world_poly);

        time_it!("world.apply_heights", 0.0f64, {
            world.apply_heights(&plate_sim);
        });

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
            display_state: DisplayState::World,
            plate_sim_batches: plate_sim_batches,
            world: world
        }
    }

    fn toggle_display_state(&mut self,
                            forward: bool) {
        self.display_state = match self.display_state {
            DisplayState::World => if forward {
                DisplayState::PlateSimBatch(0)
            } else {
                DisplayState::PlateSimBatch(self.plate_sim_batches.len() - 1)
            },
            DisplayState::PlateSimBatch(idx) => {
                if forward {
                    if idx == self.plate_sim_batches.len() - 1 {
                        DisplayState::World
                    } else {
                        DisplayState::PlateSimBatch(idx + 1)
                    }
                } else if idx == 0 {
                    DisplayState::World
                } else {
                    DisplayState::PlateSimBatch(idx - 1)
                }
            }
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
                glfw::Key::Num0 =>
                    self.display_state = DisplayState::World,
                glfw::Key::Left =>
                    self.toggle_display_state(false),
                glfw::Key::Right =>
                    self.toggle_display_state(true),
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
    let mut ctx = batch::Context::new();
    let batch = time_it!("world.to_batch", 0.0f64, {
                             game.world.to_batch(&mut ctx, &mut game.dev)
                         });

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

        time_it!("render frame", 0.02f64, {
            game.renderer.clear(clear_data, gfx::COLOR | gfx::DEPTH, frame);

            match game.display_state {
                DisplayState::World =>
                    game.renderer.draw((&batch, &game.uniforms, &ctx), frame),
                DisplayState::PlateSimBatch(idx) => {
                    let &(ref batch, ref ctx) = &game.plate_sim_batches[idx];
                    game.renderer.draw((batch, &game.uniforms, ctx), frame);
                }
            };

            game.dev.submit(game.renderer.as_buffer());
            game.renderer.reset();

            game.wnd.swap_buffers();
        });
    }
}

fn main() {
    let cmdline_args = match cmdline::Args::parse() {
        Ok(args) => args,
        Err(exit_status) => {
            os::set_exit_status(exit_status);
            return;
        }
    };

    let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.set_error_callback(glfw::FAIL_ON_ERRORS);

    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenglForwardCompat(true));
    glfw.window_hint(glfw::WindowHint::OpenglProfile(glfw::OpenGlProfileHint::Core));

    let (wnd, events) = glfw.create_window(cmdline_args.resolution[0],
                                           cmdline_args.resolution[1],
                                           "world", glfw::WindowMode::Windowed)
                            .expect("Failed to create GLFW window");
    wnd.make_current();
    wnd.set_key_polling(true);

    let (width, height) = wnd.get_framebuffer_size();
    let frame = gfx::Frame::new(width as u16, height as u16);

    let mut state = GameState::new(&cmdline_args, &wnd);
    game_loop(&mut state, &glfw, &events, &frame);
}
