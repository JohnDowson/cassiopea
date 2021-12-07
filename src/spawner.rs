use std::{cmp::min, collections::HashSet};

use crate::{components::*, player::Player, random};
use rltk::RGB;
use specs::{
    prelude::*,
    saveload::{MarkedBuilder, SimpleMarker},
};
pub fn room_table(layer: i32) -> random::Table {
    random::Table::new()
        .insert("Skel", min(layer, 10))
        .insert("Snake", 10 - layer)
        .insert("Healing cell", 3)
        .insert("Laser cell", 3)
        .insert("Compact missile", 2)
        .insert("Energy Shield", 2)
        .insert("Vibro Blade", 2)
        .insert("Memory Shard", 1)
        .insert("Energy Cell", 2)
}

pub fn player(ecs: &mut World) -> Player {
    let mut slots = HashSet::new();
    slots.insert(Slot::Body);
    slots.insert(Slot::Hands);
    let entity = ecs
        .create_entity()
        .with(Position { x: 0, y: 0 })
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
            base_power: 3,
            base_hp: 10,
            hp: 10,
            base_defense: 1,
            compute: 10,
            base_compute: 10,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .with(HasInventory)
        .with(Slots { slots })
        .with(TraceTimer { timer: 400 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
    Player {
        entity,
        position: Position { x: 0, y: 0 },
    }
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
            base_power: 3,
            base_hp: 5,
            hp: 5,
            base_defense: 1,
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
            base_power: 8,
            base_hp: 10,
            hp: 10,
            base_defense: 3,
            compute: 0,
            base_compute: 0,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

pub fn healing_cell(ecs: &mut World, x: i32, y: i32) {
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
        .with(Effect::HealSelf(10))
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

pub fn laser_cell(ecs: &mut World, x: i32, y: i32) {
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

pub fn compact_missile(ecs: &mut World, x: i32, y: i32) {
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

pub fn energy_shield(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('0'),
            fg: RGB::named(rltk::SKYBLUE),
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

pub fn vibro_blade(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('/'),
            fg: RGB::named(rltk::RED),
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

pub fn memory_shard(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('σ'),
            fg: RGB::named(rltk::PURPLE),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Name {
            name: "Memory Shard".to_string(),
        })
        .with(Item)
        .with(Consumable)
        .with(LevelUp { amount: 5 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

pub fn energy_cell(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('~'),
            fg: RGB::named(rltk::PURPLE),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Name {
            name: "Energy Cell".to_string(),
        })
        .with(Item)
        .with(Consumable)
        .with(Effect::Recharge(5))
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
