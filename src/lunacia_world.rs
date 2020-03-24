//! Simple echo websocket server.
//! Open `http://localhost:8080/ws/index.html` in browser
//! or [python console client](https://github.com/actix/examples/blob/master/websocket/websocket-client.py)
//! could be used for testing.

use std::time::{Duration, Instant, SystemTime};

use actix::prelude::*;
use actix_files as fs;
use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use std::{io, thread};
use std::collections::HashMap;
use std::collections::VecDeque;

use legion::prelude::*;

use super::*;
use crate::ecs::components::{*};

use crate::ecs::submap::{TileMap};
use crate::ecs::types::{*};
use crate::ecs::systems;

#[derive(Message)]
#[rtype(result = "()")]
pub struct PingWorld {
    pub data: String
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct WorldPong;

#[derive(Message)]
#[rtype(result = "()")]
pub struct StartWorld;

#[derive(Message)]
#[rtype(result = "()")]
pub struct UpdateWorld;

#[derive(Default)]
pub struct LunaciaWorldActor {
    up_time: u128,
    running_time_ms: u128,
    accumulated_time: u128,
    fixed_time_step: u64,
    universe: Option::<Universe>,
    world: Option::<World>,
    schedule: Option<Schedule>,
    resources: Option<Resources>,
    inputs: Vec<PlayerInputRequest>,
    outputs: Vec<WorldPong>,
    inputing: bool
}

impl Actor for LunaciaWorldActor {
    type Context = Context<Self>;
}
impl actix::Supervised for LunaciaWorldActor {}

impl ArbiterService for LunaciaWorldActor {
   fn service_started(&mut self, ctx: &mut Context<Self>) {
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => {
                self.up_time = n.as_millis();
            },
            Err(_) => {
                panic!("SystemTime before UNIX EPOCH!")
            }
        }
        self.fixed_time_step = 200; 
        self.inputing = true;

        let mut resources = Resources::default();
        let mut tile_map = TileMap::new(390, 390);

        let mut init_axies = Vec::<(LandPos, HomeLand)>::new();
        let mut init_resource_nodes = Vec::<(LandPos,)>::new();
        match tile_map.load_map() {
            Ok(v) => {
               
                // for y in 30..100 {
                //     for x in 30..100 {
                //         if tile_map.is_land_tile(&(x, y)) {
                //             let land_pos = LandPos(x, y);
                //             if init_axies.len() == 0 {
                //                 init_axies.push((land_pos, HomeLand(land_pos)));
                //             }
                //         } else if tile_map.is_resource_tile(&(x, y)) {
                //             //println!("resource node {:?} {:?}", x, y);
                //             if init_resource_nodes.len() ==0 {
                //                 init_resource_nodes.push((LandPos(x, y),));
                                
                //         }
                //     }
                // }
                for y in 30..100 {
                    for x in 30..100 {
                        if tile_map.is_land_tile(&(x, y)) {
                            let land_pos = LandPos(x, y);
                            init_axies.push((land_pos, HomeLand(land_pos)));
                        } else if tile_map.is_resource_tile(&(x, y)) {
                            init_resource_nodes.push((LandPos(x, y),));
                        }
                    }
                }

                
                resources.insert(TileMapResource(tile_map));
            },
            Err(e) => {
                println!("error parsing header: {:?}", e);
                panic!();
            }
        }

        resources.insert(GameConfigResource{
            fixed_time_ms: self.fixed_time_step, 
            number_of_updates: 0,
            map_width: 390, 
            map_height: 390,
            tmp_focusing_pos: (0, 0)
        });
        resources.insert(EmitEventResource(Vec::<(i32, LunaciaWorldEvent)>::new()));
        resources.insert(QuadrantDataHashMapResource(HashMap::new()));
        resources.insert(PathwayHashMapResource(HashMap::new()));
        self.resources = Some(resources);

        let universe = Universe::new();
        let mut world = universe.create_world();

        //Init static building
        if init_resource_nodes.len() > 0 {
            println!("Total ressource nodes: {:?}", init_resource_nodes.len());
            world.insert(
                (Model(BuildingModelType::ResourceNode as u32), Static,),
                init_resource_nodes
            );
        }

        // world.insert(
        //     (),
        //     vec![
        //         (LandPos(5, 5), ChimeraSpawner{ count: 1, cooldown_ms: 20001, tick_ms: 20000}),
        //     ],
        // );

        // TODO load state
        if init_axies.len() > 0 {
            println!("Total axie: {:?}", init_axies.len());
            world.insert(
                (Owner(1), Model(UnitModelType::Axie as u32), AutoCollect),
                init_axies
            );
        }

        self.universe = Some(universe);
        self.world = Some(world);

        let update_chimera_spawners = systems::build_update_chimera_spawners();
        let update_positions = systems::build_update_moving();
        let update_follow_paths = systems::build_update_follow_paths();
        let update_new_pos = systems::build_update_new_pos();

        // update positions using a system
        let set_quadrant_data_hash_map = systems::build_set_quadrant_data_hash_map();

        let thread_local_example = Box::new(|world: &mut World, _resources: &mut Resources| {
            let mut tmp_focusing_pos = LandPos(0, 0);
            if let Some(conf) = &_resources.get::<GameConfigResource>() {
                tmp_focusing_pos = LandPos(conf.tmp_focusing_pos.0, conf.tmp_focusing_pos.1);
            }
            // if let Some(p) = &mut _resources.get_mut::<PlayerInputResource>() {
            //     let ins = &mut p.0;
            //     //TODO quick verify valid input?
            //     while let Some(inp) = &self.inputs.pop_front() {
            //         //ins.push_back(*inp);
            //     }
            //     // let act = IOWorldActior::from_registry();
            //     // act.do_send(WorldPong);
            // }

            

            if let Some(p) = &mut _resources.get_mut::<EmitEventResource>() {
                let evts = &mut p.0;
                if evts.len() > 0 {
                    let visible_chunk_keys = tmp_focusing_pos.get_hash_map_key_successors(1);
                    for (chunk_key, evt) in evts.iter() {
                        if visible_chunk_keys.contains(&chunk_key){
                            match evt {
                                LunaciaWorldEvent::EventSpawn{frame, id, model, tx, ty} => {
                                    println!("EventSpawn: {:?} {:?} {:?} {:?},{:?}", frame, id, model, tx, ty);
                                },
                                LunaciaWorldEvent::EventRelocation{frame, id, tx, ty} => {
                                    println!("EventRelocation: {:?} {:?} {:?},{:?}", frame, id, tx, ty);
                                },
                            }
                        }
                    }
                    evts.clear();
                }
            };

            if let Some(p) = &mut _resources.get_mut::<QuadrantDataHashMapResource>() {
                let hm = &mut p.0;
                {
                    let query = <(Write<PlayerInput>, Read<PlayerInputGetStateAroundLand>)>::query();
                    for (mut pi, lp) in query.iter_mut(world) {
                        match &pi.status {
                            0 => {
                                tmp_focusing_pos = LandPos(lp.0, lp.1);
                                let visible_chunk_keys = tmp_focusing_pos.get_hash_map_key_successors(1);
                                let mut total = 0;
                                for chunk_key in visible_chunk_keys.iter() {
                                    //println!("chunk_key {:?}", chunk_key);
                                    match hm.get(chunk_key) {
                                        Some(chunk) => {
                                            
                                            for objs in chunk.values() {
                                                for (entity, qd) in objs.iter() {
                                                    total += 1;
                                                    //println!("State: {:?} - {:?},{:?}", entity.index(), qd.land_pos.0, qd.land_pos.1);
                                                }
                                            }
                                        },
                                        None => {}
                                    }
                                }
                                println!("Focus at {:?},{:?} total entities: {:?}", lp.0, lp.1, total);
                                pi.status += 1;
                            },
                            _ => ()
                        }
                    }
                }

                {
                    let mut input_hm : HashMap <(u32, u32), PlayerInputAxieGatherResource> = HashMap::new();

                    let query = <(Write<PlayerInput>, Read<PlayerInputAxie>, Read<PlayerInputAxieGatherResource>)>::query();
                    for (mut pi, ax, gr) in query.iter_mut(world) {
                        match &pi.status {
                            0 => {
                                pi.status += 1;
                                input_hm.insert((pi.owner, ax.axie_index), *gr);
                            },
                            _ => ()
                        }
                    }

                    for (key, val) in input_hm.iter() {
                        let owner = Owner(key.0);
                        let axie_query = <(Read<LandPos>)>::query()
                            .filter(tag_value(&owner) & tag_value(&Model(UnitModelType::Axie as u32)));
                        let mut axie_found : Option<Entity> = None;
                        for (mut axie_entity, (axie_pos)) in axie_query.iter_entities_mut(world) {
                            if axie_entity.index() == key.1 {
                                axie_found = Some(axie_entity);
                            }
                        }
                        match axie_found {
                            Some(entity) => {
                                // if world.get_component::<GGoal>(entity) == None {
                                //     world.add_component(entity, GatherResourceGoal{
                                //         step:0, 
                                //         home_pos:LandPos(5, 5), 
                                //         target_pos:LandPos(21, 21)}
                                //     );
                                // } else{
                                //     println!("Axie busing");
                                // }
                                
                            },
                            None => {
                                println!("Invalid axie input");
                            }
                        }
                        //println!("key: {} val: {}", key, val);
                    }
                }
                
                //hm.clear();
            }
            if let Some(conf) = &mut _resources.get_mut::<GameConfigResource>() {
                conf.number_of_updates += 1;
                conf.tmp_focusing_pos = (tmp_focusing_pos.0, tmp_focusing_pos.1);
            }
        });

        let mut schedule = Schedule::builder()
            .add_system(set_quadrant_data_hash_map)
            //.add_system(update_chimeras_as_boid)
            .add_system(update_follow_paths)
            .add_system(update_positions)
            .add_system(update_chimera_spawners)
            .add_system(update_new_pos)

            .add_system(systems::build_gather_resource_goals())
            .add_system(systems::build_gather_resource_actions())
            .add_system(systems::build_release_resource_actions())

            //.add_system(systems::build_player_input_axie_gather_resource())

            .add_system(systems::build_player_input_cleans())

            .add_system(systems::build_auto_collect_resources())
            
            // This flushes all command buffers of all systems.
            .flush()
            // a thread local system or function will wait for all previous systems to finish running,
            // and then take exclusive access of the world.
            .add_thread_local_fn(thread_local_example)
            .build();

        self.schedule = Some(schedule);

        println!("LunaciaWorldActor Service started");
   }
}

impl Handler<StartWorld> for LunaciaWorldActor {
   type Result = ();

   fn handle(&mut self, _: StartWorld, ctx: &mut Context<Self>) {
        println!("StartWorld");

        let addr = ctx.address();

        ctx.run_later(Duration::from_millis(1000), move |act, _| {
            addr.do_send(UpdateWorld);
        });
   }
}

impl Handler<PingWorld> for LunaciaWorldActor {
    type Result = ();
 
    fn handle(&mut self, msg: PingWorld, ctx: &mut Context<Self>) {
        if msg.data.len() > 1 {
            let mut iter = msg.data.split_ascii_whitespace();
            match iter.next() {
                Some("i") => {
                    let mut tx = 0;
                    if let Some(v_str) = iter.next() {
                        if let Ok(v) = v_str.parse::<i32>() {
                            tx = v;
                        }
                    }
                    let mut ty = 0;
                    if let Some(v_str) = iter.next() {
                        if let Ok(v) = v_str.parse::<i32>() {
                            ty = v;
                        }
                    }

                    self.inputs.push(PlayerInputRequest::GetPlayerState {
                        request_id: 0,
                        owner: 1,
                        tx: tx,
                        ty: ty
                    });
                },
                Some("g") => {
                    let mut e_index = 0;
                    if let Some(v_str) = iter.next() {
                        if let Ok(v) = v_str.parse::<u32>() {
                            e_index = v;
                        }
                    }
                    if e_index > 0 {
                        println!("PlayerInputRequested {:?}", e_index);
                        self.inputs.push(PlayerInputRequest::GatherResource {
                            request_id: 0,
                            owner: 1,
                            axie_index: e_index
                        });
                    }

                    self.inputing = false;
                    
                },
                _ => {
                    println!("skipped {:?}", msg.data);
                }
            }
        }
    }
 }

impl Handler<UpdateWorld> for LunaciaWorldActor {
    type Result = ();

    fn handle(&mut self, _: UpdateWorld, ctx: &mut Context<Self>) {
        if let Some(resources) = &mut self.resources {
            if let Some(schedule) = &mut self.schedule {
                if let Some(world) = &mut self.world {

                    if self.inputs.len() > 0 {
                        let mut input_axies = Vec::new();
                        let mut input_get_states = Vec::new();
                        for input in &self.inputs {
                            match input {
                                PlayerInputRequest::GatherResource{request_id, owner, axie_index} => {
                                    input_axies.push((PlayerInput{request_id: *request_id, owner: *owner, status: 0}, PlayerInputAxie{axie_index: *axie_index}, PlayerInputAxieGatherResource{resource_id: 1}))
                                },
                                PlayerInputRequest::GetPlayerState{request_id, owner, tx, ty} => {
                                    input_get_states.push((PlayerInput{request_id: *request_id, owner: *owner, status: 0}, PlayerInputGetStateAroundLand(*tx, *ty)))
                                },
                                _ => {}
                            }
                        }
                        self.inputs.clear();
                        if input_axies.len() > 0 {
                            world.insert(
                                (),
                                input_axies
                            );
                        }
                        if input_get_states.len() > 0 {
                            world.insert(
                                (),
                                input_get_states
                            );
                        }
                    }
                    
                


                    let mut now_ms : u128 = 0;
                    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                        Ok(n) => {
                            now_ms = n.as_millis();
                        },
                        Err(_) => {
                            panic!("SystemTime before UNIX EPOCH!")
                        }
                    }
                    let dt_time = (now_ms - self.up_time) - self.running_time_ms;
                    println!("dt_time {:?}", dt_time);

                    self.running_time_ms += dt_time;
                    self.accumulated_time += dt_time;

                    while self.accumulated_time >= self.fixed_time_step as u128 {
                        schedule.execute(world, resources);
                        self.accumulated_time -= self.fixed_time_step as u128;
                    }
                };
            };
        };
 
        let addr = ctx.address();

        ctx.run_later(Duration::from_millis(self.fixed_time_step as u64), move |act, _| {
             addr.do_send(UpdateWorld);
        });
    }
 }
