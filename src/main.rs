#![feature(globs)]

mod polyhedron;

mod mesh {
    use std::vec::Vec;
    use std::num::Float;
    use polyhedron::*;

    struct Polyhedron {
        pub vertices: Vec<Vertex>,
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
            vertex(0.0,  dv,  du),
            vertex(0.0,  dv, -du),
            vertex(0.0, -dv,  du),
            vertex(0.0, -dv, -du),
            vertex( du, 0.0,  dv),
            vertex(-du, 0.0,  dv),
            vertex( du, 0.0, -dv),
            vertex(-du, 0.0, -dv),
            vertex( dv,  du, 0.0),
            vertex( dv, -du, 0.0),
            vertex(-dv,  du, 0.0),
            vertex(-dv, -du, 0.0)
        ]);
        ret.edges.push_all(&[
            edge( 0,  1), edge( 0,  4), edge( 0,  5), edge( 0,  8), edge( 0, 10),
            edge( 1,  6), edge( 1,  7), edge( 1,  8), edge( 1, 10), edge( 2,  3),
            edge( 2,  4), edge( 2,  5), edge( 2,  9), edge( 2, 11), edge( 3,  6),
            edge( 3,  7), edge( 3,  9), edge( 3, 11), edge( 4,  5), edge( 4,  8),
            edge( 4,  9), edge( 5, 10), edge( 5, 11), edge( 6,  7), edge( 6,  8),
            edge( 6,  9), edge( 7, 10), edge( 7, 11), edge( 8,  9), edge(10, 11)
        ]);
        ret.faces.push_all(&[
            face(0,  1,  8,  0,  7,  3),
            face(0,  4,  5,  1, 18,  2),
            face(0,  5, 10,  2, 21,  4),
            face(0,  8,  4,  3, 19,  1),
            face(0, 10,  1,  4,  8,  0),
            face(1,  6,  8,  5, 24,  7),
            face(1,  7,  6,  6, 23,  5),
            face(1, 10,  7,  8, 26,  6),
            face(2,  3, 11,  9, 17, 13),
            face(2,  4,  9, 10, 20, 12),
            face(2,  5,  4, 11, 18, 10),
            face(2,  9,  3, 12, 16,  9),
            face(2, 11,  5, 13, 22, 11),
            face(3,  6,  7, 14, 23, 15),
            face(3,  7, 11, 15, 27, 17),
            face(3,  9,  6, 16, 25, 14),
            face(4,  8,  9, 19, 28, 20),
            face(5, 11, 10, 22, 29, 21),
            face(6,  9,  8, 25, 28, 24),
            face(7, 10, 11, 26, 29, 27)
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
}

fn main() {
    let sphere = mesh::make_sphere();

    for v in sphere.vertices.iter() {
        println!("{}\t{}\t{}", v.pos.x, v.pos.y, v.pos.z);
    }
}
