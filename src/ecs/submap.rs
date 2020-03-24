
use std::fs::File;
use std::io::Read;
// pub type Board = Vec<Vec<u8>>;
// use types::TileMap;


#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TileMap {
    pub w : usize,
    pub h : usize,
    pub data: Vec<Vec<u8>>
}

impl TileMap {
    pub fn new(w: usize, h: usize) -> Self {
        let mut data = vec![vec![0; h]; w];
        TileMap {w, h, data}
    }

    fn get_tile_value(&self, pos: &(i32, i32)) -> u8 {
        self.data[pos.1 as usize][pos.0 as usize]
    }

    pub fn load_map(&mut self) -> std::io::Result<()>  {
        let mut file_sub_map = File::open("sub-map.bin")?;
        // read the same file back into a Vec of bytes
        let mut sub_map_buffer = Vec::<u8>::new();
        file_sub_map.read_to_end(&mut sub_map_buffer)?;

        let w = self.w;
        let h = self.h;

        for y in 0..h {
            for x in 0..w {
                self.data[y as usize][x as usize] = sub_map_buffer[y * w + x];
            }
        }
        
        Ok(())
    }

    
    pub fn is_land_tile(&self, pos: &(i32, i32)) -> bool {
        let tile = self.data[pos.1 as usize][pos.0 as usize];
        match tile {
            1 => true,
            2 => true,
            3 => true,
            4 => true,
            5 => true,
            _ => false
        }
    }

    pub fn is_road_tile(&self, pos: &(i32, i32)) -> bool {
        let tile = self.data[pos.1 as usize][pos.0 as usize];
        match tile {
            6 => true,
            _ => false
        }
    }

    pub fn is_alley_tile(&self, pos: &(i32, i32)) -> bool {
        let tile = self.data[pos.1 as usize][pos.0 as usize];
        match tile {
            11 => true,
            _ => false
        }
    }

    pub fn is_deadend_tile(&self, pos: &(i32, i32)) -> bool {
        let tile = self.get_tile_value(pos);
        //println!("is_deadend_tile  {:?} {:?}: {:?}", pos.0, pos.1, tile);
        match tile {
            6 => false,
            11 => false,
            _ => true
        }
    }

    pub fn is_resource_tile(&self, pos: &(i32, i32)) -> bool {
        let tile = self.data[pos.1 as usize][pos.0 as usize];
        match tile {
            8 => true,
            _ => false
        }
    }

    pub fn can_move_to(&self, pos: &(i32, i32)) -> bool {
        let tile = self.get_tile_value(pos);
        if tile == 0 {
            return false;
        }
        // if !self.is_deadend_tile(pos) {
        //     return true;
        // }
        let &(x, y) = pos;
        if !self.is_deadend_tile(&(x - 1, y)) {
            return true;
        }
        if !self.is_deadend_tile(&(x + 1, y)) {
            return true;
        }
        if !self.is_deadend_tile(&(x, y - 1)) {
            return true;
        }
        if !self.is_deadend_tile(&(x, y + 1)) {
            return true;
        }
        return false;
    }

    pub fn get_move_cost(&self, pos: &(i32, i32)) -> u32 {
        let tile = self.data[pos.1 as usize][pos.0 as usize];
        match tile {
            6 => 1,
            _ => 5
        }
    }
    

    pub fn successors(&self, pos: &(i32, i32)) -> Vec<((i32, i32), u32)> {
        let tile = self.get_tile_value(pos);
        // println!("successors  {:?} {:?}: {:?}", pos.0, pos.1, tile);
        match tile{
            0 => {Vec::new()},
            _ => {vec![(pos.0 - 1, pos.1), (pos.0 + 1, pos.1), (pos.0, pos.1 - 1), (pos.0, pos.1 + 1)]
                .into_iter().map(|p| (p, self.get_move_cost(&p))).collect()}
        }
    }
}


// pub fn load_map() -> std::io::Result<(TileMap)>  {
//     let mut file_sub_map = File::open("sub-map.bin")?;
//     // read the same file back into a Vec of bytes
//     let mut sub_map_buffer = Vec::<u8>::new();
//     file_sub_map.read_to_end(&mut sub_map_buffer)?;

//     let mut board = vec![vec![0; 390]; 390];
//     for y in 0..390 {
//         for x in 0..390 {
//             board[y as usize][x as usize] = sub_map_buffer[y * 390 + x];
//         }
//     }
//     let mut tilemap = TileMap {
//         w: 390,
//         h: 390,
//         data: board
//     }
//     Ok((tilemap))
// }
