use macroquad::prelude::*;
use std::collections::HashMap;
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

        /// RPM to spin model at in view.
        #[structopt(short, long, default_value = "5")]
        rpm: u32,

        /// FPS to display pattern at inside of view.
        #[structopt(short, long, default_value = "30")]
        fps: u32,
    },
    Export {
        /// File to write output to
        output: String,

        #[structopt(flatten)]
        common: CommonFlags,

        /// Maximum number of frames of pattern to export.
        #[structopt(long = "max-frames", default_value = "1000")]
        max_frames: usize,
    },
}

#[derive(Debug, StructOpt)]
struct CommonFlags {
    /// Tree to use as model.
    #[structopt(short, long, default_value = "data/mattparker_2021.csv")]
    tree: String,

    /// Extra arguments to pass into pattern. Semicolon-separated key=value pairs.
    #[structopt(long = "pattern-args")]
    pattern_args: Option<String>,
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

fn parse_extra_args(args: Option<String>) -> HashMap<String, String> {
    if let Some(args) = args {
        let mut map = HashMap::new();
        for arg in args.split(';') {
            let mut kv = arg.split('=');
            let key = String::from(kv.next().unwrap());
            let val = String::from(kv.next().unwrap());
            map.insert(key, val);
        }

        map
    } else {
        HashMap::new()
    }
}

async fn render_loop<T: Pattern>(
    tree: Vec<tree::Pixel>,
    mut pattern: T,
    rpm: u32,
    fps: u32,
    args: HashMap<String, String>,
) {
    // Pre-calculate rotational velocity of scene
    let rot_vel: f32 = std::f32::consts::PI * 2. * (rpm as f32 / 60.);
    // Too lazy to do fixed-point math
    let frame_time_ms: u32 = (1. / fps as f32 * 1000.) as u32;

    // Prep rotation
    let mut prev_frame_time = Instant::now();
    let mut theta: f32 = 0.;

    let mut current_frame = pattern.next_frame().unwrap();

    loop {
        // Set up basic scene
        clear_background(DARKGRAY);

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
                    pattern = T::from_tree(&tree, &args);
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

#[macroquad::main("Merry Chrysler")]
async fn main() -> std::io::Result<()> {
    let opts = Opt::from_args();

    let flags = match opts.command {
        Command::View {
            ref common,
            rpm: _,
            fps: _,
        } => common,
        Command::Export {
            output: _,
            ref common,
            max_frames: _,
        } => common,
    };

    let tree = tree::import_tree(flags.tree.as_str())?;
    let extra_args = parse_extra_args(flags.pattern_args.clone());

    // Prep pattern
    // TODO make this dynamic CLI (unfortunately requires dyn stuff I'm not comfortable with yet)
    let mut pattern = patterns::balls::BallPattern::from_tree(&tree, &extra_args);

    match opts.command {
        Command::Export {
            output,
            common: _,
            max_frames,
        } => {
            export::export_pattern(&tree, &mut pattern, max_frames, output.as_str())?;
            return Ok(());
        }
        Command::View {
            common: _,
            rpm,
            fps,
        } => {
            render_loop(tree, pattern, rpm, fps, extra_args).await;
            //patterns::balls::run_ball_loop(pattern, rpm, fps).await;
            Ok(())
        }
        _ => Ok(()),
    }
}
