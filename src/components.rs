use crate::{map::Map, random};
use rltk::{Point, RGB};
use serde::{Deserialize, Serialize};
#[allow(deprecated)]
use specs::{
    error::NoError,
    prelude::*,
    saveload::{ConvertSaveload, Marker},
};
use specs_derive::{Component, ConvertSaveload};
use std::collections::HashSet;

#[derive(Component, Debug, Clone, Copy, ConvertSaveload)]
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

#[derive(Component, ConvertSaveload, Clone)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: u8,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Control;

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Viewshed {
    #[serde(skip)]
    pub visible_tiles: HashSet<rltk::Point>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Enemy;

#[derive(Component, ConvertSaveload, Clone)]
pub struct Name {
    pub name: String,
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Blocker;

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Stats {
    pub base_power: i32,
    pub base_hp: i32,
    pub hp: i32,
    pub base_defense: i32,
    pub compute: i32,
    pub base_compute: i32,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct MeleeAttack {
    pub target: Entity,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct TakeDamage {
    pub amount: i32,
}

impl TakeDamage {
    pub fn new_damage(store: &mut WriteStorage<Self>, target: Entity, amount: i32) {
        let dmg = TakeDamage { amount };
        store.insert(target, dmg).expect("Unable to insert damage");
    }
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Item;

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Consumable;

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Equippable {
    pub slot: Slot,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Slot {
    Head,
    Hands,
    Body,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Equipped {
    pub owner: Entity,
    pub slot: Slot,
    pub item: Entity,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Slots {
    pub slots: HashSet<Slot>,
}

#[derive(Component, Debug, Clone, Copy, ConvertSaveload)]
pub struct InInventory {
    pub owner: Entity,
    pub item: Entity,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct HasInventory;

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToPickUp {
    pub collector: Entity,
    pub item: Entity,
}

#[derive(Debug, ConvertSaveload, Clone)]
pub enum Target {
    Itself,
    Other(Entity),
    Tile(i32, i32),
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct WantsToUseItem {
    pub item: Entity,
    pub target: Target,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
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
    Recharge(i32),
}
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub enum EquipBonus {
    Defense(i32),
    Attack(i32),
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct SerializeMe;

#[derive(Component, ConvertSaveload, Clone)]
pub struct SerializationHelper {
    pub map: Map,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct LevelUp {
    pub amount: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Particle {
    pub lifetime: f32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct TraceTimer {
    pub timer: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Drops {
    pub table: random::Table,
}
