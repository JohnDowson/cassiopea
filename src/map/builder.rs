use super::{Map, Rect, Tile};
use crate::{components::Position, random::room_table, spawner};
use rltk::RandomNumberGenerator;
use specs::prelude::*;
use std::collections::HashSet;

pub trait MapBuilder {
    fn build(
        &mut self,
        dim_x: i32,
        dim_y: i32,
        layer: i32,
        rng: &mut RandomNumberGenerator,
    ) -> (Map, Position);
    fn spawn(&mut self, map: &Map, ecs: &mut World, layer: i32);
}

pub struct SimpleMapBuilder;

impl SimpleMapBuilder {
    fn simple_map(map: &mut Map) {
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

    fn spawn_rooms(map: &Map, ecs: &mut World) {
        let mut spawn_points = HashSet::new();
        {
            let mut rng = ecs.write_resource::<RandomNumberGenerator>();
            let num_spawns = dbg! {rng.roll_dice(1, 4 + map.layer) - 1};
            for room in &map.rooms {
                for _ in 0..num_spawns {
                    let mut added = false;
                    let mut tries = 0;
                    while !added && tries < 20 {
                        let (x1, y1, x2, y2) = room.coords();
                        let x = x1 + rng.roll_dice(1, i32::abs(x2 - x1));
                        let y = y1 + rng.roll_dice(1, i32::abs(y2 - y1));

                        added = dbg! {spawn_points.insert((x, y))};
                        if !added {
                            tries += 1
                        }
                    }
                }
            }
        }

        let spawn_table = room_table();
        let spawns = {
            let mut rng = ecs.write_resource::<RandomNumberGenerator>();
            spawn_points
                .into_iter()
                .map(|s| (s, spawn_table.roll(&mut rng)))
                .collect::<Vec<_>>()
        };
        for ((x, y), spawn) in spawns.into_iter() {
            if let Some(spawn) = spawn {
                match dbg! {spawn} {
                    "Skel" => spawner::skel(ecs, x, y),
                    "Snake" => spawner::snake(ecs, x, y),
                    "Healing cell" => spawner::healing_cell(ecs, x, y),
                    "Laser cell" => spawner::laser_cell(ecs, x, y),
                    "Compact missile" => spawner::compact_missile(ecs, x, y),
                    "Energy Shield" => spawner::energy_shield(ecs, x, y),
                    "Vibro Blade" => spawner::vibro_blade(ecs, x, y),
                    _ => {}
                }
            }
        }
    }
}

impl MapBuilder for SimpleMapBuilder {
    fn build(
        &mut self,
        dim_x: i32,
        dim_y: i32,
        layer: i32,
        rng: &mut RandomNumberGenerator,
    ) -> (Map, Position) {
        let mut new = Map::new(dim_x, dim_y, layer);
        Self::simple_map(&mut new);
        let (x, y) = rng.random_slice_entry(&new.rooms).unwrap().center();
        (new, Position { x, y })
    }

    fn spawn(&mut self, map: &Map, ecs: &mut World, _layer: i32) {
        Self::spawn_rooms(map, ecs)
    }
}
