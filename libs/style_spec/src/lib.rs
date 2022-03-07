pub mod layer;
pub mod source;

use crate::layer::Layer;
use crate::source::Source;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Style {
    version: u16,
    name: String,
    metadata: HashMap<String, String>,
    sources: HashMap<String, Source>,
    layers: Vec<Layer>,
}
