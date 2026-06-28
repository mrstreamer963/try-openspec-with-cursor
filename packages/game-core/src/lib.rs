use wasm_bindgen::prelude::*;

pub mod components;
pub mod events;
pub mod game;
pub mod pathfinding;
pub mod systems;
pub mod world;

pub use game::Game;

#[wasm_bindgen(start)]
pub fn init() {}
