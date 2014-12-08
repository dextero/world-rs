extern crate cgmath;
extern crate gfx;

use std::vec::Vec;
use std::rand::{task_rng, Rng};
use core::f32::consts::{PI, FRAC_PI_3};
use std::num::{Float, FloatMath};

use time;
use cgmath::{EuclideanVector, Vector, Vector3, Basis3, Rotation, Rotation3, FixedArray, Rad, rad};
use gfx::batch::Context;
use gfx::{GlDevice, Device, DeviceHelper, ToSlice};

use polyhedron::{Edge, Polyhedron};
use rendering::{PolyhedronBatch, Vertex};
use rendering;

include!("macros.rs")

struct PlatePoint {
    pos: Vector3<f32>,
    nbr_indices: Vec<uint>,
    pub speed: Rad<f32>
}

impl PlatePoint {
    fn new(pos: &Vector3<f32>,
           nbr_indices: Vec<uint>) -> PlatePoint {
        PlatePoint {
            pos: *pos,
            nbr_indices: nbr_indices,
            speed: rad(1.0)
        }
    }

    fn move_around(&mut self, move_axis: &Vector3<f32>) {
        let rot: Basis3<f32> = Rotation3::from_axis_angle(move_axis, self.speed);
        self.pos = rot.rotate_vector(&self.pos);
    }
}

struct Plate {
    pub vertex_indices: Vec<uint>,
    pub move_axis: Vector3<f32>,
    pub move_speed: Rad<f32>,
    pub height: f32
}

fn random_axis() -> Vector3<f32> {
    let mut rng = task_rng();

    Vector3::new(rng.gen_range(0.0001f32, 1.0),
                 rng.gen_range(0.0001f32, 1.0),
                 rng.gen_range(0.0001f32, 1.0)).normalize()
}

impl Plate {
    fn new(vertex_indices: Vec<uint>,
           move_axis: &Vector3<f32>,
           move_speed: Rad<f32>,
           height: f32) -> Plate {
        Plate {
            vertex_indices: vertex_indices,
            move_axis: *move_axis,
            move_speed: move_speed,
            height: height
        }
    }

    fn from_points(vertex_indices: Vec<uint>) -> Plate {
        let mut rng = task_rng();

        Plate::new(vertex_indices,
                   &random_axis(),
                   rad(rng.gen_range(0.01f32, 0.1)),
                   rng.gen_range(0.8f32, 1.2))
    }

    fn simulate(&self, vertices: &mut Vec<PlatePoint>) {
        for &idx in self.vertex_indices.iter() {
            vertices[idx].move_around(&self.move_axis);
        }
    }
}

pub struct World {
    poly: Polyhedron,
    verts: Vec<PlatePoint>,
    plates: Vec<Plate>
}

fn get_nbr_idx(edge: &Edge, vert_idx: uint) -> uint {
    if edge.vertex_indices[0] == vert_idx {
        edge.vertex_indices[1]
    } else {
        edge.vertex_indices[0]
    }
}

fn assign_neighbors(plate_points: &mut Vec<Vec<uint>>,
                    plate_id_for_verts: &mut Vec<int>,
                    plate_idx: uint,
                    nbr_indices: &Vec<uint>) -> uint {
    let mut num_assigned = 0u;

    for &nbr_idx in nbr_indices.iter() {
        if plate_id_for_verts[nbr_idx] == -1 {
            plate_id_for_verts[nbr_idx] = plate_idx as int;
            plate_points[plate_idx].push(nbr_idx);
            num_assigned += 1;
        }
    }

    num_assigned
}

fn flood_fill(verts: &Vec<PlatePoint>,
              plate_id_for_verts: &mut Vec<int>,
              plate_points: &mut Vec<Vec<uint>>) {
    let mut filled_points = plate_points.len();

    while filled_points < verts.len() {
        for plate_idx in range(0u, plate_points.len()) {
            for point_idx in range(0u, plate_points[plate_idx].len()) {
                filled_points += assign_neighbors(plate_points, plate_id_for_verts, plate_idx,
                                                  &verts[point_idx].nbr_indices);
            }
        }
    }
}

fn random_partition(verts: &Vec<PlatePoint>,
                    num_plates: uint) -> Vec<Plate> {
    let mut plate_id_for_verts = Vec::from_elem(verts.len(), -1i);
    let mut plate_points = Vec::with_capacity(num_plates);

    for plate_idx in range(0u, num_plates) {
        loop {
            let idx = task_rng().gen_range(0u, verts.len());

            if plate_id_for_verts[idx] == -1 {
                plate_id_for_verts[idx] = plate_idx as int;

                let plate_idx = plate_points.len();
                plate_points.push(Vec::new());
                plate_points[plate_idx].push(idx);

                break
            }
        }
    }

    flood_fill(verts, &mut plate_id_for_verts, &mut plate_points);
    plate_points.iter().map(|points| Plate::from_points(points.clone())).collect()
}

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


impl World {
    pub fn new(poly: Polyhedron,
               num_plates: uint) -> World {
        if poly.faces.len() < num_plates {
            panic!("cannot split {} faces into {} plates", poly.faces.len(), num_plates);
        }

        println!("splitting world into {} plates", num_plates);
        let mut verts = Vec::with_capacity(poly.vertices.len());

        for vert_idx in range(0u, poly.vertices.len()) {
            let vert = &poly.vertices[vert_idx];
            let nbr_indices = vert.edge_indices.iter()
                                  .map(|&i| get_nbr_idx(&poly.edges[i], vert_idx))
                                  .collect();
            verts.push(PlatePoint::new(&vert.pos, nbr_indices));
        }

        let plates = random_partition(&verts, num_plates);
        for plate in plates.iter() {
            for &vert_idx in plate.vertex_indices.iter() {
                verts[vert_idx].speed = plate.move_speed;
            }
        }

        World {
            poly: poly,
            verts: verts,
            plates: plates
        }
    }

    pub fn get_poly(&self) -> &Polyhedron {
        &self.poly
    }

    fn simulate_plates_step(&mut self) {
        for plate in self.plates.iter() {
            plate.simulate(&mut self.verts);
        }
    }

    pub fn simulate_plates(&mut self, steps: uint) {
        println!("simulating {} tectonic plate steps", steps);

        for _ in range(0u, steps) {
            time_it!("step", 1.0f64, {
                self.simulate_plates_step();
            });
        }
    }

    fn get_vertices(&self) -> Vec<Vertex> {
        let poly = &self.poly;
        let mut vertices = Vec::with_capacity(poly.faces.len() * 3u);

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

    pub fn to_batch(&self,
                    ctx: &mut Context,
                    dev: &mut GlDevice) -> PolyhedronBatch {
        let vertices = self.get_vertices();
        let mesh = dev.create_mesh(vertices.as_slice());

        let indices = range(0u32, vertices.len() as u32).collect::<Vec<u32>>();
        let idx_slice = dev.create_buffer_static(indices.as_slice())
                           .to_slice(gfx::PrimitiveType::TriangleList);

        let shader = dev.link_program(rendering::VS_SOURCE.clone(), rendering::FS_SOURCE.clone())
                        .unwrap();
        let state = gfx::DrawState::new().depth(gfx::state::Comparison::LessEqual, true);

        ctx.make_batch(&shader, &mesh, idx_slice, &state).unwrap()
    }
}

