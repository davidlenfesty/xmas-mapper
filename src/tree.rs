//! # Tree importing and manipulation tools.
//!
//! Coordinates are a *bit* wonky.
//! - X is increasing towards the observer
//! - Y is increasing left-to-right
//! - Z is increasing upwards.
//! - X and Y are scaled -1 to 1, origin is placed "at the bottom of the trunk"
//! - Z is just same physical scale as X and Y, up to however tall the tree is.

use std::io::{Error, ErrorKind};

pub struct Pixel {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub fn import_tree(path: &str) -> std::io::Result<Vec<Pixel>> {
    let mut tree = std::fs::read_to_string(path)?;

    // Strip UTF-8 BOM from beginning of string, if there
    // (Ugh why do I have to deal with this)
    if tree.as_bytes()[0..3] == [0xEF, 0xBB, 0xBF] {
        // TODO make this not bad
        tree = String::from(&tree[3..tree.len()]);
    }
    
    let mut pixels = Vec::new();
    for line in tree.lines() {
        let mut pix = line.split(',');

        // I know a macro is the easy way but there must be a cleaner impl of this
        // TODO remove unwraps
        let x: f32 = match pix.next() {
            Some(x) => str::parse(x.trim()).unwrap(),
            None => {
                return Err(Error::new(ErrorKind::InvalidData, "No x"));
            }
        };

        let y: f32 = match pix.next() {
            Some(y) => str::parse(y).unwrap(),
            None => {
                return Err(Error::new(ErrorKind::InvalidData, "No y"));
            }
        };

        let z: f32 = match pix.next() {
            Some(z) => str::parse(z).unwrap(),
            None => {
                return Err(Error::new(ErrorKind::InvalidData, "No z"));
            }
        };

        pixels.push(Pixel {
            x, y, z
        });
    }

    Ok(pixels)
}