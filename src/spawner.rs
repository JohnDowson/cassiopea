use std::collections::HashSet;

use crate::{components::*, map::Map, player::Player};
use rltk::{to_cp437, Point, RandomNumberGenerator, RGB};
use specs::{
    prelude::*,
    saveload::{MarkedBuilder, SimpleMarker},
};

pub fn player(ecs: &mut World, x: i32, y: i32) -> Player {
    let entity = ecs
        .create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 1,
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
            base_hp: 20,
            hp: 20,
            defense: 3,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .with(HasInventory)
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    let position = Point::new(x, y);
    Player { entity, position }
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
            render_order: 2,
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
            base_hp: 5,
            hp: 5,
            defense: 5,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

pub fn spawn_room(ecs: &mut World) {
    let mut monster_spawn_points = HashSet::new();
    let mut item_spawn_points = HashSet::new();
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let map = ecs.read_resource::<Map>();
        let num_monsters = rng.roll_dice(1, 2);
        let num_items = rng.roll_dice(1, 3);
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
        random_item(ecs, x as i32, y as i32);
    }
}

fn random_item(ecs: &mut World, x: i32, y: i32) {
    let loot_table = [healing_cell, laser_cell, compact_missile];
    let roll = {
        ecs.write_resource::<RandomNumberGenerator>()
            .random_slice_entry(&loot_table)
            .unwrap()
    };
    roll(ecs, x, y)
}

fn healing_cell(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437(';'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Name {
            name: "Healing cell".to_string(),
        })
        .with(Item)
        .with(Consumable)
        .with(Effect::HealSelf(9))
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn laser_cell(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('\''),
            fg: RGB::named(rltk::GREEN),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Name {
            name: "Laser Cell".to_string(),
        })
        .with(Item)
        .with(Consumable)
        .with(Effect::DamageRanged {
            range: 5,
            damage: 10,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn compact_missile(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('>'),
            fg: RGB::named(rltk::GREEN4),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Name {
            name: "Compact Missile".to_string(),
        })
        .with(Item)
        .with(Consumable)
        .with(Effect::DamageAOE {
            range: 5,
            damage: 10,
            radius: 3,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
