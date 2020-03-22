use super::*;
// use crate::submap::{TileMap};
use crate::ecs::submap::{*};
use std::collections::HashMap;

#[derive(Clone)]
pub struct GameConfigResource {
    pub number_of_updates: u32,
    pub fixed_time_ms: u64,
    pub map_width: usize,
    pub map_height: usize
}

#[derive(Clone)]
pub struct QuadrantDataHashMapResource(pub HashMap <i32, Vec<(i32, i32)>>);

#[derive(Clone)]
pub struct TileMapResource(pub TileMap);

// pub type Board = Vec<Vec<u8>>;
// #[derive(Clone)]
// pub struct EventSpawn {
//     //pub state: i32 //0: none, 1: request path, 2: moving, 3:finished,
//     pub frame: u32,
//     pub id: u32,
//     pub model: usize,
//     pub tx: i32,
//     pub ty: i32,
// }

#[derive(Clone)]
pub struct EmitEventResource(pub Vec<LunaciaWorldEvent>);

#[derive(Clone)]
pub enum LunaciaWorldEvent {
    EventSpawn {
        //pub state: i32 //0: none, 1: request path, 2: moving, 3:finished,
        frame: u32,
        id: u32,
        model: usize,
        tx: i32,
        ty: i32,
    },
    EventRelocation {
        frame: u32,
        id: u32,
        tx: i32,
        ty: i32,
    }
}
