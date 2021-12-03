use std::collections::HashSet;

use crate::{components::*, map::Map};
use rltk::{to_cp437, RandomNumberGenerator, RGB};
use specs::prelude::*;

pub fn player(ecs: &mut World, x: i32, y: i32) -> Entity {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Control)
        .with(Viewshed {
            visible_tiles: Default::default(),
            range: 8,
            dirty: true,
        })
        .with(Blocker)
        .with(Stats {
            base_power: 10,
            base_health: 20,
            hp: 20,
            defense: 3,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .build()
}

pub fn random_enemy(ecs: &mut World, x: i32, y: i32) {
    let (glyph, name) = ecs
        .write_resource::<RandomNumberGenerator>()
        .random_slice_entry(&[('&', "Snake"), ('$', "Skel")])
        .unwrap();
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: to_cp437(*glyph),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Viewshed {
            visible_tiles: Default::default(),
            range: 8,
            dirty: true,
        })
        .with(Enemy)
        .with(Name {
            name: name.to_string(),
        })
        .with(Blocker)
        .with(Stats {
            base_power: 5,
            base_health: 5,
            hp: 5,
            defense: 5,
        })
        .build();
}

pub fn spawn_room(ecs: &mut World) {
    let mut monster_spawn_points = HashSet::new();
    let mut item_spawn_points = HashSet::new();
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let map = ecs.read_resource::<Map>();
        let num_monsters = rng.roll_dice(1, 4);
        let num_items = rng.roll_dice(1, 2);
        for room in &map.rooms {
            for _i in 0..num_monsters {
                let mut added = false;
                while !added {
                    let (x1, y1, x2, y2) = room.coords();
                    let x = (x1 + rng.roll_dice(1, i32::abs(x2 - x1))) as usize;
                    let y = (y1 + rng.roll_dice(1, i32::abs(y2 - y1))) as usize;
                    if !monster_spawn_points.contains(&(x, y)) {
                        monster_spawn_points.insert((x, y));
                        added = true;
                    }
                }
            }
            for _i in 0..num_items {
                let mut added = false;
                while !added {
                    let (x1, y1, x2, y2) = room.coords();
                    let x = (x1 + rng.roll_dice(1, i32::abs(x2 - x1))) as usize;
                    let y = (y1 + rng.roll_dice(1, i32::abs(y2 - y1))) as usize;
                    if !item_spawn_points.contains(&(x, y)) {
                        item_spawn_points.insert((x, y));
                        added = true;
                    }
                }
            }
        }
    }
    for &(x, y) in monster_spawn_points.iter() {
        random_enemy(ecs, x as i32, y as i32);
    }
    for &(x, y) in item_spawn_points.iter() {
        health_potion(ecs, x as i32, y as i32);
    }
}

fn health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Name {
            name: "Health Potion".to_string(),
        })
        .with(Item)
        .with(Effect::Heal(9))
        .build();
}
