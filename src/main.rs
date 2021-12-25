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
    rpm: u32,

    #[structopt(short = "f", long = "fps", default_value = "30")]
    fps: u32
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

trait Pattern {
    fn from_tree(tree: &Vec<tree::Pixel>) -> Self;
}

struct Green {
    storage: Vec<Color>
}

impl Pattern for Green {
    fn from_tree(tree: &Vec<tree::Pixel>) -> Self {
        // TODO do the iter way
        let mut storage = Vec::new();
        for _ in tree {
            storage.push(Color::from_rgba(0, 255, 0, 255));
        }

        Self {
            storage,
        }
    }
}

impl Iterator for Green {
    type Item = Vec<Color>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.storage.clone())
    }
}

struct Rainbow {
    phase: f32,
    len: usize,
}

impl Pattern for Rainbow {
    fn from_tree(tree: &Vec<tree::Pixel>) -> Self {
        Self {
            phase: 0.,
            len: tree.len(),
        }
    }
}

impl Iterator for Rainbow {
    type Item = Vec<Color>;

    fn next(&mut self) -> Option<Self::Item> {
        // What do I want here? 2 rainbows per frame
        let color_wavelen: f32 = 256. * 3.; // phase should go up to this value
        let index_to_phase: f32 = color_wavelen / (self.len as f32);
        // 2 here is the number of rainbows per tree
        let index_to_phase = index_to_phase * 3.;

        // TODO figure out this number based on frequency
        self.phase += 4.;
        if self.phase > color_wavelen {
            self.phase = 0.;
        }

        let mut frame = Vec::new();
        for i in 0..self.len {
            // TODO figure out how to make n rainbows
            let phase = ((i as f32) * index_to_phase + self.phase) as usize % (256 * 3 - 1);

            // TODO hold on this is alll wrong
            let color = if phase < 255 {
                let phase = phase as u8;
                Color::from_rgba(255 - phase, phase, 0, 255)
            } else if phase < (256 * 2 - 1) {
                let phase = (phase - 255) as u8;
                Color::from_rgba(0, 255 - phase, phase, 255)
            } else if phase < (256 * 3 - 1) {
                let phase = (phase - 511) as u8;
                Color::from_rgba(phase, 0, 255 - phase, 255)
            } else {
                // TODO figure out how to remove this artifact
                Color::from_rgba(255, 0, 0, 255)
            };

            frame.push(color);
        }

        Some(frame)
    }
}

#[macroquad::main("Merry Chrysler")]
async fn main() {
    let opts = Opt::from_args();

    // TODO better error handling
    let tree = tree::import_tree(opts.tree.as_str()).expect("Could not import tree!");

    // Pre-calculate rotational velocity of scene
    let rot_vel: f32 = std::f32::consts::PI * 2. * (opts.rpm as f32 / 60.);
    // Too lazy to do fixed-point math
    let frame_time_ms: u32 = (1. / opts.fps as f32 * 1000.) as u32;

    // Prep rotation
    let mut prev_frame_time = Instant::now();
    let mut theta: f32 = 0.;

    // Prep pattern
    let mut pattern = Rainbow::from_tree(&tree);
    let mut current_frame = pattern.next().unwrap();

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
            current_frame = match pattern.next() {
                Some(frame) => frame,
                None => {
                    pattern = Rainbow::from_tree(&tree);
                    pattern.next().unwrap()
                },
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
