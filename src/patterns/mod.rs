use crate::tree::Pixel;

use macroquad::color::Color;

pub mod green;
pub mod rainbow;

pub trait Pattern {
    fn from_tree(tree: &Vec<Pixel>) -> Self;

    fn next_frame(&mut self) -> Option<Vec<Color>>;
}
