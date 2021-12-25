use structopt::StructOpt;
use macroquad::prelude::*;
use std::time::Instant;

mod tree;

#[derive(Debug, StructOpt)]
#[structopt(name = "Christmas Tree Mapper", about = "A thing that maps christmas trees")]
struct Opt {
    tree: String,
}

#[macroquad::main("Merry Chrysler")]
async fn main() {
    let opts = Opt::from_args();

    // TODO better error handling
    let tree = tree::import_tree(opts.tree.as_str()).expect("Could not import tree!");

    let mut prev_frame_time = Instant::now();
    let mut theta: f32 = 0.;

    loop {
        // Set up basic scene
        clear_background(GRAY);

        // Set up camera
        // TODO rotate camera
        let frame_time = Instant::now();
        let delta = frame_time - prev_frame_time;
        prev_frame_time = frame_time; // update time

        // Update camera angle
        theta += (delta.as_millis() as f32) / 1000. * (std::f32::consts::PI / 2.);
        set_camera(&Camera3D {
            position: vec3(theta.sin() * 3., theta.cos() * 3., 1.),
            target: vec3(0., 0., 1.),
            up: vec3(0., 0., 1.),
            ..Default::default()
        });

        for pixel in &tree {
            draw_sphere(vec3(pixel.x, pixel.y, pixel.z), 0.01, None, GREEN);
        }
        //draw_sphere(vec3(0., 0., 1.), 0.25, None, GREEN);

        //set_default_camera();
        next_frame().await;

    }
}
