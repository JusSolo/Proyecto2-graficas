//cube.rs
use crate::material::Material;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::texture::Texture;
use nalgebra_glm::Vec3;

pub struct Cube {
    pub min: Vec3,
    pub max: Vec3,
    pub material: Material,
    pub textures: [Option<Texture>; 6], // [ -X, +X, -Y, +Y, -Z, +Z ]
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_dir: &Vec3) -> Intersect {
        // Método de slabs (intersección AABB)
        let mut tmin = (self.min.x - ray_origin.x) / ray_dir.x;
        let mut tmax = (self.max.x - ray_origin.x) / ray_dir.x;
        if tmin > tmax {
            std::mem::swap(&mut tmin, &mut tmax);
        }

        let mut tymin = (self.min.y - ray_origin.y) / ray_dir.y;
        let mut tymax = (self.max.y - ray_origin.y) / ray_dir.y;
        if tymin > tymax {
            std::mem::swap(&mut tymin, &mut tymax);
        }

        if (tmin > tymax) || (tymin > tmax) {
            return Intersect::empty();
        }
        if tymin > tmin {
            tmin = tymin;
        }
        if tymax < tmax {
            tmax = tymax;
        }

        let mut tzmin = (self.min.z - ray_origin.z) / ray_dir.z;
        let mut tzmax = (self.max.z - ray_origin.z) / ray_dir.z;
        if tzmin > tzmax {
            std::mem::swap(&mut tzmin, &mut tzmax);
        }

        if (tmin > tzmax) || (tzmin > tmax) {
            return Intersect::empty();
        }
        if tzmin > tmin {
            tmin = tzmin;
        }
        //if tzmax < tmax {
        //    tmax = tzmax;
        //}

        if tmin < 0.0 {
            return Intersect::empty();
        }

        // Punto de impacto
        let point = ray_origin + ray_dir * tmin;

        // Determinar la cara golpeada
        let mut normal = Vec3::new(0.0, 0.0, 0.0);
        let mut face_index = 0;

        if (point.x - self.min.x).abs() < 1e-3 {
            normal = Vec3::new(-1.0, 0.0, 0.0);
            face_index = 0;
        } else if (point.x - self.max.x).abs() < 1e-3 {
            normal = Vec3::new(1.0, 0.0, 0.0);
            face_index = 1;
        } else if (point.y - self.min.y).abs() < 1e-3 {
            normal = Vec3::new(0.0, -1.0, 0.0);
            face_index = 2;
        } else if (point.y - self.max.y).abs() < 1e-3 {
            normal = Vec3::new(0.0, 1.0, 0.0);
            face_index = 3;
        } else if (point.z - self.min.z).abs() < 1e-3 {
            normal = Vec3::new(0.0, 0.0, -1.0);
            face_index = 4;
        } else if (point.z - self.max.z).abs() < 1e-3 {
            normal = Vec3::new(0.0, 0.0, 1.0);
            face_index = 5;
        }

        let mut hit = Intersect::new(point, normal, tmin, self.material);

        // Mapear coordenadas UV según la cara
        if let Some(tex) = &self.textures[face_index] {
            let (u, v) = match face_index {
                0 => (
                    // -X → usa (z,y)
                    (point.z - self.min.z) / (self.max.z - self.min.z),
                    (point.y - self.min.y) / (self.max.y - self.min.y),
                ),
                1 => (
                    // +X → usa (z,y)
                    (point.z - self.min.z) / (self.max.z - self.min.z),
                    (point.y - self.min.y) / (self.max.y - self.min.y),
                ),
                2 => (
                    // -Y → usa (x,z)
                    (point.x - self.min.x) / (self.max.x - self.min.x),
                    (point.z - self.min.z) / (self.max.z - self.min.z),
                ),
                3 => (
                    // +Y → usa (x,z)
                    (point.x - self.min.x) / (self.max.x - self.min.x),
                    (point.z - self.min.z) / (self.max.z - self.min.z),
                ),
                4 => (
                    // -Z → usa (x,y)
                    (point.x - self.min.x) / (self.max.x - self.min.x),
                    (point.y - self.min.y) / (self.max.y - self.min.y),
                ),
                5 => (
                    // +Z → usa (x,y)
                    (point.x - self.min.x) / (self.max.x - self.min.x),
                    (point.y - self.min.y) / (self.max.y - self.min.y),
                ),
                _ => (0.0, 0.0),
            };

            hit.material.diffuse = tex.get_color(u, v);
        }

        hit
    }
}
