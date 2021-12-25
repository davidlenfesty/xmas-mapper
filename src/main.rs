use macroquad::prelude::*;
use std::time::Instant;
use structopt::StructOpt;

mod export;
mod patterns;
mod tree;

use patterns::Pattern;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Christmas Tree Mapper",
    about = "A thing that maps christmas trees"
)]
struct Opt {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    View {
        #[structopt(flatten)]
        common: CommonFlags,
    },
    Export {
        #[structopt(flatten)]
        common: CommonFlags,
    },
}

#[derive(Debug, StructOpt)]
struct CommonFlags {
    #[structopt(short, long, default_value = "data/mattparker_2021.csv")]
    tree: String,

    #[structopt(short, long, default_value = "5")]
    rpm: u32,

    #[structopt(short, long, default_value = "30")]
    fps: u32,
}

// TODO assure pixels/frame line up
fn render_frame(tree: &Vec<tree::Pixel>, frame: &Vec<Color>) {
    let mut i = 0;
    for pixel in tree {
        let location = vec3(pixel.x, pixel.y, pixel.z);
        let color = frame[i];
        draw_sphere(location, 0.01, None, color);
        i += 1;
    }
}

#[macroquad::main("Merry Chrysler")]
async fn main() {
    let opts = Opt::from_args();

    // Pre-calculate rotational velocity of scene
    let flags = match opts.command {
        Command::View { ref common } => common,
        Command::Export { ref common } => common,
    };

    // TODO better error handling
    let tree = tree::import_tree(flags.tree.as_str()).expect("Could not import tree!");

    let rot_vel: f32 = std::f32::consts::PI * 2. * (flags.rpm as f32 / 60.);
    // Too lazy to do fixed-point math
    let frame_time_ms: u32 = (1. / flags.fps as f32 * 1000.) as u32;

    // Prep rotation
    let mut prev_frame_time = Instant::now();
    let mut theta: f32 = 0.;

    // Prep pattern
    let mut pattern = patterns::rainbow::Rainbow::from_tree(&tree);

    match opts.command {
        Command::Export { common: _ } => {
            export::export_pattern(&tree, &mut pattern, 1000, "thing.csv").unwrap();
            return;
        }
        _ => (),
    };

    let mut current_frame = pattern.next_frame().unwrap();

    loop {
        // Set up basic scene
        clear_background(GRAY);

        let frame_time = Instant::now();
        let delta = frame_time - prev_frame_time;

        // Approximate an Nfps cap, probably better ways to do this that I can impl later
        if delta.as_millis() as u32 > frame_time_ms {
            prev_frame_time = frame_time; // update previous frame time

            // Update pattern
            // TODO improve this iteration strategy
            current_frame = match pattern.next_frame() {
                Some(frame) => frame,
                None => {
                    pattern = patterns::rainbow::Rainbow::from_tree(&tree);
                    pattern.next_frame().unwrap()
                }
            }
        }

        // Set up camera
        theta += (delta.as_millis() as f32) / 1000. * rot_vel; // Update camera angle
        set_camera(&Camera3D {
            position: vec3(theta.sin() * 4., theta.cos() * 4., 3.),
            target: vec3(0., 0., 1.5),
            up: vec3(0., 0., 1.),
            ..Default::default()
        });

        // Draw pixels
        render_frame(&tree, &current_frame);

        set_default_camera();
        next_frame().await;
    }
}
