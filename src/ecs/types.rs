use super::*;
// use crate::submap::{TileMap};
use crate::ecs::submap::{*};
use std::collections::HashMap;

#[derive(Clone)]
pub struct GameConfigResource {
    pub fixed_time_ms: u64,
    pub map_width: usize,
    pub map_height: usize
}

#[derive(Clone)]
pub struct QuadrantDataHashMapResource(pub HashMap <i32, Vec<(i32, i32)>>);

#[derive(Clone)]
pub struct TileMapResource(pub TileMap);

// pub type Board = Vec<Vec<u8>>;
