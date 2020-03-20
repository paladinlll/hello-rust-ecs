use actix::prelude::*;

use super::*;
// use lunacia_world::{LunaciaWorldActor, PingWorld, WorldPong, StartWorld};

#[derive(Default)]
pub struct IOWorldActior;

impl actix::Supervised for IOWorldActior {}

impl ArbiterService for IOWorldActior {
   fn service_started(&mut self, ctx: &mut Context<Self>) {
        println!("IOWorldActior Service started");
   }
}

impl Actor for IOWorldActior {
   type Context = Context<Self>;

   fn started(&mut self, _: &mut Context<Self>) {
      // get LunaciaWorldActor address from the registry
      let act = LunaciaWorldActor::from_registry();
      act.do_send(StartWorld);
   }
}

impl Handler<PingWorld> for IOWorldActior {
    type Result = ();
 
    fn handle(&mut self, msg: PingWorld, ctx: &mut Context<Self>) {
         println!("Request PingWorld");
         let act = LunaciaWorldActor::from_registry();
        act.do_send(PingWorld{data: msg.data});
    }
}

impl Handler<WorldPong> for IOWorldActior {
    type Result = ();

    fn handle(&mut self, _: WorldPong, ctx: &mut Context<Self>) {
            println!("WorldPong");
    }
}