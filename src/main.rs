use image::{RgbImage, Rgb};

type Vec3 = glam::Vec3A;

#[derive(Debug)]
struct Ray {
    position : Vec3,
    direction : Vec3,
}

impl Ray {
    fn new(position: Vec3, direction : Vec3) -> Ray {
        Ray { position, direction }
    }
}

trait Sdf {
    fn distance(&self, point: Vec3) -> f32;
}

struct Sphere {
    center: Vec3,
    radius: f32,
}

impl Sdf for Sphere {
    fn distance(&self, point: Vec3) -> f32 {
        (point - self.center).length() - self.radius
    }
}

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

fn main() {
    let mut img = RgbImage::new(WIDTH, HEIGHT);

    let sphere = Sphere {
        center: Vec3::new(0.0, 0.0, 3.0),
        radius: 1.0,
    };
    for u in 0 .. WIDTH {
        for v in 0 .. HEIGHT {
            let x = (u as f32) / (WIDTH as f32);
            let y = (v as f32) / (HEIGHT as f32);
            let ray = Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(x, y, 1.0));


            let mut t = 0.0;
            let mut p = ray.position;
            for _ in 0 .. 100 {
                let d = sphere.distance(p);
                p = p + ray.direction * d;
                t += d;
                if d < 0.001 {
                    img.put_pixel(u, v, Rgb([255, 0, 0]));
                    println!("x: {}, y: {}", x, y);
                    break;
                }
            }

        }
    }
    img.save("output.png").unwrap();

}
