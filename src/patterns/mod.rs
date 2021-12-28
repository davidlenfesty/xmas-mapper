use crate::tree::Pixel;
use std::collections::HashMap;

use macroquad::color::Color;

pub mod balls;
pub mod green;
pub mod rainbow;

pub trait Pattern {
    fn from_tree(tree: &Vec<Pixel>, args: &HashMap<String, String>) -> Self;

    fn next_frame(&mut self) -> Option<Vec<Color>>;
}
