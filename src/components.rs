use std::collections::HashSet;

use rltk::{Point, RGB};
use specs::prelude::*;
use specs_derive::Component;

#[derive(Component, Debug, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn as_point(&self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: u8,
}

#[derive(Component, Debug)]
pub struct Control;

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: HashSet<rltk::Point>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Component, Debug)]
pub struct Enemy;

#[derive(Component)]
pub struct Name {
    pub name: String,
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Component, Debug)]
pub struct Blocker;

#[derive(Component, Debug)]
pub struct Stats {
    pub base_power: i32,
    pub base_hp: i32,
    pub hp: i32,
    pub defense: i32,
}

#[derive(Component, Debug)]
pub struct MeleeAttack {
    pub target: Entity,
}

#[derive(Component, Debug)]
pub struct TakeDamage {
    pub amount: i32,
}

impl TakeDamage {
    pub fn new_damage(store: &mut WriteStorage<Self>, target: Entity, amount: i32) {
        let dmg = TakeDamage { amount };
        store.insert(target, dmg).expect("Unable to insert damage");
    }
}

#[derive(Component, Debug)]
pub struct Item;

#[derive(Component, Debug)]
pub struct Consumable;

#[derive(Component, Debug, Clone, Copy)]
pub struct InInventory {
    pub owner: Entity,
    pub item: Entity,
}

#[derive(Component, Debug)]
pub struct HasInventory;

#[derive(Component, Debug)]
pub struct WantsToPickUp {
    pub collector: Entity,
    pub item: Entity,
}

#[derive(Debug)]
pub enum Target {
    Itself,
    Other(Entity),
    Tile(i32, i32),
}

#[derive(Component, Debug)]
pub struct WantsToUseItem {
    pub item: Entity,
    pub target: Target,
}

#[derive(Component, Debug)]
pub enum Effect {
    HealSelf(i32),
    DamageRanged {
        range: i32,
        damage: i32,
    },
    DamageAOE {
        range: i32,
        damage: i32,
        radius: i32,
    },
}
