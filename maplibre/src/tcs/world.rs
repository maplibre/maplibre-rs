use std::default::Default;

use crate::tcs::{resources::Resources, tiles::Tiles};

#[derive(Default)]
pub struct World {
    pub resources: Resources,
    pub tiles: Tiles,
}

impl World {}
