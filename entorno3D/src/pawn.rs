/**
 * Tronco de cono (frustum) como primitiva
 */
use nalgebra_glm::Vec3;

use crate::{
    material::Material,
    ray_intersect::{Intersect, RayIntersect},
};

/// Tronco de cono (frustum) como primitiva
pub struct Frustum {
    pub base: Vec3,         // posición de la base inferior (centro)
    pub radius_bottom: f32, // radio inferior
    pub radius_top: f32,    // radio superior
    pub height: f32,        // altura
    pub material: Material,
}

impl RayIntersect for Frustum {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_dir: &Vec3) -> Intersect {
        // 🚧 Aquí pondrías la intersección real con frustum,
        // para empezar puedes usar una aproximación con un cilindro
        // (más fácil y suficiente para debug).
        // Luego afinamos la geometría real.
        let axis = Vec3::new(0.0, 1.0, 0.0);
        let center = self.base + axis * (self.height * 0.5);
        let radius = self.radius_bottom.max(self.radius_top);

        // === fallback: bounding sphere ===
        let l = center - ray_origin;
        let tca = l.dot(ray_dir);
        if tca < 0.0 {
            return Intersect::empty();
        }
        let d2 = l.dot(&l) - tca * tca;
        if d2 > radius * radius {
            return Intersect::empty();
        }
        let thc = (radius * radius - d2).sqrt();
        let t0 = tca - thc;
        let t1 = tca + thc;

        let distance = if t0 < 0.0 { t1 } else { t0 };
        if distance < 0.0 {
            return Intersect::empty();
        }

        let point = ray_origin + ray_dir * distance;
        let normal = (point - center).normalize();

        Intersect {
            is_intersecting: true,
            distance,
            point,
            normal,
            material: self.material,
        }
    }
}

/// El peón: 2 frustums apilados
pub struct Pawn {
    pub base: Vec3,
    pub scale: f32,
    pub material: Material,
}

impl RayIntersect for Pawn {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_dir: &Vec3) -> Intersect {
        // dimensiones relativas
        let h1 = 1.0 * self.scale;
        let h2 = 1.2 * self.scale;

        let bottom = Frustum {
            base: self.base,
            radius_bottom: 0.6 * self.scale,
            radius_top: 0.3 * self.scale,
            height: h1,
            material: self.material,
        };

        let top = Frustum {
            base: self.base + Vec3::new(0.0, h1, 0.0),
            radius_bottom: 0.3 * self.scale,
            radius_top: 0.5 * self.scale,
            height: h2,
            material: self.material,
        };

        // chequear intersección con ambos
        let i1 = bottom.ray_intersect(ray_origin, ray_dir);
        let i2 = top.ray_intersect(ray_origin, ray_dir);

        if i1.is_intersecting && (!i2.is_intersecting || i1.distance < i2.distance) {
            i1
        } else {
            i2
        }
    }
}
