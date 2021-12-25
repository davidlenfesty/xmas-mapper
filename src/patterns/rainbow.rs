use super::Pattern;
use crate::tree::Pixel;
use macroquad::color::Color;

pub struct Rainbow {
    phase: f32,
    len: usize,
}

impl Pattern for Rainbow {
    fn from_tree(tree: &Vec<Pixel>) -> Self {
        Self {
            phase: 0.,
            len: tree.len(),
        }
    }

    fn next_frame(&mut self) -> Option<Vec<Color>> {
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
