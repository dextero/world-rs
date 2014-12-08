extern crate cgmath;

use cgmath::{Point, Vector, Point3, Vector3, EuclideanVector};
use polyhedron::Polyhedron;

#[deriving(Show)]
pub struct Ray {
    orig: Vector3<f32>,
    dir: Vector3<f32>
}

#[deriving(Show)]
pub struct Plane {
    normal: Vector3<f32>,
    d: f32
}

#[deriving(PartialEq, Eq)]
pub enum PlaneSide {
    Above,
    On,
    Below
}

impl Plane {
    pub fn from_points(v1: &Vector3<f32>,
                       v2: &Vector3<f32>,
                       v3: &Vector3<f32>) -> Plane {
        let e21 = v1.sub(v2);
        let e32 = v2.sub(v3);
        let e13 = v3.sub(v1);

        Plane {
            normal: Vector3::new(v1.y * e32.z + v2.y * e13.z + v3.y * e21.z,
                                 v1.z * e32.x + v2.z * e13.x + v3.z * e21.x,
                                 v1.x * e32.y + v2.x * e13.y + v3.x * e21.y).normalize(),
            d: -(v1.x * (v2.y * v3.z - v3.y * v2.z) +
                 v2.x * (v3.y * v1.z - v1.y * v3.z) +
                 v3.x * (v1.y * v2.z - v2.y * v1.z))
        }
    }

    pub fn intersection_point(&self, ray: &Ray) -> (Vector3<f32>, f32) {
        let t = -(ray.orig.dot(&self.normal) + self.d) / ray.dir.dot(&self.normal);
        (ray.orig.add(&ray.dir.mul_s(t)), t)
    }

    pub fn get_plane_side(&self, point: &Vector3<f32>) -> PlaneSide {
        let dist = -point.dot(&self.normal);

        if dist < 0.0f32 {
            PlaneSide::Above
        } else if dist > 0.0f32 {
            PlaneSide::Below
        } else {
            PlaneSide::On
        }
    }
}

impl Ray {
    pub fn towards_center(orig: &Point3<f32>) -> Ray {
        let v = orig.to_vec();

        Ray {
            orig: v,
            dir: v.neg().normalize()
        }
    }

    pub fn intersection_dist(&self, verts: &[&Vector3<f32>, ..3]) -> Option<f32> {
        let plane = Plane::from_points(verts[0], verts[1], verts[2]);
        let (intersection, dist) = plane.intersection_point(self);

        let planes = [Plane::from_points(&self.orig, verts[0], verts[1]),
                      Plane::from_points(&self.orig, verts[1], verts[2]),
                      Plane::from_points(&self.orig, verts[2], verts[0])];

        if planes[0].get_plane_side(&intersection) != PlaneSide::Below
                && planes[1].get_plane_side(&intersection) != PlaneSide::Below
                && planes[2].get_plane_side(&intersection) != PlaneSide::Below {
            Some(dist)
        } else {
            None
        }
    }
}

pub fn intersecting_triangle_id(poly: &Polyhedron,
                            ray: &Ray) -> Option<uint> {
    let mut nearest: Option<(uint, f32)> = None;

    for i in range(0u, poly.faces.len()) {
        let face = &poly.faces[i];
        let dist = ray.intersection_dist(&[&poly.vertices[face.vertex_indices[0]].pos,
                                           &poly.vertices[face.vertex_indices[1]].pos,
                                           &poly.vertices[face.vertex_indices[2]].pos]);
        match dist {
            Some(dist) => match nearest {
                Some((_, old_dist)) => {
                    if old_dist > dist {
                        nearest = Some((i, dist))
                    }
                },
                None => nearest = Some((i, dist))
            },
            None => {}
        }
    }

    match nearest {
        Some((nearest_idx, _)) => Some(nearest_idx),
        None => None
    }
}

