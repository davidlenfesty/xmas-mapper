//! Tools for generating and exporting frame info.
//!
//! Frame data is exported as a CSV in the following form:
//! FRAME_ID, R_0, G_0, B_0, R_1, G_1, B_1, ...

use crate::patterns::Pattern;
use crate::tree::Pixel;

pub fn export_pattern<T: Pattern>(
    tree: &Vec<Pixel>,
    pattern: &mut T,
    frame_limit: usize,
    filename: &str,
) -> std::io::Result<()> {
    let mut out = String::new();

    // Push out header
    let num_pixels = tree.len();
    out.push_str("FRAME_ID");
    for i in 0..num_pixels {
        out.push_str(format!(",R_{},G_{},B_{}", i, i, i).as_str());
    }
    out.push_str("\n");

    let mut i = 0;
    while let Some(frame) = pattern.next_frame() {
        // Frame index
        out.push_str(format!("{}", i).as_str());

        // Write out RGB value per pixel
        for pixel in frame {
            out.push_str(format!(",{},{},{}", pixel.r, pixel.g, pixel.b).as_str());
        }

        out.push_str("\n");

        i += 1;
        if i >= frame_limit {
            break;
        }
    }

    std::fs::write(filename, out)?;
    Ok(())
}
