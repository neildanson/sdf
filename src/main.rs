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

struct Cube {
    center: Vec3,
    size: f32,
}

impl Sdf for Cube {
    fn distance(&self, point: Vec3) -> f32 {
        let q = (point - self.center).abs() - Vec3::splat(self.size);
        q.max(Vec3::ZERO).length() + q.max(Vec3::ZERO).min(Vec3::ZERO).length()
    }
}

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const MAX_DEPTH : f32 = 20.0f32;

fn main() {
    let mut img = RgbImage::new(WIDTH, HEIGHT);

    let sphere = Sphere {
        center: Vec3::new(1.0, 0.0, 3.0),
        radius: 1.0,
    };

    let cube = Cube {
        center: Vec3::new(-1.0, 0.0, 3.0),
        size: 0.5,
    };

    let shapes : Vec<&dyn Sdf> = vec![&sphere, &cube];
    let aspect_ratio = WIDTH as f32 / HEIGHT as f32;
    let start = std::time::Instant::now();
    for u in 0 .. WIDTH {
        for v in 0 .. HEIGHT {
            let x = (u as f32) / (WIDTH as f32) * 2.0 - 1.0;
            let y = (v as f32) / (HEIGHT as f32) * 2.0 - 1.0;
            let x = x * aspect_ratio;
            let ray = Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(x, y, 1.0).normalize());


            let mut p = ray.position;
            loop {
                let mut min_distance = f32::MAX;
                for shape in &shapes {
                    let d = shape.distance(p);
                    if d < min_distance {
                        min_distance = d;
                    }
                }
                p = p + ray.direction * min_distance;
                if min_distance > MAX_DEPTH {
                    break;
                }
                if min_distance < 0.001 {
                    img.put_pixel(u, v, Rgb([255, 0, 0]));
                    break;
                }
            }

        }
    }
    let elapsed = start.elapsed();
    println!("Elapsed: {:?}", elapsed);
    img.save("output.png").unwrap();

}
