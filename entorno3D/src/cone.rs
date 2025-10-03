use crate::material::Material;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::texture::Texture;
use nalgebra_glm::{Vec3, dot};

pub struct ConeSection {
    pub apex: Vec3,  // vértice superior
    pub height: f32, // altura del cono
    pub r1: f32,     // radio inferior
    pub r2: f32,     // radio superior
    pub material: Material,
    pub texture: Option<Texture>,
}

impl RayIntersect for ConeSection {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_dir: &Vec3) -> Intersect {
        // El cono truncado es la interpolación entre r1 y r2
        // Fórmula: r(y) = r1 + (r2 - r1) * ( (y - y0) / h )
        let y0 = self.apex.y;
        let y1 = y0 - self.height;

        // Paramétrico para un rayo: O + tD
        // Ecuación implícita lateral (sin tapas):
        // (x^2 + z^2) - (r(y))^2 = 0

        // Para simplificar, movemos coordenadas al eje del cono
        let ox = ray_origin.x - self.apex.x;
        let oy = ray_origin.y - self.apex.y;
        let oz = ray_origin.z - self.apex.z;

        let dx = ray_dir.x;
        let dy = ray_dir.y;
        let dz = ray_dir.z;

        // Relación de radios
        let k = (self.r2 - self.r1) / self.height;

        // Expresión cuadrática en t
        // (ox + t dx)^2 + (oz + t dz)^2 = (r1 + k*(oy + t dy + h))^2
        // Expandir: A t^2 + B t + C = 0
        let mut A = dx * dx + dz * dz - (k * dy) * (k * dy);
        let mut B = 2.0 * (ox * dx + oz * dz - (k * dy) * (self.r1 + k * oy));
        let mut C = ox * ox + oz * oz - (self.r1 + k * oy) * (self.r1 + k * oy);

        let disc = B * B - 4.0 * A * C;
        if disc < 0.0 {
            return Intersect::empty();
        }

        let sqrt_disc = disc.sqrt();
        let mut t = (-B - sqrt_disc) / (2.0 * A);
        if t < 0.0 {
            t = (-B + sqrt_disc) / (2.0 * A);
            if t < 0.0 {
                return Intersect::empty();
            }
        }

        let y_hit = oy + t * dy;
        if y_hit > 0.0 || y_hit < -self.height {
            return Intersect::empty();
        }

        let point = ray_origin + ray_dir * t;

        // Normal aproximada
        let r_hit = self.r1 + k * (-y_hit);
        let normal = Vec3::new(point.x - self.apex.x, r_hit * k, point.z - self.apex.z).normalize();

        let mut hit = Intersect::new(point, normal, t, self.material);

        // texturizado cilíndrico (u,v)
        if let Some(tex) = &self.texture {
            let u = (point.x.atan2(point.z) / std::f32::consts::PI + 1.0) * 0.5;
            let v = (-y_hit / self.height).clamp(0.0, 1.0);
            hit.material.diffuse = tex.get_color(u, v);
        }

        hit
    }
}
