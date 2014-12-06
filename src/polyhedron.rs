extern crate cgmath;

use std::vec::Vec;
use std::clone::Clone;
use std::num::Float;
use std::cmp::Eq;
use std::collections::TreeMap;
use cgmath::EuclideanVector;

pub struct PolyVertex {
    pub pos: cgmath::Vector3<f32>,
    pub edge_indices: Vec<uint>,
    pub face_indices: Vec<uint>
}

impl PolyVertex {
    fn from_vec(pos: &cgmath::Vector3<f32>) -> PolyVertex {
        PolyVertex {
            pos: *pos,
            edge_indices: Vec::new(),
            face_indices: Vec::new()
        }
    }

    fn from_xyz(x: f32, y: f32, z: f32) -> PolyVertex {
        PolyVertex::from_vec(&cgmath::Vector3::new(x, y, z))
    }

    fn new() -> PolyVertex {
        PolyVertex::from_xyz(0.0, 0.0, 0.0)
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
        PolyVertex::from_xyz(0.0,  dv,  du),
        PolyVertex::from_xyz(0.0,  dv, -du),
        PolyVertex::from_xyz(0.0, -dv,  du),
        PolyVertex::from_xyz(0.0, -dv, -du),
        PolyVertex::from_xyz( du, 0.0,  dv),
        PolyVertex::from_xyz(-du, 0.0,  dv),
        PolyVertex::from_xyz( du, 0.0, -dv),
        PolyVertex::from_xyz(-du, 0.0, -dv),
        PolyVertex::from_xyz( dv,  du, 0.0),
        PolyVertex::from_xyz( dv, -du, 0.0),
        PolyVertex::from_xyz(-dv,  du, 0.0),
        PolyVertex::from_xyz(-dv, -du, 0.0)
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

fn get_or_create<T: Ord>(map: &mut TreeMap<T, uint>,
                         val: T) -> uint {
    match map.get(&val) {
        Some(&idx) => idx,
        None => {
            let new_idx = map.len();
            map.insert(val, new_idx);
            new_idx
        }
    }
}

struct VecWrapper {
    vec: cgmath::Vector3<f32>
}

impl VecWrapper {
    fn new(vec: &cgmath::Vector3<f32>) -> VecWrapper {
        VecWrapper { vec: *vec }
    }
}

fn almost_eq(a: f32, b: f32) -> bool {
    const EPSILON: f32 = 0.00001;
    (a - b).abs() < EPSILON
}

impl PartialEq for VecWrapper {
    fn eq(&self, other: &VecWrapper) -> bool {
        almost_eq(self.vec.x, other.vec.x)
            && almost_eq(self.vec.y, other.vec.y)
            && almost_eq(self.vec.z, other.vec.z)
    }
}
impl Eq for VecWrapper {}

impl PartialOrd for VecWrapper {
    fn partial_cmp(&self, other: &VecWrapper) -> Option<Ordering> {
        Some(if !almost_eq(self.vec.x, other.vec.x) {
                 if self.vec.x < other.vec.x { Less } else { Greater }
             } else if !almost_eq(self.vec.y, other.vec.y) {
                 if self.vec.y < other.vec.y { Less } else { Greater }
             } else if !almost_eq(self.vec.z, other.vec.z) {
                 if self.vec.z < other.vec.z { Less } else { Greater }
             } else {
                 Equal
             })
    }
}

impl Ord for VecWrapper {
    fn cmp(&self, other: &VecWrapper) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

fn refine(poly: &Polyhedron) -> Polyhedron {
    let mut ret = Polyhedron::new();
    let mut verts = TreeMap::new();
    let mut edges = TreeMap::new();

    for &face in poly.faces.iter() {
        let v1 = &poly.vertices[face.vertex_indices[0]].pos;
        let v2 = &poly.vertices[face.vertex_indices[1]].pos;
        let v3 = &poly.vertices[face.vertex_indices[2]].pos;
        let v12 = v1.add(v2).normalize();
        let v23 = v2.add(v3).normalize();
        let v31 = v3.add(v1).normalize();

        let v1_idx = get_or_create(&mut verts, VecWrapper::new(v1));
        let v2_idx = get_or_create(&mut verts, VecWrapper::new(v2));
        let v3_idx = get_or_create(&mut verts, VecWrapper::new(v3));
        let v12_idx = get_or_create(&mut verts, VecWrapper::new(&v12));
        let v23_idx = get_or_create(&mut verts, VecWrapper::new(&v23));
        let v31_idx = get_or_create(&mut verts, VecWrapper::new(&v31));

        let e1_12 = if v1_idx < v12_idx { (v1_idx, v12_idx) } else { (v12_idx, v1_idx) };
        let e12_2 = if v12_idx < v2_idx { (v12_idx, v2_idx) } else { (v2_idx, v12_idx) };
        let e2_23 = if v2_idx < v23_idx { (v2_idx, v23_idx) } else { (v23_idx, v2_idx) };
        let e23_3 = if v23_idx < v3_idx { (v23_idx, v3_idx) } else { (v3_idx, v23_idx) };
        let e3_31 = if v3_idx < v31_idx { (v3_idx, v31_idx) } else { (v31_idx, v3_idx) };
        let e31_1 = if v31_idx < v1_idx { (v31_idx, v1_idx) } else { (v1_idx, v31_idx) };

        let e1_12_idx = get_or_create(&mut edges, e1_12);
        let e12_2_idx = get_or_create(&mut edges, e12_2);
        let e2_23_idx = get_or_create(&mut edges, e2_23);
        let e23_3_idx = get_or_create(&mut edges, e23_3);
        let e3_31_idx = get_or_create(&mut edges, e3_31);
        let e31_1_idx = get_or_create(&mut edges, e31_1);
        let e12_31_idx = get_or_create(&mut edges, (v12_idx, v31_idx));
        let e12_23_idx = get_or_create(&mut edges, (v12_idx, v23_idx));
        let e23_31_idx = get_or_create(&mut edges, (v23_idx, v31_idx));

        ret.faces.push(Face::new(v1_idx, v12_idx, v31_idx, e1_12_idx, e12_31_idx, e31_1_idx));
        ret.faces.push(Face::new(v12_idx, v2_idx, v23_idx, e12_2_idx, e2_23_idx, e12_23_idx));
        ret.faces.push(Face::new(v23_idx, v3_idx, v31_idx, e23_3_idx, e3_31_idx, e23_31_idx));
        ret.faces.push(Face::new(v12_idx, v23_idx, v31_idx, e12_23_idx, e23_31_idx, e12_31_idx));
    }

    ret.vertices.grow(verts.len(), PolyVertex::new());
    for (vert, &idx) in verts.iter() {
        ret.vertices[idx] = PolyVertex::from_vec(&vert.vec);
    }

    ret.edges.grow(edges.len(), Edge::new(-1, -1));
    for (&(a, b), &idx) in edges.iter() {
        ret.edges[idx] = Edge::new(a, b);
    }

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

pub fn make_sphere(detail_level: uint) -> Polyhedron {
    let mut sphere = make_icosahedron();
    for _ in range(0, detail_level) {
        sphere = refine(&sphere);
    }

    sphere
}
