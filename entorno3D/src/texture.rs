// texture.rs
use crate::color::Color;
use image::{DynamicImage, GenericImageView};

pub struct Texture {
    pub image: DynamicImage,
}

impl Texture {
    pub fn from_file(path: &str) -> Self {
        let img = image::open(path).expect("No se pudo cargar la textura");
        Texture { image: img }
    }

    pub fn get_color(&self, u: f32, v: f32) -> Color {
        let (w, h) = self.image.dimensions();

        // Nos aseguramos que u,v estén entre 0 y 1
        let u = u.fract();
        let v = v.fract();

        // Mapear a coordenadas de la imagen
        let x = (u * w as f32).clamp(0.0, (w - 1) as f32) as u32;
        let y = ((1.0 - v) * h as f32).clamp(0.0, (h - 1) as f32) as u32;

        let pixel = self.image.get_pixel(x, y);
        Color::new(pixel[0], pixel[1], pixel[2])
    }
}
