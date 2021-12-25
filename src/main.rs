use macroquad::prelude::*;
use std::time::Instant;
use structopt::StructOpt;

mod export;
mod tree;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Christmas Tree Mapper",
    about = "A thing that maps christmas trees"
)]
struct Opt {
    tree: String,

    #[structopt(short = "r", long = "rpm", default_value = "5")]
    rpm: i32,
}

#[macroquad::main("Merry Chrysler")]
async fn main() {
    let opts = Opt::from_args();

    // TODO better error handling
    let tree = tree::import_tree(opts.tree.as_str()).expect("Could not import tree!");

    // Pre-calculate rotational velocity of scene
    let rot_vel: f32 = std::f32::consts::PI * 2. * (opts.rpm as f32 / 60.);

    // Prep rotation
    let mut prev_frame_time = Instant::now();
    let mut theta: f32 = 0.;

    loop {
        // Set up basic scene
        clear_background(GRAY);

        let frame_time = Instant::now();
        let delta = frame_time - prev_frame_time;
        prev_frame_time = frame_time; // update previous frame time

        // Set up camera
        theta += (delta.as_millis() as f32) / 1000. * rot_vel; // Update camera angle
        set_camera(&Camera3D {
            position: vec3(theta.sin() * 4., theta.cos() * 4., 3.),
            target: vec3(0., 0., 1.5),
            up: vec3(0., 0., 1.),
            ..Default::default()
        });

        // Draw pixels
        for pixel in &tree {
            draw_sphere(vec3(pixel.x, pixel.y, pixel.z), 0.01, None, GREEN);
        }

        set_default_camera();
        next_frame().await;
    }
}
