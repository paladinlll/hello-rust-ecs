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

use legion::prelude::*;

use super::*;
// use io_world::{IOWorldActior};

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
    fixed_time_step: u32,
    number_of_updates: u128,
    universe: Option::<Universe>,
    world: Option::<World>,
    inputs: Vec<PingWorld>,
    outputs: Vec<WorldPong>,
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


        let universe = Universe::new();
        let mut world = universe.create_world();

        self.universe = Some(universe);
        self.world = Some(world);

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
        self.inputs.push(msg);
    }
 }

impl Handler<UpdateWorld> for LunaciaWorldActor {
    type Result = ();
 
    fn handle(&mut self, _: UpdateWorld, ctx: &mut Context<Self>) {
        //println!("UpdateWorld {:?}", self.number_of_updates);

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

            self.accumulated_time -= self.fixed_time_step as u128;
            self.number_of_updates += 1;
        }
 
        let addr = ctx.address();

        ctx.run_later(Duration::from_millis(self.fixed_time_step as u64), move |act, _| {
             addr.do_send(UpdateWorld);
        });
    }
 }
