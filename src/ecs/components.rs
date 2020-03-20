

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pos(pub i32, pub i32);

impl Pos {
    pub fn get_hash_map_key(&self) -> i32  {
        let ret = (self.0 / 3) as i32 + 1000 * ((self.0 / 3) as i32);
        (ret)
    }

    pub fn distance(&self, other: &Pos) -> u32 {
        ((self.0 - other.0).abs() + (self.1 - other.1).abs()) as u32
      }
}

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Model(pub usize);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Chimera;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChimeraState {
    pub state: i32 //0: idle, 1:moving,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FollowPath {
    //pub state: i32 //0: none, 1: request path, 2: moving, 3:finished,
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
