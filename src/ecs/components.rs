

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LandPos(pub i32, pub i32);

impl LandPos {
    pub fn get_hash_map_key(&self) -> i32  {
        let ret = (self.0 / 6) as i32 + 1000 * ((self.1 / 6) as i32);
        (ret)
    }

    pub fn distance(&self, other: &LandPos) -> u32 {
        ((self.0 - other.0).abs() + (self.1 - other.1).abs()) as u32
    }

    pub fn get_hash_map_key_successors(&self, range: u32) -> Vec<i32> {
        let mut lst = Vec::<i32>::new();
        let i_range = range as i32;
        for y in -i_range..=i_range {
            for x in -i_range..=i_range {
                let lp = LandPos(self.0 + x * 6, self.1 + y * 6);
                // println!("{:?}, {:?} = {:?}", lp.0, lp.1, lp.get_hash_map_key());
                lst.push(lp.get_hash_map_key());
            }
        }
        (lst)
    }

    pub fn get_hash_map_key_successors_at_radius(&self, range: u32) -> Vec<i32> {
        let mut lst = Vec::<i32>::new();
        let i_range = range as i32;

        let mut x = -i_range;
        while x <= i_range {
            lst.push(LandPos(self.0 + x * 6, self.1 - i_range * 6).get_hash_map_key());
            lst.push(LandPos(self.0 + x * 6, self.1 + i_range * 6).get_hash_map_key());

            x += 1;
        }

        let mut y = -(i_range - 1);
        while y <= (i_range - 1) {
            lst.push(LandPos(self.0 - i_range * 6, self.1 - y * 6).get_hash_map_key());
            lst.push(LandPos(self.0 + i_range * 6, self.1 + y * 6).get_hash_map_key());

            y += 1;
        }
        (lst)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NewPos(pub i32, pub i32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vel(pub i32, pub i32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Moving{
    pub vx: i32,
    pub vy: i32,
    pub speed: u64, // 1/maxstep per ms
    pub cost: u32,
    pub step: u64,
    pub maxstep: u64,
}

impl Moving {
    pub fn new() -> Self {
        Moving {vx: 0, vy: 0, speed: 2, cost: 1, step: 0, maxstep: 0}
    }
}

pub enum UnitModelType {
    None = 0,
    Axie,
    Chimera,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuadrantKey(pub i32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HomeLand(pub LandPos);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Owner(pub u32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Model(pub u32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AutoCollect;

pub enum BuildingModelType {
    None = 1000,
    ResourceNode,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResourceNode;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Static;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Chimera;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChimeraState {
    pub state: i32 //0: idle, 1:moving,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FollowPath {
    //pub state: i32 //0: none, 1: request path, 2: moving, 3:finished,
    pub sx: i32,
    pub sy: i32,
    pub tx: i32,
    pub ty: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChimeraSpawner {
    // x: i32,
    // y: i32,
    // w: i32,
    // h: i32,
    pub count: i32,
    pub cooldown_ms: i32,
    pub tick_ms: i32,
}



pub enum WorldStateType {
    None,
    GatherResource,
    GatherResourceDone,
    ReleaseResource,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldState(pub i32, pub i32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GAction{
    //pub id: i32,
    //pub target: Option<LandPos>,
    //pub cost: u32,
    pub duration_ms: u32,
    pub pre_conditions: WorldState,
    pub after_effects: WorldState,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GActionGatherResource;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GActionReleaseResource;

impl GAction {
    pub fn new_gather_resource_action() -> Self {
        let pre_conditions = WorldState(WorldStateType::GatherResource as i32, 1);
        let after_effects = WorldState(WorldStateType::GatherResourceDone as i32, 1);
        GAction {
            duration_ms: 5000,
            pre_conditions: pre_conditions,
            after_effects: after_effects,
        }
    }

    pub fn new_release_resource_action() -> Self {
        let pre_conditions = WorldState(WorldStateType::GatherResourceDone as i32, 1);
        let after_effects = WorldState(WorldStateType::ReleaseResource as i32, 1);

        GAction {
            duration_ms: 1000,
            pre_conditions: pre_conditions,
            after_effects: after_effects,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GGoal;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GatherResourceGoal {
    pub step: i32,
    pub home_pos: LandPos,
    pub target_pos: LandPos
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerInput{
    pub owner: u32,
    pub request_id: u32,
    pub status: u32, //0: requesting, 1: responsed, _:will delete
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerInputGetStateAroundLand(pub i32, pub i32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerInputAxie{
    pub axie_index: u32
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerInputAxieGatherResource{
    pub resource_id: u32
}
