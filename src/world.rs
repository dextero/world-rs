extern crate cgmath;
extern crate gfx;

use std::vec::Vec;
use std::num::{Float, FloatMath};

use cgmath::{EuclideanVector, Vector, Vector3, FixedArray};
use gfx::batch::Context;
use gfx::{GlDevice, Device, DeviceHelper, ToSlice};

use polyhedron::{Polyhedron};
use rendering;
use rendering::{PolyhedronBatch, Vertex, color_by_height};
use plate_simulation::PlateSimulation;

pub struct World {
    poly: Polyhedron
}

fn get_min_max_length<Iter: Iterator<Vector3<f32>>>(iter: &mut Iter) -> (f32, f32) {
    let mut min_len_sq = iter.next().unwrap().length2();
    let mut max_len_sq = min_len_sq;

    loop {
        match iter.next() {
            Some(v) => {
                min_len_sq = min_len_sq.min(v.length2());
                max_len_sq = max_len_sq.max(v.length2());
            },
            None => break
        }
    }

    (min_len_sq.sqrt(), max_len_sq.sqrt())
}

impl World {
    pub fn new(poly: Polyhedron) -> World {
        World { poly: poly }
    }

    pub fn get_poly(&self) -> &Polyhedron {
        &self.poly
    }

    fn get_vertices(&self) -> Vec<Vertex> {
        let poly = &self.poly;
        let (min_h, max_h) = get_min_max_length(&mut self.poly.vertices.iter().map(|v| v.pos));
        let mut vertices = Vec::with_capacity(poly.faces.len() * 3u);

        for face_idx in range(0u, poly.faces.len()) {
            let face = &poly.faces[face_idx];
            let verts = [&poly.vertices[face.vertex_indices[0]].pos,
                         &poly.vertices[face.vertex_indices[1]].pos,
                         &poly.vertices[face.vertex_indices[2]].pos];

            let mean_pos = verts[0].add(verts[1]).add(verts[2]).div_s(3.0);
            let face_col = color_by_height(mean_pos.length(), min_h, max_h);

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

    pub fn apply_heights(&mut self,
                         plate_sim: &PlateSimulation) {
        const DOT_THRESHOLD: f32 = 0.1;

        let mut deltas = Vec::with_capacity(self.poly.vertices.len());
        let mut min_delta = deltas.len() as f32;
        let mut max_delta = 0.0f32;

        for v in self.poly.vertices.iter() {
            let mut delta = 0.0f32;
            let mut nbr_count = 0u;

            for v2 in plate_sim.verts.iter() {
                let dot = v.pos.dot(&v2.pos);
                if dot > DOT_THRESHOLD {
                    delta += dot;
                    nbr_count += 1;
                }
            }

            delta /= nbr_count as f32;
            deltas.push(delta);

            min_delta = min_delta.min(delta);
            max_delta = max_delta.max(delta);
        }

        let half = (min_delta + max_delta) / 2.0;
        let diff = max_delta - min_delta;
        let factor = 2.0 / diff;
        let scale = |i| 1.0 + (deltas[i] - half) * factor;

        for i in range(0u, self.poly.vertices.len()) {
            let v = &mut self.poly.vertices[i].pos;
            *v = v.mul_s(scale(i));
        }
    }
}

