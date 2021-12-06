use super::{spawn_room, MapBuilder};
use crate::{
    components::Position,
    map::{Map, Rect, Tile},
};
use rltk::RandomNumberGenerator;
use specs::prelude::*;

pub struct SimpleMapBuilder {
    map: Map,
    snapshots: Vec<Map>,
}

impl SimpleMapBuilder {
    pub fn new(dim_x: i32, dim_y: i32, layer: i32) -> Self {
        Self {
            map: Map::new(dim_x, dim_y, layer),
            snapshots: Vec::new(),
        }
    }

    fn simple_map(&mut self) {
        let map = &mut self.map;
        let dim_x = map.dim_x;
        let dim_y = map.dim_x;

        for x in 0..dim_x {
            map[(x, 0)] = Tile::Wall;
            map[(x, dim_y - 1)] = Tile::Wall;
        }
        for y in 0..dim_y {
            map[(0, y)] = Tile::Wall;
            map[(dim_x - 1, y)] = Tile::Wall;
        }

        let mut rng = rltk::RandomNumberGenerator::new();
        for _ in 0..dim_x / 4 {
            // FIXME: Fix rooms sticking out of bounds
            let w = rng.range(5, 20);
            let h = rng.range(3, 10);
            let x = rng.roll_dice(1, (dim_x - 1) - w);
            let y = rng.roll_dice(1, (dim_y - 1) - h);
            let new_room = Rect::new(x, y, w, h);
            let ok = !map.rooms.iter().any(|room| room.intersects(&new_room));
            if ok {
                map.add_room(&new_room);

                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        map.add_horizontal_tunnel(prev_x, new_x, prev_y);
                        map.add_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        map.add_vertical_tunnel(prev_y, new_y, prev_x);
                        map.add_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }

                map.rooms.push(new_room);
            }
        }
        if rng.range(0, 2) != 0 {
            let coords = rng
                .random_slice_entry(&map.rooms[..map.rooms.len() - 1])
                .unwrap()
                .center();
            map[coords] = Tile::TerminalService;
        }
        let coords = map.rooms.last().unwrap().center();
        map[coords] = Tile::TerminalDown;
        map.populate_passable();
    }
}

impl MapBuilder for SimpleMapBuilder {
    fn build(&mut self, _rng: &mut RandomNumberGenerator) {
        self.simple_map()
    }

    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_player_spawn(&self, rng: &mut RandomNumberGenerator) -> Position {
        let (x, y) = rng.random_slice_entry(&self.map.rooms).unwrap().center();
        Position { x, y }
    }

    fn spawn(&mut self, ecs: &mut World) {
        for room in &self.map.rooms {
            spawn_room(&self.map, room, ecs)
        }
    }

    fn take_snapshot(&mut self) {
        //self.snapshots.push(self.map.as_ref().unwrap().clone())
    }

    fn get_snapshots(&self) -> Vec<Map> {
        self.snapshots.clone()
    }
}
