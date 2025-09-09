use crate::color::Color;

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub diffuse: Color,
    pub specular: f32,
    pub albedo: [f32; 4], // 0: difuso, 1: especular, 2: reflexión, 3: refracción
}

impl Material {
    pub fn new(diffuse: Color, specular: f32, albedo: [f32; 4]) -> Self {
        Material {
            diffuse,
            specular,
            albedo,
        }
    }

    pub fn black() -> Self {
        Material {
            diffuse: Color::new(0, 0, 0),
            specular: 0.0,
            albedo: [0.0, 0.0, 0.0, 0.0],
        }
    }
}
