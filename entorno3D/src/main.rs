use minifb::{Key, Window, WindowOptions};
use nalgebra_glm::{normalize, Vec3};
use std::f32::consts::PI;
use std::time::Duration;

mod camera;
mod color;
mod framebuffer;
mod light;
mod material;
mod ray_intersect;
mod sphere;

use camera::Camera;
use color::Color;
use framebuffer::Framebuffer;
use light::Light;
use material::Material;
use ray_intersect::{Intersect, RayIntersect};
use sphere::Sphere;

const SHADOW_BIAS: f32 = 1e-4;

fn reflect(incident: &Vec3, normal: &Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(normal) * normal
}

fn refract(incident: &Vec3, normal: &Vec3, eta: f32) -> Option<Vec3> {
    let n_dot_i = normal.dot(incident);
    let k = 1.0 - eta * eta * (1.0 - n_dot_i * n_dot_i);

    if k < 0.0 {
        None // Reflexión interna total
    } else {
        Some(eta * incident - (eta * n_dot_i + k.sqrt()) * normal)
    }
}

fn cast_shadow(intersect: &Intersect, light: &Light, objects: &[Sphere]) -> f32 {
    let light_dir = (light.position - intersect.point).normalize();
    let light_distance = (light.position - intersect.point).magnitude();

    let offset_normal = intersect.normal * SHADOW_BIAS;
    let shadow_ray_origin = if light_dir.dot(&intersect.normal) < 0.0 {
        intersect.point - offset_normal
    } else {
        intersect.point + offset_normal
    };

    let mut shadow_intensity = 0.0;

    for object in objects {
        let shadow_intersect = object.ray_intersect(&shadow_ray_origin, &light_dir);
        if shadow_intersect.is_intersecting && shadow_intersect.distance < light_distance {
            let distance_ratio = shadow_intersect.distance / light_distance;
            shadow_intensity = 1.0 - distance_ratio.powf(2.0).min(1.0);
            break;
        }
    }

    shadow_intensity
}

pub fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Sphere],
    light: &Light,
    depth: u32, // Añade profundidad para recursión
) -> Color {
    const MAX_DEPTH: u32 = 5;

    if depth > MAX_DEPTH {
        return Color::new(4, 12, 36); // Color del cielo
    }

    let mut intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;

    for object in objects {
        let i = object.ray_intersect(ray_origin, ray_direction);
        if i.is_intersecting && i.distance < zbuffer {
            zbuffer = i.distance;
            intersect = i;
        }
    }

    if !intersect.is_intersecting {
        return Color::new(4, 12, 36);
    }

    // Cálculos de iluminación difusa y especular...
    let light_dir = (light.position - intersect.point).normalize();
    let view_dir = (ray_origin - intersect.point).normalize();
    let reflect_dir = reflect(&-light_dir, &intersect.normal);

    let shadow_intensity = cast_shadow(&intersect, light, objects);
    let light_intensity = light.intensity * (1.0 - shadow_intensity);

    let diffuse_intensity = intersect.normal.dot(&light_dir).max(0.0).min(1.0);
    let diffuse = intersect.material.diffuse
        * intersect.material.albedo[0]
        * diffuse_intensity
        * light_intensity;

    let specular_intensity = view_dir
        .dot(&reflect_dir)
        .max(0.0)
        .powf(intersect.material.specular);
    let specular =
        light.color * intersect.material.albedo[1] * specular_intensity * light_intensity;

    // Componente de reflexión
    let mut reflection_color = Color::new(0, 0, 0);
    if intersect.material.albedo[2] > 0.0 {
        let reflected_dir = reflect(ray_direction, &intersect.normal);
        let reflection_origin = intersect.point + intersect.normal * SHADOW_BIAS;
        reflection_color = cast_ray(
            &reflection_origin,
            &reflected_dir,
            objects,
            light,
            depth + 1,
        ) * intersect.material.albedo[2];
    }

    // Componente de refracción
    let mut refraction_color = Color::new(0, 0, 0);
    if intersect.material.albedo[3] > 0.0 {
        let eta = if ray_direction.dot(&intersect.normal) < 0.0 {
            1.0 / intersect.material.albedo[3] // De aire a material
        } else {
            intersect.material.albedo[3] // De material a aire
        };

        if let Some(refracted_dir) = refract(ray_direction, &intersect.normal, eta) {
            let refraction_origin = intersect.point - intersect.normal * SHADOW_BIAS;
            refraction_color = cast_ray(
                &refraction_origin,
                &refracted_dir,
                objects,
                light,
                depth + 1,
            ) * intersect.material.albedo[3];
        }
    }

    // Luz reflejada (iluminación indirecta)
    let mut reflected_light_color = Color::new(0, 0, 0);
    if intersect.material.albedo[2] > 0.0 {
        // Solo calcular si el material tiene reflexión
        let reflected_dir = reflect(ray_direction, &intersect.normal);
        let reflection_origin = intersect.point + intersect.normal * SHADOW_BIAS;

        // Muestrear en varias direcciones alrededor de la reflexión perfecta para suavizar
        for i in 0..3 {
            let jitter = Vec3::new(
                rand::random::<f32>() - 0.5,
                rand::random::<f32>() - 0.5,
                rand::random::<f32>() - 0.5,
            ) * 0.1;

            let jittered_dir = (reflected_dir + jitter).normalize();

            reflected_light_color = reflected_light_color
                + cast_ray(&reflection_origin, &jittered_dir, objects, light, depth + 1);
        }

        reflected_light_color = reflected_light_color * (1.0 / 3.0) * intersect.material.albedo[2];
    }

    diffuse + specular + reflection_color + refraction_color + reflected_light_color
}

pub fn render(framebuffer: &mut Framebuffer, objects: &[Sphere], camera: &Camera, light: &Light) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = PI / 3.0;
    let perspective_scale = (fov * 0.33).tan();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            // Map the pixel coordinate to screen space [-1, 1]
            let screen_x = (2.0 * x as f32) / width - 1.0;
            let screen_y = -(2.0 * y as f32) / height + 1.0;

            // Adjust for aspect ratio and perspective
            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;

            // Calculate the direction of the ray for this pixel
            let ray_direction = normalize(&Vec3::new(screen_x, screen_y, -1.0));

            // Apply camera rotation to the ray direction
            let rotated_direction = camera.basis_change(&ray_direction);

            // Cast the ray and get the pixel color
            let pixel_color = cast_ray(&camera.eye, &rotated_direction, objects, light, 0);

            // Draw the pixel on screen with the returned color
            framebuffer.set_current_color(pixel_color.to_hex());
            framebuffer.point(x, y);
        }
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Rust Graphics - Raytracer Example",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    // move the window around
    window.set_position(500, 500);
    window.update();

    // Material con refracción (vidrio)
    let glass = Material::new(
        Color::new(180, 180, 225),
        125.0,
        [0.1, 0.1, 0.1, 0.8], // Bajo difuso/especular, alta refracción
    );

    let ivory = Material::new(
        Color::new(10, 230, 105),
        50.0,
        [0.6, 0.3, 0.1, 0.0], // Sin refracción
    );

    // Esfera con reflexión completa
    let mirror = Material::new(
        Color::new(105, 210, 210),
        400.0,
        [0.1, 0.1, 1.0, 0.3], // Reflexión completa (albedo[2] = 1.0)
    );

    let objects = [
        Sphere {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: 1.0,
            material: glass,
        },
        Sphere {
            center: Vec3::new(0.0, 0.0, 1.5),
            radius: 0.35,
            material: ivory,
        },
        Sphere {
            center: Vec3::new(-2.0, 1.0, -3.0),
            radius: 1.0,
            material: mirror, // Esfera con reflexión completa
        },
    ];

    // Initialize camera
    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 5.0), // eye: Initial camera position
        Vec3::new(0.0, 0.0, 0.0), // center: Point the camera is looking at (origin)
        Vec3::new(0.0, 1.0, 0.0), // up: World up vector
    );
    let rotation_speed = PI / 50.0;

    let light = Light::new(Vec3::new(0.0, 3.0, 5.0), Color::new(255, 223, 250), 2.0);

    while window.is_open() {
        // listen to inputs
        if window.is_key_down(Key::Escape) {
            break;
        }

        //  camera orbit controls
        if window.is_key_down(Key::Left) {
            camera.orbit(rotation_speed, 0.);
        }
        if window.is_key_down(Key::Right) {
            camera.orbit(-rotation_speed, 0.);
        }
        if window.is_key_down(Key::Up) {
            camera.orbit(0., -rotation_speed);
        }
        if window.is_key_down(Key::Down) {
            camera.orbit(0., rotation_speed);
        }

        // draw some points
        render(&mut framebuffer, &objects, &camera, &light);

        // update the window with the framebuffer contents
        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}
