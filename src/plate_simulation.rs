extern crate cgmath;
extern crate gfx;

use std::vec::Vec;
use std::num::FloatMath;
use std::rand::Rng;

use time;
use cgmath::{EuclideanVector, Vector, Vector3, Basis3, Rotation, Rotation3, Rad, rad, FixedArray};
use gfx::batch::Context;
use gfx::{GlDevice, Device, DeviceHelper, ToSlice};

use polyhedron::{Edge, Polyhedron};
use rendering;
use rendering::{PolyhedronBatch, Vertex, color_by_index};

include!("macros.rs")

struct PlatePoint {
    pub pos: Vector3<f32>,
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

fn random_axis<R: Rng>(rng: &mut R) -> Vector3<f32> {
    Vector3::new(rng.gen_range(0.0001f32, 1.0),
                 rng.gen_range(0.0001f32, 1.0),
                 rng.gen_range(0.0001f32, 1.0)).normalize()
}

fn make_vertex(pos: &Vector3<f32>,
               color: &[f32, ..4]) -> Vertex {
    Vertex {
        pos: *pos.as_fixed(),
        color: *color,
        id: -2
    }
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

    fn from_points<R: Rng>(rng: &mut R,
                           vertex_indices: Vec<uint>) -> Plate {
        const HEIGHT_DEV: f32 = 0.02;

        Plate::new(vertex_indices,
                   &random_axis(rng),
                   rad(rng.gen_range(0.01f32, 0.1)),
                   rng.gen_range(1.0 - HEIGHT_DEV, 1.0 + HEIGHT_DEV))
    }

    fn simulate(&self, vertices: &mut Vec<PlatePoint>) {
        for &idx in self.vertex_indices.iter() {
            vertices[idx].move_around(&self.move_axis);
        }
    }
}

fn get_nbr_idx(edge: &Edge, vert_idx: uint) -> uint {
    if edge.vertex_indices[0] == vert_idx {
        edge.vertex_indices[1]
    } else {
        edge.vertex_indices[0]
    }
}

fn assign_neighbors(plate_points: &mut Vec<Vec<uint>>,
                    new_frontier: &mut Vec<uint>,
                    plate_id_for_verts: &mut Vec<int>,
                    plate_idx: uint,
                    nbr_indices: &Vec<uint>) -> uint {
    let mut num_assigned = 0u;

    for &nbr_idx in nbr_indices.iter() {
        if plate_id_for_verts[nbr_idx] == -1 {
            plate_id_for_verts[nbr_idx] = plate_idx as int;
            plate_points[plate_idx].push(nbr_idx);
            new_frontier.push(nbr_idx);
            num_assigned += 1;
        }
    }

    num_assigned
}

fn flood_fill(verts: &Vec<PlatePoint>,
              plate_id_for_verts: &mut Vec<int>,
              plate_points: &mut Vec<Vec<uint>>) {
    let mut filled_points = plate_points.len();
    let mut frontier_points = plate_points.clone();

    while filled_points < verts.len() {
        //println!("{} points to go", verts.len() - filled_points);

        for plate_idx in range(0u, plate_points.len()) {
            let mut new_frontier = Vec::new();

            for &point_idx in frontier_points[plate_idx].iter() {
                filled_points += assign_neighbors(plate_points, &mut new_frontier,
                                                  plate_id_for_verts, plate_idx,
                                                  &verts[point_idx].nbr_indices);
            }

            frontier_points[plate_idx] = new_frontier;
        }
    }
}

fn random_partition<R: Rng>(rng: &mut R,
                            verts: &Vec<PlatePoint>,
                            num_plates: uint) -> Vec<Plate> {
    let mut plate_id_for_verts = Vec::from_elem(verts.len(), -1i);
    let mut plate_points = Vec::with_capacity(num_plates);

    for plate_idx in range(0u, num_plates) {
        //println!("finding origin of plate {}/{} ({} verts total)", plate_idx, num_plates, verts.len());
        loop {
            let idx = rng.gen_range(0u, verts.len());

            if plate_id_for_verts[idx] == -1 {
                plate_id_for_verts[idx] = plate_idx as int;

                let plate_idx = plate_points.len();
                plate_points.push(Vec::new());
                plate_points[plate_idx].push(idx);

                break
            }
        }
    }

    time_it!("flood fill", 5.0f64, {
        flood_fill(verts, &mut plate_id_for_verts, &mut plate_points);
    });

    plate_points.iter().map(|points| Plate::from_points(rng, points.clone())).collect()
}

pub struct PlateSimulation {
    initial_distance: f32,
    pub verts: Vec<PlatePoint>,
    plates: Vec<Plate>
}

fn get_edge_length(poly: &Polyhedron) -> f32 {
    let edge = &poly.edges[0];
    let verts = [&poly.vertices[edge.vertex_indices[0]],
                 &poly.vertices[edge.vertex_indices[1]]];
    let diff = verts[0].pos.sub(&verts[1].pos);
    diff.length()
}

impl PlateSimulation {
    pub fn new<R: Rng>(poly: &Polyhedron,
                       num_plates: uint,
                       rng: &mut R) -> PlateSimulation {
        if poly.faces.len() < num_plates {
            panic_bt!("cannot split {} faces into {} plates", poly.faces.len(), num_plates);
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

        let plates = random_partition(rng, &verts, num_plates);
        for plate in plates.iter() {
            for &vert_idx in plate.vertex_indices.iter() {
                verts[vert_idx].speed = plate.move_speed;
            }
        }

        PlateSimulation {
            initial_distance: get_edge_length(poly),
            verts: verts,
            plates: plates
        }
    }

    fn simulate_plates_step(&mut self) {
        const DOT_THRESHOLD: f32 = 0.5;
        let initial_distance: f32 = self.initial_distance;

        for plate in self.plates.iter() {
            plate.simulate(&mut self.verts);
        }

        let mut avg_distances = Vec::with_capacity(self.verts.len());
        let mut min_dist = avg_distances.len() as f32;
        let mut max_dist = 0.0f32;

        for i in range(0u, self.verts.len()) {
            let v = &self.verts[i];
            let mut sum = 0.0f32;

            for v2 in self.verts.iter() {
                sum += v.pos.dot(&v2.pos).max(DOT_THRESHOLD);
            }

            sum -= self.verts.len() as f32 * DOT_THRESHOLD;
            let avg_dist = sum / self.verts.len() as f32;
            avg_distances.push(avg_dist);

            min_dist = min_dist.min(avg_dist);
            max_dist = max_dist.max(avg_dist);
        }

        let speed_scale = |i| 1.0 - (avg_distances[i] / initial_distance);

        for i in range(0u, self.verts.len()) {
            self.verts[i].speed.s *= speed_scale(i);
        }
    }

    pub fn simulate_plates(&mut self, steps: uint) {
        //println!("simulating {} tectonic plate steps", steps);

        for _ in range(0u, steps) {
            time_it!("step", 1.0f64, {
                self.simulate_plates_step();
            });
        }
    }

    fn get_vertices(&self) -> Vec<Vertex> {
        let mut vertices = Vec::with_capacity(self.verts.len() * 2 + 1);
        vertices.push(make_vertex(&Vector3::new(0.0, 0.0, 0.0), &[0.0, 0.0, 0.0, 1.0]));

        for plate_idx in range(0u, self.plates.len()) {
            let plate = &self.plates[plate_idx];
            let plate_color = color_by_index(plate_idx, self.plates.len());

            for &vert_idx in plate.vertex_indices.iter() {
                let v = &self.verts[vert_idx];
                let rot: Basis3<f32> = Rotation3::from_axis_angle(&plate.move_axis, v.speed);

                vertices.push(make_vertex(&v.pos, &plate_color));
                vertices.push(make_vertex(&rot.rotate_vector(&v.pos), &plate_color));
            }
        }

        vertices
    }

    pub fn to_batch(&self,
                    ctx: &mut Context,
                    dev: &mut GlDevice) -> PolyhedronBatch {
        let vertices = self.get_vertices();
        let mesh = dev.create_mesh(vertices.as_slice());

        let mut indices: Vec<u32> = Vec::with_capacity(self.verts.len() * 4);
        for i in range(0u32, self.verts.len() as u32) {
            indices.push(0);
            indices.push(i * 2 + 1);
            indices.push(i * 2 + 1);
            indices.push(i * 2 + 2);
        }
        let idx_slice = dev.create_buffer_static(indices.as_slice())
                           .to_slice(gfx::PrimitiveType::Line);

        let shader = dev.link_program(rendering::VS_SOURCE.clone(), rendering::FS_SOURCE.clone())
                        .unwrap();
        let state = gfx::DrawState::new().depth(gfx::state::Comparison::LessEqual, true);

        ctx.make_batch(&shader, &mesh, idx_slice, &state).unwrap()
    }
}

