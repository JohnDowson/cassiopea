use std::collections::HashSet;

use crate::{components::*, map::Map, player::Player, random};
use rltk::{Point, RandomNumberGenerator, RGB};
use specs::{
    prelude::*,
    saveload::{MarkedBuilder, SimpleMarker},
};

pub fn player(ecs: &mut World, x: i32, y: i32) -> Player {
    let mut slots = HashSet::new();
    slots.insert(Slot::Body);
    slots.insert(Slot::Hands);
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
            defense: 100,
            compute: 10,
            base_compute: 10,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .with(HasInventory)
        .with(Slots { slots })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    let position = Point::new(x, y);
    Player { entity, position }
}

fn room_table() -> random::Table {
    use random::*;
    Table::new()
        .insert("Skel", 4)
        .insert("Snake", 6)
        .insert("Healing cell", 8)
        .insert("Laser cell", 3)
        .insert("Compact missile", 8)
        .insert("Energy Shield", 8)
        .insert("Vibro Blade", 8)
}

pub fn snake(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('&'),
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
            name: "Snake".to_string(),
        })
        .with(Blocker)
        .with(Stats {
            base_power: 5,
            base_hp: 5,
            hp: 5,
            defense: 5,
            compute: 0,
            base_compute: 0,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

pub fn skel(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('$'),
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
            name: "Skel".to_string(),
        })
        .with(Blocker)
        .with(Stats {
            base_power: 5,
            base_hp: 5,
            hp: 5,
            defense: 5,
            compute: 0,
            base_compute: 0,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

pub fn spawn_room(ecs: &mut World) {
    let mut spawn_points = HashSet::new();
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let map = ecs.read_resource::<Map>();
        let num_spawns = rng.roll_dice(1, 4 + map.layer) - 1;
        for room in &map.rooms {
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
    }

    let spawn_table = room_table();
    let spawns = {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        spawn_points
            .into_iter()
            .map(|s| (s, spawn_table.roll(&mut *rng)))
            .collect::<Vec<_>>()
    };
    for ((x, y), spawn) in spawns.into_iter() {
        if let Some(spawn) = spawn {
            match spawn {
                "Skel" => skel(ecs, x, y),
                "Snake" => snake(ecs, x, y),
                "Healing cell" => healing_cell(ecs, x, y),
                "Laser cell" => laser_cell(ecs, x, y),
                "Compact missile" => compact_missile(ecs, x, y),
                "Energy Shield" => energy_shield(ecs, x, y),
                "Vibro Blade" => vibro_blade(ecs, x, y),
                _ => {}
            }
        }
    }
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

fn energy_shield(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('0'),
            fg: RGB::named(rltk::BLUE),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Name {
            name: "Energy Shield".to_string(),
        })
        .with(Item)
        .with(Equippable { slot: Slot::Body })
        .with(EquipBonus::Defense(5))
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn vibro_blade(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('/'),
            fg: RGB::named(rltk::BLUE),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Name {
            name: "Vibro Blade".to_string(),
        })
        .with(Item)
        .with(Equippable { slot: Slot::Hands })
        .with(EquipBonus::Attack(5))
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
