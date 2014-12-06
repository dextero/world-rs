extern crate cgmath;

use std::vec::Vec;
use std::clone::Clone;
use std::num::Float;
use cgmath::EuclideanVector;

pub struct PolyVertex {
    pub pos: cgmath::Vector3<f32>,
    pub edge_indices: Vec<uint>,
    pub face_indices: Vec<uint>
}

impl PolyVertex {
    fn new(x: f32, y: f32, z: f32) -> PolyVertex {
        PolyVertex {
            pos: cgmath::Vector3::new(x, y, z),
            edge_indices: Vec::new(),
            face_indices: Vec::new()
        }
    }
}

impl Clone for PolyVertex {
    fn clone(&self) -> PolyVertex {
        PolyVertex {
            pos: self.pos,
            edge_indices: self.edge_indices.clone(),
            face_indices: self.face_indices.clone()
        }
    }
}

pub struct Edge {
    pub vertex_indices: [uint, ..2],
    pub face_indices: Vec<uint>
}

impl Edge {
    fn new(a_idx: uint, b_idx: uint) -> Edge {
        Edge {
            vertex_indices: [ a_idx, b_idx ],
            face_indices: Vec::new()
        }
    }
}

impl Clone for Edge {
    fn clone(&self) -> Edge {
        Edge {
            vertex_indices: self.vertex_indices,
            face_indices: self.face_indices.clone()
        }
    }
}

pub struct Face {
    pub vertex_indices: [uint, ..3],
    pub edge_indices: [uint, ..3]
}

impl Face {
    fn new(a_vert_idx: uint, b_vert_idx: uint, c_vert_idx: uint,
           a_edge_idx: uint, b_edge_idx: uint, c_edge_idx: uint) -> Face {
        Face {
            vertex_indices: [ a_vert_idx, b_vert_idx, c_vert_idx ],
            edge_indices: [ a_edge_idx, b_edge_idx, c_edge_idx ]
        }
    }
}

impl Clone for Face {
    fn clone(&self) -> Face {
        Face {
            vertex_indices: self.vertex_indices,
            edge_indices: self.edge_indices
        }
    }
}

pub struct Polyhedron {
    pub vertices: Vec<PolyVertex>,
    pub edges: Vec<Edge>,
    pub faces: Vec<Face>
}

impl Polyhedron {
    fn new() -> Polyhedron {
        Polyhedron {
            vertices: Vec::new(),
            edges: Vec::new(),
            faces: Vec::new()
        }
    }
}

fn make_icosahedron() -> Polyhedron {
    let phi = (1.0 + 5.0f32.sqrt()) / 2.0;
    let du = 1.0 / (phi * phi + 1.0).sqrt();
    let dv = phi * du;

    let mut ret = Polyhedron::new();

    ret.vertices.push_all(&[
        PolyVertex::new(0.0,  dv,  du),
        PolyVertex::new(0.0,  dv, -du),
        PolyVertex::new(0.0, -dv,  du),
        PolyVertex::new(0.0, -dv, -du),
        PolyVertex::new( du, 0.0,  dv),
        PolyVertex::new(-du, 0.0,  dv),
        PolyVertex::new( du, 0.0, -dv),
        PolyVertex::new(-du, 0.0, -dv),
        PolyVertex::new( dv,  du, 0.0),
        PolyVertex::new( dv, -du, 0.0),
        PolyVertex::new(-dv,  du, 0.0),
        PolyVertex::new(-dv, -du, 0.0)
    ]);
    ret.edges.push_all(&[
        Edge::new( 0,  1), Edge::new( 0,  4), Edge::new( 0,  5), Edge::new( 0,  8), Edge::new( 0, 10),
        Edge::new( 1,  6), Edge::new( 1,  7), Edge::new( 1,  8), Edge::new( 1, 10), Edge::new( 2,  3),
        Edge::new( 2,  4), Edge::new( 2,  5), Edge::new( 2,  9), Edge::new( 2, 11), Edge::new( 3,  6),
        Edge::new( 3,  7), Edge::new( 3,  9), Edge::new( 3, 11), Edge::new( 4,  5), Edge::new( 4,  8),
        Edge::new( 4,  9), Edge::new( 5, 10), Edge::new( 5, 11), Edge::new( 6,  7), Edge::new( 6,  8),
        Edge::new( 6,  9), Edge::new( 7, 10), Edge::new( 7, 11), Edge::new( 8,  9), Edge::new(10, 11)
    ]);
    ret.faces.push_all(&[
        Face::new(0,  1,  8,  0,  7,  3),
        Face::new(0,  4,  5,  1, 18,  2),
        Face::new(0,  5, 10,  2, 21,  4),
        Face::new(0,  8,  4,  3, 19,  1),
        Face::new(0, 10,  1,  4,  8,  0),
        Face::new(1,  6,  8,  5, 24,  7),
        Face::new(1,  7,  6,  6, 23,  5),
        Face::new(1, 10,  7,  8, 26,  6),
        Face::new(2,  3, 11,  9, 17, 13),
        Face::new(2,  4,  9, 10, 20, 12),
        Face::new(2,  5,  4, 11, 18, 10),
        Face::new(2,  9,  3, 12, 16,  9),
        Face::new(2, 11,  5, 13, 22, 11),
        Face::new(3,  6,  7, 14, 23, 15),
        Face::new(3,  7, 11, 15, 27, 17),
        Face::new(3,  9,  6, 16, 25, 14),
        Face::new(4,  8,  9, 19, 28, 20),
        Face::new(5, 11, 10, 22, 29, 21),
        Face::new(6,  9,  8, 25, 28, 24),
        Face::new(7, 10, 11, 26, 29, 27)
    ]);

    for i in range(0, ret.edges.len()) {
        for &vert_idx in ret.edges[i].vertex_indices.iter() {
            ret.vertices[vert_idx].edge_indices.push(i);
        }
    }

    for i in range(0, ret.faces.len()) {
        for &vert_idx in ret.faces[i].vertex_indices.iter() {
            ret.vertices[vert_idx].face_indices.push(i);
        }
        for &edge_idx in ret.faces[i].edge_indices.iter() {
            ret.edges[edge_idx].face_indices.push(i);
        }
    }

    ret
}

pub fn make_sphere() -> Polyhedron {
    make_icosahedron()
}
