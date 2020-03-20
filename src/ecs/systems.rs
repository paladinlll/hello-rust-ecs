
use super::*;
use crate::ecs::types::{*};
use crate::ecs::components::{*};

// use crate::submap::{TileMap};

// use submap::TileMap;

// // mod types;
// use types::GameConfigResource;
// use types::QuadrantDataHashMapResource;
// use types::TileMapResource;

// // mod components;
// use components::Pos;
// use components::Vel;
// use components::Moving;
// use components::Chimera;
// use components::ChimeraState;
// use components::FollowPath;
// use components::ChimeraSpawner;

use astar::astar;
//use super::prelude::{GameConfigResource, Pos, Vel, Chimera, ChimeraSpawner, ChimeraState};
use legion::prelude::*;

pub fn build_update_chimera_spawners() -> Box<dyn Schedulable>  {
    SystemBuilder::new("update_chimera_spawners")
        .read_resource::<GameConfigResource>()
        .with_query(<(Read<Pos>, Write<ChimeraSpawner>)>::query())
        .build(move |command_buffer, mut world, (res0), query| {
            let dt_ms = res0.fixed_time_ms;
            for (pos, mut spawner) in query.iter_mut(&mut world) {
                spawner.tick_ms += dt_ms as i32;
                if spawner.tick_ms >= spawner.cooldown_ms {
                    spawner.tick_ms -= spawner.cooldown_ms;
                    println!("spawn chimera {:?} - {:?}", spawner.tick_ms, dt_ms);

                    let entities: &[Entity] = command_buffer.insert(
                        ((), Chimera),
                        vec![
                            (Pos(pos.0, pos.1), Vel(1, 0), ChimeraState{state: 0})
                        ],
                    );
                }
            }
        })
}

// pub fn build_update_chimeras_as_boid() -> Box<dyn Schedulable>  {
//     SystemBuilder::new("update_chimeras")
//         .read_resource::<GameConfigResource>()
//         .read_resource::<QuadrantDataHashMapResource>()
//         .with_query(<(Read<Pos>, Write<Vel>)>::query()
//             .filter(tag::<Chimera>()))
//         .build(|_, mut world, (res0, res1), query| {
//             //res1.0 = res2.0.clone(); // Write the mutable resource from the immutable resource
//             let dt_time = res0.fixed_time_ms  as f64 * 0.001;
//             let mw = res0.map_width as f32;
//             let mh = res0.map_height as f32;
//             let hm = &res1.0;

//             for (mut pos, mut vel) in query.iter_mut(&mut world) {
//                 let mut v_pos = Vector2::new(pos.0 as f64, pos.1 as f64);
                

//                 let hash_map_key = get_position_hash_map_key(pos.0, pos.1);
//                 let mut group_size = 0;
//                 let mut v_center = Vector2::new(0.0, 0.0);
//                 let mut v_avoid = Vector2::new(0.0, 0.0);
//                 if let Some(vec) = hm.get(&hash_map_key) {
//                     for &x in vec {
//                         let dir = v_pos - x;
//                         let n_distance = dir.magnitude();
//                         if n_distance < 3.0 {
//                             v_center += x;
//                             group_size += 1;

//                             if n_distance < 1.0 {
//                                 v_avoid += dir;
//                             }
//                         }
//                     }
//                     //println!("chunk {:?} - neighbor {:?}", hash_map_key, vec.len());
//                 }

//                 if group_size > 0 {
//                     v_center = v_center / group_size as f64;
//                     v_center += v_avoid;
//                     let mut dir = v_center - v_pos;
//                     let n_distance = dir.magnitude();
//                     if n_distance > 0.0 { 
//                         dir.normalize();
//                     } else {
//                         dir = Vector2::new(vel.0 as f64, vel.1 as f64);
//                         dir.normalize();
//                     }

//                     if dir.x.abs() > dir.y.abs() {
//                         if dir.x > 0.0  {
//                             vel.0 = 1;
//                             vel.1 = 0;
//                         } else {
//                             vel.0 = -1;
//                             vel.1 = 0;
//                         }
//                     } else {
//                         if dir.y > 0.0  {
//                             vel.0 = 0;
//                             vel.1 = 1;
//                         } else {
//                             vel.0 = 0;
//                             vel.1 = -1;
//                         }
//                     }
//                 }
//             }
//         })
// }

pub fn build_update_moving() -> Box<dyn Schedulable>  {
    SystemBuilder::new("update_moving")
        .read_resource::<GameConfigResource>()
        .with_query(<(Read<Pos>, Write<Moving>)>::query()
            .filter(!component::<NewPos>()))
        .build(move |command_buffer, mut world, (res0), query| {
            //res1.0 = res2.0.clone(); // Write the mutable resource from the immutable resource
            //let dt_time = res0.fixed_time_ms  as f64 * 0.001;
            for (mut entity, (pos, mut mv)) in query.iter_entities_mut(&mut world) {
                //let mut v_pos = Vector2::new(pos.0 as f64, pos.1 as f64);
                if mv.vx == 0 && mv.vy == 0 {
                    continue;
                }
                mv.step += mv.speed * res0.fixed_time_ms / mv.cost as u64;
                if mv.step >= mv.maxstep {
                    command_buffer.add_component(entity, NewPos(pos.0 + mv.vx, pos.1 + mv.vy));
                    // pos.0 += mv.vx;
                    // pos.1 += mv.vy;
                    mv.step -= mv.maxstep;
                    mv.vx = 0;
                    mv.vy = 0;
                }
            }
        })
}

pub fn build_update_new_pos() -> Box<dyn Schedulable>  {
    SystemBuilder::new("update_moving")
        .with_query(<(Write<Pos>, Read<NewPos>)>::query())
        .build(move |command_buffer, mut world, (), query| {
            //res1.0 = res2.0.clone(); // Write the mutable resource from the immutable resource
            //let dt_time = res0.fixed_time_ms  as f64 * 0.001;
            for (mut entity, (mut pos, newpos)) in query.iter_entities_mut(&mut world) {
                pos.0 = newpos.0;
                pos.1 = newpos.1;
                command_buffer.remove_component::<NewPos>(entity);
            }
        })
}

pub fn build_update_follow_paths() -> Box<dyn Schedulable>  {
    SystemBuilder::new("update_follow_paths")
        .read_resource::<TileMapResource>()
        .with_query(<(Read<FollowPath>, Read<Pos>, Write<Moving>)>::query()
            .filter(!component::<NewPos>()))
        .build(move |command_buffer, mut world, (res0), query| {
            let tm = &res0.0;
            
            for (mut entity, (fp, pos, mut mv)) in query.iter_entities_mut(&mut world) {
            //for (fp, pos, mut mv) in query.iter_mut(&mut world) {
                if mv.vx == 0 && mv.vy == 0 {
                    if !tm.can_move_to(&(fp.tx, fp.ty)) {
                        command_buffer.remove_component::<FollowPath>(entity);
                        command_buffer.remove_component::<Moving>(entity);
                    } else if pos.distance(&Pos(fp.tx, fp.ty)) <= 1{
                        println!("{:?},{:?} -> {:?},{:?} Reach target", pos.0, pos.1, fp.tx, fp.ty);
                        command_buffer.remove_component::<FollowPath>(entity);
                        command_buffer.remove_component::<Moving>(entity);
                    } else {
                        let goal: (i32, i32) = (fp.tx, fp.ty);
                        let result = astar::astar(&(pos.0, pos.1),
                            |&p| tm.successors(&p),
                            |&(x, y)| (x - goal.0).abs() as u32 + (y - goal.1).abs() as u32,
                            |&p| p == goal);
                        match result {
                            Some((paths, cost)) => {
                                println!("{:?},{:?} -> {:?},{:?} Path found length: {:?}. cost: {:?}", pos.0, pos.1, goal.0, goal.1, paths.len(), cost);

                                mv.vx = paths[1].0 - paths[0].0;
                                mv.vy = paths[1].1 - paths[0].1;
                                mv.cost = tm.get_move_cost(&paths[1]);
                                mv.maxstep = 1000;
                            }
                            None => {
                                println!("No path found");
                                command_buffer.remove_component::<FollowPath>(entity);
                                command_buffer.remove_component::<Moving>(entity);
                            }
                        }

                        
                    }
                } else {

                }
                //let mut v_pos = Vector2::new(pos.0 as f64, pos.1 as f64);

                // let dx = fp.tx - pos.0 as i32;
                // let dy = fp.ty - pos.1 as i32;
                // if dx == 0 && dy == 0 {
                //     vel.0 = 0;
                //     vel.1 = 0;
                // } else if dx.abs() > dy.abs() {
                //     if dx > 0  {
                //         vel.0 = 1;
                //         vel.1 = 0;
                //     } else {
                //         vel.0 = -1;
                //         vel.1 = 0;
                //     }
                // } else {
                //     if dy > 0  {
                //         vel.0 = 0;
                //         vel.1 = 1;
                //     } else {
                //         vel.0 = 0;
                //         vel.1 = -1;
                //     }
                // }
            }
        })
}

pub fn build_update_chimera_state() -> Box<dyn Schedulable>  {
    SystemBuilder::new("update_chimera_state")
        .read_resource::<TileMapResource>()
        .with_query(<(Read<Pos>, Write<ChimeraState>)>::query()
            .filter(tag::<Chimera>() & !component::<Moving>()))
        .build(move |command_buffer, mut world, (res0), query| {
            
            for (mut entity, (pos, mut cs)) in query.iter_entities_mut(&mut world) {
                //let mut v_pos = Vector2::new(pos.0 as f64, pos.1 as f64);
                match cs.state {
                    0 => {
                        command_buffer.add_component(entity, FollowPath {tx: 4, ty: 3});
                        command_buffer.add_component(entity, Moving {vx: 0, vy: 0, speed: 2, cost: 1, step: 0, maxstep: 0});
                        cs.state = 1;
                    },
                    _ => {
                        // if vel.0 == 0 && vel.1 == 0 {
                        //     command_buffer.delete(entity);
                        // }
                    }
                }
            }
        })
}

pub fn build_set_quadrant_data_hash_map() -> Box<dyn Schedulable>  {
    SystemBuilder::new("set_quadrant_data_hash_map")
        .write_resource::<QuadrantDataHashMapResource>()
        .with_query(<(Read<Pos>)>::query())
        .build(|_, mut world, (conf), query| {
            let hm = &mut conf.0;
        
            for (pos) in query.iter_mut(&mut world) {
                //let v_pos = Vector2::new(pos.0 as f64, pos.1 as f64);
                let hash_map_key = pos.get_hash_map_key();
                hm.entry(hash_map_key)
                    // If there's no entry for key 3, create a new Vec and return a mutable ref to it
                    .or_insert_with(Vec::new)
                    // and insert the item onto the Vec
                    .push((pos.0, pos.1));
            }
        })
}
