
use super::*;
use crate::ecs::types::{*};
use crate::ecs::components::{*};
use astar::astar;
use legion::prelude::*;
use std::collections::HashMap;
use rand::Rng;

pub fn build_update_chimera_spawners() -> Box<dyn Schedulable>  {
    SystemBuilder::new("update_chimera_spawners")
        .read_resource::<GameConfigResource>()
        .write_resource::<EmitEventResource>()
        .with_query(<(Read<LandPos>, Write<ChimeraSpawner>)>::query())
        .build(move |command_buffer, mut world, (res0, res1), query| {
            let conf = &res0;

            let emit_event = &mut res1.0;
            for (pos, mut spawner) in query.iter_mut(&mut world) {
                spawner.tick_ms += conf.fixed_time_ms as i32;
                if spawner.tick_ms >= spawner.cooldown_ms {
                    spawner.tick_ms -= spawner.cooldown_ms;
                    //println!("spawn chimera {:?} - {:?}", spawner.tick_ms, dt_ms);

                    let entities: &[Entity] = command_buffer.insert(
                        ((Model(UnitModelType::Chimera as u32)), Chimera),
                        vec![
                            (LandPos(pos.0, pos.1), Vel(0, 0), ChimeraState{state: 0})
                        ],
                    );

                    emit_event.push((pos.get_hash_map_key(), LunaciaWorldEvent::EventSpawn{
                        frame: conf.number_of_updates,
                        id: entities[0].index(),
                        model: 1,
                        tx: pos.0,
                        ty: pos.1,
                    }));
                }
            }
        })
}

pub fn build_update_moving() -> Box<dyn Schedulable>  {
    SystemBuilder::new("update_moving")
        .read_resource::<GameConfigResource>()
        .with_query(<(Read<LandPos>, Write<Moving>)>::query()
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
                    // if entity.index() == 575 {
                    //     println!("{:?}  Reach next {:?} {:?} {:?} - vel {:?} {:?}", res0.number_of_updates, entity.index(), pos.0, pos.1, mv.vx, mv.vy);
                    // }
                    mv.step -= mv.maxstep;
                    mv.vx = 0;
                    mv.vy = 0;
                }
            }
        })
}

pub fn build_update_new_pos() -> Box<dyn Schedulable>  {
    SystemBuilder::new("update_moving")
        .read_resource::<GameConfigResource>()
        .write_resource::<EmitEventResource>()
        .with_query(<(Write<LandPos>, Read<NewPos>)>::query())
        .build(move |command_buffer, mut world, (res0, res1), query| {
            let conf = &res0;
            let emit_event = &mut res1.0;
            for (mut entity, (mut pos, newpos)) in query.iter_entities_mut(&mut world) {
                // if entity.index() == 575 {
                //     println!("{:?} NewPos {:?} {:?},{:?} -> {:?},{:?}", conf.number_of_updates, entity.index(), pos.0, pos.1, newpos.0, newpos.1);
                // }
                pos.0 = newpos.0;
                pos.1 = newpos.1;
                command_buffer.remove_component::<NewPos>(entity);

                emit_event.push((pos.get_hash_map_key(), LunaciaWorldEvent::EventRelocation{
                    frame: conf.number_of_updates,
                    id: entity.index(),
                    tx: pos.0,
                    ty: pos.1,
                }));
            }
        })
}

pub fn build_update_follow_paths() -> Box<dyn Schedulable>  {
    SystemBuilder::new("update_follow_paths")
        .read_resource::<TileMapResource>()
        .write_resource::<PathwayHashMapResource>()
        .with_query(<(Read<FollowPath>, Read<LandPos>, Write<Moving>)>::query()
            .filter(!component::<NewPos>()))
        .build(move |command_buffer, mut world, (res0, res1), query| {
            let tm = &res0.0;
            let pw = &mut res1.0;
            
            for (mut entity, (fp, pos, mut mv)) in query.iter_entities_mut(&mut world) {
            //for (fp, pos, mut mv) in query.iter_mut(&mut world) {
                if mv.vx == 0 && mv.vy == 0 {
                    if !tm.can_move_to(&(fp.tx, fp.ty)) {
                        //println!("can_t_move_to {:?} {:?}", fp.tx, fp.ty);
                        command_buffer.remove_component::<FollowPath>(entity);
                        command_buffer.remove_component::<Moving>(entity);
                    } else if pos.distance(&LandPos(fp.tx, fp.ty)) <= 1{
                        // if entity.index() == 575 {
                        //     println!("{:?} {:?},{:?} -> {:?},{:?} Reach target", entity.index(), pos.0, pos.1, fp.tx, fp.ty);
                        // }
                        command_buffer.remove_component::<FollowPath>(entity);
                        command_buffer.remove_component::<Moving>(entity);
                    } else {
                        let goal: (i32, i32) = (fp.tx, fp.ty);
                        let pathway_key = ((fp.sx, fp.sy), (fp.tx, fp.ty));

                        

                        match pw.get(&pathway_key) {
                            Some(paths) => {
                                let mut current_index = 0;
                                while current_index < paths.len() {
                                    if paths[current_index] == (pos.0, pos.1) {
                                        break;
                                    }
                                    current_index += 1;
                                }

                                if current_index < paths.len() - 1 {
                                    mv.vx = paths[current_index+1].0 - paths[current_index].0;
                                    mv.vy = paths[current_index+1].1 - paths[current_index].1;

                                    // if entity.index() == 575 {
                                    //     println!("{:?} check... {:?} {:?} - vel {:?} {:?}", entity.index(), pos.0, pos.1, mv.vx, mv.vy);
                                    // }
                                    mv.cost = tm.get_move_cost(&paths[current_index+1]);
                                    mv.maxstep = 1000;
                                } else {
                                    println!("Invalid pathway cache {:?} {:?},{:?} -> {:?},{:?} : {:?},{:?}", entity.index(), fp.sx, fp.sy, fp.tx, fp.ty, pos.0, pos.1);
                                    // for path in paths.iter() {
                                    //     println!("----{:?},{:?}", path.0, path.1);
                                    // }
                                    command_buffer.remove_component::<FollowPath>(entity);
                                    command_buffer.remove_component::<Moving>(entity);
                                }
                            },
                            None => {
                                let result = astar::astar(&(pos.0, pos.1),
                                    |&p| tm.successors(&p),
                                    |&(x, y)| (x - goal.0).abs() as u32 + (y - goal.1).abs() as u32,
                                    |&p| p == goal);
                                match result {
                                    Some((paths, cost)) => {
                                        //println!("{:?},{:?} -> {:?},{:?} Path found length: {:?}. cost: {:?}", pos.0, pos.1, goal.0, goal.1, paths.len(), cost);

                                        mv.vx = paths[1].0 - paths[0].0;
                                        mv.vy = paths[1].1 - paths[0].1;
                                        mv.cost = tm.get_move_cost(&paths[1]);
                                        mv.maxstep = 1000;

                                        pw.insert(pathway_key, paths);
                                    }
                                    None => {
                                        println!("No path found {:?},{:?} -> {:?},{:?}", pos.0, pos.1, fp.tx, fp.ty);
                                        command_buffer.remove_component::<FollowPath>(entity);
                                        command_buffer.remove_component::<Moving>(entity);
                                    }
                                }
                            }
                        }
                    }
                } else {

                }
            }
        })
}

pub fn build_set_quadrant_data_hash_map() -> Box<dyn Schedulable>  {
    SystemBuilder::new("set_quadrant_data_hash_map")
        .write_resource::<QuadrantDataHashMapResource>()
        .with_query(<(Read<LandPos>, Tagged<Model>)>::query()
            .filter(!component::<QuadrantKey>()))
        .build(move |command_buffer, mut world, (conf), query| {
            let hm = &mut conf.0;
        
            for (mut entity, (pos, model)) in query.iter_entities_mut(&mut world) {
                //let v_pos = Vector2::new(pos.0 as f64, pos.1 as f64);
                let hash_map_key = pos.get_hash_map_key();
                command_buffer.add_component(entity, QuadrantKey(hash_map_key));
                hm.entry(hash_map_key)
                    .or_insert_with(HashMap::<u32, Vec<(Entity, QuadrantData)>>::new)
                    .entry(model.0)
                    .or_insert_with(Vec::<(Entity, QuadrantData)>::new)
                    .push((entity, QuadrantData{model: model.0, land_pos: (pos.0, pos.1)}));
            }
        })
}

pub fn build_gather_resource_goals() -> Box<dyn Schedulable>  {
    SystemBuilder::new("gather_resource_goals")
        .read_resource::<TileMapResource>()
        .with_query(<(Write<GatherResourceGoal>, Read<LandPos>)>::query()
            .filter(!component::<GAction>() & !component::<GAction>()))
        .build(move |command_buffer, mut world, (res0), query| {
             
            for (mut entity, (mut goal, pos)) in query.iter_entities_mut(&mut world) {
                match goal.step {
                    0 => {
                        // if entity.index() == 575 {
                        //     println!("{:?} Will GActionGatherResource", entity.index());
                        // }
                        goal.step += 1;
                        command_buffer.add_component(entity, FollowPath {sx: pos.0, sy: pos.1, tx: goal.target_pos.0, ty: goal.target_pos.1});
                        command_buffer.add_component(entity, Moving::new());
                        command_buffer.add_component(entity, GAction::new_gather_resource_action());
                        command_buffer.add_tag(entity, GActionGatherResource);
                    },
                    1 => {
                        // if entity.index() == 575 {
                        //     println!("{:?} Will GActionReleaseResource", entity.index());
                        // }
                        goal.step += 1;
                        command_buffer.add_component(entity, FollowPath {sx: pos.0, sy: pos.1, tx: goal.home_pos.0, ty: goal.home_pos.1});
                        command_buffer.add_component(entity, Moving::new());
                        command_buffer.add_component(entity, GAction::new_release_resource_action());
                        command_buffer.add_tag(entity, GActionReleaseResource);
                    },
                    _ => {
                        // if entity.index() == 575 {
                        //     println!("{:?} Done GatherResourceGoal", entity.index());
                        // }
                        command_buffer.remove_component::<GatherResourceGoal>(entity);
                        command_buffer.remove_tag::<GGoal>(entity);
                    }
                }
                
            }
        })
}

pub fn build_gather_resource_actions() -> Box<dyn Schedulable>  {
    SystemBuilder::new("build_gather_resource_actions")
        .read_resource::<TileMapResource>()
        .with_query(<(Write<GAction>)>::query()
            .filter(!component::<Moving>() & tag::<GActionGatherResource>()))
        .build(move |command_buffer, mut world, (res0), query| {
             
            for (mut entity, (mut action)) in query.iter_entities_mut(&mut world) {
                command_buffer.remove_tag::<GActionGatherResource>(entity);
                command_buffer.remove_component::<GAction>(entity);
            }
        })
}

pub fn build_release_resource_actions() -> Box<dyn Schedulable>  {
    SystemBuilder::new("build_release_resource_actions")
        .read_resource::<TileMapResource>()
        .with_query(<(Write<GAction>)>::query()
            .filter(!component::<Moving>() & tag::<GActionReleaseResource>()))
        .build(move |command_buffer, mut world, (res0), query| {
             
            for (mut entity, (mut action)) in query.iter_entities_mut(&mut world) {
                command_buffer.remove_tag::<GActionReleaseResource>(entity);
                command_buffer.remove_component::<GAction>(entity);
            }
        })
}

pub fn build_player_input_cleans() -> Box<dyn Schedulable>  {
    SystemBuilder::new("build_player_input_cleans")
        .with_query(<(Read<PlayerInput>)>::query())
        .build(move |command_buffer, mut world, (res0), query| {
            for (mut entity, (pi)) in query.iter_entities_mut(&mut world) {
                match &pi.status {
                    1 => {
                        command_buffer.delete(entity);
                    },
                    _ => ()
                }
            }
        })
}

pub fn build_auto_collect_resources() -> Box<dyn Schedulable>  {
    SystemBuilder::new("build_auto_collect_resources")
        .read_resource::<TileMapResource>()
        .read_resource::<QuadrantDataHashMapResource>()
        .with_query(<(Write<HomeLand>)>::query()
            .filter(tag::<AutoCollect>() & !tag::<GGoal>()))
        .build(move |command_buffer, mut world, (res0, res1), query| {
            let hm = &res1.0;

            let mut rng = rand::thread_rng();

            for (mut entity, (mut hl)) in query.iter_entities_mut(&mut world) {
                let n: u32 = rng.gen_range(0, 100);
                if n > 10 {
                    continue;
                }
                let home_pos = hl.0;

                let mut search_range = 1;
                let search_model = BuildingModelType::ResourceNode as u32;
                let mut nearest_distance = -1;
                let mut target_pos : Option<LandPos> = None;
                while search_range < 10 {
                    let visible_chunk_keys = home_pos.get_hash_map_key_successors_at_radius(search_range);
                    for chunk_key in visible_chunk_keys.iter() {
                        match hm.get(chunk_key) {
                            Some(chunk) => {
                                match chunk.get(&search_model) {
                                    Some(objs) => {
                                        for (_, qd) in objs.iter() {
                                            let check_pos = LandPos(qd.land_pos.0, qd.land_pos.1);
                                            let dist = home_pos.distance(&check_pos);
                                            if nearest_distance == -1 || dist < nearest_distance as u32 {
                                                nearest_distance = dist as i32;
                                                target_pos = Some(check_pos);
                                            }
                                        }
                                    },
                                    None => {}
                                }
                                
                            },
                            None => {}
                        }
                    }
                    search_range += 1;
                }
                
                match target_pos {
                    Some(p) => {
                        command_buffer.add_tag(entity, GGoal);
                        command_buffer.add_component(entity, GatherResourceGoal{
                            step:0, 
                            home_pos: home_pos, 
                            target_pos: p
                        });
                    },
                    None => {
                        //println!("No resource node near. will stop AutoCollect");
                        command_buffer.remove_tag::<AutoCollect>(entity);
                    }
                }
                //let goal: (i32, i32) = (fp.tx, fp.ty);
                    
                
            }
        })
}
