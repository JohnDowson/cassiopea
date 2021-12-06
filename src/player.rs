use std::cmp::{max, min};
use std::collections::HashSet;

use crate::components::*;
use crate::gui::GameLog;
use crate::map::Tile;
use crate::{map::Map, state::RunState};
use rltk::{Rltk, VirtualKeyCode, RGB};
use serde::{Deserialize, Serialize};
#[allow(deprecated)]
use specs::error::NoError;
use specs::prelude::*;
use specs::saveload::{ConvertSaveload, MarkedBuilder, Marker, SimpleMarker};
use specs_derive::ConvertSaveload;

#[derive(ConvertSaveload, Clone)]
pub struct Player {
    pub entity: Entity,
    pub position: Position,
}

pub fn player_input(ecs: &mut World, ctx: &mut Rltk) -> RunState {
    use VirtualKeyCode::*;
    match ctx.key {
        None => return RunState::AwaitingInput,
        Some(key) => match key {
            A => try_move_player(ecs, -1, 0),
            D => try_move_player(ecs, 1, 0),
            W => try_move_player(ecs, 0, -1),
            S => try_move_player(ecs, 0, 1),
            Q => try_move_player(ecs, -1, -1),
            E => try_move_player(ecs, 1, -1),
            Z => try_move_player(ecs, -1, 1),
            X => try_move_player(ecs, 1, 1),
            G => get_item(ecs),
            R => return try_interact(ecs),
            I => return RunState::ShowInventory,
            Escape => return RunState::MainMenu(crate::gui::MainMenuSelection::SaveGame),
            Space => return RunState::PlayerTurn,
            _ => return RunState::AwaitingInput,
        },
    }
    RunState::NPCTurn
}

fn try_interact(ecs: &mut World) -> RunState {
    let player = ecs.fetch::<Player>();
    let map = ecs.fetch::<Map>();

    match map[(player.position.x, player.position.y)] {
        Tile::TerminalDown => RunState::NextLayer,
        Tile::TerminalUp => todo!(),
        Tile::TerminalService => RunState::RevealMap(0),
        _ => RunState::AwaitingInput,
    }
}

fn get_item(ecs: &mut World) {
    let player = ecs.fetch::<Player>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player.position.x && position.y == player.position.y {
            target_item = Some(item_entity);
        }
    }
    match target_item {
        None => gamelog.entry("There is nothing here to pick up.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickUp>();
            pickup
                .insert(
                    player.entity,
                    WantsToPickUp {
                        collector: player.entity,
                        item,
                    },
                )
                .expect("Unable to insert want to pickup");
        }
    }
}

fn try_move_player(ecs: &mut World, delta_x: i32, delta_y: i32) {
    let mut positions = ecs.write_storage::<Position>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let mut players = ecs.write_storage::<Control>();
    let mut player = ecs.write_resource::<Player>();
    let stats = ecs.read_storage::<Stats>();
    let mut melee = ecs.write_storage::<MeleeAttack>();
    let map = ecs.fetch::<Map>();

    for (_, pos, vis) in (&mut players, &mut positions, &mut viewsheds).join() {
        let x = min(map.dim_x - 1, max(0, pos.x + delta_x));
        let y = min(map.dim_y - 1, max(0, pos.y + delta_y));
        for maybe_target in map.tile_content[map.coords_to_idx(x, y)].iter() {
            if let Some(_t) = stats.get(*maybe_target) {
                melee
                    .insert(
                        player.entity,
                        MeleeAttack {
                            target: *maybe_target,
                        },
                    )
                    .expect("Can't insert Melee");
                return;
            }
        }
        if map.passable[map.coords_to_idx(x, y)] {
            pos.x = x;
            pos.y = y;
            player.position.x = x;
            player.position.y = y;
        }
        vis.dirty = true;
    }
}

pub fn player(ecs: &mut World, pos: Position) -> Player {
    let mut slots = HashSet::new();
    slots.insert(Slot::Body);
    slots.insert(Slot::Hands);
    let entity = ecs
        .create_entity()
        .with(pos)
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
    let position = pos;
    Player { entity, position }
}
