#![allow(dead_code)]

extern crate cgmath;
extern crate glfw;

use std::num::Float;
use std::num::FloatMath;
use core::f32::consts::{PI_2, FRAC_PI_2};

use cgmath::{Point3, Vector3, Matrix4};
use glfw::Action;

const DEFAULT_CAMERA_DISTANCE: f32 = 5.0;

bitflags! {
    flags CameraRotationFlags: u32 {
        const CAMERA_STILL    = 0x0,
        const CAMERA_UP       = 0x1,
        const CAMERA_DOWN     = 0x2,
        const CAMERA_LEFT     = 0x4,
        const CAMERA_RIGHT    = 0x8,
        const CAMERA_ZOOM_IN  = 0x10,
        const CAMERA_ZOOM_OUT = 0x20,
    }
}

pub struct Camera {
    angle_xz: f32,
    angle_y: f32,
    distance: f32,

    eye: Point3<f32>,

    rotate: CameraRotationFlags
}

fn eye_from_angles_distance(xz: f32, y: f32, dist: f32) -> Point3<f32> {
    let (sin_xz, cos_xz) = xz.sin_cos();
    let (sin_y, cos_y) = y.sin_cos();

    let x = dist * cos_xz * cos_y;
    let y = dist * sin_y;
    let z = dist * sin_xz * cos_y;

    Point3::new(x, y, z)
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            angle_xz: 0.0,
            angle_y: 0.0,
            distance: DEFAULT_CAMERA_DISTANCE,
            eye: eye_from_angles_distance(0.0, 0.0, DEFAULT_CAMERA_DISTANCE),
            rotate: CAMERA_STILL
        }
    }

    pub fn get_eye(&self) -> Point3<f32> {
        self.eye
    }

    pub fn to_view_matrix(&mut self) -> Matrix4<f32> {
        self.eye = eye_from_angles_distance(self.angle_xz, self.angle_y,
                                            self.distance);

        Matrix4::look_at(&self.eye,
                         &Point3::new(0.0, 0.0, 0.0),
                         &Vector3::unit_y())
    }

    pub fn update(&mut self, dt: f32) {
        const EPSILON: f32 = 0.00001;
        const MIN_CAMERA_DISTANCE: f32 = 1.1;

        let left = self.rotate.contains(CAMERA_LEFT);
        let right = self.rotate.contains(CAMERA_RIGHT);
        let up = self.rotate.contains(CAMERA_UP);
        let down = self.rotate.contains(CAMERA_DOWN);
        let zoom_in = self.rotate.contains(CAMERA_ZOOM_IN);
        let zoom_out = self.rotate.contains(CAMERA_ZOOM_OUT);

        let zoom_speed = 1.0 / (1.0 + (-self.distance / 3.0 + 4.0).exp());
        let rotate_speed = 0.25 + 3.0 / (1.0 + (-self.distance / 3.0 + 3.0).exp());

        let dir_xz = (left as f32 - right as f32) * rotate_speed;
        let dir_y = (up as f32 - down as f32) * rotate_speed;
        let dir_zoom = (zoom_out as f32 - zoom_in as f32) * zoom_speed;

        self.angle_xz = (self.angle_xz + dt * dir_xz) % PI_2;
        self.angle_y = (self.angle_y + dt * dir_y).min(FRAC_PI_2 - EPSILON)
                                                  .max(-FRAC_PI_2 + EPSILON);
        self.distance = (self.distance + dir_zoom).max(MIN_CAMERA_DISTANCE);
    }

    pub fn handle_key_action(&mut self,
                             dir: CameraRotationFlags,
                             action: Action) {
        match action {
            Action::Press => self.rotate.insert(dir),
            Action::Release => self.rotate.remove(dir),
            _ => {}
        }
    }
}

