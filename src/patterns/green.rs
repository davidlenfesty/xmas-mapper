use super::Pattern;
use crate::tree::Pixel;
use macroquad::color::Color;
use std::collections::HashMap;

pub struct Green {
    storage: Vec<Color>,
}

impl Pattern for Green {
    fn from_tree(tree: &Vec<Pixel>, _args: &HashMap<String, String>) -> Self {
        // TODO do the iter way
        let mut storage = Vec::new();
        for _ in tree {
            storage.push(Color::from_rgba(0, 255, 0, 255));
        }

        Self { storage }
    }

    fn next_frame(&mut self) -> Option<Vec<Color>> {
        Some(self.storage.clone())
    }
}
