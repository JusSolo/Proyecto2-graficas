// plane.rs
use crate::material::Material;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::texture::Texture;
use nalgebra_glm::Vec3;

pub struct Plane {
    pub point: Vec3,
    pub normal: Vec3,
    pub material: Material,
    pub texture: Option<Texture>,
    pub scale: f32, // Tamaño de repetición de la textura
}

impl RayIntersect for Plane {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        let denom = self.normal.dot(ray_direction);
        if denom.abs() > 1e-6 {
            let t = (self.point - ray_origin).dot(&self.normal) / denom;
            if t >= 0.0 {
                let hit_point = ray_origin + ray_direction * t;

                // Coordenadas de textura (usando x y z del punto de impacto)
                let u = hit_point.x / self.scale;
                let v = hit_point.z / self.scale;

                // Si hay textura, usamos el color de ella
                let mut mat = self.material;
                if let Some(tex) = &self.texture {
                    mat.diffuse = tex.get_color(u, v);
                }

                return Intersect::new(hit_point, self.normal, t, mat);
            }
        }
        Intersect::empty()
    }
}
