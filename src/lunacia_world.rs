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

use legion::prelude::*;

use super::*;
use crate::ecs::components::{*};

use crate::ecs::submap::{TileMap};
use crate::ecs::types::{TileMapResource, GameConfigResource, QuadrantDataHashMapResource, LunaciaWorldEvent, EmitEventResource};
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
    inputs: Vec<PingWorld>,
    outputs: Vec<WorldPong>,
    inputing: bool,
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
        match tile_map.load_map() {
            Ok(v) => {
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
            map_height: 390
        });
        resources.insert(EmitEventResource(Vec::<LunaciaWorldEvent>::new()));
        self.resources = Some(resources);

        let universe = Universe::new();
        let mut world = universe.create_world();

        //Init static building
        world.insert(
            (BuildingModel(BuildingModelType::ResourceNode as i32), Static,),
            vec![
                (LandPos(21, 21),),
            ]
        );

        // world.insert(
        //     (),
        //     vec![
        //         (LandPos(5, 5), ChimeraSpawner{ count: 1, cooldown_ms: 20001, tick_ms: 20000}),
        //     ],
        // );

        // TODO load state
        world.insert(
            (UnitModel(UnitModelType::Axie as i32),),
            vec![
                (LandPos(5, 5), GatherResourceGoal{step:0, home_pos:LandPos(5, 5), target_pos:LandPos(21, 21)},)
            ]
        );

        self.universe = Some(universe);
        self.world = Some(world);

        let update_chimera_spawners = systems::build_update_chimera_spawners();
        let update_positions = systems::build_update_moving();
        let update_follow_paths = systems::build_update_follow_paths();
        let update_chimera_state = systems::build_update_chimera_state();
        let update_new_pos = systems::build_update_new_pos();

        // update positions using a system
        let set_quadrant_data_hash_map = systems::build_set_quadrant_data_hash_map();

        let thread_local_example = Box::new(|world: &mut World, _resources: &mut Resources| {
            if let Some(p) = &mut _resources.get_mut::<EmitEventResource>() {
                let evts = &mut p.0;
                if evts.len() > 0 {
                    for evt in evts.iter() {
                        match evt {
                            LunaciaWorldEvent::EventSpawn{frame, id, model, tx, ty} => {
                                println!("EventSpawn: {:?} {:?} {:?} {:?},{:?}", frame, id, model, tx, ty);
                            },
                            LunaciaWorldEvent::EventRelocation{frame, id, tx, ty} => {
                                println!("EventRelocation: {:?} {:?} {:?},{:?}", frame, id, tx, ty);
                            },
                        }
                    }
                    evts.clear();
                }
            };
            if let Some(conf) = &mut _resources.get_mut::<GameConfigResource>() {
                conf.number_of_updates += 1;
            }
        });

        let mut schedule = Schedule::builder()
            .add_system(set_quadrant_data_hash_map)
            .add_system(update_chimera_state)
            //.add_system(update_chimeras_as_boid)
            .add_system(update_follow_paths)
            .add_system(update_positions)
            .add_system(update_chimera_spawners)
            .add_system(update_new_pos)

            .add_system(systems::build_gather_resource_goals())
            .add_system(systems::build_gather_resource_actions())
            .add_system(systems::build_release_resource_actions())

            
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
        if self.inputing {
            if msg.data.len() > 1 {
                let mut iter = msg.data.split_ascii_whitespace();
                match iter.next() {
                    Some("i") => {
                        self.inputing = false;
                        self.inputs.push(msg);
                    },
                    _ => {
                        println!("skipped {:?}", msg.data);
                    }
                }
            }
        } else if msg.data.len() <= 1 {
            self.inputing = true;
            println!("inputing on");
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
                        println!("Got {:?} inputs", self.inputs.len());
                        for input in &self.inputs {
                            println!("input: {:?}", input.data);
                        }
                        
                        self.inputs.clear();

                        let act = IOWorldActior::from_registry();
                        act.do_send(WorldPong);
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
                    //println!("dt_time {:?}", dt_time);

                    self.running_time_ms += dt_time;
                    self.accumulated_time += dt_time;

                    while self.accumulated_time >= self.fixed_time_step as u128 {
                        let mut quadrant_data = QuadrantDataHashMapResource(HashMap::new());

                        resources.insert(quadrant_data);
                        
                        schedule.execute(world, resources);

                        let hm : Option<QuadrantDataHashMapResource> = resources.remove();

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
