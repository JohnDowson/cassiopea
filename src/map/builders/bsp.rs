use crate::{
    components::Position,
    map::{Map, Rect, Tile},
};
use rltk::RandomNumberGenerator;
use specs::prelude::*;

use super::{spawn_room, MapBuilder};

pub struct BspBuilder {
    map: Map,
    // snapshots: Vec<Map>,
    starting_position: Position,
    rects: Vec<Rect>,
}

impl BspBuilder {
    pub fn new(dim_x: i32, dim_y: i32, layer: i32) -> Self {
        Self {
            map: Map::new(dim_x, dim_y, layer),
            // snapshots: Vec::new(),
            starting_position: Position { x: 0, y: 0 },
            rects: Vec::new(),
        }
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator) {
        self.rects
            .push(Rect::new(1, 1, self.map.dim_x - 2, self.map.dim_y - 2));
        let first = self.rects[0];
        self.subdivide(first);

        let mut n_rooms = 0;
        while n_rooms < 512 {
            let rect = self.get_rekt(rng);
            let candidate = self.get_sub_rekt(rect, rng);

            if self.is_possible(candidate) {
                self.map.add_room(&candidate);
                self.map.rooms.push(candidate);
                self.subdivide(rect);
            }

            n_rooms += 1;
        }
        self.map
            .rooms
            .sort_by(|a, b| a.top_left.0.cmp(&b.top_left.0));

        for i in 0..self.map.rooms.len() - 1 {
            let room = self.map.rooms[i];
            let next_room = self.map.rooms[i + 1];
            let start_x = room.x1() + (rng.roll_dice(1, i32::abs(room.x1() - room.x2())) - 1);
            let start_y = room.y1() + (rng.roll_dice(1, i32::abs(room.y1() - room.y2())) - 1);
            let end_x =
                next_room.x1() + (rng.roll_dice(1, i32::abs(next_room.x1() - next_room.x2())) - 1);
            let end_y =
                next_room.y1() + (rng.roll_dice(1, i32::abs(next_room.y1() - next_room.y2())) - 1);
            self.draw_corridor(start_x, start_y, end_x, end_y);
        }

        let start = self.map.rooms[0].center();
        self.starting_position = Position {
            x: start.0,
            y: start.1,
        };
        let coords = rng
            .random_slice_entry(&self.map.rooms[1..])
            .unwrap()
            .center();
        self.map[coords] = Tile::TerminalService;

        let coords = self.map.rooms.last().unwrap().center();
        self.map[coords] = Tile::TerminalDown;
    }
    fn draw_corridor(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        let mut x = x1;
        let mut y = y1;

        while x != x2 || y != y2 {
            if x < x2 {
                x += 1;
            } else if x > x2 {
                x -= 1;
            } else if y < y2 {
                y += 1;
            } else if y > y2 {
                y -= 1;
            }
            self.map[(x, y)] = Tile::Floor;
        }
    }
    fn is_possible(&self, mut rect: Rect) -> bool {
        rect.top_left.0 -= 2;
        rect.bottom_right.0 += 2;
        rect.top_left.1 -= 2;
        rect.bottom_right.1 += 2;

        let mut can_build = true;

        for y in rect.top_left.1..=rect.bottom_right.1 {
            for x in rect.top_left.0..=rect.bottom_right.0 {
                if x > self.map.dim_x - 2 {
                    can_build = false;
                }
                if y > self.map.dim_y - 2 {
                    can_build = false;
                }
                if x < 1 {
                    can_build = false;
                }
                if y < 1 {
                    can_build = false;
                }
                if can_build && self.map[(x, y)] != Tile::Wall {
                    can_build = false;
                }
            }
        }

        can_build
    }

    fn subdivide(&mut self, rect: Rect) {
        let width = i32::abs(rect.x1() - rect.x2());
        let height = i32::abs(rect.y1() - rect.y2());
        let half_width = i32::max(width / 2, 1);
        let half_height = i32::max(height / 2, 1);

        self.rects
            .push(Rect::new(rect.x1(), rect.y1(), half_width, half_height));
        self.rects.push(Rect::new(
            rect.x1(),
            rect.y1() + half_height,
            half_width,
            half_height,
        ));
        self.rects.push(Rect::new(
            rect.x1() + half_width,
            rect.y1(),
            half_width,
            half_height,
        ));
        self.rects.push(Rect::new(
            rect.x1() + half_width,
            rect.y1() + half_height,
            half_width,
            half_height,
        ));
    }

    fn get_rekt(&self, rng: &mut RandomNumberGenerator) -> Rect {
        *rng.random_slice_entry(&self.rects).unwrap()
    }

    fn get_sub_rekt(&self, mut rect: Rect, rng: &mut RandomNumberGenerator) -> Rect {
        let rect_width = i32::abs(rect.top_left.0 - rect.bottom_right.0);
        let rect_height = i32::abs(rect.top_left.1 - rect.bottom_right.1);

        let w = i32::max(3, rng.roll_dice(1, i32::min(rect_width, 10)) - 1) + 1;
        let h = i32::max(3, rng.roll_dice(1, i32::min(rect_height, 10)) - 1) + 1;

        rect.top_left.0 += rng.roll_dice(1, 6) - 1;
        rect.top_left.1 += rng.roll_dice(1, 6) - 1;
        rect.bottom_right.0 = rect.top_left.0 + w;
        rect.bottom_right.1 = rect.top_left.1 + h;

        rect
    }
}

impl MapBuilder for BspBuilder {
    fn build(&mut self, rng: &mut RandomNumberGenerator) {
        self.build(rng)
    }

    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_player_spawn(&self, _rng: &mut RandomNumberGenerator) -> Position {
        self.starting_position
    }

    fn spawn(&mut self, ecs: &mut World) {
        for room in &self.map.rooms {
            spawn_room(&self.map, room, ecs)
        }
    }

    fn take_snapshot(&mut self) {
        todo!()
    }

    fn get_snapshots(&self) -> Vec<Map> {
        todo!()
    }
}
