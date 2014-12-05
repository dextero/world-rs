use std::vec::Vec;
use std::clone::Clone;

pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

pub fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3 { x: x, y: y, z: z }
}

pub struct Vertex {
    pub pos: Vec3,
    pub edge_indices: Vec<uint>,
    pub face_indices: Vec<uint>
}

pub fn vertex(x: f32, y: f32, z: f32) -> Vertex {
    Vertex {
        pos: vec3(x, y, z),
        edge_indices: Vec::new(),
        face_indices: Vec::new()
    }
}

impl Clone for Vertex {
    fn clone(&self) -> Vertex {
        Vertex {
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

pub fn edge(a_idx: uint, b_idx: uint) -> Edge {
    Edge {
        vertex_indices: [ a_idx, b_idx ],
        face_indices: Vec::new()
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

pub fn face(a_vert_idx: uint, b_vert_idx: uint, c_vert_idx: uint,
            a_edge_idx: uint, b_edge_idx: uint, c_edge_idx: uint) -> Face {
    Face {
        vertex_indices: [ a_vert_idx, b_vert_idx, c_vert_idx ],
        edge_indices: [ a_edge_idx, b_edge_idx, c_edge_idx ]
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

