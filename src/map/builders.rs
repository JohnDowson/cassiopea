use crate::{
    components::Position,
    map::{Map, Rect},
    spawner,
};
use rltk::RandomNumberGenerator;
use specs::prelude::*;
use std::collections::HashSet;

pub mod bsp;
pub mod simple;

pub trait MapBuilder {
    fn build(&mut self, rng: &mut RandomNumberGenerator);
    fn get_map(&self) -> Map;
    fn get_player_spawn(&self, rng: &mut RandomNumberGenerator) -> Position;
    fn spawn(&mut self, ecs: &mut World);
    fn take_snapshot(&mut self);
    fn get_snapshots(&self) -> Vec<Map>;
}

fn spawn_room(map: &Map, room: &Rect, ecs: &mut World) {
    let mut spawn_points = HashSet::new();
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_spawns = rng.roll_dice(1, 4 + map.layer) - 1;
        for _ in 0..num_spawns {
            let mut added = false;
            let mut tries = 0;
            while !added && tries < 20 {
                let (x1, y1, x2, y2) = room.coords();
                let x = x1 + rng.roll_dice(1, i32::abs(x2 - x1));
                let y = y1 + rng.roll_dice(1, i32::abs(y2 - y1));

                added = spawn_points.insert((x, y));
                if !added {
                    tries += 1
                }
            }
        }
    }

    let spawn_table = spawner::room_table(map.layer);
    let spawns = {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        spawn_points
            .into_iter()
            .map(|s| (s, spawn_table.roll(&mut rng)))
            .collect::<Vec<_>>()
    };
    for ((x, y), spawn) in spawns.into_iter() {
        if let Some(spawn) = spawn {
            match spawn {
                "Skel" => spawner::skel(ecs, x, y),
                "Snake" => spawner::snake(ecs, x, y),
                "Healing cell" => spawner::healing_cell(ecs, x, y),
                "Laser cell" => spawner::laser_cell(ecs, x, y),
                "Compact missile" => spawner::compact_missile(ecs, x, y),
                "Energy Shield" => spawner::energy_shield(ecs, x, y),
                "Vibro Blade" => spawner::vibro_blade(ecs, x, y),
                "Memory Shard" => spawner::memory_shard(ecs, x, y),
                "Energy Cell" => spawner::energy_cell(ecs, x, y),
                _ => {}
            }
        }
    }
}
