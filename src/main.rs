
type Vec3 = glam::Vec3A;

struct Ray {
    position : Vec3,
}

impl Ray {
    fn new(position: Vec3) -> Ray {
        Ray { position }
    }
}


fn main() {
    let ray = Ray::new(Vec3::new(0.0, 0.0, 0.0));
    println!("Ray position: {:?}", ray.position);
}
